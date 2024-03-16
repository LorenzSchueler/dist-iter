use mpi::{
    topology::{Process, SimpleCommunicator},
    traits::{Destination, Equivalence},
};
use tracing::trace;

use crate::{TaskInstanceId, UninitBuffer};

pub(super) struct ChunkDistributor<Iter, const N: usize>
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
        task_instance_id: TaskInstanceId,
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
            trace!(
                "sending data of length {} to worker {} ...",
                self.buf.len(),
                process.rank()
            );
            process.send_with_tag(&*self.buf, *task_instance_id);
            trace!("data sent to worker {}", process.rank());
            self.buf.clear();
            true
        } else {
            false
        }
    }
}
