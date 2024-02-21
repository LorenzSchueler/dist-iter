use mpi::traits::Equivalence;

use crate::{iter::*, task::*};

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
