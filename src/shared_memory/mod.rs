use super::{ReadOnly, ReadWrite};
use ffi::*;

use std::ffi::CString;

pub struct SharedMemoryObject<Mode> {
    handle: FileHandle,
    name: CString,
    mode: Mode,
}

#[cfg(unix)]
mod posix;
