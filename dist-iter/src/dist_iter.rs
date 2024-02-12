use mpi::{
    topology::SimpleCommunicator,
    traits::{Communicator, Source},
};

use crate::task::Task;

pub struct DistIter<'w, I>
where
    I: Iterator,
    I::Item: Task,
{
    inner: I,
    init: bool,
    send_count: usize,
    recv_count: usize,
    world: &'w SimpleCommunicator,
}

impl<'w, I> Iterator for DistIter<'w, I>
where
    I: Iterator,
    I::Item: Task,
{
    type Item = <I::Item as Task>::OUT;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.init {
            for dest in 1..self.world.size() {
                if let Some(task) = self.inner.next() {
                    task.send(self.world.process_at_rank(dest));
                    self.send_count += 1;
                }
            }
        }
        if self.recv_count < self.send_count {
            let tag = <I::Item as Task>::TAG;
            let (data, status) = self.world.any_process().receive_with_tag(tag);
            self.recv_count += 1;

            if let Some(task) = self.inner.next() {
                task.send(self.world.process_at_rank(status.source_rank()));
                self.send_count += 1;
            }
            Some(data)
        } else {
            None
        }
    }
}

pub trait MyIterExt {
    fn into_dist_iter(self, world: &SimpleCommunicator) -> DistIter<Self>
    where
        Self: Iterator + Sized,
        Self::Item: Task;
}

impl<I> MyIterExt for I
where
    I: Iterator,
    I::Item: Task,
{
    fn into_dist_iter(self, world: &SimpleCommunicator) -> DistIter<Self> {
        DistIter {
            inner: self,
            init: false,
            send_count: 0,
            recv_count: 0,
            world,
        }
    }
}
