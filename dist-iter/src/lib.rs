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

#[doc(hidden)]
pub use crate::task::Task;
use crate::universe_guard::UniverseGuard;
pub use crate::{
    dist_iter::IntoDistIter,
    function_registry::{RegistryEntry, FUNCTION_REGISTRY},
};

pub fn main(master: fn(&SimpleCommunicator)) {
    function_registry::check_registry();

    let universe = UniverseGuard::new(mpi::initialize().unwrap());
    let world = universe.world();

    if world.rank() == 0 {
        master(&world);
    } else {
        worker(&world);
    }
}

fn worker(world: &SimpleCommunicator) {
    loop {
        let (msg, status) = world.any_process().matched_probe();

        let execute = function_registry::tag_to_execute(status.tag());
        let stop = execute(msg, world.process_at_rank(0));
        if stop {
            break;
        }
    }
}
