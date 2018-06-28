use super::*;

use std::ffi::CStr;
use std::io;

use libc;

#[allow(non_camel_case_types)]
pub trait ReadOnly_ReadWrite {
    const OFLAGS: i32;
}

impl ReadOnly_ReadWrite for ReadOnly {
    const OFLAGS: i32 = libc::O_RDONLY;
}

impl ReadOnly_ReadWrite for ReadWrite {
    const OFLAGS: i32 = libc::O_RDWR;
}

fn shm_open<T: ReadOnly_ReadWrite>(name: &CStr, _: &T) -> Result<FileHandle, ErrCode> {
    match unsafe { libc::shm_open(name.as_ptr(), T::OFLAGS, 0) } {
        -1 => Err(ErrCode::last_error().into()),
        handle => Ok(handle),
    }
}

fn shm_create<T: ReadOnly_ReadWrite>(name: &CStr, _: &T, perm: u32) -> Result<FileHandle, ErrCode> {
    match unsafe { libc::shm_open(name.as_ptr(), T::OFLAGS | libc::O_CREAT | libc::O_EXCL, perm) } {
        -1 => Err(ErrCode::last_error().into()),
        handle => {
            unsafe { libc::fchmod(handle, perm); }
            Ok(handle)
        }
    }
}

impl<T: ReadOnly_ReadWrite> SharedMemoryObject<T> {
    pub fn create(name: &CStr, mode: T, perm: Perm) -> io::Result<Self> {
        Ok(shm_create(name, &mode, perm.into()).map(|handle| SharedMemoryObject {
            handle: handle,
            mode: mode,
            name: name.into(),
        })?)
    }

    pub fn open(name: &CStr, mode: T) -> io::Result<Self> {
        Ok(shm_open(name, &mode).map(|handle| SharedMemoryObject {
            handle: handle,
            mode: mode,
            name: name.into(),
        })?)
    }

    pub fn open_or_create(name: &CStr, mode: T, perm: Perm) -> io::Result<Self> {
        let perm = perm.into();
        Ok(loop {
            match shm_create(name, &mode, perm) {
                Err(ec) if ec == FILE_EXISTS => {
                    // continue
                },
                res => break res,
            }
            match shm_open(name, &mode) {
                Err(ec) if ec == NO_SUCH_FILE_OR_DIRECTORY => {
                    // continue
                }
                res => break res,
            }
        }.map(|handle| SharedMemoryObject {
            handle: handle,
            mode: mode,
            name: name.into()
        })?)
    }
}

impl SharedMemoryObject<ReadWrite> {
    pub unsafe fn remove(name: &CStr) -> bool {
        libc::shm_unlink(name.as_ptr()) == 0
    }
}

#[test]
fn test_create_and_remove() {
    let name = CString::new("hoge").unwrap();
    unsafe { SharedMemoryObject::<ReadWrite>::remove(&name); }
    let obj = SharedMemoryObject::create(&name, ReadWrite, Perm).unwrap();
}
