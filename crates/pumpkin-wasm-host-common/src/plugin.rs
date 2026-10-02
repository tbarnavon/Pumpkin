use std::{any::Any, sync::Arc};

use arc_swap::ArcSwap;
use thiserror::Error;

use crate::{
    concurrent_store::LegacyStore, jobs::JobRunner, modded::GuestCalls, scheduler::TaskScheduler,
};

#[derive(Error, Debug)]
pub enum PluginInitError {
    #[error("Engine creation failed: {0}")]
    EngineCreationFailed(wasmtime::Error),
    #[error("Failed to setup linker: {0}")]
    LinkerSetupFailed(wasmtime::Error),
    #[error("Could not identify the plugin API version (does not export pumpkin:plugin/metadata?)")]
    UnknownApiVersion(),
    #[error("Plugin is built against an unsupported version of the API: {0}")]
    UnsupportedApiVersion(String),
    #[error("Plugin was built against a different iteration of the API: {0}")]
    ApiMismatch(wasmtime::Error),
    #[error("Failed to read plugin file: {0}")]
    FileReadFailed(std::io::Error),
    #[error("Failed to load plugin as component: {0}")]
    ComponentNewFailed(wasmtime::Error),
    #[error("Failed to create cache data for plugin: {0}")]
    ComponentCacheSerializeFailed(wasmtime::Error),
    #[error("Failed to write cache file for plugin: {0}")]
    ComponentCacheWriteFailed(std::io::Error),
    #[error("Failed to instantiate plugin: {0}")]
    InstantiationFailed(wasmtime::Error),
    #[error("Calling `init_plugin` failed: {0}")]
    CallInitPluginFailed(wasmtime::Error),
    #[error("Calling `get_metadata` failed: {0}")]
    CallGetMetadataFailed(wasmtime::Error),
    #[error("Failed to get absolute path: {0}")]
    PathResolutionFailed(std::io::Error),
    #[error("Failed to create cache: {0}")]
    CacheCreationFailed(wasmtime::Error),
}

#[derive(Copy, Clone)]
pub enum PluginApiVersion {
    V0_1,
    V0_2,
}

/// One instantiation of a plugin. Exported functions only work with the store they came from,
/// so callers take both from the same generation.
pub struct PluginGeneration {
    /// The guest exports of the plugin, as generated for its API version.
    pub plugin_instance: Arc<dyn Any + Send + Sync>,
    pub store: LegacyStore,
}

impl PluginGeneration {
    /// The guest exports of this generation. Each API version's host only ever sees plugins of
    /// its own version.
    ///
    /// # Panics
    /// Panics when `T` is not the type generated for this plugin's API version.
    #[must_use]
    pub fn instance<T: 'static>(&self) -> &T {
        self.plugin_instance
            .downcast_ref()
            .unwrap_or_else(|| panic!("Unexpected plugin version in host path."))
    }
}

impl Drop for PluginGeneration {
    fn drop(&mut self) {
        self.store.discard();
    }
}

pub struct WasmPlugin {
    generation: ArcSwap<PluginGeneration>,
    pub api_version: PluginApiVersion,
    pub task_scheduler: Arc<TaskScheduler>,
    /// The calls into the guest that depend on its API version, for the shared modded runtime.
    pub guest: &'static dyn GuestCalls,
    /// Worker instances for `scheduler.spawn-job`.
    pub jobs: Box<dyn JobRunner>,
}

impl WasmPlugin {
    #[must_use]
    pub fn new(
        generation: Arc<PluginGeneration>,
        api_version: PluginApiVersion,
        task_scheduler: Arc<TaskScheduler>,
        guest: &'static dyn GuestCalls,
        jobs: Box<dyn JobRunner>,
    ) -> Self {
        Self {
            generation: ArcSwap::from(generation),
            api_version,
            task_scheduler,
            guest,
            jobs,
        }
    }

    /// The live instance and store. A trap replaces them (see the host's restart).
    #[must_use]
    pub fn current(&self) -> Arc<PluginGeneration> {
        self.generation.load_full()
    }

    /// Swaps in a new generation after the old one trapped.
    pub fn replace_generation(&self, generation: Arc<PluginGeneration>) {
        self.generation.store(generation);
    }
}
