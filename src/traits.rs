use mpi::{topology::SimpleCommunicator, Rank};

pub trait Task {
    fn send(&self, world: &SimpleCommunicator, dest: Rank);
}
