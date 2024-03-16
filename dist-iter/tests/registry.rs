use dist_iter::{
    mpi::point_to_point::Message, RegistryEntry, TaskId, WorkerMode, FUNCTION_REGISTRY,
};

#[test]
#[should_panic(expected = "task ids are not unique")]
#[dist_iter::test]
fn registry_tag_uniqueness() {
    fn execute(_msg: Message) -> WorkerMode {
        WorkerMode::Continue
    }

    #[linkme::distributed_slice(FUNCTION_REGISTRY)]
    static REGISTRY_ENTRY1: RegistryEntry = RegistryEntry::new(TaskId::new(0), execute);

    #[linkme::distributed_slice(FUNCTION_REGISTRY)]
    static REGISTRY_ENTRY2: RegistryEntry = RegistryEntry::new(TaskId::new(0), execute);
}
