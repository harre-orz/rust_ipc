use {Privilege, ReadWrite, MappedRegion, FileMapping, Perm};
use mem_algo::{MemAlgo, SimpleSeqFit};
use std::io;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::sync::Arc;

pub struct ManagedMappedFile<P: Privilege, A: MemAlgo = SimpleSeqFit> {
    region: Arc<MappedRegion<P::Mode>>,
    _marker: PhantomData<(P, A)>,
}

impl<P: Privilege, A: MemAlgo> ManagedMappedFile<P, A> {
    pub fn create(name: &str, size: usize, mode: P) -> io::Result<Self> {
        let name = CString::new(name).unwrap();
        let region = FileMapping::open_or_create(&name, mode, size, 0, Perm)?;
        let algo = unsafe { &mut *(region.as_ptr() as *mut _ as *mut A) };
        algo.init(size);
        Ok(ManagedMappedFile {
            region: Arc::new(region),
            _marker: PhantomData,
        })
    }

    pub fn segment_manager(&self) -> &mut A {
        unsafe { &mut *(self.region.as_ptr() as *const A as *mut A) }
    }
}

#[test]
fn test_managed_mapped_file() {
    let mfile: ManagedMappedFile<_, SimpleSeqFit> = ManagedMappedFile::create("mfile-test", 4096, ReadWrite).unwrap();
    let alloc = mfile.segment_manager();
    let x = unsafe { &mut *(alloc.alloc::<u8>(1024)) };
    let mut s = unsafe { ::std::slice::from_raw_parts_mut(x, 1024) };

    for i in 0..1024 {
        s[i] = 0x41;
    }

    let x = unsafe { &mut *(alloc.alloc::<u8>(8)) };
    let mut s = unsafe { ::std::slice::from_raw_parts_mut(x, 8) };
    for i in 0..8 {
        s[i] = 0x42;
    }
}
