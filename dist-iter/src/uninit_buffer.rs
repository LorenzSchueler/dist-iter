use std::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};

use mpi::{
    point_to_point::Message,
    traits::{Equivalence, Source},
    Rank, Tag,
};

pub struct UninitBuffer<T, const N: usize> {
    buf: [MaybeUninit<T>; N],
    start: usize,
    end: usize,
}

impl<T, const N: usize> UninitBuffer<T, N> {
    pub fn new() -> Self {
        Self {
            // SAFETY: An uninitialized `[MaybeUninit<_>; LEN]` is valid.
            buf: unsafe { MaybeUninit::uninit().assume_init() },
            //buf: MaybeUninit::uninit_array(),
            start: 0,
            end: 0,
        }
    }

    pub fn init_count(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.end == 0
    }

    pub fn is_full(&self) -> bool {
        self.end == N
    }

    pub fn push_handle(&mut self) -> Option<UninitBufferPushHandle<T, N>> {
        if !self.is_full() {
            Some(UninitBufferPushHandle { buffer: self })
        } else {
            None
        }
    }

    pub fn push_back_unchecked(&mut self, item: T) {
        self.push_back(item)
    }

    // only accessed through push handle or push_unchecked
    fn push_back(&mut self, item: T) {
        self.buf[self.end] = MaybeUninit::new(item);
        self.end += 1;
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if self.start < self.end {
            let result = unsafe { self.buf[self.start].assume_init_read() };
            self.start += 1;
            Some(result)
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.start = 0;
        self.end = 0;
    }

    pub(crate) fn receive_into_with_tag(&mut self, process: impl Source, tag: Tag) -> Rank
    where
        T: Equivalence,
    {
        // SAFETY: only safe if used for writes
        let full_buffer = unsafe { &mut *((&mut self.buf) as *mut [MaybeUninit<T>] as *mut [T]) };
        //unsafe { MaybeUninit::slice_assume_init_mut(&mut self.buf) },
        let status = process.receive_into_with_tag(full_buffer, tag);
        self.start = 0;
        self.end = status.count(<T as Equivalence>::equivalent_datatype()) as usize;
        status.source_rank()
    }

    pub fn matched_receive_into(&mut self, from: Message) -> Tag
    where
        T: Equivalence,
    {
        // SAFETY: only safe if used for writes
        let full_buffer = unsafe { &mut *((&mut self.buf) as *mut [MaybeUninit<T>] as *mut [T]) };
        //unsafe { MaybeUninit::slice_assume_init_mut(&mut self.buf) },
        let status = from.matched_receive_into(full_buffer);
        self.start = 0;
        self.end = status.count(<T as Equivalence>::equivalent_datatype()) as usize;
        status.tag()
    }
}

impl<T, const N: usize> Default for UninitBuffer<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> Iterator for UninitBuffer<T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.pop_front()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.init_count();
        (size, Some(size))
    }
}

impl<T, const N: usize> ExactSizeIterator for UninitBuffer<T, N> {}

impl<T, const N: usize> Deref for UninitBuffer<T, N> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        // SAFETY: only the initialized part is returned
        unsafe { &*((&self.buf[self.start..self.end]) as *const [MaybeUninit<T>] as *const [T]) }
        //unsafe { MaybeUninit::slice_assume_init_ref(&self.buf[..self.count]) },
    }
}

impl<T, const N: usize> DerefMut for UninitBuffer<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: only the initialized part is returned
        unsafe {
            &mut *((&mut self.buf[self.start..self.end]) as *mut [MaybeUninit<T>] as *mut [T])
        }
        //unsafe { MaybeUninit::slice_assume_init_mut(&mut self.buf[..self.count]) },
    }
}

pub struct UninitBufferPushHandle<'b, T, const N: usize> {
    buffer: &'b mut UninitBuffer<T, N>,
}

impl<'b, T, const N: usize> UninitBufferPushHandle<'b, T, N> {
    pub fn push_back(&mut self, item: T) {
        self.buffer.push_back(item)
    }
}
