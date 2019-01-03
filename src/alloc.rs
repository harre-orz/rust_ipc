use ptr::Pointer;
use std::mem;
use std::ptr::NonNull;
use std::marker::PhantomData;
use libc;

pub trait Allocator<T> {
    type Pointer: Pointer<T>;

    fn alloc(size: usize, ptr: &mut Self::Pointer) -> bool;

    fn realloc(size: usize, ptr: &mut Self::Pointer) -> bool;

    fn dealloc(size: usize, ptr: &mut Self::Pointer);
}


#[derive(Clone)]
pub struct Alloc<T> {
    _marker: PhantomData<T>,
}

impl<T> Alloc<T> {
    pub fn new() -> Self {
        Alloc {
            _marker: PhantomData,
        }
    }
}

impl<T> Allocator<T> for Alloc<T> {
    type Pointer = NonNull<T>;

    fn alloc(size: usize, ptr: &mut Self::Pointer) -> bool {
        unsafe {
            if let Some(value) = size.checked_mul(mem::size_of::<T>())
                .map(|size| libc::malloc(size) as *mut T)
                .and_then(|ptr| ptr.as_mut())
            {
                *ptr = NonNull::new_unchecked(value);
                true
            } else {
                false
            }
        }
    }

    fn realloc(size: usize, ptr: &mut Self::Pointer) -> bool {
        unsafe {
            if let Some(value) = size.checked_mul(mem::size_of::<T>())
                .map(|size| libc::realloc(ptr.clone().as_ptr() as *mut libc::c_void, size) as *mut T)
                .and_then(|ptr| ptr.as_mut())
            {
                *ptr = NonNull::new_unchecked(value);
                true
            } else {
                false
            }
        }
    }

    fn dealloc(size: usize, ptr: &mut Self::Pointer) {
        unsafe {
            libc::free(ptr.clone().as_ptr() as *mut libc::c_void);
        }
    }
}
