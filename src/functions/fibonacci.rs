use std::any::Any;

use mpi::{
    point_to_point::Message,
    topology::SimpleCommunicator,
    traits::{Communicator, Destination},
    Rank, Tag,
};

use crate::traits::{Function, Task};

pub struct Fibonacci {}

impl Function for Fibonacci {
    fn execute(&self, msg: Message, world: &SimpleCommunicator) -> bool {
        let (data, status) = msg.matched_receive();
        let result = fibonacci(data);
        world
            .process_at_rank(0)
            .send_with_tag(&result, status.tag());
        false
    }

    fn receive(&self, msg: Message) -> Box<dyn Any> {
        let (data, status) = msg.matched_receive::<u64>();
        println!("root got data {:?} from {}", data, status.source_rank());
        Box::new(data)
    }
}

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

pub const FIBONACCI: Fibonacci = Fibonacci {};

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
