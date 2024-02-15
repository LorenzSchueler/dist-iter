use std::marker::PhantomData;

use mpi::{
    topology::SimpleCommunicator,
    traits::{Communicator, Source},
};

use crate::{iter::dist_iterator::DistIterator, Task};

#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct Map<const N: usize, I, T>
where
    I: DistIterator<N>,
    T: Task,
{
    inner: I,
    task: PhantomData<T>,
    buf: Vec<T::OUT>, // TODO use array
    send_count: usize,
    recv_count: usize,
    init: bool,
    world: SimpleCommunicator,
}

impl<const N: usize, I, T> Map<N, I, T>
where
    I: DistIterator<N>,
    T: Task<IN = I::Item>,
{
    pub(super) fn new(inner: I, _task: T) -> Self {
        Map {
            inner,
            task: PhantomData,
            buf: Vec::new(),
            send_count: 0,
            recv_count: 0,
            init: false,
            world: SimpleCommunicator::world(),
        }
    }
}

impl<const N: usize, I, T> Iterator for Map<N, I, T>
where
    I: DistIterator<N>,
    T: Task,
{
    type Item = T::OUT;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.buf.pop() {
            return Some(item);
        }
        if !self.init {
            for dest in 1..self.world.size() {
                let process = self.world.process_at_rank(dest);
                if self.inner.send_next_to(process, T::TAG) {
                    self.send_count += 1;
                }
            }
        }
        if self.recv_count < self.send_count {
            let (data, status) = self.world.any_process().receive_vec_with_tag(T::TAG);
            self.buf = data;
            self.recv_count += 1;

            let process = self.world.process_at_rank(status.source_rank());
            if self.inner.send_next_to(process, T::TAG) {
                self.send_count += 1;
            }
        }
        if let Some(item) = self.buf.pop() {
            Some(item)
        } else {
            None
        }
    }
}

//let count = status
//.count(Msg::equivalent_datatype())
//.value_as()
//.expect("Message element count cannot be expressed as a usize.");

//let (message, status) = self;
//let count = status
//.count(Msg::equivalent_datatype())
//.value_as()
//.expect("Message element count cannot be expressed as a usize.");

//#[repr(transparent)]
//struct UninitMsg<M>(MaybeUninit<M>);

//unsafe impl<M: Equivalence> Equivalence for UninitMsg<M> {
//type Out = M::Out;

//fn equivalent_datatype() -> Self::Out {
//M::equivalent_datatype()
//}
//}

//let mut res = (0..count)
//.map(|_| UninitMsg::<Msg>(MaybeUninit::uninit()))
//.collect::<Vec<_>>();

//let status = message.matched_receive_into(&mut res[..]);

//let res = unsafe { transmute(res) };

//(res, status)
