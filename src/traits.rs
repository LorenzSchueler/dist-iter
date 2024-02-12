use std::any::Any;
use std::fmt::Debug;

use mpi::{point_to_point::Message, topology::SimpleCommunicator, traits::Equivalence, Rank};

pub trait Task {
    fn send(&self, world: &SimpleCommunicator, dest: Rank);
}

pub fn receive<T: 'static + Equivalence + Debug>(msg: Message) -> Box<dyn Any> {
    let (data, status) = msg.matched_receive::<T>();
    println!("root got data {:?} from {}", data, status.source_rank());
    Box::new(data)
}
