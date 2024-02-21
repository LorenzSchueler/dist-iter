use mpi::traits::Equivalence;

use crate::{
    iter::{map_chunk::MapChunk, reduce::Reduce},
    task::*,
};

pub trait DistIterator: IntoIterator
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
        MapChunk::new(self.into_iter(), task.task)
    }

    fn dist_map<T, const IN: usize>(self, task: MapTask<T>) -> impl Iterator<Item = T::Out>
    where
        Self: Sized,
        T: Task<In = Self::Item, IN = { IN }, OUT = { IN }>,
    {
        MapChunk::new(self.into_iter(), task.task)
    }

    fn dist_filter<T, const IN: usize>(self, task: FilterTask<T>) -> impl Iterator<Item = T::Out>
    where
        Self: Sized,
        T: Task<In = Self::Item, IN = { IN }, OUT = { IN }>,
    {
        MapChunk::new(self.into_iter(), task.task)
    }

    fn dist_reduce<T, F, const IN: usize>(self, (task, f): (ReduceTask<T>, F)) -> Option<Self::Item>
    where
        Self: Sized,
        T: Task<In = Self::Item, Out = Self::Item, IN = { IN }, OUT = { 1 }>,
        F: FnMut(Self::Item, Self::Item) -> Self::Item,
    {
        //MapChunk::new(self.into_iter(), task).reduce(f)
        Reduce::new(self.into_iter(), task.task, f).value()
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
