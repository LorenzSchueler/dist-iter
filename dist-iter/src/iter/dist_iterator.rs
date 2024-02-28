use mpi::traits::Equivalence;

use crate::{
    iter::{
        map_chunk::{MapChunk, MapChunkCollect},
        reduce::Reduce,
    },
    task::*,
};

pub trait DistIterator: Iterator
where
    Self::Item: Equivalence,
{
    fn dist_map_chunk<T, const IN: usize, const OUT: usize>(
        self,
        task: MapChunkTask<T>,
    ) -> impl Iterator<Item = T::Out>
    where
        Self: Sized,
        T: Task<In = Self::Item, IN = { IN }, OUT = { OUT }>,
    {
        MapChunk::new(self, task.task)
    }

    fn dist_map_chunk_collect<T, const IN: usize, const OUT: usize>(
        self,
        task: MapChunkTask<T>,
    ) -> Vec<T::Out>
    where
        Self: Sized,
        T: Task<In = Self::Item, IN = { IN }, OUT = { OUT }>,
    {
        MapChunkCollect::new(self, task.task).collect()
    }

    fn dist_map<T, const IN: usize>(self, task: MapTask<T>) -> impl Iterator<Item = T::Out>
    where
        Self: Sized,
        T: Task<In = Self::Item, IN = { IN }, OUT = { IN }>,
    {
        MapChunk::new(self, task.task)
    }

    fn dist_map_collect<T, const IN: usize>(self, task: MapTask<T>) -> Vec<T::Out>
    where
        Self: Sized,
        T: Task<In = Self::Item, IN = { IN }, OUT = { IN }>,
    {
        MapChunkCollect::new(self, task.task).collect()
    }

    fn dist_filter<T, const IN: usize>(self, task: FilterTask<T>) -> impl Iterator<Item = T::Out>
    where
        Self: Sized,
        T: Task<In = Self::Item, IN = { IN }, OUT = { IN }>,
    {
        MapChunk::new(self, task.task)
    }

    fn dist_filter_collect<T, const IN: usize>(self, task: FilterTask<T>) -> Vec<T::Out>
    where
        Self: Sized,
        T: Task<In = Self::Item, IN = { IN }, OUT = { IN }>,
    {
        MapChunkCollect::new(self, task.task).collect()
    }

    fn dist_reduce<T, F, const IN: usize>(self, (task, f): (ReduceTask<T>, F)) -> Option<Self::Item>
    where
        Self: Sized,
        T: Task<In = Self::Item, Out = Self::Item, IN = { IN }, OUT = { 1 }>,
        F: FnMut(Self::Item, Self::Item) -> Self::Item,
    {
        //MapChunk::new(self, task).reduce(f)
        Reduce::new(self, task.task, f).value()
    }

    // dist_find
    // dist_for_each
}

impl<I> DistIterator for I
where
    I: Iterator,
    I::Item: Equivalence,
{
}
