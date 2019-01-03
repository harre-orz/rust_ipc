use sync::Mutex;
use mem_algo::MemAlgo;
use std::ptr;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::marker::PhantomData;

pub fn align<T>(addr: *const T, offset: usize) -> *mut T {
    let align = mem::align_of::<T>();
    (((addr as usize + offset - 1) / align + 1) * align) as *mut T
}

struct OffsetCell<T> {
    offset: usize,
    _marker: PhantomData<T>,
}

impl<T> OffsetCell<T> {
    pub fn set(&mut self, ptr: *const T) {
        let this = self as *const Self as usize;
        self.offset = ptr as usize - this;
    }
}

impl<T> Deref for OffsetCell<T> {
    type Target = T;

    fn deref(&self) -> &T {
        let this = self as *const Self as usize;
        unsafe { &*((self.offset + this) as *const T) }
    }
}

impl<T> DerefMut for OffsetCell<T> {
    fn deref_mut(&mut self) -> &mut T {
        let this = self as *const Self as usize;
        unsafe { &mut *((self.offset + this) as *mut T) }
    }
}

const BLOCK_CTRL_ALIGNMENT: usize = 16;

struct BlockCtrl {
    next: OffsetCell<BlockCtrl>,
    size: usize,
}

pub struct SimpleSeqFit<M> {
    root: BlockCtrl,
    allocate_size: usize,
    mutex: M,
}

impl<M> SimpleSeqFit<M>
    where M: Mutex
{
    fn place_new(&mut self, segment_bytes: usize) {
        self.mutex.place_new();

        let root = &self.root as *const BlockCtrl;
        let block1 = align(root, mem::align_of::<Self>());
        self.allocate_size = 0;
        self.root.size = segment_bytes;
        self.root.next.set(block1);
        self.root.next.size = 0;
        self.root.next.next.set(root);
    }

    fn sanity_check(&self) -> bool {
        let root = &self.root as *const BlockCtrl;
        let mut cur = &*self.root.next as *const BlockCtrl;

        if cur < root {
            return false
        }

        let last = (root as usize + self.root.size) as *const BlockCtrl;
        if cur > last {
            return false
        }

        while cur != root {
            let next = unsafe { &*(*cur).next } as *const BlockCtrl;
            if next > last {
                return false
            }
            if next < cur {
                return false
            }
            cur = next;
        }

        return true
    }
}

impl<M> MemAlgo for SimpleSeqFit<M>
    where M: Mutex
{
    fn alloc<T>(&mut self, size: usize) -> *mut T {
        let mut cur = &mut *self.root.next as *mut BlockCtrl;
        let mut prev = &mut self.root as *mut BlockCtrl;
        let root = prev;
        ptr::null_mut()
    }

    fn dealloc<T>(&mut self, ptr: *mut T, size: usize) {
    }

    fn realloc<T>(&mut self, ptr: *mut T, size: usize) -> *mut T{
        ptr::null_mut()
    }
}

