use mpi::{
    topology::SimpleCommunicator,
    traits::{Communicator, Equivalence},
};

use crate::{iter::chunk_distributor::ChunkDistributor, task::Task, uninit_buffer::UninitBuffer};

#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub(super) struct MapChunk<I, T, const IN: usize, const OUT: usize>
where
    I: Iterator,
    I::Item: Equivalence,
    T: Task<In = I::Item, IN = { IN }, OUT = { OUT }>,
{
    chunk_distributor: ChunkDistributor<I, IN>,
    buf: UninitBuffer<T::Out, OUT>,
    send_count: usize,
    recv_count: usize,
    init: bool,
    world: SimpleCommunicator,
}

impl<I, T, const IN: usize, const OUT: usize> MapChunk<I, T, IN, OUT>
where
    I: Iterator,
    I::Item: Equivalence,
    T: Task<In = I::Item, IN = { IN }, OUT = { OUT }>,
{
    pub(super) fn new(iter: I, _task: T) -> Self {
        MapChunk {
            chunk_distributor: ChunkDistributor::new(iter),
            buf: UninitBuffer::new(),
            send_count: 0,
            recv_count: 0,
            init: false,
            world: SimpleCommunicator::world(),
        }
    }
}

impl<I, T, const IN: usize, const OUT: usize> Iterator for MapChunk<I, T, IN, OUT>
where
    I: Iterator,
    I::Item: Equivalence,
    T: Task<In = I::Item, IN = { IN }, OUT = { OUT }>,
{
    type Item = T::Out;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.buf.next() {
            return Some(item);
        }
        if !self.init {
            for dest in 1..self.world.size() {
                let process = self.world.process_at_rank(dest);
                if self.chunk_distributor.send_next_to(process, T::TAG) {
                    self.send_count += 1;
                }
            }
        }
        while self.recv_count < self.send_count {
            let process = self.world.any_process();
            let rank = self.buf.receive_into_with_tag(process, T::TAG);
            self.recv_count += 1;
            eprintln!("< data of length {:?}", self.buf.len());

            let process = self.world.process_at_rank(rank);
            if self.chunk_distributor.send_next_to(process, T::TAG) {
                self.send_count += 1;
            }
            // if chunk was empty receive next one til we get a non empty one or recv_count == send_count
            if let Some(item) = self.buf.next() {
                return Some(item);
            }
        }
        self.buf.next()
    }
}
