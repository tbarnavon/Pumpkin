//! Brings a Wasm plugin back after a trap.
//!
//! A trap poisons the component instance and ends its store driver, so every later call fails.
//! The supervisor started in `on_load` waits for that, then instantiates the plugin again,
//! swaps the new generation in and runs `on_load` on it. Only in-memory plugin state is lost.
//! Registrations that outlive the instance (event handlers, commands, tasks) are dropped first
//! so `on_load` can make them again; block and item hooks point at the `WasmPlugin`, not at a
//! generation, so they carry over.

use std::{
    any::Any,
    sync::{
        Arc, Mutex, OnceLock, Weak,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use pumpkin_core::plugin::{Context, PluginMetadata};
use pumpkin_plugin_runtime::RuntimeSpawner;
use pumpkin_wasm_host_common::{
    concurrent_store,
    plugin::{PluginGeneration, PluginInitError, WasmPlugin},
    signature::PumpkinMetadata,
};
use wasmtime::Engine;

use crate::runtime::AnyPluginPre;

/// A plugin that traps more often than this within `RESTART_WINDOW` stays down.
const MAX_RESTARTS: usize = 3;
const RESTART_WINDOW: Duration = Duration::from_secs(60);

pub struct Restarter {
    engine: Engine,
    plugin_pre: AnyPluginPre,
    legacy_sync_reentry: concurrent_store::LegacySyncReentry,
    spawner: Arc<dyn RuntimeSpawner>,
    marketplace_metadata: Option<PumpkinMetadata>,
    plugin: OnceLock<Weak<WasmPlugin>>,
    context: Mutex<Option<Arc<Context>>>,
    stopped: AtomicBool,
    restarts: Mutex<Vec<Instant>>,
}

impl Restarter {
    pub fn new(
        engine: Engine,
        plugin_pre: AnyPluginPre,
        legacy_sync_reentry: concurrent_store::LegacySyncReentry,
        spawner: Arc<dyn RuntimeSpawner>,
        marketplace_metadata: Option<PumpkinMetadata>,
    ) -> Self {
        Self {
            engine,
            plugin_pre,
            legacy_sync_reentry,
            spawner,
            marketplace_metadata,
            plugin: OnceLock::new(),
            context: Mutex::new(None),
            stopped: AtomicBool::new(false),
            restarts: Mutex::new(Vec::new()),
        }
    }

    /// Instantiates the component and runs its `init_plugin`, in a fresh store.
    pub async fn instantiate(
        &self,
    ) -> Result<(Arc<PluginGeneration>, PluginMetadata), PluginInitError> {
        let (plugin_instance, store, metadata) = match &self.plugin_pre {
            AnyPluginPre::V0_1(plugin_pre) => {
                let (plugin, store, metadata) = pumpkin_wasm_host_v0_1::init_plugin(
                    &self.engine,
                    plugin_pre.clone(),
                    &self.legacy_sync_reentry,
                )
                .await?;
                (
                    Arc::new(plugin) as Arc<dyn Any + Send + Sync>,
                    store,
                    metadata,
                )
            }
            AnyPluginPre::V0_2(plugin_pre) => {
                let (plugin, store, metadata) = pumpkin_wasm_host_v0_2::init_plugin(
                    &self.engine,
                    plugin_pre.clone(),
                    &self.legacy_sync_reentry,
                )
                .await?;
                (
                    Arc::new(plugin) as Arc<dyn Any + Send + Sync>,
                    store,
                    metadata,
                )
            }
        };
        let store = concurrent_store::start_legacy_store(
            store,
            self.legacy_sync_reentry.clone(),
            Arc::clone(&self.spawner),
        )
        .await
        .map_err(PluginInitError::InstantiationFailed)?;
        Ok((
            Arc::new(PluginGeneration {
                plugin_instance,
                store,
            }),
            metadata,
        ))
    }

    /// Points a generation's host state back at its plugin.
    pub async fn attach(
        &self,
        plugin: &Arc<WasmPlugin>,
        generation: &PluginGeneration,
    ) -> wasmtime::Result<()> {
        let weak_plugin = Arc::downgrade(plugin);
        let _ = self.plugin.set(weak_plugin.clone());
        let marketplace_metadata = self.marketplace_metadata.clone();
        generation
            .store
            .call(move |accessor| {
                Box::pin(async move {
                    accessor.with(|mut store| {
                        store.data_mut().plugin = Some(weak_plugin);
                        store.data_mut().marketplace_metadata = marketplace_metadata;
                    });
                    Ok(())
                })
            })
            .await
    }

    /// Restarts the plugin if this generation's store driver fails.
    pub fn watch(self: &Arc<Self>, context: &Arc<Context>, generation: &PluginGeneration) {
        *self
            .context
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Arc::clone(context));
        let Some(plugin) = self.plugin.get().cloned() else {
            return;
        };
        let restarter = Arc::clone(self);
        let mut join = generation.store.driver_join();
        let spawned = self.spawner.spawn(Box::pin(async move {
            if join.wait().await.is_ok() {
                return;
            }
            if let Some(plugin) = plugin.upgrade() {
                restarter.restart(plugin).await;
            }
        }));
        if let Err(error) = spawned {
            tracing::error!(%error, "Failed to watch Wasm plugin for traps");
        }
    }

    /// Called on unload, so the store shutting down isn't taken for a trap.
    pub fn stop(&self) {
        self.stopped.store(true, Ordering::Release);
    }

    fn allow_restart(&self) -> bool {
        let mut restarts = self
            .restarts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let now = Instant::now();
        restarts.retain(|at| now.duration_since(*at) < RESTART_WINDOW);
        if restarts.len() >= MAX_RESTARTS {
            return false;
        }
        restarts.push(now);
        true
    }

    async fn restart(self: Arc<Self>, plugin: Arc<WasmPlugin>) {
        if self.stopped.load(Ordering::Acquire) {
            return;
        }
        let Some(context) = self
            .context
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
        else {
            return;
        };
        let name = context.get_metadata().name.clone();
        if !self.allow_restart() {
            tracing::error!(
                plugin = %name,
                "Wasm plugin trapped {MAX_RESTARTS} times within {}s, not restarting it",
                RESTART_WINDOW.as_secs()
            );
            return;
        }
        // The store driver has already logged the trap itself.
        tracing::warn!(plugin = %name, "Wasm plugin trapped, restarting it");

        plugin.task_scheduler.cancel_plugin_tasks(&plugin);
        context.plugin_manager.unregister_handlers(&name);
        context.unregister_commands();

        let generation = match self.instantiate().await {
            Ok((generation, _)) => generation,
            Err(error) => {
                tracing::error!(plugin = %name, %error, "Failed to restart Wasm plugin");
                return;
            }
        };
        if let Err(error) = self.attach(&plugin, &generation).await {
            tracing::error!(plugin = %name, %error, "Failed to restart Wasm plugin");
            return;
        }
        plugin.replace_generation(generation);

        match crate::runtime::on_load(&plugin, &self, context).await {
            Ok(Ok(())) => tracing::info!(plugin = %name, "Restarted Wasm plugin"),
            Ok(Err(error)) => {
                tracing::error!(plugin = %name, %error, "Restarted Wasm plugin failed to load");
            }
            Err(error) => {
                tracing::error!(plugin = %name, %error, "Restarted Wasm plugin failed to load");
            }
        }
    }
}
