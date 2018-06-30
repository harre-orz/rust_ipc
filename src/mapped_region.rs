use super::*;
use ffi::*;

use std::io;
use std::ptr;
use std::marker::PhantomData;
use libc;

#[cfg(unix)]
pub struct MappedRegion<M> {
    base: *mut libc::c_void,
    size: usize,
    offset: usize,
    mode: PhantomData<M>,
}

pub enum Advise {
    Normal,
    Sequential,
    Random,
    Willneed,
    Dontneed,
}

fn advise<M>(mapping: &MappedRegion<M>, adv: Advise) -> bool {
    false
}

fn flush<M>(mapping: &mut MappedRegion<M>, offset: usize, mut bytes: usize) -> io::Result<()> {
    if offset > mapping.size || offset + bytes > mapping.size {
        return Err(ErrCode::last_error().into());
    }
    if bytes == 0 {
        bytes = mapping.size - offset;
    }
    match unsafe { libc::msync(mapping.base.offset(offset as isize), bytes, libc::MS_SYNC) } {
        -1 => Err(ErrCode::last_error().into()),
        _ => Ok(()),
    }
}

fn shrink_by<M>(mapping: &mut MappedRegion<M>, bytes: usize, from_back: bool) -> bool {
    false
}

impl MappedRegion<CopyOnWrite> {
    pub fn advise(&self, adv: Advise) -> bool {
        advise(self, adv)
    }

    pub fn as_ptr(&self) -> &libc::c_void {
        unsafe { &*self.base }
    }

    pub fn as_mut_ptr(&mut self) -> &mut libc::c_void {
        unsafe { &mut *self.base }
    }

    pub fn flush(&mut self, offset: usize, mut bytes: usize) -> io::Result<()> {
        flush(self, offset, bytes)
    }

    pub fn shrink_by(&mut self, bytes: usize, from_back: bool) -> bool {
        shrink_by(self, bytes, from_back)
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

impl MappedRegion<ReadOnly> {
    pub fn advise(&self, adv: Advise) -> bool {
        advise(self, adv)
    }

    pub fn as_ptr(&self) -> &libc::c_void {
        unsafe { &*self.base }
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

impl MappedRegion<ReadPrivate> {
    pub fn advise(&self, adv: Advise) -> bool {
        advise(self, adv)
    }

    pub fn as_ptr(&self) -> &libc::c_void {
        unsafe { &*self.base }
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

impl MappedRegion<ReadWrite> {
    pub fn advise(&self, adv: Advise) -> bool {
        advise(self, adv)
    }

    pub fn as_ptr(&self) -> &libc::c_void {
        unsafe { &*self.base }
    }

    pub fn as_mut_ptr(&mut self) -> &mut libc::c_void {
        unsafe { &mut *self.base }
    }

    pub fn flush(&mut self, offset: usize, mut bytes: usize) -> io::Result<()> {
        flush(self, offset, bytes)
    }

    pub fn shrink_by(&mut self, bytes: usize, from_back: bool) -> bool {
        shrink_by(self, bytes, from_back)
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

impl<M> Drop for MappedRegion<M> {
    fn drop(&mut self) {
        unsafe { libc::munmap(self.base, self.size); }
    }
}

#[cfg(unix)]
impl SharedMemoryObject<ReadOnly> {
    pub fn mapping(self, size: usize) -> io::Result<MappedRegion<ReadOnly>> {
        let handle = self.mapping_handle();

        let prot = libc::PROT_READ;
        let flags = libc::MAP_SHARED;

        match unsafe { libc::mmap(ptr::null_mut(), size, prot, flags, handle, 0) } {
            libc::MAP_FAILED => Err(ErrCode::last_error().into()),
            base => Ok(MappedRegion {
                base: base,
                size: size,
                offset: 0,
                mode: PhantomData,
            }),
        }
    }
}

#[cfg(unix)]
impl SharedMemoryObject<ReadWrite> {
    pub fn mapping(self, size: usize) -> io::Result<MappedRegion<ReadWrite>> {
        let handle = self.mapping_handle();

        let prot = libc::PROT_WRITE;
        let flags = libc::MAP_SHARED;

        match unsafe { libc::mmap(ptr::null_mut(), size, prot, flags, handle, 0) } {
            libc::MAP_FAILED => Err(ErrCode::last_error().into()),
            base => Ok(MappedRegion {
                base: base,
                size: size,
                offset: 0,
                mode: PhantomData,
            }),
        }
    }
}

#[test]
fn test_mapped_region() {
    use std::ffi::CString;

    let name = CString::new("hoge").unwrap();
    unsafe { SharedMemoryObject::<ReadWrite>::remove(&name); }
    let shm = SharedMemoryObject::open_or_create(&name, ReadWrite, Perm).unwrap();
    let mmap = shm.mapping(4096).unwrap();
}
