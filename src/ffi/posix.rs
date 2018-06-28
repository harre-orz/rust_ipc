#![allow(dead_code)]

use std::io;
use libc;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ErrCode(i32);

impl ErrCode {
    pub fn last_error() -> Self {
        ErrCode(unsafe { *libc::__errno_location() })
    }
}

impl From<ErrCode> for io::Error {
    fn from(ec: ErrCode) -> Self {
        io::Error::from_raw_os_error(ec.0)
    }
}

pub const FILE_EXISTS: ErrCode = ErrCode(libc::EEXIST);

pub const NO_SUCH_FILE_OR_DIRECTORY: ErrCode = ErrCode(libc::ENOENT);

pub type FileHandle = i32;

pub struct Perm;

impl Into<u32> for Perm {
    fn into(self) -> u32 {
        0644
    }
}
