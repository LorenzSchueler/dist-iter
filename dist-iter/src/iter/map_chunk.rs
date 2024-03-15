use std::marker::PhantomData;

use mpi::{
    datatype::MutView,
    topology::SimpleCommunicator,
    traits::{Communicator, Equivalence, Source},
    Count,
};
use tracing::{error_span, trace};

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
        Self {
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
        let _span = error_span!("task", id = T::TAG).entered();

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
            trace!("init send complete");
        }
        while self.recv_count < self.send_count {
            let process = self.world.any_process();
            let rank = self.buf.receive_into_with_tag(process, T::TAG);
            self.recv_count += 1;
            trace!(
                "received chunk of length {} from worker {}",
                self.buf.len(),
                rank
            );

            let process = self.world.process_at_rank(rank);
            if self.chunk_distributor.send_next_to(process, T::TAG) {
                self.send_count += 1;
            }
            // if chunk was empty, receive next one until a non empty one is received or recv_count == send_count
            if let Some(item) = self.buf.next() {
                return Some(item);
            }
        }
        None
    }
}

pub(super) struct MapChunkCollect<I, T, const IN: usize, const OUT: usize>
where
    I: Iterator,
    I::Item: Equivalence,
    T: Task<In = I::Item, IN = { IN }, OUT = { OUT }>,
{
    chunk_distributor: ChunkDistributor<I, IN>,
    task: PhantomData<T>,
}

impl<I, T, const IN: usize, const OUT: usize> MapChunkCollect<I, T, IN, OUT>
where
    I: Iterator,
    I::Item: Equivalence,
    T: Task<In = I::Item, IN = { IN }, OUT = { OUT }>,
{
    pub(super) fn new(iter: I, _task: T) -> Self {
        Self {
            chunk_distributor: ChunkDistributor::new(iter),
            task: PhantomData,
        }
    }

    pub(super) fn collect(mut self) -> Vec<T::Out> {
        let world = SimpleCommunicator::world();
        let mut send_count = 0;
        let mut recv_count = 0;
        let mut vec = Vec::new();

        for dest in 1..world.size() {
            let process = world.process_at_rank(dest);
            if self.chunk_distributor.send_next_to(process, T::TAG) {
                send_count += 1;
            }
        }
        while recv_count < send_count {
            let datatype = T::Out::equivalent_datatype();

            vec.reserve(T::OUT);
            let len = vec.len();
            let vec_start: *mut T::Out = vec.as_mut_ptr();
            // SAFETY: vec has capacity for at least T::OUT more elements. buf is only used to let mpi write into it. the length is set after the recv_len is known.
            let mut buf = unsafe {
                let vec_end = &mut *(vec_start).add(len);
                MutView::with_count_and_datatype(vec_end, T::OUT as Count, &datatype)
            };

            let process = world.any_process();
            let status = process.receive_into_with_tag(&mut buf, T::TAG);
            let rank = status.source_rank();
            let recv_len = status.count(datatype) as usize;
            // SAFETY: recv_len additional elements have been written at the end of the vector (within its reserved capacity)
            unsafe { vec.set_len(len + recv_len) };
            recv_count += 1;
            trace!("received chunk of length {}", recv_len);

            let process = world.process_at_rank(rank);
            if self.chunk_distributor.send_next_to(process, T::TAG) {
                send_count += 1;
            }
        }

        vec
    }
}
