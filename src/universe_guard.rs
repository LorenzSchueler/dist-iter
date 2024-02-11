use std::{any::Any, ops::Deref};

use mpi::{
    datatype::DynBufferMut,
    environment::Universe,
    point_to_point::Message,
    topology::SimpleCommunicator,
    traits::{Communicator, Destination},
    Tag,
};

use crate::traits::Function;

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

pub struct End {}

impl Function for End {
    fn execute(&self, msg: Message, _world: &SimpleCommunicator) -> bool {
        let mut g: [u8; 0] = [];
        let mut buf = DynBufferMut::new(&mut g);
        let _ = msg.matched_receive_into(&mut buf);
        true
    }

    fn receive(&self, _msg: Message) -> Box<dyn Any> {
        Box::new(())
    }
}

const END_TAG: Tag = 0;
