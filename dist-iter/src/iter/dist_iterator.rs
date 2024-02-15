use mpi::{
    topology::{Process, SimpleCommunicator},
    traits::{Destination, Equivalence},
    Tag,
};

use crate::{iter::map::Map, task::Task};

pub trait DistIterator<const N: usize> {
    type Item: Equivalence;

    fn send_next_to(&mut self, dest: Process<'_, SimpleCommunicator>, tag: Tag) -> bool;

    fn map<T>(self, task: T) -> Map<N, Self, T>
    where
        Self: Sized,
        T: Task<IN = Self::Item>,
    {
        Map::new(self, task)
    }

    // fn reduce();
}

pub trait IntoDistIterator {
    type Iter<const N: usize>: DistIterator<N>;

    fn into_dist_iter<const N: usize>(self) -> Self::Iter<N>;
}

impl<I> IntoDistIterator for I
where
    I: IntoIterator,
    I::Item: Equivalence,
{
    type Iter<const N: usize> = IntoDistIter<I::IntoIter, N>;

    fn into_dist_iter<const N: usize>(self) -> Self::Iter<N> {
        IntoDistIter {
            inner: self.into_iter(),
            buf: Vec::new(),
        }
    }
}

pub struct IntoDistIter<Iter, const N: usize>
where
    Iter: Iterator,
    Iter::Item: Equivalence,
{
    inner: Iter,
    buf: Vec<Iter::Item>, // TODO use array
}

impl<Iter, const N: usize> DistIterator<N> for IntoDistIter<Iter, N>
where
    Iter: Iterator,
    Iter::Item: Equivalence,
{
    type Item = Iter::Item;

    fn send_next_to(&mut self, process: Process<'_, SimpleCommunicator>, tag: Tag) -> bool {
        loop {
            if self.buf.len() < N {
                if let Some(item) = self.inner.next() {
                    self.buf.push(item);
                    continue;
                }
            }
            break;
        }
        if self.buf.len() > 0 {
            eprintln!("sending vec of length {:?}", self.buf.len());
            process.send_with_tag(&self.buf, tag);
            self.buf.clear();
            true
        } else {
            false
        }
    }
}
