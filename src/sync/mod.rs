pub trait Mutex {
    fn place_new(&mut self);

    fn lock(&mut self);

    fn try_lock(&mut self) -> bool;

    fn unlock(&mut self);
}


pub struct LockGuard<'a, M: Mutex>(&'a mut M);

impl<'a, M: Mutex> Drop for LockGuard<'a, M> {
    fn drop(&mut self) {
        self.0.unlock();
    }
}


pub fn lock_guard<'a, M: Mutex>(mutex: &'a mut M) -> LockGuard<'a, M> {
    mutex.lock();
    LockGuard(mutex)
}

#[cfg(unix)]
mod posix;

#[cfg(unix)]
pub use self::posix::*;
