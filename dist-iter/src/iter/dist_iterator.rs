use mpi::{
    topology::{Process, SimpleCommunicator},
    traits::{Destination, Equivalence},
    Tag,
};

use crate::{iter::map::Map, task::Task};

pub trait DistIterator {
    type Item: Equivalence;

    fn send_next_to(&mut self, dest: Process<'_, SimpleCommunicator>, tag: Tag) -> bool;

    fn map<T>(self, task: T) -> Map<Self, T>
    where
        Self: Sized,
        T: Task<IN = Self::Item>,
    {
        Map::new(self, task)
    }

    // fn reduce();
}

pub trait IntoDistIterator {
    type Iter: DistIterator;

    fn into_dist_iter(self) -> Self::Iter;
}

impl<I> IntoDistIterator for I
where
    I: IntoIterator,
    I::Item: Equivalence,
{
    type Iter = IntoDistIter<I::IntoIter>;

    fn into_dist_iter(self) -> Self::Iter {
        IntoDistIter {
            inner: self.into_iter(),
        }
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
