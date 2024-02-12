use linkme::distributed_slice;
use mpi::{point_to_point::Message, topology::Process, topology::SimpleCommunicator, Tag};

#[distributed_slice]
pub static FUNCTION_REGISTRY: [(
    Tag,
    fn(Message, mpi::topology::Process<'_, SimpleCommunicator>) -> bool,
)];

pub fn tag_to_execute(tag: Tag) -> fn(Message, Process<'_, SimpleCommunicator>) -> bool {
    FUNCTION_REGISTRY
        .iter()
        .find(|(t, _)| *t == tag)
        .map(|(_, execute)| *execute)
        .unwrap()
}
