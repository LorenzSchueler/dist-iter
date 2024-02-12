use mpi::{
    point_to_point::Message,
    topology::{Process, SimpleCommunicator},
    traits::{Destination, Equivalence},
    Tag,
};

pub trait Task {
    type IN: Equivalence;
    type OUT: Equivalence;

    const TAG: Tag;

    fn get_data(&self) -> &Self::IN;

    fn send(&self, process: Process<'_, SimpleCommunicator>) {
        process.send_with_tag(self.get_data(), Self::TAG);
    }

    fn receive<T: 'static + Equivalence>(msg: Message) -> T {
        let (data, _) = msg.matched_receive::<T>();
        data
    }
}
