use std::any::Any;

use linkme::distributed_slice;
use mpi::{point_to_point::Message, topology::SimpleCommunicator, Tag};

#[distributed_slice]
pub static FUNCTIONS: [(
    Tag,
    fn(msg: Message, _world: &SimpleCommunicator) -> bool,
    fn(msg: Message) -> Box<dyn Any>,
)];

pub fn tag_to_execute(tag: Tag) -> fn(msg: Message, _world: &SimpleCommunicator) -> bool {
    FUNCTIONS
        .iter()
        .find(|(t, _, _)| *t == tag)
        .map(|(_, execute, _)| *execute)
        .unwrap()
}

pub fn tag_to_receive(tag: Tag) -> fn(msg: Message) -> Box<dyn Any> {
    FUNCTIONS
        .iter()
        .find(|(t, _, _)| *t == tag)
        .map(|(_, _, receive)| *receive)
        .unwrap()
}
