use mpi::{
    point_to_point::Message,
    topology::{Process, SimpleCommunicator},
    traits::Equivalence,
    Tag,
};

pub trait Task {
    type IN;
    type OUT: Equivalence;

    const TAG: Tag;

    fn send(&self, process: Process<'_, SimpleCommunicator>);

    fn receive<T: 'static + Equivalence>(msg: Message) -> T {
        let (data, _) = msg.matched_receive::<T>();
        data
    }
}
