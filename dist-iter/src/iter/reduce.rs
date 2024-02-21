use std::marker::PhantomData;

use mpi::{
    topology::SimpleCommunicator,
    traits::{Communicator, Equivalence, Source},
};

use crate::{iter::chunk_distributor::ChunkDistributor, task::Task};

#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub(super) struct Reduce<I, T, F, const IN: usize>
where
    I: Iterator,
    I::Item: Equivalence,
    T: Task<In = I::Item, Out = I::Item, IN = { IN }, OUT = { 1 }>,
    F: FnMut(I::Item, I::Item) -> I::Item,
{
    chunk_distributor: ChunkDistributor<I, IN>,
    task: PhantomData<T>,
    f: F,
}

impl<I, T, F, const IN: usize> Reduce<I, T, F, IN>
where
    I: Iterator,
    I::Item: Equivalence,
    T: Task<In = I::Item, Out = I::Item, IN = { IN }, OUT = { 1 }>,
    F: FnMut(I::Item, I::Item) -> I::Item,
{
    pub(super) fn new(iter: I, _task: T, f: F) -> Self {
        Reduce {
            chunk_distributor: ChunkDistributor::new(iter),
            task: PhantomData,
            f,
        }
    }

    pub(super) fn value(mut self) -> Option<I::Item> {
        let world = SimpleCommunicator::world();
        let mut send_count = 0;
        let mut recv_count = 0;

        for dest in 1..world.size() {
            let process = world.process_at_rank(dest);
            if self.chunk_distributor.send_next_to(process, T::TAG) {
                send_count += 1;
            }
        }

        if send_count > 0 {
            let (result, status) = world.any_process().receive_with_tag(T::TAG);
            recv_count += 1;
            eprintln!("< reduce result");

            let process = world.process_at_rank(status.source_rank());
            if self.chunk_distributor.send_next_to(process, T::TAG) {
                send_count += 1;
            }
            let mut acc = result;

            while recv_count < send_count {
                let (result, status) = world.any_process().receive_with_tag(T::TAG);
                recv_count += 1;
                eprintln!("< reduce result");

                let process = world.process_at_rank(status.source_rank());
                if self.chunk_distributor.send_next_to(process, T::TAG) {
                    send_count += 1;
                }
                acc = (self.f)(acc, result);
            }

            Some(acc)
        } else {
            None
        }
    }
}
