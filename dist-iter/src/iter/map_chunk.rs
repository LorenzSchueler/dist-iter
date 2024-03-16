use std::marker::PhantomData;

use mpi::{
    datatype::MutView,
    topology::SimpleCommunicator,
    traits::{Communicator, Destination, Equivalence, Source},
    Count,
};
use tracing::{error_span, trace};

use crate::{
    function_registry::{register_new_task, TaskInstanceMapping, REGISTER_TASK_ID},
    iter::chunk_distributor::ChunkDistributor,
    task::Task,
    uninit_buffer::UninitBuffer,
    TaskInstanceId,
};

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
    task_instance_id: TaskInstanceId,
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
            task_instance_id: register_new_task(T::ID),
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
        let _span = error_span!("task", id = %self.task_instance_id).entered();

        if let Some(item) = self.buf.next() {
            return Some(item);
        }
        if !self.init {
            self.init = true;

            let task_instance_mapping = TaskInstanceMapping::new(T::ID, self.task_instance_id);
            for dest in 1..self.world.size() {
                trace!(
                    "sending task mapping {} -> {} to worker {} ...",
                    task_instance_mapping.task_instance_id(),
                    task_instance_mapping.task_id(),
                    dest
                );
                let process = self.world.process_at_rank(dest);
                process.send_with_tag(&task_instance_mapping, *REGISTER_TASK_ID);
                trace!("task mapping sent to worker {}", dest);
            }

            for dest in 1..self.world.size() {
                let process = self.world.process_at_rank(dest);
                if self
                    .chunk_distributor
                    .send_next_to(process, self.task_instance_id)
                {
                    self.send_count += 1;
                }
            }
            trace!("init send complete");
        }
        while self.recv_count < self.send_count {
            trace!("receiving response ...");
            let process = self.world.any_process();
            let rank = self
                .buf
                .receive_into_with_task_instance_id(process, self.task_instance_id);
            self.recv_count += 1;
            trace!(
                "received response of length {} from worker {}",
                self.buf.len(),
                rank
            );

            let process = self.world.process_at_rank(rank);
            if self
                .chunk_distributor
                .send_next_to(process, self.task_instance_id)
            {
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
        let task_instance_id = register_new_task(T::ID);
        let _span = error_span!("task", id = %task_instance_id).entered();

        let world = SimpleCommunicator::world();
        let mut send_count = 0;
        let mut recv_count = 0;
        let mut vec = Vec::new();

        let task_instance_mapping = TaskInstanceMapping::new(T::ID, task_instance_id);
        for dest in 1..world.size() {
            trace!(
                "sending task mapping {} -> {} to worker {} ...",
                task_instance_mapping.task_instance_id(),
                task_instance_mapping.task_id(),
                dest
            );
            let process = world.process_at_rank(dest);
            process.send_with_tag(&task_instance_mapping, *REGISTER_TASK_ID);
            trace!("task mapping sent to worker {}", dest);
        }

        for dest in 1..world.size() {
            let process = world.process_at_rank(dest);
            if self
                .chunk_distributor
                .send_next_to(process, task_instance_id)
            {
                send_count += 1;
            }
        }
        trace!("init send complete");

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

            trace!("receiving response ...");
            let process = world.any_process();
            let status = process.receive_into_with_tag(&mut buf, *task_instance_id);
            let rank = status.source_rank();
            let recv_len = status.count(datatype) as usize;
            // SAFETY: recv_len additional elements have been written at the end of the vector (within its reserved capacity)
            unsafe { vec.set_len(len + recv_len) };
            recv_count += 1;
            trace!(
                "received response of length {} from worker {}",
                recv_len,
                rank
            );

            let process = world.process_at_rank(rank);
            if self
                .chunk_distributor
                .send_next_to(process, task_instance_id)
            {
                send_count += 1;
            }
        }

        vec
    }
}
