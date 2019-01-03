use mapped_region::{Advise, page_size, adjust_page_offset};
use err::{ErrCode, INVALID_ARGUMENT, PERMISSION_DENIED,
          FILE_EXISTS, NO_SUCH_FILE_OR_DIRECTORY};
use std::io;
use std::ptr;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use libc;

pub struct CopyOnWrite;
pub struct ReadOnly;
pub struct ReadPrivate;
pub struct ReadWrite;

/// Close socket on exit scope.
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

/// Unix permission compatible.
pub struct Perm(u32);

pub struct MappedRegion {
    base: *mut libc::c_void,
    size: usize,
    page_offset: usize,
    is_xsi: bool,
}

impl MappedRegion {
    fn new(mut size: usize, prot: i32, flags: i32, fd: Handle, offset: usize) -> io::Result<Self> {
        let page_offset = adjust_page_offset(offset);
        if size == 0 {
            size = fd.size()?;
            if size < offset {
                return Err(PERMISSION_DENIED.into());
            }
            size -= (offset - page_offset) as usize;
        }

        match unsafe {
            libc::mmap(ptr::null_mut(),
                       size + page_offset as usize,
                       prot,
                       flags,
                       fd.0,
                       (offset - page_offset) as i64) }
        {
            libc::MAP_FAILED => Err(ErrCode::last_error().into()),
            base => Ok(MappedRegion {
                base: unsafe { base.offset(page_offset as isize) },
                size: size,
                page_offset: page_offset,
                is_xsi: false,
            }),
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub unsafe fn base(&self) -> *mut libc::c_void {
        self.base
    }
}

impl Drop for MappedRegion {
    fn drop(&mut self) {
        unsafe {
            if self.is_xsi {
            } else {
                libc::munmap(self.base.offset(-(self.page_offset as isize)),
                             self.size + self.page_offset);
            }
        }
    }
}

pub struct FileMapping<P> {
    name: CString,
    perm: Perm,
    size: usize,
    offset: usize,
    file_flag: i32,
    mmap_flag: i32,
    mmap_prot: i32,
    mode: PhantomData<P>,
}

impl<P> FileMapping<P> {
    fn file_create(&self) -> Result<Handle, ErrCode> {
        match unsafe { libc::open(self.name.as_ptr(),
                                  self.file_flag | libc::O_CREAT | libc::O_EXCL,
                                  self.perm.0) }
        {
            -1 => Err(ErrCode::last_error()),
            fd => Ok(Handle(fd)),
        }
    }

    fn file_open(&self) -> Result<Handle, ErrCode> {
        match unsafe { libc::open(self.name.as_ptr(), self.file_flag) } {
            -1 => Err(ErrCode::last_error()),
            fd => Ok(Handle(fd)),
        }
    }

    pub fn create(self) -> io::Result<MappedRegion> {
        let fd = self.file_create()?;
        let file_size = self.size + self.offset;
        fd.truncate(file_size)?;
        MappedRegion::new(self.size, self.mmap_flag, self.mmap_prot, fd, self.offset)
    }

    pub fn open(self) -> io::Result<MappedRegion> {
        let fd = self.file_open()?;
        let file_size = self.size + self.offset;
        if (fd.size()?) < file_size {
            fd.truncate(file_size)?;
        }
        MappedRegion::new(self.size, self.mmap_flag, self.mmap_prot, fd, self.offset)
    }

    pub fn open_or_create(self) -> io::Result<MappedRegion> {
        let fd = loop {
            match self.file_create() {
                Ok(fd) => break fd,
                Err(ec) => if ec != FILE_EXISTS {
                    return Err(ec.into())
                },
            }
            match self.file_open() {
                Ok(fd) => break fd,
                Err(ec) => if ec != NO_SUCH_FILE_OR_DIRECTORY {
                    return Err(ec.into())
                },
            }
        };
        let file_size = self.size + self.offset;
        if (fd.size()?) < file_size {
            fd.truncate(file_size)?;
        }
        MappedRegion::new(self.size, self.mmap_flag, self.mmap_prot, fd, self.offset)
    }

    pub fn remove(self) -> bool {
        unsafe { libc::unlink(self.name.as_ptr()) == 0 }
    }

    pub fn offset(self, offset: usize) -> Self {
        FileMapping {
            name: self.name,
            perm: self.perm,
            size: self.size,
            offset: offset,
            file_flag: self.file_flag,
            mmap_flag: self.mmap_flag,
            mmap_prot: self.mmap_prot,
            mode: self.mode,
        }
    }

    pub fn permission(self, perm: Perm) -> Self {
        FileMapping {
            name: self.name,
            perm: perm,
            size: self.size,
            offset: self.offset,
            file_flag: self.file_flag,
            mmap_flag: self.mmap_flag,
            mmap_prot: self.mmap_prot,
            mode: self.mode,
        }
    }

    pub fn size(self, size: usize) -> Self {
        FileMapping {
            name: self.name,
            perm: self.perm,
            size: size,
            offset: self.offset,
            file_flag: self.file_flag,
            mmap_flag: self.mmap_flag,
            mmap_prot: self.mmap_prot,
            mode: self.mode,
        }
    }

    pub fn read_only(self) -> FileMapping<ReadOnly> {
        FileMapping {
            name: self.name,
            perm: self.perm,
            size: self.size,
            offset: self.offset,
            file_flag: libc::O_RDONLY,
            mmap_flag: libc::MAP_SHARED,
            mmap_prot: libc::PROT_READ,
            mode: PhantomData,
        }
    }

    pub fn read_write(self) -> FileMapping<ReadWrite> {
        FileMapping {
            name: self.name,
            perm: self.perm,
            size: self.size,
            offset: self.offset,
            file_flag: libc::O_RDWR,
            mmap_flag: libc::MAP_SHARED,
            mmap_prot: libc::PROT_READ | libc::PROT_WRITE,
            mode: PhantomData,
        }
    }

    pub fn read_private(self) -> FileMapping<ReadPrivate> {
            FileMapping {
                name: self.name,
                perm: self.perm,
                size: self.size,
                offset: self.offset,
                file_flag: libc::O_RDONLY,
            mmap_flag: libc::MAP_PRIVATE,
            mmap_prot: libc::PROT_READ,
            mode: PhantomData,
        }
    }

    pub fn copy_on_write(self) -> FileMapping<CopyOnWrite> {
        FileMapping {
            name: self.name,
            perm: self.perm,
            size: self.size,
            offset: self.offset,
            file_flag: libc::O_RDWR,
            mmap_flag: libc::MAP_PRIVATE,
            mmap_prot: libc::PROT_READ | libc::PROT_WRITE,
            mode: PhantomData,
        }
    }
}

pub fn file_mapping<T>(name: T) -> FileMapping<ReadWrite>
    where T: AsRef<str>
{
    FileMapping {
        name: CString::new(name.as_ref()).unwrap(),
        perm: Perm(0o644),
        size: 0,
        offset: 0,
        file_flag: libc::O_RDWR,
        mmap_flag: libc::MAP_SHARED,
        mmap_prot: libc::PROT_READ | libc::PROT_WRITE,
        mode: PhantomData,
    }
}

pub struct SharedMemory<P> {
    name: CString,
    perm: Perm,
    size: usize,
    offset: usize,
    shm_flag: i32,
    mmap_flag: i32,
    mmap_prot: i32,
    mode: PhantomData<P>,
}

impl<P> SharedMemory<P> {
    fn shm_create(&self) -> Result<Handle, ErrCode> {
        match unsafe { libc::shm_open(self.name.as_ptr(),
                                      self.shm_flag | libc::O_CREAT | libc::O_EXCL,
                                      self.perm.0) }
        {
            -1 => Err(ErrCode::last_error()),
            fd => Ok(Handle(fd)),
        }
    }

    fn shm_open(&self) -> Result<Handle, ErrCode> {
        match unsafe { libc::shm_open(self.name.as_ptr(), self.shm_flag, 0) } {
            -1 => Err(ErrCode::last_error()),
            fd => Ok(Handle(fd)),
        }
    }

    pub fn create(self) -> io::Result<MappedRegion> {
        let fd = self.shm_create()?;
        let shm_size = self.size + self.offset;
        fd.truncate(shm_size)?;
        MappedRegion::new(self.size, self.mmap_flag, self.mmap_prot, fd, self.offset)
    }

    pub fn open(self) -> io::Result<MappedRegion> {
        let fd = self.shm_open()?;
        let shm_size = self.size + self.offset;
        if (fd.size()?) < shm_size {
            fd.truncate(shm_size)?;
        }
        MappedRegion::new(self.size, self.mmap_flag, self.mmap_prot, fd, self.offset)
    }

    pub fn open_or_create(self) -> io::Result<MappedRegion> {
        let fd = loop {
            match self.shm_create() {
                Ok(fd) => break fd,
                Err(ec) => if ec != FILE_EXISTS {
                    return Err(ec.into())
                },
            }
            match self.shm_open() {
                Ok(fd) => break fd,
                Err(ec) => if ec != NO_SUCH_FILE_OR_DIRECTORY {
                    return Err(ec.into())
                },
            }
        };
        let shm_size = self.size + self.offset;
        if (fd.size()?) < shm_size {
            fd.truncate(shm_size)?;
        }
        MappedRegion::new(self.size, self.mmap_flag, self.mmap_prot, fd, self.offset)
    }

    pub fn remove(self) -> bool {
        unsafe { libc::shm_unlink(self.name.as_ptr()) == 0 }
    }

    pub fn offset(self, offset: usize) -> Self {
        SharedMemory {
            name: self.name,
            perm: self.perm,
            size: self.size,
            offset: offset,
            shm_flag: self.shm_flag,
            mmap_flag: self.mmap_flag,
            mmap_prot: self.mmap_prot,
            mode: self.mode,
        }
    }

    pub fn permission(self, perm: Perm) -> Self {
        SharedMemory {
            name: self.name,
            perm: perm,
            size: self.size,
            offset: self.offset,
            shm_flag: self.shm_flag,
            mmap_flag: self.mmap_flag,
            mmap_prot: self.mmap_prot,
            mode: self.mode,
        }
    }

    pub fn size(self, size: usize) -> Self {
        SharedMemory {
            name: self.name,
            perm: self.perm,
            size: size,
            offset: self.offset,
            shm_flag: self.shm_flag,
            mmap_flag: self.mmap_flag,
            mmap_prot: self.mmap_prot,
            mode: self.mode,
        }
    }

    pub fn read_only(self) -> SharedMemory<ReadOnly> {
        SharedMemory {
            name: self.name,
            perm: self.perm,
            size: self.size,
            offset: self.offset,
            shm_flag: libc::O_RDONLY,
            mmap_flag: libc::MAP_SHARED,
            mmap_prot: libc::PROT_READ,
            mode: PhantomData,
        }
    }

    pub fn read_write(self) -> SharedMemory<ReadWrite> {
        SharedMemory {
            name: self.name,
            perm: self.perm,
            size: self.size,
            offset: self.offset,
            shm_flag: libc::O_RDWR,
            mmap_flag: libc::MAP_SHARED,
            mmap_prot: libc::PROT_READ | libc::PROT_WRITE,
            mode: PhantomData,
        }
    }

    pub fn read_private(self) -> SharedMemory<ReadOnly> {
        SharedMemory {
            name: self.name,
            perm: self.perm,
            size: self.size,
            offset: self.offset,
            shm_flag: libc::O_RDONLY,
            mmap_flag: libc::MAP_PRIVATE,
            mmap_prot: libc::PROT_READ,
            mode: PhantomData,
        }
    }

    pub fn copy_on_write(self) -> SharedMemory<ReadWrite> {
        SharedMemory {
            name: self.name,
            perm: self.perm,
            size: self.size,
            offset: self.offset,
            shm_flag: libc::O_RDWR,
            mmap_flag: libc::MAP_PRIVATE,
            mmap_prot: libc::PROT_READ | libc::PROT_WRITE,
            mode: PhantomData,
        }
    }
}

pub fn shared_memory<T>(name: T) -> SharedMemory<ReadWrite>
    where T: AsRef<str>
{
    SharedMemory {
        name: CString::new(name.as_ref()).unwrap(),
        perm: Perm(0o644),
        size: 0,
        offset: 0,
        shm_flag: 0,
        mmap_flag: libc::MAP_SHARED,
        mmap_prot: libc::PROT_READ | libc::PROT_WRITE,
        mode: PhantomData,
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
    size: usize,
    perm: Perm,
    shm_flag: i32,
    mode: PhantomData<P>,
}

impl<P> XsiSharedMemory<P> {
    pub fn create(self) -> io::Result<MappedRegion> {
        let shmid = self.xsi_create()?;
        Ok(MappedRegion {
            base: self.xsi_at(shmid)?,
            size: self.size,
            page_offset: 0,
            is_xsi: true,
        })
    }

    pub fn open(mut self) -> io::Result<MappedRegion> {
        let shmid = self.xsi_open()?;
        self.size = self.xsi_size(shmid)?;
        Ok(MappedRegion {
            base: self.xsi_at(shmid)?,
            size: self.size,
            page_offset: 0,
            is_xsi: true,
        })
    }

    pub fn open_or_create(mut self) -> io::Result<MappedRegion> {
        let shmid = loop {
            match self.xsi_create() {
                Ok(shmid) => break shmid,
                Err(ec) => if ec != FILE_EXISTS {
                    return Err(ec.into())
                },
            }
            match self.xsi_open() {
                Ok(shmid) => break shmid,
                Err(ec) => if ec != NO_SUCH_FILE_OR_DIRECTORY {
                    return Err(ec.into())
                },
            }
        };
        self.size = self.xsi_size(shmid)?;
        Ok(MappedRegion {
            base: self.xsi_at(shmid)?,
            size: self.size,
            page_offset: 0,
            is_xsi: true,
        })
    }

    pub fn remove(self) -> bool {
        if let Ok(shmid) = self.xsi_open() {
            if let Ok(base) = self.xsi_at(shmid) {
                unsafe { libc::shmdt(base) == 0 }
            } else {
                false
            }
        } else {
            false
        }
    }

    fn xsi_create(&self) -> Result<i32, ErrCode> {
        match unsafe { libc::shmget(self.key.0,
                                    self.size,
                                    self.perm.0 as i32 | libc::IPC_CREAT | libc::IPC_EXCL) }
        {
            -1 => Err(ErrCode::last_error()),
            shmid => Ok(shmid),
        }
    }

    fn xsi_open(&self) -> Result<i32, ErrCode> {
        match unsafe { libc::shmget(self.key.0,
                                    self.size,
                                    0) }
        {
            -1 => Err(ErrCode::last_error()),
            shmid => Ok(shmid),
        }
    }

    fn xsi_size(&self, shmid: i32) -> Result<usize, ErrCode> {
        use std::mem;
        let mut shm: libc::shmid_ds = unsafe { mem::uninitialized() };
        match unsafe { libc::shmctl(shmid, libc::IPC_STAT, &mut shm) } {
            -1 => Err(ErrCode::last_error()),
            _ => Ok(shm.shm_segsz as usize),
        }
    }

    fn xsi_at(&self, shmid: i32) -> Result<*mut libc::c_void, ErrCode> {
        let base = unsafe { libc::shmat(shmid, ptr::null(), self.shm_flag) };
        if base != (usize::max_value() as *mut libc::c_void) {
            Ok(base)
        } else {
            Err(ErrCode::last_error())
        }
    }

    pub fn read_only(self) -> XsiSharedMemory<ReadOnly> {
        XsiSharedMemory {
            key: self.key,
            size: self.size,
            perm: self.perm,
            shm_flag: libc::SHM_RDONLY,
            mode: PhantomData,
        }
    }

    pub fn read_write(self) -> XsiSharedMemory<ReadWrite> {
        XsiSharedMemory {
            key: self.key,
            size: self.size,
            perm: self.perm,
            shm_flag: 0,
            mode: PhantomData,
        }
    }
}

pub fn xsi_shared_memory(key: XsiKey) -> XsiSharedMemory<ReadWrite>
{
    XsiSharedMemory {
        key: key,
        size: 0,
        perm: Perm(0o644),
        shm_flag: 0,
        mode: PhantomData,
    }
}

pub fn anon_shared_memory(size: usize) -> io::Result<MappedRegion> {
    match unsafe { libc::mmap(ptr::null_mut(),
                              size,
                              libc::PROT_READ | libc::PROT_WRITE,
                              libc::MAP_ANONYMOUS | libc::MAP_SHARED,
                              -1,
                              0) }
    {
        libc::MAP_FAILED => Err(ErrCode::last_error().into()),
        base => Ok(MappedRegion {
            base: base,
            size: size,
            page_offset: 0,
            is_xsi: false,
        }),
    }
}
