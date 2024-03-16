use std::marker::PhantomData;

use mpi::{
    topology::SimpleCommunicator,
    traits::{Communicator, Destination, Equivalence, Source},
};
use tracing::{error_span, trace};

use crate::{
    function_registry::{register_new_task, TaskInstanceMapping, REGISTER_TASK_ID},
    iter::chunk_distributor::ChunkDistributor,
    task::Task,
};

pub(super) struct ForEach<I, T, const IN: usize>
where
    I: Iterator,
    I::Item: Equivalence,
    T: Task<In = I::Item, Out = u8, IN = { IN }, OUT = { 0 }>,
{
    chunk_distributor: ChunkDistributor<I, IN>,
    task: PhantomData<T>,
}

impl<I, T, const IN: usize> ForEach<I, T, IN>
where
    I: Iterator,
    I::Item: Equivalence,
    T: Task<In = I::Item, Out = u8, IN = { IN }, OUT = { 0 }>,
{
    pub(super) fn new(iter: I, _task: T) -> Self {
        Self {
            chunk_distributor: ChunkDistributor::new(iter),
            task: PhantomData,
        }
    }

    pub(super) fn for_each(mut self) {
        let task_instance_id = register_new_task(T::ID);
        let _span = error_span!("task", id = %task_instance_id).entered();

        let world = SimpleCommunicator::world();
        let mut send_count = 0;
        let mut recv_count = 0;

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

        let mut buf: [T::Out; 0] = [];
        while recv_count < send_count {
            trace!("receiving response ...",);
            let status = world
                .any_process()
                .receive_into_with_tag(&mut buf, *task_instance_id);
            recv_count += 1;
            trace!("received response from worker {}", status.source_rank());

            let process = world.process_at_rank(status.source_rank());
            if self
                .chunk_distributor
                .send_next_to(process, task_instance_id)
            {
                send_count += 1;
            }
        }
    }
}
