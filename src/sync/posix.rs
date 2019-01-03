use sync::Mutex;
use std::mem;
use libc;

pub struct NullMutex {
    _mutex: libc::pthread_mutex_t,
}

impl Mutex for NullMutex {
    fn place_new(&mut self) {}
    fn lock(&mut self) {}
    fn try_lock(&mut self) -> bool { true }
    fn unlock(&mut self) {}
}


pub struct SharedMutex {
    mutex: libc::pthread_mutex_t,
}

impl Mutex for SharedMutex {
    fn place_new(&mut self) {
        unsafe {
            let mut attr: libc::pthread_mutexattr_t = mem::uninitialized();
            libc::pthread_mutexattr_setpshared(&mut attr, libc::PTHREAD_PROCESS_SHARED);
            libc::pthread_mutex_init(&mut self.mutex, &attr);
        }
    }

    fn lock(&mut self) {
        unsafe { libc::pthread_mutex_lock(&mut self.mutex) };
    }

    fn try_lock(&mut self) -> bool {
        unsafe { libc::pthread_mutex_trylock(&mut self.mutex) != libc::EBUSY }
    }

    fn unlock(&mut self) {
        unsafe { libc::pthread_mutex_unlock(&mut self.mutex) };
    }
}


pub struct PrivateMutex {
    mutex: libc::pthread_mutex_t,
}

impl Mutex for PrivateMutex {
    fn place_new(&mut self) {
        unsafe {
            let mut attr: libc::pthread_mutexattr_t = mem::uninitialized();
            libc::pthread_mutexattr_setpshared(&mut attr, libc::PTHREAD_PROCESS_PRIVATE);
            libc::pthread_mutex_init(&mut self.mutex, &attr);
        }
    }

    fn lock(&mut self) {
        unsafe { libc::pthread_mutex_lock(&mut self.mutex) };
    }

    fn try_lock(&mut self) -> bool {
        unsafe { libc::pthread_mutex_trylock(&mut self.mutex) != libc::EBUSY }
    }

    fn unlock(&mut self) {
        unsafe { libc::pthread_mutex_unlock(&mut self.mutex) };
    }
}
