use std::any::Any;

use linkme::distributed_slice;
use mpi::{
    point_to_point::Message,
    topology::{Process, SimpleCommunicator},
    traits::Destination,
    Tag,
};

use crate::{
    dispatch::FUNCTIONS,
    traits::{receive, Task},
};

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

fn execute(msg: Message, process: Process<'_, SimpleCommunicator>) -> bool {
    let (data, status) = msg.matched_receive();
    let result = fibonacci(data);
    process.send_with_tag(&result, status.tag());
    false
}

#[distributed_slice(FUNCTIONS)]
static FIBONACCI: (
    Tag,
    fn(Message, Process<'_, SimpleCommunicator>) -> bool,
    fn(Message) -> Box<dyn Any>,
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
    fn send(&self, process: Process<'_, SimpleCommunicator>) {
        process.send_with_tag(&self.data, FIBONACCI_TAG);
    }
}
