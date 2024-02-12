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
    let result = square(data);
    world
        .process_at_rank(0)
        .send_with_tag(&result, status.tag());
    false
}

#[distributed_slice(FUNCTIONS)]
pub static SQUARE: (
    Tag,
    fn(msg: Message, _world: &SimpleCommunicator) -> bool,
    fn(msg: Message) -> Box<dyn Any>,
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
    fn send(&self, world: &SimpleCommunicator, dest: Rank) {
        world
            .process_at_rank(dest)
            .send_with_tag(&self.data, SQUARE_TAG);
    }
}

const SQUARE_TAG: Tag = 2;

fn square(n: i32) -> i32 {
    n * n
}
