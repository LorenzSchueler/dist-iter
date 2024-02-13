mod dist_iter;
mod function_registry;
mod task;
mod universe_guard;

pub use dist_iter_macros::main;
#[doc(hidden)]
pub use linkme;
#[doc(hidden)]
pub use mpi;
use mpi::{
    topology::SimpleCommunicator,
    traits::{Communicator, Source},
};

pub use crate::dist_iter::IntoDistIter;
use crate::universe_guard::UniverseGuard;
#[doc(hidden)]
pub use crate::{
    function_registry::{RegistryEntry, FUNCTION_REGISTRY},
    task::Task,
};

pub fn main(master: fn()) {
    function_registry::check_registry();

    let universe = UniverseGuard::new(mpi::initialize().unwrap());

    if universe.world().rank() == 0 {
        master();
    } else {
        worker();
    }
}

fn worker() {
    let world = SimpleCommunicator::world();
    loop {
        let (msg, status) = world.any_process().matched_probe();

        let execute = function_registry::tag_to_execute(status.tag());
        let stop = execute(msg, world.process_at_rank(0));
        if stop {
            break;
        }
    }
}
