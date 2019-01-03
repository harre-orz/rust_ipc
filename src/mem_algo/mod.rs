pub trait MemAlgo {
    fn alloc<T>(&mut self, size: usize) -> *mut T;

    fn dealloc<T>(&mut self, ptr: *mut T, size: usize);

    fn realloc<T>(&mut self, ptr: *mut T, size: usize) -> *mut T;
}

mod simple_seq_fit;
pub use self::simple_seq_fit::*;
