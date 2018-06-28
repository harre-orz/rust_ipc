
#[macro_use]
extern crate bitflags;

extern crate libc;

pub struct ReadOnly;

pub struct ReadWrite;

pub struct CopyOnWrite;

pub struct ReadPrivate;

mod ffi;
pub use self::ffi::{Perm};

mod shared_memory;
pub use self::shared_memory::*;
