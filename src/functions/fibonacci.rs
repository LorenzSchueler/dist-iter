use std::any::Any;

use linkme::distributed_slice;
use mpi::{
    point_to_point::Message,
    topology::SimpleCommunicator,
    traits::{Communicator, Destination},
    Rank, Tag,
};

use crate::{
    dispatch::FUNCTIONS,
    traits::{receive, Task},
};

fn execute(msg: Message, world: &SimpleCommunicator) -> bool {
    let (data, status) = msg.matched_receive();
    let result = fibonacci(data);
    world
        .process_at_rank(0)
        .send_with_tag(&result, status.tag());
    false
}

#[distributed_slice(FUNCTIONS)]
pub static FIBONACCI: (
    Tag,
    fn(msg: Message, _world: &SimpleCommunicator) -> bool,
    fn(msg: Message) -> Box<dyn Any>,
) = (FIBONACCI_TAG, execute, receive::<u64>);

pub struct FibonacciTask {
    data: u64,
}

impl FibonacciTask {
    pub fn new(data: u64) -> Self {
        Self { data }
    }
}

impl Task for FibonacciTask {
    fn send(&self, world: &SimpleCommunicator, dest: Rank) {
        world
            .process_at_rank(dest)
            .send_with_tag(&self.data, FIBONACCI_TAG);
    }
}

const FIBONACCI_TAG: Tag = 1;

fn fibonacci(n: u64) -> u64 {
    if n == 0 {
        return 0;
    } else if n == 1 {
        return 1;
    } else {
        return fibonacci(n - 1) + fibonacci(n - 2);
    }
}
