use super::*;
use ptr::{RawCell, RawPtr, OffsetPtr};
//use sync::Mutex;
use std::mem;
use std::ptr;
use libc;

const BLOCK_CTRL_ALIGNMENT: usize = 16;

struct BlockCtrl {
    next: OffsetPtr<BlockCtrl>,
    size: usize,
}

pub struct SimpleSeqFit {
    root: RawCell<BlockCtrl>,
    segment_bytes: usize,
    allocated_bytes: usize,
}

impl SimpleSeqFit {
    fn dump(&self) {
        let root = self.root.get();
        let mut block = self.root.next.get();
        print!("[  ");
        while (block > root) {
            print!("offset:{} size:{}  ", block - root, block.size);
            assert!(block != block.next.get(), "block {} != block.next {}", block, block.next.get());
            block = block.next.get();
        }
        println!("]");
    }
}

impl MemAlgo for SimpleSeqFit {
    fn init(&mut self, segment_bytes: usize) {
        self.segment_bytes = segment_bytes;
        self.allocated_bytes = 0;
        self.root.size = 0;

        let root = self.root.get();
        let block1 = root.offset(mem::size_of::<Self>()).align::<BlockCtrl>();
        self.root.next.set(block1);
        self.root.next.next.set(root);
        self.root.next.size = segment_bytes - (block1 - root);
    }

    fn alloc<T>(&mut self, size: usize) -> *mut T {
        assert!(mem::align_of::<T>() <= BLOCK_CTRL_ALIGNMENT);

        let root = self.root.get();
        let mut prev = root;
        let mut block = self.root.next.get();
        while block != root {
            let alloc_ptr = block.offset(BLOCK_CTRL_ALIGNMENT).align::<T>();
            let alloc_size = alloc_ptr + (mem::size_of::<T>() * size) - block;
            if alloc_size <= block.size {
                let mut next = (block + alloc_size).align::<BlockCtrl>();
                let remain_size = block.size - alloc_size;
                prev.next.size = 0;
                if remain_size > BLOCK_CTRL_ALIGNMENT {
                    prev.next.set(next);
                    prev.next.size = remain_size;
                    prev.next.next.set(block.next.get());
                } else {
                    prev.next.set(block.next.get());
                }
                self.allocated_bytes += alloc_size;
                return alloc_ptr.into();
            }
            prev = block;
            block = block.next.get();
        }
        ptr::null_mut()
    }

    fn dealloc<T>(&mut self, ptr: *mut T, size: usize) {
        assert!(mem::align_of::<T>() <= BLOCK_CTRL_ALIGNMENT);

        let ptr = (RawPtr::new(ptr) - BLOCK_CTRL_ALIGNMENT).align::<BlockCtrl>();
        let root = self.root.get();
        let mut prev = root;
        let mut block = self.root.next.get();
        while block != root {
            if prev < ptr && ptr < block {
                let next = block.next.get();
                prev.next.set(ptr);
                prev.next.next.set(block);
                prev.next.size = BLOCK_CTRL_ALIGNMENT + mem::size_of::<T>() * size;
                return;
            }
            prev = block;
            block = block.next.get();
        }
    }

    fn realloc<T>(&mut self, ptr: *mut T, size: usize) -> *mut T {
        ptr::null_mut()
    }
}

// #[test]
// fn test_align() {
//     assert_eq!(align(1 as *const i32), 4 as *const i32);
//     assert_eq!(align(4 as *const i32), 4 as *const i32);
//     assert_eq!(align(100 as *const f64), 104 as *const f64);
//     assert_eq!(align(101 as *const f64), 104 as *const f64);
//
//     struct Align8 {
//         x: i32,
//         y: i64,
//         z: f64,
//     }
//     assert_eq!(align(100 as *const Align8), 104 as *const Align8);
//
//     struct Align4 {
//         x: [u8; 8],
//         y: i32,
//     }
//     assert_eq!(align(100 as *const Align4), 100 as *const Align4);
//
//     struct Align0;
//     assert_eq!(align(101 as *const Align0), 101 as *const Align0);
// }

#[test]
fn test_alloc() {
    let base: &mut SimpleSeqFit = unsafe { &mut *(libc::malloc(4096) as *mut _) };
    base.init(4096);
    base.dump();

    let x = base.alloc::<i32>(1);
    println!("x = {:p}", x);
    base.dump();

    let y = base.alloc::<i32>(1);
    println!("y = {:p}", y);
    base.dump();

    base.dealloc(y, 1);
    base.dump();

    base.dealloc(x, 1);
    base.dump();

    let a = base.alloc::<i32>(1000);
    base.dump();

    unsafe { libc::free(base as *mut _ as *mut _) };
}
