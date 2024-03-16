#![feature(associated_const_equality)]
#![feature(maybe_uninit_slice)]
#![feature(maybe_uninit_uninit_array)]
#![feature(lazy_cell)]

use std::process::ExitCode;

pub use dist_iter_macros::{main, test};
#[doc(hidden)]
pub use linkme;
#[doc(hidden)]
pub use mpi;
use mpi::{
    topology::SimpleCommunicator,
    traits::{Communicator, Source},
    Rank,
};
#[doc(hidden)]
pub use tracing;
use tracing::{error_span, trace};

mod function_registry;
mod iter;
mod task;
mod uninit_buffer;
pub mod universe_guard;
mod utils;

pub use crate::iter::DistIterator;
use crate::{function_registry::TaskInstanceId, universe_guard::UniverseGuard};
#[doc(hidden)]
pub use crate::{
    function_registry::{gen_task_id, RegistryEntry, TaskId, WorkerMode, FUNCTION_REGISTRY},
    task::*,
    uninit_buffer::UninitBuffer,
};

#[doc(hidden)]
pub const MASTER: Rank = 0;

#[doc(hidden)]
pub fn main(master: fn()) -> ExitCode {
    function_registry::check_registry();

    let universe = UniverseGuard::new(mpi::initialize().unwrap());
    let world = universe.world();

    if world.size() < 2 {
        eprintln!("dist-iter needs at least 2 ranks");
        return ExitCode::FAILURE;
    }

    if world.rank() == MASTER {
        let _span = error_span!("master").entered();
        master();
    } else {
        let _span = error_span!("worker", id = world.rank()).entered();
        worker();
    }

    ExitCode::SUCCESS
}

fn worker() {
    let world = SimpleCommunicator::world();
    loop {
        trace!(target: "dist_iter::worker_loop", "waiting for task ...");
        let (msg, status) = world.process_at_rank(MASTER).matched_probe();
        trace!(target: "dist_iter::worker_loop", "task available");

        let task_instance_id = TaskInstanceId::new(status.tag());
        let _span = error_span!("task", id = %task_instance_id).entered();
        trace!(target: "dist_iter::worker_loop", "processing task ...");
        let execute = function_registry::task_instance_id_to_function(task_instance_id);
        let worker_mode = execute(msg);
        trace!(target: "dist_iter::worker_loop", "finished task");

        if worker_mode.is_terminate() {
            trace!(target: "dist_iter::worker_loop", "shutting down ...");
            break;
        }
    }
}
