use super::*;
use err::*;
use std::io;
use std::ptr;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use libc;

pub trait Privilege {
    fn is_shared(&self) -> bool;

    fn writable(&self) -> bool;

    #[doc(hidden)]
    const O_RD: i32;

    #[doc(hidden)]
    const MAP_PROT: i32;

    #[cfg(unix)]
    const SHMFLG: i32 = 0;
}

impl Privilege for ReadOnly {
    fn writable(&self) -> bool {
        false
    }

    fn is_shared(&self) -> bool {
        true
    }

    #[cfg(unix)]
    const O_RD: i32 = libc::O_RDONLY;

    #[cfg(unix)]
    const MAP_PROT: i32 = libc::PROT_READ;

    #[cfg(unix)]
    const SHMFLG: i32 = libc::SHM_RDONLY;
}

impl Privilege for ReadWrite {
    fn writable(&self) -> bool {
        true
    }

    fn is_shared(&self) -> bool {
        true
    }

    #[doc(hidden)]
    const O_RD: i32 = libc::O_RDWR;

    #[doc(hidden)]
    const MAP_PROT: i32 = libc::PROT_READ | libc::PROT_WRITE;
}

pub struct Perm(u32);

pub struct MappedRegion<P> {
    base: *mut libc::c_void,
    size: usize,
    page_offset: isize,
    is_xsi: bool,
    mode: PhantomData<P>,
}

