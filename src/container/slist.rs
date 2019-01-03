use alloc::Allocator;
use intrusive::Adapter;
use intrusive::slist;
use container::Data;
use std::marker::PhantomData;

pub struct Inner<T, A: Allocator<Self>> {
    data: T,
    link: slist::Link<Adapt<T, A>>,
}

struct Adapt<T, A> {
    alloc: A,
    _marker: PhantomData<T>,
}

impl<T, A: Allocator<Inner<T, A>>> Adapter for Adapt<T, A> {
    type Link = slist::Link<Self>;
    type Value = Inner<T, A>;
    type Pointer = A::Pointer;
    type Size = usize;

    fn get_link(value: &Self::Value) -> *const Self::Link {
        &value.link
    }
}

pub struct SinglyLinkedList<T, A: Allocator<Inner<T, A>>> {
    slist: Box<slist::SinglyLinkedList<Adapt<T, A>>>,
    _marker: PhantomData<T>,
}

impl<T: Data, A: Allocator<Inner<T, A>>> SinglyLinkedList<T, A> {
    pub fn new(alloc: A) -> Self {
        SinglyLinkedList {
            slist: Box::new(slist::SinglyLinkedList::new(Adapt {
                alloc: alloc,
                _marker: PhantomData,
            })),
            _marker: PhantomData,
        }
    }

    pub fn push_front(&mut self, value: T) {
    }

    pub fn pop_front(&mut self) -> Option<T> {
        None
    }
}
