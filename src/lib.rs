extern crate libc;

mod err;

pub mod ptr;

pub mod alloc;

pub mod intrusive;

pub mod container;

pub mod sync;

pub mod mem_algo;

pub mod indexes;

mod mapped_region;
pub use self::mapped_region::*;

pub struct SegmentManager<A, I> {
    region: mapped_region::MappedRegion,
    _marker: std::marker::PhantomData<(A, I)>,
}

mod segment_manager_impl;

mod managed_shared_memory;
pub use self::managed_shared_memory::*;

mod managed_mapped_file;
pub use self::managed_mapped_file::*;
