use std::{marker::PhantomData, usize};

use mpi::{
    topology::{Process, SimpleCommunicator},
    traits::{Communicator, Destination, Equivalence, Source},
    Tag,
};

use crate::task::Task;

pub trait DistIterator {
    type Item: Equivalence;

    fn send_next_to(&mut self, dest: Process<'_, SimpleCommunicator>, tag: Tag) -> bool;

    fn map<T>(self, _task: T) -> Map<Self, T>
    where
        Self: Sized,
        T: Task<IN = Self::Item>,
    {
        Map {
            inner: self,
            task: PhantomData,
            send_count: 0,
            recv_count: 0,
            init: false,
            world: SimpleCommunicator::world(),
        }
    }

    // fn reduce();
}

pub trait IntoDistIterator {
    type Iter: DistIterator;

    fn into_dist_iter(self) -> Self::Iter;
}

impl<T> IntoDistIterator for T
where
    T: Iterator,
    T::Item: Equivalence,
{
    type Iter = IntoDistIter<T>;

    fn into_dist_iter(self) -> Self::Iter {
        IntoDistIter { inner: self }
    }
}

pub struct IntoDistIter<Iter>
where
    Iter: Iterator,
    Iter::Item: Equivalence,
{
    inner: Iter,
}

impl<Iter> DistIterator for IntoDistIter<Iter>
where
    Iter: Iterator,
    Iter::Item: Equivalence,
{
    type Item = Iter::Item;

    fn send_next_to(&mut self, process: Process<'_, SimpleCommunicator>, tag: Tag) -> bool {
        if let Some(item) = self.inner.next() {
            process.send_with_tag(&item, tag);
            true
        } else {
            false
        }
    }
}

#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct Map<I: DistIterator, T: Task> {
    inner: I,
    task: PhantomData<T>,
    send_count: usize,
    recv_count: usize,
    init: bool,
    world: SimpleCommunicator,
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
