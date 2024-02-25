use std::ops::Deref;

use linkme::distributed_slice;
use mpi::{
    environment::Universe,
    point_to_point::Message,
    traits::{Communicator, Destination},
    Tag,
};

use crate::function_registry::{RegistryEntry, WorkerMode, FUNCTION_REGISTRY};

pub(crate) struct UniverseGuard {
    universe: Universe,
}

impl UniverseGuard {
    pub fn new(universe: Universe) -> Self {
        Self { universe }
    }
}

impl Drop for UniverseGuard {
    fn drop(&mut self) {
        let world = self.world();
        if world.rank() == 0 {
            for dest in 1..world.size() {
                world.process_at_rank(dest).send_with_tag(&0u8, END_TAG);
            }
        }
    }
}

impl Deref for UniverseGuard {
    type Target = Universe;

    fn deref(&self) -> &Self::Target {
        &self.universe
    }
}

fn execute(msg: Message) -> WorkerMode {
    msg.matched_receive::<u8>();
    WorkerMode::Terminate
}

#[distributed_slice(FUNCTION_REGISTRY)]
static END: RegistryEntry = (END_TAG, execute);

const END_TAG: Tag = 0;
