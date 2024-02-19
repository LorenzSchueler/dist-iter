use std::marker::PhantomData;

use mpi::{topology::SimpleCommunicator, traits::Communicator};

use crate::{iter::dist_iterator::DistIterator, task::FilterTask, uninit_buffer::UninitBuffer};

#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct Filter<I, T, const N: usize>
where
    I: DistIterator<N>,
    T: FilterTask<N, Item = I::Item>,
{
    inner: I,
    task: PhantomData<T>,
    buf: UninitBuffer<T::Item, N>,
    send_count: usize,
    recv_count: usize,
    init: bool,
    world: SimpleCommunicator,
}

impl<I, T, const N: usize> Filter<I, T, N>
where
    I: DistIterator<N>,
    T: FilterTask<N, Item = I::Item>,
{
    pub(super) fn new(inner: I, _task: T) -> Self {
        Filter {
            inner,
            task: PhantomData,
            buf: UninitBuffer::new(),
            send_count: 0,
            recv_count: 0,
            init: false,
            world: SimpleCommunicator::world(),
        }
    }
}

impl<I, T, const N: usize> Iterator for Filter<I, T, N>
where
    I: DistIterator<N>,
    T: FilterTask<N, Item = I::Item>,
{
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.buf.pop_front() {
            return Some(item);
        }
        if !self.init {
            for dest in 1..self.world.size() {
                let process = self.world.process_at_rank(dest);
                if self.inner.send_next_to(process, T::TAG) {
                    self.send_count += 1;
                }
            }
        }
        while self.recv_count < self.send_count {
            let process = self.world.any_process();
            let rank = self.buf.receive_into_with_tag(process, T::TAG);
            self.recv_count += 1;
            eprintln!("< data of length {:?}", self.buf.init_count());

            let process = self.world.process_at_rank(rank);
            if self.inner.send_next_to(process, T::TAG) {
                self.send_count += 1;
            }
            if let Some(item) = self.buf.pop_front() {
                return Some(item);
            }
        }
        None
    }
}
