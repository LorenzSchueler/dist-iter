use std::mem::MaybeUninit;

use mpi::{
    traits::{Equivalence, Source},
    Rank, Tag,
};

pub struct UninitBuffer<T, const N: usize> {
    buf: [MaybeUninit<T>; N],
    count: usize,
}

impl<T, const N: usize> UninitBuffer<T, N> {
    pub fn new() -> Self {
        Self {
            // SAFETY: An uninitialized `[MaybeUninit<_>; LEN]` is valid.
            buf: unsafe { MaybeUninit::uninit().assume_init() },
            //buf: MaybeUninit::uninit_array(),
            count: 0,
        }
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn is_full(&self) -> bool {
        self.count == N
    }

    pub fn push_handle(&mut self) -> Option<UninitBuferPushHandle<T, N>> {
        if !self.is_full() {
            Some(UninitBuferPushHandle { buffer: self })
        } else {
            None
        }
    }

    // only accessed through push handle
    fn push(&mut self, item: T) {
        self.buf[self.count] = MaybeUninit::new(item);
        self.count += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.count > 0 {
            let result = unsafe { self.buf[self.count - 1].assume_init_read() };
            self.count -= 1;
            Some(result)
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.count = 0;
    }

    pub fn init_buffer_ref(&self) -> &[T] {
        // SAFETY: only the initialized part is returned
        unsafe { &*((&self.buf[..self.count]) as *const [MaybeUninit<T>] as *const [T]) }
        //unsafe { MaybeUninit::slice_assume_init_ref(&mut self.buf[..self.count]) },
    }

    pub fn receive_into_with_tag(&mut self, process: impl Source, tag: Tag) -> Rank
    where
        T: Equivalence,
    {
        // SAFETY: only safe if used for writes
        let full_buffer = unsafe { &mut *((&mut self.buf) as *mut [MaybeUninit<T>] as *mut [T]) };
        //unsafe { MaybeUninit::slice_assume_init_mut(&mut self.buf) },
        let status = process.receive_into_with_tag(full_buffer, tag);
        self.count = status.count(<T as Equivalence>::equivalent_datatype()) as usize;
        status.source_rank()
    }
}

pub struct UninitBuferPushHandle<'b, T, const N: usize> {
    buffer: &'b mut UninitBuffer<T, N>,
}

impl<'b, T, const N: usize> UninitBuferPushHandle<'b, T, N> {
    pub fn push(&mut self, item: T) {
        self.buffer.push(item)
    }
}
