use std::mem;
use std::ptr;
use std::fmt;
use std::cmp::{Ordering};
use std::ops::{Deref, DerefMut, Add, Sub};
use std::marker::PhantomData;

fn align(addr: usize, align: usize) -> usize {
    assert_ne!(addr, 0);
    ((addr - 1) / align + 1) * align
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RawCell<T> {
    cell: T,
}

impl<T> RawCell<T> {
    pub fn get(&self) -> RawPtr<T> {
        RawPtr {
            addr: &self.cell as *const _ as usize,
            _marker: PhantomData,
        }
    }
}

impl<T> Deref for RawCell<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.cell
    }
}

impl<T> DerefMut for RawCell<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.cell
    }
}


pub struct RawPtr<T> {
    addr: usize,
    _marker: PhantomData<T>,
}

impl<T> RawPtr<T> {
    pub fn new(ptr: *mut T) -> Self {
        assert_eq!((ptr as usize) % mem::align_of::<T>(), 0);
        RawPtr {
            addr: ptr as usize,
            _marker: PhantomData,
        }
    }

    pub fn is_null(&self) -> bool {
        self.addr == 0
    }

    pub fn offset(self, off: usize) -> RawPtr<T> {
        RawPtr {
            addr: self.addr + off,
            _marker: PhantomData,
        }
    }

    pub fn align<U>(self) -> RawPtr<U> {
        RawPtr {
            addr: align(self.addr, mem::align_of::<U>()),
            _marker: PhantomData,
        }
    }
}

impl<T> Clone for RawPtr<T> {
    fn clone(&self) -> Self {
        RawPtr {
            addr: self.addr,
            _marker: PhantomData,
        }
    }
}

impl<T> Copy for RawPtr<T> {
}

impl<T> Eq for RawPtr<T> {
}

impl<T, U> PartialEq<RawPtr<U>> for RawPtr<T> {
    fn eq(&self, other: &RawPtr<U>) -> bool {
        self.addr == other.addr
    }
}

impl<T> Ord for RawPtr<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.addr.cmp(&other.addr)
    }
}

impl<T, U> PartialOrd<RawPtr<U>> for RawPtr<T> {
    fn partial_cmp(&self, other: &RawPtr<U>) -> Option<Ordering> {
        self.addr.partial_cmp(&other.addr)
    }
}

impl<T> AsRef<T> for RawPtr<T> {
    fn as_ref(&self) -> &T {
        unsafe { &*(self.addr as *const T) }
    }
}

impl<T> Deref for RawPtr<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*(self.addr as *const T) }
    }
}

impl<T> DerefMut for RawPtr<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *(self.addr as *mut T) }
    }
}

impl<T, U> Into<*const U> for RawPtr<T> {
    fn into(self) -> *const U {
        assert_eq!(self.addr % mem::align_of::<U>(), 0);
        self.addr as *const _
    }
}

impl<T, U> Into<*mut U> for RawPtr<T> {
    fn into(self) -> *mut U {
        assert_eq!(self.addr % mem::align_of::<U>(), 0);
        self.addr as *mut _
    }
}

impl<T> fmt::Display for RawPtr<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:p}", self.addr as *const T)
    }
}

impl<T> Add<usize> for RawPtr<T> {
    type Output = RawPtr<T>;

    fn add(self, rhs: usize) -> RawPtr<T> {
        RawPtr {
            addr: self.addr + rhs,
            _marker: PhantomData,
        }
    }
}

impl<T> Sub<usize> for RawPtr<T> {
    type Output = RawPtr<T>;

    fn sub(self, rhs: usize) -> RawPtr<T> {
        assert!(self.addr >= rhs, "self {} >= rhs {}", self, rhs);
        RawPtr {
            addr: self.addr - rhs,
            _marker: PhantomData,
        }
    }
}

impl<T, U> Sub<RawPtr<U>> for RawPtr<T> {
    type Output = usize;

    fn sub(self, rhs: RawPtr<U>) -> Self::Output {
        assert!(self.addr >= rhs.addr, "{} >= {}", self, rhs);
        self.addr - rhs.addr
    }
}


pub struct OffsetPtr<T> {
    offset: isize,
    _marker: PhantomData<T>,
}

impl<T> OffsetPtr<T> {
    pub fn null(&mut self) {
        self.offset = -1;
    }

    pub fn init(&mut self, raw: &RawPtr<T>) {
        assert_eq!(raw.is_null(), false);
        self.offset = raw.addr as isize - self as *const _ as isize
    }

    fn off2addr(&self) -> usize {
        (self.offset + self as *const _ as isize) as usize
    }

    pub fn get(&self) -> RawPtr<T> {
        RawPtr {
            addr: self.off2addr(),
            _marker: PhantomData,
        }
    }

    pub fn set<U: AsRef<T>>(&mut self, ptr: U) {
        let addr = ptr.as_ref() as *const _ as isize;
        self.offset = addr - self as *const _ as isize;
    }
}

impl<T> Deref for OffsetPtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.off2addr() as *const _) }
    }
}

impl<T> DerefMut for OffsetPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self.off2addr() as *mut _) }
    }
}

// #[test]
// fn test_offset_ptr() {
//     let mut ptr: OffsetPtr<i64> = OffsetPtr {
//         offset: 0,
//         _marker: PhantomData,
//     };
//     OffsetPtr::null(&mut ptr);
//     assert_eq!(OffsetPtr::is_null(&ptr), true);
//     assert_eq!(OffsetPtr::as_ptr(&ptr), ptr::null());
//
//     let y: i64 = 100;
//     OffsetPtr::init(&mut ptr, &y);
//     assert_ne!(OffsetPtr::is_null(&ptr), true);
//     assert_ne!(OffsetPtr::as_ptr(&ptr), ptr::null());
//     assert_eq!(*ptr, 100);
//
//     let z: i64 = 200;
//     OffsetPtr::init(&mut ptr, &z);
//     assert_ne!(OffsetPtr::is_null(&ptr), true);
//     assert_ne!(OffsetPtr::as_ptr(&ptr), ptr::null());
//     assert_eq!(*ptr, 200);
// }
