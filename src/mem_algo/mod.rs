
mod simple_seq_fit;
pub use self::simple_seq_fit::*;

pub trait MemAlgo {
    fn init(&mut self, segment_bytes: usize);

    fn alloc<T>(&mut self, size: usize) -> *mut T;

    fn dealloc<T>(&mut self, ptr: *mut T, size: usize);

    fn realloc<T>(&mut self, ptr: *mut T, size: usize) -> *mut T;
}
