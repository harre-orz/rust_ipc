use std::mem;
use std::ptr::NonNull;
use std::marker::PhantomData;

pub trait Pointer<T> : Sized {
    fn as_ref(&self) -> &T;
    fn as_mut(&mut self) -> &mut T;
    unsafe fn set_value(me: &mut Option<Self>, value: &mut T);
    unsafe fn clone_from(me: &mut Option<Self>, other: &mut Option<Self>);
}

impl<T> Pointer<T> for NonNull<T> {
    fn as_ref(&self) -> &T {
        unsafe { self.as_ref() }
    }

    fn as_mut(&mut self) -> &mut T {
        unsafe { &mut *(self.clone().as_ptr() as *mut _) }
    }

    unsafe fn set_value(me: &mut Option<Self>, value: &mut T) {
        *me = NonNull::new(value as *const _ as *mut T);
    }

    unsafe fn clone_from(me: &mut Option<Self>, other: &mut Option<Self>) {
        *me = *other;
    }
}

fn into_ptr<T>(me: &OffsetPtr<T>) -> NonNull<T> {
    let ptr = me.offset + me as *const _ as isize;
    unsafe {
        NonNull::new_unchecked(ptr as *mut T)
    }
}

fn from_ptr<T>(me: &mut OffsetPtr<T>, ptr: &mut T) {
    let off = ptr as *mut _ as isize - me as *mut _ as isize;
    me.offset = off;
}


#[derive(Debug)]
pub struct OffsetPtr<T> {
    offset: isize,
    _marker: PhantomData<T>,
}

impl<T> Pointer<T> for OffsetPtr<T> {
    fn as_ref(&self) -> &T {
        unsafe { &*into_ptr(self).as_ptr() }
    }

    fn as_mut(&mut self) -> &mut T {
        unsafe { &mut *into_ptr(self).as_ptr() }
    }

    unsafe fn set_value(me: &mut Option<Self>, value: &mut T) {
        *me = Some(mem::uninitialized());
        from_ptr(me.as_mut().unwrap(), value);
    }

    unsafe fn clone_from(me: &mut Option<Self>, other: &mut Option<Self>) {
        if let Some(ptr) = other {
            *me = Some(mem::uninitialized());
            from_ptr(me.as_mut().unwrap(), ptr.as_mut());
        } else {
            *me = None;
        }
    }
}
