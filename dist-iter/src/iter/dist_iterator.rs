use mpi::{
    topology::{Process, SimpleCommunicator},
    traits::{Destination, Equivalence},
    Tag,
};

use crate::{iter::*, task::*, UninitBuffer};

pub trait DistIterator<const N: usize>: IntoIterator
where
    Self::Item: Equivalence,
{
    fn dist_map<T>(self, task: T) -> MapChunk<Self::IntoIter, T, N>
    where
        Self: Sized,
        T: MapChunkTask<N, In = Self::Item>,
    {
        MapChunk::new(self.into_iter(), task)
    }

    fn dist_map_chunk<T>(self, task: T) -> MapChunk<Self::IntoIter, T, N>
    where
        Self: Sized,
        T: MapChunkTask<N, In = Self::Item>,
    {
        MapChunk::new(self.into_iter(), task)
    }

    fn dist_filter<T>(self, task: T) -> MapChunk<Self::IntoIter, T, N>
    where
        Self: Sized,
        T: MapChunkTask<N, In = Self::Item>,
    {
        MapChunk::new(self.into_iter(), task)
    }

    fn dist_reduce<T, F>(self, (task, f): (T, F)) -> Option<Self::Item>
    where
        Self: Sized,
        T: MapChunkTask<N, In = Self::Item, Out = Self::Item>,
        F: FnMut(Self::Item, Self::Item) -> Self::Item,
    {
        Reduce::new(self.into_iter(), task, f).value()
    }

    //fn all() -> bool;
    //fn any() -> bool;
    //fn collect<B>(self) -> B
    //where
    //B: FromIterator<Self::Item>,
    //Self: Sized;
}

impl<const N: usize, I> DistIterator<N> for I
where
    I: IntoIterator,
    I::Item: Equivalence,
{
}

#[doc(hidden)]
pub struct ChunkDistributor<Iter, const N: usize>
where
    Iter: Iterator,
    Iter::Item: Equivalence,
{
    iter: Iter,
    buf: UninitBuffer<Iter::Item, N>,
}

impl<I, const N: usize> ChunkDistributor<I, N>
where
    I: Iterator,
    I::Item: Equivalence,
{
    pub(super) fn new(iter: I) -> Self {
        Self {
            iter,
            buf: UninitBuffer::new(),
        }
    }

    pub(super) fn send_next_to(
        &mut self,
        process: Process<'_, SimpleCommunicator>,
        tag: Tag,
    ) -> bool {
        loop {
            if let Some(mut push_handle) = self.buf.push_handle() {
                if let Some(item) = self.iter.next() {
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
