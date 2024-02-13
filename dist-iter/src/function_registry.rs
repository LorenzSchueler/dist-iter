use linkme::distributed_slice;
use mpi::{
    point_to_point::Message,
    topology::{Process, SimpleCommunicator},
    Tag,
};

#[doc(hidden)]
pub type RegistryEntry = (
    Tag,
    fn(Message, mpi::topology::Process<'_, SimpleCommunicator>) -> bool,
);

#[doc(hidden)]
#[distributed_slice]
pub static FUNCTION_REGISTRY: [RegistryEntry];

#[doc(hidden)]
pub fn tag_to_execute(tag: Tag) -> fn(Message, Process<'_, SimpleCommunicator>) -> bool {
    FUNCTION_REGISTRY
        .iter()
        .find(|(t, _)| *t == tag)
        .map(|(_, execute)| *execute)
        .unwrap()
}

pub(crate) fn check_registry() {
    let mut sorted_tags = FUNCTION_REGISTRY
        .iter()
        .map(|(tag, _)| *tag)
        .collect::<Vec<_>>();
    sorted_tags.sort();
    for i in 0..(sorted_tags.len() - 1) {
        if sorted_tags[i] == sorted_tags[i + 1] {
            panic!(
                "tags are not unique: tag {} exists at least twice",
                sorted_tags[i]
            );
        }
    }
}
