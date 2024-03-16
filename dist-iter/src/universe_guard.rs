use std::ops::Deref;

use mpi::{
    environment::Universe,
    traits::{Communicator, Destination},
};
use tracing::{error_span, trace};

use crate::{function_registry::SHUTDOWN_TASK_ID, utils::CommunicatorExt, MASTER};

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
        if world.rank() == MASTER {
            let _span = error_span!("master").entered();
            let buf: [u8; 0] = [];
            for process in world.workers() {
                trace!("sending shutdown message to worker {}", process.rank());
                process.send_with_tag(&buf, *SHUTDOWN_TASK_ID);
                trace!("shutdown message sent to worker {}", process.rank());
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
