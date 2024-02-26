use dist_iter::{mpi::point_to_point::Message, RegistryEntry, WorkerMode, FUNCTION_REGISTRY};

#[test]
#[should_panic(expected = "tags are not unique")]
#[dist_iter::test]
fn registry_tag_uniqueness() {
    fn execute(_msg: Message) -> WorkerMode {
        WorkerMode::Continue
    }

    #[linkme::distributed_slice(FUNCTION_REGISTRY)]
    static REGISTRY_ENTRY1: RegistryEntry = (0, execute);

    #[linkme::distributed_slice(FUNCTION_REGISTRY)]
    static REGISTRY_ENTRY2: RegistryEntry = (0, execute);
}
