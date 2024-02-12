use std::any::Any;
use std::fmt::Debug;

use mpi::{
    point_to_point::Message,
    topology::{Process, SimpleCommunicator},
    traits::Equivalence,
};

pub trait Task {
    fn send(&self, process: Process<'_, SimpleCommunicator>);
}

pub fn receive<T: 'static + Equivalence + Debug>(msg: Message) -> Box<dyn Any> {
    let (data, status) = msg.matched_receive::<T>();
    println!("root got data {:?} from {}", data, status.source_rank());
    Box::new(data)
}
