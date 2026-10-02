//! Worker instances for `scheduler.spawn-job` (see `pumpkin_wasm_host_common::jobs`).
//!
//! A worker is an instance of the plugin's component in its own store, with no server. Workers
//! are kept for reuse.

use std::future::Future;
use std::pin::Pin;
use std::sync::Mutex;

use pumpkin_wasm_host_common::{
    concurrent_store::LegacySyncReentry,
    jobs::{JobOutput, JobRunner},
    state::PluginHostState,
};
use wasmtime::{Engine, Store};

use crate::runtime::AnyPluginPre;

/// Idle workers one plugin keeps; jobs beyond that make more, dropped when they finish.
const MAX_IDLE_WORKERS: usize = 2;

enum WorkerPlugin {
    V0_1(pumpkin_wasm_host_v0_1::Plugin),
    V0_2(pumpkin_wasm_host_v0_2::Plugin),
}

struct Worker {
    store: Store<PluginHostState>,
    plugin: WorkerPlugin,
}

pub struct JobWorkers {
    engine: Engine,
    plugin_pre: AnyPluginPre,
    legacy_sync_reentry: LegacySyncReentry,
    idle: Mutex<Vec<Worker>>,
}

impl JobWorkers {
    pub const fn new(
        engine: Engine,
        plugin_pre: AnyPluginPre,
        legacy_sync_reentry: LegacySyncReentry,
    ) -> Self {
        Self {
            engine,
            plugin_pre,
            legacy_sync_reentry,
            idle: Mutex::new(Vec::new()),
        }
    }

    async fn instantiate(&self) -> wasmtime::Result<Worker> {
        let mut store = Store::new(&self.engine, PluginHostState::new());
        store.limiter(|state| &mut state.limits);
        let reentry = &self.legacy_sync_reentry;
        let plugin = match &self.plugin_pre {
            AnyPluginPre::V0_1(plugin_pre) => WorkerPlugin::V0_1(
                reentry
                    .scope_bootstrap(plugin_pre.instantiate_async(&mut store))
                    .await?,
            ),
            AnyPluginPre::V0_2(plugin_pre) => WorkerPlugin::V0_2(
                reentry
                    .scope_bootstrap(plugin_pre.instantiate_async(&mut store))
                    .await?,
            ),
        };
        store
            .run_concurrent(async |accessor| match &plugin {
                WorkerPlugin::V0_1(plugin) => {
                    reentry
                        .scope_bootstrap(plugin.call_init_plugin(accessor))
                        .await
                }
                WorkerPlugin::V0_2(plugin) => {
                    reentry
                        .scope_bootstrap(plugin.call_init_plugin(accessor))
                        .await
                }
            })
            .await??;
        Ok(Worker { store, plugin })
    }

    async fn run_job(&self, kind: u32, input: Vec<u8>) -> JobOutput {
        let idle = self
            .idle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .pop();
        let mut worker = match idle {
            Some(worker) => worker,
            None => self.instantiate().await?,
        };
        let reentry = &self.legacy_sync_reentry;
        let plugin = &worker.plugin;
        // A trap drops the worker with the `?`; the next job gets a fresh one.
        let output = worker
            .store
            .run_concurrent(async |accessor| match plugin {
                WorkerPlugin::V0_1(plugin) => {
                    reentry
                        .scope_bootstrap(plugin.call_run_job(accessor, kind, input))
                        .await
                }
                WorkerPlugin::V0_2(plugin) => {
                    reentry
                        .scope_bootstrap(plugin.call_run_job(accessor, kind, input))
                        .await
                }
            })
            .await??;
        let mut idle = self
            .idle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if idle.len() < MAX_IDLE_WORKERS {
            idle.push(worker);
        }
        Ok(output)
    }
}

impl JobRunner for JobWorkers {
    fn run(&self, kind: u32, input: Vec<u8>) -> Pin<Box<dyn Future<Output = JobOutput> + '_>> {
        Box::pin(self.run_job(kind, input))
    }
}
