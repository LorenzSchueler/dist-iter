use std::ops::Deref;

use mpi::{
    environment::Universe,
    traits::{Communicator, Destination},
};

use crate::END_TAG;

pub struct UniverseGuard {
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
            let size = world.size();
            let function = END_TAG;
            let data: [u8; 0] = [];
            mpi::request::scope(|scope| {
                let mut send_requests = vec![];
                println!("sending end");
                for dest in 1..size {
                    send_requests.push(
                        world
                            .process_at_rank(dest)
                            .immediate_send_with_tag(scope, &data, function),
                    );
                }
                for req in send_requests {
                    req.wait_without_status();
                }
                println!("done");
            });
        }
    }
}

impl Deref for UniverseGuard {
    type Target = Universe;

    fn deref(&self) -> &Self::Target {
        &self.universe
    }
}
