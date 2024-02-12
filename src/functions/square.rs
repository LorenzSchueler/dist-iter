use std::any::Any;

use mpi::{
    point_to_point::Message,
    topology::SimpleCommunicator,
    traits::{Communicator, Destination},
    Rank, Tag,
};

use crate::traits::{Function, Task};

pub struct Square {}

impl Function for Square {
    fn execute(&self, msg: Message, world: &SimpleCommunicator) -> bool {
        let (data, status) = msg.matched_receive();
        let result = square(data);
        world
            .process_at_rank(0)
            .send_with_tag(&result, status.tag());
        false
    }

    fn receive(&self, msg: Message) -> Box<dyn Any> {
        let (data, status) = msg.matched_receive::<i32>();
        println!("root got data {:?} from {}", data, status.source_rank());
        Box::new(data)
    }
}

pub struct SquareTask {
    data: i32,
}

impl SquareTask {
    pub fn new(data: i32) -> Self {
        Self { data }
    }
}

impl Task for SquareTask {
    fn send(&self, world: &SimpleCommunicator, dest: Rank) {
        world
            .process_at_rank(dest)
            .send_with_tag(&self.data, SQUARE_TAG);
    }
}

pub const SQUARE: Square = Square {};

const SQUARE_TAG: Tag = 2;

fn square(n: i32) -> i32 {
    n * n
}
