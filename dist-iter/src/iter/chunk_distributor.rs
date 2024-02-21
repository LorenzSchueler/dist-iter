use mpi::{
    topology::{Process, SimpleCommunicator},
    traits::{Destination, Equivalence},
    Tag,
};

use crate::UninitBuffer;

#[doc(hidden)]
pub struct ChunkDistributor<Iter, const N: usize>
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
        tag: Tag,
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
            eprintln!("> data of length {:?}", self.buf.len());
            process.send_with_tag(&*self.buf, tag);
            self.buf.clear();
            true
        } else {
            false
        }
    }
}
