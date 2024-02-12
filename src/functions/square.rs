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

fn execute(msg: Message, process: Process<'_, SimpleCommunicator>) -> bool {
    let (data, status) = msg.matched_receive();
    let result = square(data);
    process.send_with_tag(&result, status.tag());
    false
}

#[distributed_slice(FUNCTIONS)]
pub static SQUARE: (
    Tag,
    fn(Message, Process<'_, SimpleCommunicator>) -> bool,
    fn(Message) -> Box<dyn Any>,
) = (SQUARE_TAG, execute, receive::<i32>);

pub struct SquareTask {
    data: i32,
}

impl SquareTask {
    pub fn new(data: i32) -> Self {
        Self { data }
    }
}

impl Task for SquareTask {
    fn send(&self, process: Process<'_, SimpleCommunicator>) {
        process.send_with_tag(&self.data, SQUARE_TAG);
    }
}

const SQUARE_TAG: Tag = 2;

fn square(n: i32) -> i32 {
    n * n
}
