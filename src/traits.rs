use std::any::Any;

use mpi::{point_to_point::Message, topology::SimpleCommunicator, Rank};

pub trait Function {
    fn execute(&self, msg: Message, world: &SimpleCommunicator) -> bool;

    fn receive(&self, msg: Message) -> Box<dyn Any>;
}

pub trait Task {
    fn send(&self, world: &SimpleCommunicator, dest: Rank);
}
