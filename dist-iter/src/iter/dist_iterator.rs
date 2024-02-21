use mpi::traits::Equivalence;

use crate::{
    iter::{map_chunk::MapChunk, reduce::Reduce},
    task::*,
};

pub trait DistIterator: IntoIterator
where
    Self::Item: Equivalence,
{
    fn dist_map<T, const N: usize>(self, task: T) -> impl Iterator<Item = T::Out>
    where
        Self: Sized,
        T: MapChunkTask<In = Self::Item, N = { N }>,
    {
        MapChunk::new(self.into_iter(), task)
    }

    fn dist_map_chunk<T, const N: usize>(self, task: T) -> impl Iterator<Item = T::Out>
    where
        Self: Sized,
        T: MapChunkTask<In = Self::Item, N = { N }>,
    {
        MapChunk::new(self.into_iter(), task)
    }

    fn dist_filter<T, const N: usize>(self, task: T) -> impl Iterator<Item = T::Out>
    where
        Self: Sized,
        T: MapChunkTask<In = Self::Item, N = { N }>,
    {
        MapChunk::new(self.into_iter(), task)
    }

    fn dist_reduce<T, F, const N: usize>(self, (task, f): (T, F)) -> Option<Self::Item>
    where
        Self: Sized,
        T: MapChunkTask<In = Self::Item, Out = Self::Item, N = { N }>,
        F: FnMut(Self::Item, Self::Item) -> Self::Item,
    {
        //MapChunk::new(self.into_iter(), task).reduce(f)
        Reduce::new(self.into_iter(), task, f).value()
    }

    //fn all() -> bool;
    //fn any() -> bool;
    //fn collect<B>(self) -> B
    //where
    //B: FromIterator<Self::Item>,
    //Self: Sized;
}

impl<I> DistIterator for I
where
    I: IntoIterator,
    I::Item: Equivalence,
{
}
