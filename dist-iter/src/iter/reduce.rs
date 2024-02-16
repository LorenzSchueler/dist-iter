use std::marker::PhantomData;

use mpi::{
    topology::SimpleCommunicator,
    traits::{Communicator, Source},
};

use crate::{iter::dist_iterator::DistIterator, task::ReduceTask};

#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct Reduce<I, T, F, const N: usize>
where
    I: DistIterator<N>,
    T: ReduceTask<N, Item = I::Item>,
    F: FnMut(I::Item, I::Item) -> I::Item,
{
    inner: I,
    task: PhantomData<T>,
    f: F,
}

impl<I, T, F, const N: usize> Reduce<I, T, F, N>
where
    I: DistIterator<N>,
    T: ReduceTask<N, Item = I::Item>,
    F: FnMut(I::Item, I::Item) -> I::Item,
{
    pub(super) fn new(inner: I, _task: T, f: F) -> Self {
        Reduce {
            inner,
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
            if self.inner.send_next_to(process, T::TAG) {
                send_count += 1;
            }
        }

        if send_count > 0 {
            let (result, status) = world.any_process().receive_with_tag(T::TAG);
            recv_count += 1;
            eprintln!("< reduce result");

            let process = world.process_at_rank(status.source_rank());
            if self.inner.send_next_to(process, T::TAG) {
                send_count += 1;
            }
            let mut acc = result;

            while recv_count < send_count {
                let (result, status) = world.any_process().receive_with_tag(T::TAG);
                recv_count += 1;
                eprintln!("< reduce result");

                let process = world.process_at_rank(status.source_rank());
                if self.inner.send_next_to(process, T::TAG) {
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
