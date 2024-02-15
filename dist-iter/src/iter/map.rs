use std::marker::PhantomData;

use mpi::{
    topology::SimpleCommunicator,
    traits::{Communicator, Source},
};

use crate::{iter::dist_iterator::DistIterator, Task};

#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct Map<I: DistIterator, T: Task> {
    inner: I,
    task: PhantomData<T>,
    send_count: usize,
    recv_count: usize,
    init: bool,
    world: SimpleCommunicator,
}

impl<I, T> Map<I, T>
where
    I: DistIterator,
    T: Task<IN = I::Item>,
{
    pub fn new(inner: I, _task: T) -> Self {
        Map {
            inner,
            task: PhantomData,
            send_count: 0,
            recv_count: 0,
            init: false,
            world: SimpleCommunicator::world(),
        }
    }
}

impl<I, T> Iterator for Map<I, T>
where
    I: DistIterator,
    T: Task,
{
    type Item = T::OUT;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.init {
            for dest in 1..self.world.size() {
                let process = self.world.process_at_rank(dest);
                if self.inner.send_next_to(process, T::TAG) {
                    self.send_count += 1;
                }
            }
        }
        if self.recv_count < self.send_count {
            let (data, status) = self.world.any_process().receive_with_tag(T::TAG);
            self.recv_count += 1;

            let process = self.world.process_at_rank(status.source_rank());
            if self.inner.send_next_to(process, T::TAG) {
                self.send_count += 1;
            }

            Some(data)
        } else {
            None
        }
    }
}
