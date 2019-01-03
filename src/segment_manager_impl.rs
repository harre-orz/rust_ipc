use sync::{Mutex, SharedMutex, PrivateMutex, NullMutex};
use mem_algo::{MemAlgo, SimpleSeqFit};
use indexes::Index;
use SegmentManager;
use std::ptr;

impl<I: Index> SegmentManager<SimpleSeqFit<SharedMutex>, I> {
    pub fn construct<T>(&mut self, key: &str) -> Option<&mut T>
    {
        None
    }

    pub fn find_or_construct<T, F>(&mut self, key: &str, func: F) -> Option<&mut T>
    {
        None
    }

    pub fn find<T>(&self, key: &str) -> Option<&T> {
        None
    }

    pub fn find_mut<T>(&mut self, key: &str) -> Option<&mut T> {
        None
    }
}

impl<I: Index> SegmentManager<SimpleSeqFit<PrivateMutex>, I> {
    pub fn construct<T>(&mut self, key: &str) -> Option<&mut T> {
        None
    }

    pub fn find<T>(&self, key: &str) -> Option<&T> {
        None
    }

    pub fn find_or_construct<T>(&mut self, key: &str) -> Option<&mut T> {
        None
    }

    pub fn find_mut<T>(&mut self, key: &str) -> Option<&mut T> {
        None
    }
}

impl<I: Index> SegmentManager<SimpleSeqFit<NullMutex>, I> {
    pub fn find<T>(&self, key: &str) -> Option<&T> {
        None
    }
}

impl<M: Mutex, I: Index> MemAlgo for SegmentManager<SimpleSeqFit<M>, I> {
    fn alloc<T>(&mut self, size: usize) -> *mut T {
        ptr::null_mut()
    }

    fn dealloc<T>(&mut self, ptr: *mut T, size: usize) {
    }

    fn realloc<T>(&mut self, ptr: *mut T, size: usize) -> *mut T {
        ptr::null_mut()
    }
}
