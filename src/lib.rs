extern crate libc;

pub struct ReadOnly;

pub struct ReadWrite;

mod err;

mod ptr;

pub mod mem_algo;

mod mapped_region;
pub use self::mapped_region::*;

// mod managed_mapped_file;
// pub use self::managed_mapped_file::ManagedMappedFile;
