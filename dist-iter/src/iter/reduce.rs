use std::marker::PhantomData;

use mpi::{
    topology::SimpleCommunicator,
    traits::{Communicator, Equivalence, Source},
};
use tracing::{error_span, trace};

use crate::{
    function_registry::{register_new_task, send_task_instance_mapping, TaskInstanceMapping},
    iter::chunk_distributor::ChunkDistributor,
    task::Task,
    utils::CommunicatorExt,
};

#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub(super) struct Reduce<I, T, F, const IN: usize>
where
    I: Iterator,
    I::Item: Equivalence,
    T: Task<In = I::Item, Out = I::Item, IN = { IN }, OUT = { 1 }>,
    F: FnMut(I::Item, I::Item) -> I::Item,
{
    chunk_distributor: ChunkDistributor<I, IN>,
    task: PhantomData<T>,
    f: F,
}

impl<I, T, F, const IN: usize> Reduce<I, T, F, IN>
where
    I: Iterator,
    I::Item: Equivalence,
    T: Task<In = I::Item, Out = I::Item, IN = { IN }, OUT = { 1 }>,
    F: FnMut(I::Item, I::Item) -> I::Item,
{
    pub(super) fn new(iter: I, _task: T, f: F) -> Self {
        Self {
            chunk_distributor: ChunkDistributor::new(iter),
            task: PhantomData,
            f,
        }
    }

    pub(super) fn value(mut self) -> Option<I::Item> {
        let task_instance_id = register_new_task(T::ID);
        let _span = error_span!("task", id = %task_instance_id).entered();

        let world = SimpleCommunicator::world();
        let mut send_count = 0;
        let mut recv_count = 0;

        let task_instance_mapping = TaskInstanceMapping::new(T::ID, task_instance_id);
        send_task_instance_mapping(task_instance_mapping, &world);

        for process in world.workers() {
            if self
                .chunk_distributor
                .send_next_to(process, task_instance_id)
            {
                send_count += 1;
            }
        }
        trace!("init send complete");

        if send_count > 0 {
            trace!("receiving response ...");
            let (result, status) = world.any_process().receive_with_tag(*task_instance_id);
            recv_count += 1;
            trace!("received response from worker {}", status.source_rank());

            let process = world.process_at_rank(status.source_rank());
            if self
                .chunk_distributor
                .send_next_to(process, task_instance_id)
            {
                send_count += 1;
            }
            let mut acc = result;

            while recv_count < send_count {
                trace!("receiving response ...");
                let (result, status) = world.any_process().receive_with_tag(*task_instance_id);
                recv_count += 1;
                trace!("received response from worker {}", status.source_rank());

                let process = world.process_at_rank(status.source_rank());
                if self
                    .chunk_distributor
                    .send_next_to(process, task_instance_id)
                {
                    send_count += 1;
                }
                acc = (self.f)(acc, result);
            }

            Some(acc)
        } else {
            None
        }
    }
}