// use super::*;
// use ptr::{RawCell, RawPtr, OffsetPtr};
// //use sync::Mutex;
// use std::mem;
// use std::ptr;
// use libc;
//
//
//
// struct BlockCtrl {
//     next: OffsetPtr<BlockCtrl>,
//     free_size: usize,
// }
//
// pub struct SimpleSeqFit {
//     root: BlockCtrl,
//     segment_bytes: usize,
//     allocated_bytes: usize,
// }
//
// impl SimpleSeqFit {
//     fn dump(&self) {
//         // let root = self.root.get();
//         // let mut block = self.root.next.get();
//         // print!("[  ");
//         // while (block > root) {
//         //     print!("offset:{} size:{}  ", block - root, block.size);
//         //     assert!(block != block.next.get(), "block {} != block.next {}", block, block.next.get());
//         //     block = block.next.get();
//         // }
//         // println!("]");
//     }
// }
//
// impl MemAlgo for SimpleSeqFit {
//     fn init(&mut self, segment_bytes: usize) {
//         self.segment_bytes = segment_bytes;
//         self.allocated_bytes = 0;
//         self.root.size = 0;
//
//         // let root = self.root.get();
//         // let block1 = root.offset(mem::size_of::<Self>()).align::<BlockCtrl>();
//         // self.root.next.set(block1);
//         // self.root.next.next.set(root);
//         // self.root.next.size = segment_bytes - (block1 - root);
//     }
//
//     fn alloc<T>(&mut self, size: usize) -> *mut T {
//         assert!(mem::align_of::<T>() <= BLOCK_CTRL_ALIGNMENT);
//
//         let root = self.root.get();
//         let mut prev = root;
//         let mut block = self.root.next.get();
//         while block != root {
//             let alloc_ptr = block.offset(BLOCK_CTRL_ALIGNMENT).align::<T>();
//             let alloc_size = alloc_ptr + (mem::size_of::<T>() * size) - block;
//             if alloc_size <= block.size {
//                 let mut next = (block + alloc_size).align::<BlockCtrl>();
//                 let remain_size = block.size - alloc_size;
//                 prev.next.size = 0;
//                 if remain_size > BLOCK_CTRL_ALIGNMENT {
//                     prev.next.set(next);
//                     prev.next.size = remain_size;
//                     prev.next.next.set(block.next.get());
//                 } else {
//                     prev.next.set(block.next.get());
//                 }
//                 self.allocated_bytes += alloc_size;
//                 return alloc_ptr.into();
//             }
//             prev = block;
//             block = block.next.get();
//         }
//         ptr::null_mut()
//     }
//
//     fn dealloc<T>(&mut self, ptr: *mut T, size: usize) {
//         assert!(mem::align_of::<T>() <= BLOCK_CTRL_ALIGNMENT);
//
//         let ptr = (RawPtr::new(ptr) - BLOCK_CTRL_ALIGNMENT).align::<BlockCtrl>();
//         let root = self.root.get();
//         let mut prev = root;
//         let mut block = self.root.next.get();
//         while block != root {
//             if prev < ptr && ptr < block {
//                 let next = block.next.get();
//                 prev.next.set(ptr);
//                 prev.next.next.set(block);
//                 prev.next.size = BLOCK_CTRL_ALIGNMENT + mem::size_of::<T>() * size;
//                 return;
//             }
//             prev = block;
//             block = block.next.get();
//         }
//     }
//
//     fn realloc<T>(&mut self, ptr: *mut T, size: usize) -> *mut T {
//         ptr::null_mut()
//     }
// }
//
// // #[test]
// // fn test_align() {
// //     assert_eq!(align(1 as *const i32), 4 as *const i32);
// //     assert_eq!(align(4 as *const i32), 4 as *const i32);
// //     assert_eq!(align(100 as *const f64), 104 as *const f64);
// //     assert_eq!(align(101 as *const f64), 104 as *const f64);
// //
// //     struct Align8 {
// //         x: i32,
// //         y: i64,
// //         z: f64,
// //     }
// //     assert_eq!(align(100 as *const Align8), 104 as *const Align8);
// //
// //     struct Align4 {
// //         x: [u8; 8],
// //         y: i32,
// //     }
// //     assert_eq!(align(100 as *const Align4), 100 as *const Align4);
// //
// //     struct Align0;
// //     assert_eq!(align(101 as *const Align0), 101 as *const Align0);
// // }
//
// #[test]
// fn test_alloc() {
//     let base: &mut SimpleSeqFit = unsafe { &mut *(libc::malloc(4096) as *mut _) };
//     base.init(4096);
//     base.dump();
//
//     let x = base.alloc::<i32>(1);
//     println!("x = {:p}", x);
//     base.dump();
//
//     let y = base.alloc::<i32>(1);
//     println!("y = {:p}", y);
//     base.dump();
//
//     base.dealloc(y, 1);
//     base.dump();
//
//     base.dealloc(x, 1);
//     base.dump();
//
//     let a = base.alloc::<i32>(1000);
//     base.dump();
//
//     unsafe { libc::free(base as *mut _ as *mut _) };
// }
