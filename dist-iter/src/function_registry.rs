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

/// Generate tag.
//
/// Uniqueness is not guaranteed at compile time but check at runtime.
pub const fn gen_tag(file: &str, line: u32, column: u32) -> Tag {
    let file_hash: [u8; 20] = const_sha1::sha1(file.as_bytes()).as_bytes();
    let tag: u32 = ((line << 16) + column)
        ^ (((file_hash[0] as u32) << 24)
            + ((file_hash[1] as u32) << 16)
            + ((file_hash[2] as u32) << 8)
            + (file_hash[3] as u32))
        ^ (((file_hash[4] as u32) << 24)
            + ((file_hash[5] as u32) << 16)
            + ((file_hash[6] as u32) << 8)
            + (file_hash[7] as u32))
        ^ (((file_hash[8] as u32) << 24)
            + ((file_hash[9] as u32) << 16)
            + ((file_hash[10] as u32) << 8)
            + (file_hash[11] as u32))
        ^ (((file_hash[12] as u32) << 24)
            + ((file_hash[13] as u32) << 16)
            + ((file_hash[14] as u32) << 8)
            + (file_hash[15] as u32));

    (tag as Tag).abs()
}
