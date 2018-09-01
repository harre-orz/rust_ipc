#![allow(dead_code)]
use std::io;
use std::fmt;
use libc;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct ErrCode(i32);

impl ErrCode {
    pub fn last_error() -> Self {
        ErrCode(unsafe { *libc::__errno_location() })
    }
}

impl fmt::Debug for ErrCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = unsafe { ::std::ffi::CStr::from_ptr(libc::strerror(self.0)) };
        write!(f, "{}", s.to_str().expect("Utf8Error"))
    }
}

impl From<ErrCode> for io::Error {
    fn from(ec: ErrCode) -> Self {
        io::Error::from_raw_os_error(ec.0)
    }
}

pub const SUCCESS: ErrCode = ErrCode(0);
pub const FILE_EXISTS: ErrCode = ErrCode(libc::EEXIST);
pub const PERMISSION_DENIED: ErrCode = ErrCode(libc::EACCES);
pub const NO_SUCH_FILE_OR_DIRECTORY: ErrCode = ErrCode(libc::ENOENT);
pub const INVALID_ARGUMENT: ErrCode = ErrCode(libc::EINVAL);
