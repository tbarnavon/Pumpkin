//! Worker instances for `scheduler.spawn-job`.
//!
//! A job runs the plugin's `run-job` export in a worker: an instance of the same component in
//! its own store, with no server, on a small thread pool off the tick. The output goes back to
//! the main instance through the task scheduler on a later tick. Workers are kept for reuse.

use std::sync::{
    Arc, Mutex, OnceLock,
    atomic::{AtomicU64, Ordering},
};

use pumpkin_host_bindings::PluginPre;
use wasmtime::{Engine, Store};

use super::{WasmPlugin, concurrent_store::LegacySyncReentry, state::PluginHostState};
use crate::server::Server;

/// Idle workers one plugin keeps; jobs beyond that make more, dropped when they finish.
const MAX_IDLE_WORKERS: usize = 2;

static NEXT_JOB_ID: AtomicU64 = AtomicU64::new(0);

struct Worker {
    store: Store<PluginHostState>,
    plugin: pumpkin_host_bindings::Plugin,
}

/// A finished job, waiting for the main instance's `handle-job-result`.
pub struct JobResult {
    pub plugin: Arc<WasmPlugin>,
    pub job_id: u64,
    pub output: Result<Vec<u8>, String>,
}

pub struct JobWorkers {
    engine: Engine,
    plugin_pre: PluginPre<PluginHostState>,
    legacy_sync_reentry: LegacySyncReentry,
    idle: Mutex<Vec<Worker>>,
}

/// Job threads, apart from Rayon's global pool so long jobs don't hold up chunk generation.
fn pool() -> Option<&'static rayon::ThreadPool> {
    static POOL: OnceLock<Option<rayon::ThreadPool>> = OnceLock::new();
    POOL.get_or_init(|| {
        let threads = std::thread::available_parallelism().map_or(1, |n| (n.get() / 4).max(1));
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .thread_name(|i| format!("Plugin-Job-{i}"))
            .build()
            .inspect_err(|error| tracing::error!(%error, "Failed to start the plugin job pool"))
            .ok()
    })
    .as_ref()
}

impl JobWorkers {
    pub(super) const fn new(
        engine: Engine,
        plugin_pre: PluginPre<PluginHostState>,
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
        let plugin = self
            .legacy_sync_reentry
            .scope_bootstrap(self.plugin_pre.instantiate_async(&mut store))
            .await?;
        let reentry = &self.legacy_sync_reentry;
        store
            .run_concurrent(async |accessor| {
                reentry
                    .scope_bootstrap(plugin.call_init_plugin(accessor))
                    .await
            })
            .await??;
        Ok(Worker { store, plugin })
    }

    async fn run(&self, kind: u32, input: Vec<u8>) -> wasmtime::Result<Result<Vec<u8>, String>> {
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
            .run_concurrent(async |accessor| {
                reentry
                    .scope_bootstrap(plugin.call_run_job(accessor, kind, input))
                    .await
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

impl WasmPlugin {
    /// `scheduler.spawn-job`: runs the job on a worker and queues its output for the task
    /// scheduler, which hands it to `handle-job-result`.
    pub fn spawn_job(self: &Arc<Self>, server: &Arc<Server>, kind: u32, input: Vec<u8>) -> u64 {
        let job_id = NEXT_JOB_ID.fetch_add(1, Ordering::Relaxed);
        let plugin = self.clone();
        let server = server.clone();
        let run = move || {
            let output = server
                .runtime
                .block_on(plugin.jobs.run(kind, input))
                .unwrap_or_else(|error| Err(format!("job failed: {error}")));
            server.task_scheduler.push_job_result(JobResult {
                plugin,
                job_id,
                output,
            });
        };
        match pool() {
            Some(pool) => pool.spawn(run),
            None => rayon::spawn(run),
        }
        job_id
    }
}
