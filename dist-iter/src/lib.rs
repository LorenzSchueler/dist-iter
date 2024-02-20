use std::process::ExitCode;

pub use dist_iter_macros::main;
#[doc(hidden)]
pub use linkme;
#[doc(hidden)]
pub use mpi;
use mpi::{
    topology::SimpleCommunicator,
    traits::{Communicator, Source},
};

mod function_registry;
mod iter;
mod task;
mod uninit_buffer;
mod universe_guard;

pub use crate::iter::*;
use crate::universe_guard::UniverseGuard;
#[doc(hidden)]
pub use crate::{
    function_registry::{gen_tag, RegistryEntry, WorkerMode, FUNCTION_REGISTRY},
    task::*,
    uninit_buffer::UninitBuffer,
};

#[doc(hidden)]
pub fn main(master: fn()) -> ExitCode {
    function_registry::check_registry();

    let universe = UniverseGuard::new(mpi::initialize().unwrap());
    let world = universe.world();

    if world.size() < 2 {
        eprintln!("dist-iter needs at least 2 ranks");
        return ExitCode::FAILURE;
    }

    if world.rank() == 0 {
        master();
    } else {
        worker();
    }

    ExitCode::SUCCESS
}

fn worker() {
    let world = SimpleCommunicator::world();
    loop {
        let (msg, status) = world.any_process().matched_probe();

        let execute = function_registry::tag_to_execute(status.tag());
        let worker_mode = execute(msg, status, world.process_at_rank(0));
        if worker_mode.is_terminate() {
            break;
        }
    }
}
