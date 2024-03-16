use std::marker::PhantomData;

use mpi::{
    topology::SimpleCommunicator,
    traits::{Communicator, Equivalence, Source},
};
use tracing::{error_span, trace};

use crate::{iter::chunk_distributor::ChunkDistributor, task::Task};

pub(super) struct ForEach<I, T, const IN: usize>
where
    I: Iterator,
    I::Item: Equivalence,
    T: Task<In = I::Item, Out = u8, IN = { IN }, OUT = { 0 }>,
{
    chunk_distributor: ChunkDistributor<I, IN>,
    task: PhantomData<T>,
}

impl<I, T, const IN: usize> ForEach<I, T, IN>
where
    I: Iterator,
    I::Item: Equivalence,
    T: Task<In = I::Item, Out = u8, IN = { IN }, OUT = { 0 }>,
{
    pub(super) fn new(iter: I, _task: T) -> Self {
        Self {
            chunk_distributor: ChunkDistributor::new(iter),
            task: PhantomData,
        }
    }

    pub(super) fn for_each(mut self) {
        let _span = error_span!("task", id = T::TAG).entered();

        let world = SimpleCommunicator::world();
        let mut send_count = 0;
        let mut recv_count = 0;

        for dest in 1..world.size() {
            let process = world.process_at_rank(dest);
            if self.chunk_distributor.send_next_to(process, T::TAG) {
                send_count += 1;
            }
        }
        trace!("init send complete");

        let mut buf: [T::Out; 0] = [];
        while recv_count < send_count {
            trace!("receiving 'for each finished' ...",);
            let status = world.any_process().receive_into_with_tag(&mut buf, T::TAG);
            recv_count += 1;
            trace!(
                "received 'for each finished' from worker {}",
                status.source_rank()
            );

            let process = world.process_at_rank(status.source_rank());
            if self.chunk_distributor.send_next_to(process, T::TAG) {
                send_count += 1;
            }
        }
    }
}
