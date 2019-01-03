use ptr::Pointer;
use std::marker::PhantomData;

pub struct RefPtr<'a, T: 'a, P: 'a> {
    ptr: &'a Option<P>,
    _marker: PhantomData<T>,
}

impl<'a, T, P: Pointer<T>> RefPtr<'a, T, P> {
    pub fn new(ptr: &'a Option<P>) -> Self {
        RefPtr {
            ptr: ptr,
            _marker: PhantomData,
        }
    }

    pub fn get(&self) -> Option<&'a T> {
        if let Some(ptr) = self.ptr {
            Some(ptr.as_ref())
        } else {
            None
        }
    }

    pub fn set(&mut self, ptr: &'a Option<P>) {
        self.ptr = ptr;
    }
}


pub struct MutPtr<'a, T: 'a, P: 'a> {
    ptr: &'a mut Option<P>,
    _marker: PhantomData<T>,
}

impl<'a, T, P: Pointer<T>> MutPtr<'a, T, P> {
    pub fn new(ptr: &'a mut Option<P>) -> Self {
        MutPtr {
            ptr: ptr,
            _marker: PhantomData,
        }
    }

    pub fn get(&mut self) -> Option<&'a mut T> {
        if let Some(ptr) = self.ptr.as_ref() {
            Some(unsafe { &mut *(ptr as *const _ as *mut P) }.as_mut())
        } else {
            None
        }
    }

    pub fn set(&mut self, ptr: &'a mut Option<P>) {
        self.ptr = ptr;
    }
}
