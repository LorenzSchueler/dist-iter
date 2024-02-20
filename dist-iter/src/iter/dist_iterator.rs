use mpi::{
    topology::{Process, SimpleCommunicator},
    traits::{Destination, Equivalence},
    Tag,
};

use crate::{iter::*, task::*, uninit_buffer::UninitBuffer};

pub trait DistIterator<const N: usize> {
    type Item: Equivalence;

    fn send_next_to(&mut self, dest: Process<'_, SimpleCommunicator>, tag: Tag) -> bool;

    fn map<T>(self, task: T) -> MapChunk<Self, T, N>
    where
        Self: Sized,
        T: MapChunkTask<N, In = Self::Item>,
    {
        MapChunk::new(self, task)
    }

    fn dist_map_chunk<T>(self, task: T) -> MapChunk<Self, T, N>
    where
        Self: Sized,
        T: MapChunkTask<N, In = Self::Item>,
    {
        MapChunk::new(self, task)
    }

    fn filter<T>(self, task: T) -> MapChunk<Self, T, N>
    where
        Self: Sized,
        T: MapChunkTask<N, In = Self::Item>,
    {
        MapChunk::new(self, task)
    }

    fn reduce<T, F>(self, (task, f): (T, F)) -> Option<Self::Item>
    where
        Self: Sized,
        T: MapChunkTask<N, In = Self::Item, Out = Self::Item>,
        F: FnMut(Self::Item, Self::Item) -> Self::Item,
    {
        Reduce::new(self, task, f).value()
    }

    //fn all() -> bool;
    //fn any() -> bool;
    //fn collect<B>(self) -> B
    //where
    //B: FromIterator<Self::Item>,
    //Self: Sized;
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
            buf: UninitBuffer::new(),
        }
    }
}

pub struct IntoDistIter<Iter, const N: usize>
where
    Iter: Iterator,
    Iter::Item: Equivalence,
{
    inner: Iter,
    buf: UninitBuffer<Iter::Item, N>,
}

impl<Iter, const N: usize> DistIterator<N> for IntoDistIter<Iter, N>
where
    Iter: Iterator,
    Iter::Item: Equivalence,
{
    type Item = Iter::Item;

    fn send_next_to(&mut self, process: Process<'_, SimpleCommunicator>, tag: Tag) -> bool {
        loop {
            if let Some(mut push_handle) = self.buf.push_handle() {
                if let Some(item) = self.inner.next() {
                    push_handle.push_back(item);
                    continue;
                }
            }
            break;
        }
        #[allow(unstable_name_collisions)]
        if !self.buf.is_empty() {
            eprintln!("> data of length {:?}", self.buf.len());
            process.send_with_tag(&*self.buf, tag);
            self.buf.clear();
            true
        } else {
            false
        }
    }
}