impl<P: Privilege> MappedRegion<P> {
    fn new(handle: Handle, mode: P, flags: i32, mut size: usize, offset: isize) -> io::Result<Self> {
        let page_offset = adjust_page_offset(offset);
        if size == 0 {
            size = handle.size()?;
            if size < offset as usize {
                return Err(PERMISSION_DENIED.into());
            }
            size -= (offset - page_offset) as usize;
        }

        match unsafe {
            // TODO: ページ開始位置の指定は使うことはがないので実装していない
            libc::mmap(ptr::null_mut(), size + page_offset as usize, P::MAP_PROT, flags, handle.0, (offset - page_offset) as i64)
        } {
            libc::MAP_FAILED => Err(ErrCode::last_error().into()),
            base => Ok(MappedRegion {
                base: unsafe { base.offset(page_offset) },
                size: size,
                page_offset: page_offset,
                is_xsi: false,
                mode: PhantomData,
            })
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn flush(&mut self, size: usize, offset: isize, async: bool) -> bool {
        if self.is_xsi {
            return false;
        }

        // TODO
        let bytes = size + offset as usize;
        unsafe { libc::msync(self.base, bytes, if async { libc::MS_ASYNC } else { libc::MS_SYNC }) == 0 }
    }

    pub fn shrink_by(&mut self, size: usize, from_back: bool) -> bool {
        if self.is_xsi {
            return false;
        }

        // TODO
        let page_start = ptr::null_mut();
        let page_bytes = 0;
        if page_bytes > 0 {
            unsafe { libc::munmap(page_start, page_bytes) == 0 }
        } else {
            false
        }
    }

    pub fn advise(&self, advice: Advice) -> bool {
        // TODO
        false
    }
}

impl MappedRegion<ReadOnly> {
    pub fn as_ptr(&self) -> *const libc::c_void {
        self.base
    }
}

impl MappedRegion<ReadWrite> {
    pub fn as_ptr(&self) -> *mut libc::c_void {
        self.base
    }
}

impl<P> Drop for MappedRegion<P> {
    fn drop(&mut self) {
        unsafe {
            if self.is_xsi {
                libc::shmdt(self.base);
            } else {
                libc::munmap(self.base.offset(-self.page_offset), self.size + self.page_offset as usize);
            }
        }
    }
}

struct Handle(i32);

impl Handle {
    fn size(&self) -> Result<usize, ErrCode> {
        unsafe {
            let mut st: libc::stat = ::std::mem::uninitialized();
            match libc::fstat(self.0, &mut st) {
                -1 => Err(ErrCode::last_error()),
                _ => Ok(st.st_size as usize)
            }
        }
    }

    fn truncate(&self, size: usize) -> Result<(), ErrCode> {
        match unsafe { libc::ftruncate(self.0, size as i64) } {
            -1 => Err(ErrCode::last_error()),
            _ => Ok(()),
        }
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe { libc::close(self.0); }
    }
}

pub struct FileMapping<P> {
    name: CString,
    mode: P,
    perm: Perm,
    size: usize,
    offset: isize,
    flags: i32,
}

impl<P: Privilege> FileMapping<P> {
    pub fn new<T>(name: T, mode: P) -> Self
        where T: AsRef<str>
    {
        FileMapping {
            name: CString::new(name.as_ref()).unwrap(),
            mode: mode,
            perm: Perm(0o644),
            size: 0,
            offset: 0,
            flags: libc::MAP_SHARED,
        }
    }

    pub fn offset(mut self, offet: isize) -> Self {
        self.offset = offet;
        self
    }

    pub fn perm(mut self, perm: Perm) -> Self {
        self.perm = perm;
        self
    }

    pub fn private(mut self) -> Self {
        self.flags = libc::MAP_PRIVATE;
        self
    }

    pub fn size(mut self, size: usize) -> Self {
        self.size = size;
        self
    }

    pub fn create(self) -> io::Result<MappedRegion<P>> {
        let handle = self.file_create()?;
        let file_size = self.size + self.offset as usize;
        handle.truncate(file_size)?;
        self.mapping(handle)
    }

    pub fn open(self) -> io::Result<MappedRegion<P>> {
        let handle = self.file_open()?;
        let file_size = self.size + self.offset as usize;
        if handle.size()? < file_size {
            handle.truncate(file_size)?;
        }
        self.mapping(handle)
    }

    pub fn open_or_create(self) -> io::Result<MappedRegion<P>> {
        let handle = loop {
            match self.file_create() {
                Err(ec) if ec == FILE_EXISTS => {}, // continue
                otherwise => break otherwise?,
            }
            match self.file_open() {
                Err(ec) if  ec == NO_SUCH_FILE_OR_DIRECTORY => {}, // continue
                otherwise => break otherwise?,
            }
        };
        let file_size = self.size + self.offset as usize;
        if handle.size()? < file_size {
            handle.truncate(file_size)?;
        }
        self.mapping(handle)
    }

    fn file_create(&self) -> Result<Handle, ErrCode> {
        match unsafe { libc::open(self.name.as_ptr(), P::O_RD | libc::O_CREAT | libc::O_EXCL, self.perm.0) } {
            -1 => Err(ErrCode::last_error()),
            fd => Ok(Handle(fd)),
        }
    }

    fn file_open(&self) -> Result<Handle, ErrCode> {
        match unsafe { libc::open(self.name.as_ptr(), P::O_RD) } {
            -1 => Err(ErrCode::last_error()),
            fd => Ok(Handle(fd)),
        }
    }

    fn mapping(self, handle: Handle) -> io::Result<MappedRegion<P>> {
        Ok(MappedRegion::new(handle, self.mode, self.flags, self.size, self.offset)?)
    }

    pub fn remove(self) -> bool {
        unsafe { libc::unlink(self.name.as_ptr()) == 0 }
    }
}


pub struct SharedMemory<P> {
    name: CString,
    mode: P,
    perm: Perm,
    size: usize,
    offset: isize,
    flags: i32,
}

impl<P: Privilege> SharedMemory<P> {
    pub fn new<T>(name: T, mode: P) -> Self
        where T: AsRef<str>,
    {
        SharedMemory {
            name: CString::new(name.as_ref()).unwrap(),
            mode: mode,
            perm: Perm(0o644),
            size: 0,
            offset: 0,
            flags: libc::MAP_SHARED,
        }
    }

    pub fn offset(mut self, offset: isize) -> Self {
        self.offset = offset;
        self
    }

    pub fn perm(mut self, perm: Perm) -> Self {
        self.perm = perm;
        self
    }

    pub fn private(mut self) -> Self {
        self.flags = libc::MAP_PRIVATE;
        self
    }

    pub fn size(mut self, size: usize) -> Self {
        self.size = size;
        self
    }

    pub fn create(self) -> io::Result<MappedRegion<P>> {
        let handle = self.shm_create()?;
        let shm_size = self.size + self.offset as usize;
        handle.truncate(shm_size)?;
        self.mapping(handle)

    }

    pub fn open(self) -> io::Result<MappedRegion<P>> {
        let handle = self.shm_open()?;
        let shm_size = self.size + self.offset as usize;
        if handle.size()? < shm_size {
            handle.truncate(shm_size)?;
        }
        self.mapping(handle)
    }

    pub fn open_or_create(self) -> io::Result<MappedRegion<P>> {
        let handle = loop {
            match self.shm_create() {
                Err(ec) if ec == FILE_EXISTS => {}, // continue
                otherwise => break otherwise?,
            }
            match self.shm_open() {
                Err(ec) if ec == NO_SUCH_FILE_OR_DIRECTORY => {}, // continue
                otherwise => break otherwise?,
            }
        };
        let shm_size = self.size + self.offset as usize;
        if handle.size()? < shm_size {
            handle.truncate(shm_size)?;
        }
        self.mapping(handle)
    }

    fn shm_create(&self) -> Result<Handle, ErrCode> {
        match unsafe { libc::shm_open(self.name.as_ptr(), P::O_RD | libc::O_CREAT | libc::O_EXCL, self.perm.0) } {
            -1 => Err(ErrCode::last_error()),
            fd => Ok(Handle(fd)),
        }
    }

    fn shm_open(&self) -> Result<Handle, ErrCode> {
        match unsafe { libc::shm_open(self.name.as_ptr(), P::O_RD, 0) } {
            -1 => Err(ErrCode::last_error()),
            fd => Ok(Handle(fd)),
        }
    }

    fn mapping(self, handle: Handle) -> io::Result<MappedRegion<P>> {
        Ok(MappedRegion::new(handle, self.mode, self.flags, self.size, self.offset)?)
    }

    pub fn remov(self) -> bool {
        unsafe { libc::shm_unlink(self.name.as_ptr()) == 0 }
    }
}

#[derive(Clone)]
pub struct XsiKey(libc::key_t);

impl XsiKey {
    pub fn private() -> Self {
        XsiKey(libc::IPC_PRIVATE)
    }

    pub fn new(name: &CStr, id: u8) -> io::Result<Self> {
        if id == 0 {
            return Err(INVALID_ARGUMENT.into());
        }

        match unsafe { libc::ftok(name.as_ptr(), id as i32) } {
            -1 => Err(ErrCode::last_error().into()),
            key => Ok(XsiKey(key)),
        }
    }
}

pub struct XsiSharedMemory<P> {
    key: XsiKey,
    mode: P,
    perm: Perm,
    size: usize,
}

impl<P: Privilege> XsiSharedMemory<P> {
    pub fn new(key: XsiKey, mode: P) -> Self {
        XsiSharedMemory {
            key: key,
            mode: mode,
            perm: Perm(0o644),
            size: 0,
        }
    }

    pub fn create(self) -> io::Result<MappedRegion<P>> {
        let shmid = self.xsi_create()?;
        Ok(self.xsi_at(shmid)?)
    }

    pub fn open(mut self) -> io::Result<MappedRegion<P>> {
        let shmid = self.xsi_open()?;
        self.size = self.xsi_size(shmid)?;
        Ok(self.xsi_at(shmid)?)
    }

    pub fn open_or_create(mut self) -> io::Result<MappedRegion<P>> {
        let shmid = loop {
            match self.xsi_create() {
                Err(ec) if ec == FILE_EXISTS => {}, // continue
                otherwise => break otherwise?,
            }
            match self.xsi_open() {
                Err(ec) if ec == NO_SUCH_FILE_OR_DIRECTORY => {}, // continue
                otherwise => break otherwise?,
            }
        };
        self.size = self.xsi_size(shmid)?;
        Ok(self.xsi_at(shmid)?)
    }

    fn xsi_at(self, shmid: i32) -> Result<MappedRegion<P>, ErrCode> {
        let base = unsafe { libc::shmat(shmid, ptr::null(), P::SHMFLG) };
        if base == (usize::max_value() as *mut libc::c_void) {
            return Err(ErrCode::last_error());
        }
        Ok(MappedRegion {
            base: base,
            size: self.size,
            page_offset: 0,
            is_xsi: true,
            mode: PhantomData,
        })
    }

    pub  fn remvove(self) -> bool {
        if let Ok(shmid)= self.xsi_open() {
            unsafe { libc::shmctl(shmid, libc::IPC_RMID, ptr::null_mut()) == 0 }
        } else {
            false
        }
    }

    fn xsi_create(&self) -> Result<i32, ErrCode> {
        match unsafe { libc::shmget(self.key.0, self.size, 0x0644 | libc::IPC_CREAT | libc::IPC_EXCL) } {
            -1 => Err(ErrCode::last_error()),
            shmid => Ok(shmid),
        }
    }

    fn xsi_open(&self) -> Result<i32, ErrCode> {
        match unsafe { libc::shmget(self.key.0, 0, 0) } {
            -1 => Err(ErrCode::last_error()),
            shmid => Ok(shmid),
        }
    }

    fn xsi_size(&self, shmid: i32) -> Result<usize, ErrCode> {
        let mut shm: libc::shmid_ds = unsafe { ::std::mem::uninitialized() };
        match unsafe { libc::shmctl(shmid, libc::IPC_STAT, &mut shm) } {
            -1 => Err(ErrCode::last_error()),
            _ => Ok(shm.shm_segsz as usize),
        }
    }
}
