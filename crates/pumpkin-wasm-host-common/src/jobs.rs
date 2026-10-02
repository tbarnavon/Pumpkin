//! `scheduler.spawn-job`.
//!
//! A job runs the plugin's `run-job` export in a worker: an instance of the same component in
//! its own store, with no server, on a small thread pool off the tick. The output goes back to
//! the main instance through the task scheduler on a later tick. The workers themselves depend on
//! the plugin's API version, so the host gives each plugin a [`JobRunner`].

use std::future::Future;
use std::pin::Pin;
use std::sync::{
    Arc, OnceLock,
    atomic::{AtomicU64, Ordering},
};

use pumpkin_core::server::Server;

use crate::plugin::WasmPlugin;

static NEXT_JOB_ID: AtomicU64 = AtomicU64::new(0);

/// The output of one job, or why it failed.
pub type JobOutput = wasmtime::Result<Result<Vec<u8>, String>>;

/// Runs a plugin's `run-job` export on worker instances of its component.
pub trait JobRunner: Send + Sync {
    fn run(&self, kind: u32, input: Vec<u8>) -> Pin<Box<dyn Future<Output = JobOutput> + '_>>;
}

/// A finished job, waiting for the main instance's `handle-job-result`.
pub struct JobResult {
    pub plugin: Arc<WasmPlugin>,
    pub job_id: u64,
    pub output: Result<Vec<u8>, String>,
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
            plugin.task_scheduler.clone().push_job_result(JobResult {
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
