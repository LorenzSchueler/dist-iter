use mpi::{topology::Process, traits::Communicator};

pub(crate) trait CommunicatorExt: Communicator {
    fn workers(&self) -> impl Iterator<Item = Process<Self>>
    where
        Self: Sized,
    {
        (1..self.size()).map(|id| self.process_at_rank(id))
    }
}

impl<C: Communicator> CommunicatorExt for C {}
