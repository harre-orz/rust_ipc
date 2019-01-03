use ptr::Pointer;
use intrusive::{Adapter};
use intrusive::size::{Size};
use intrusive::ptr::{RefPtr, MutPtr};
use std::iter::Iterator;

pub struct Link<A: Adapter> {
    next: Option<A::Pointer>,
}

impl<A: Adapter> Link<A> {
    pub fn unlinked() -> Self {
        Link {
            next: None,
        }
    }

    unsafe fn unlink(&mut self) {
        self.next = None;
    }
}


pub struct SinglyLinkedList<A: Adapter> {
    head: Option<A::Pointer>,
    size: A::Size,
    _adapter: A,
}

impl<A: Adapter<Link = Link<A>>> SinglyLinkedList<A> {
    pub fn new(adapter: A) -> Self {
        SinglyLinkedList {
            head: None,
            size: A::Size::new(),
            _adapter: adapter,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    pub fn size(&self) -> usize {
        self.size.count(self.iter())
    }

    pub fn iter(&self) -> Iter<A> {
        Iter {
            cur: RefPtr::new(&self.head),
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<A> {
        IterMut {
            cur: MutPtr::new(&mut self.head),
        }
    }

    pub fn push_front(&mut self, value: &mut A::Value) {
        let new = A::get_link(value) as *mut A::Link;
        unsafe {
            let new = &mut *new;
            if let Some(next_ptr) = &mut self.head {
                A::Pointer::set_value(&mut new.next, next_ptr.as_mut());
            } else {
                new.next = None;
            }
            A::Pointer::set_value(&mut self.head, value);
            self.size.inc();
        }
    }

    pub fn pop_front(&mut self) -> Option<&mut A::Value> {
        let head = &self.head as *const _ as *mut _;
        if let Some(del_ptr) = &mut self.head {
            let val = del_ptr.as_mut() as *mut _;
            let del = A::get_link(del_ptr.as_ref()) as *mut A::Link;
            unsafe {
                let del = &mut *del;
                A::Pointer::clone_from(&mut *head, &mut del.next);
                del.unlink();
                self.size.dec();
                Some(&mut *val)
            }
        } else {
            None
        }
    }
}


/// Iter
pub struct Iter<'a, A: Adapter> {
    cur: RefPtr<'a, A::Value, A::Pointer>,
}

impl<'a, A: Adapter<Link = Link<A>> + 'a> Iterator for Iter<'a, A> {
    type Item = &'a A::Value;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(value) = self.cur.get() {
            let link = unsafe { &*(A::get_link(value)) };
            self.cur.set(&link.next);
            Some(value)
        } else {
            None
        }
    }
}


/// IterMut
pub struct IterMut<'a, A: Adapter> {
    cur: MutPtr<'a, A::Value, A::Pointer>,
}

impl<'a, A: Adapter<Link = Link<A>> + 'a> Iterator for IterMut<'a, A> {
    type Item = &'a mut A::Value;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(value) = self.cur.get() {
            let link = A::get_link(value) as *mut A::Link;
            let link = unsafe { &mut *link };
            self.cur.set(&mut link.next);
            Some(value)
        } else {
            None
        }
    }
}


#[test]
fn test_hoge() {
    use std::ptr::NonNull;

    struct My {
        data: i32,
        link: Link<Adapt>,
    }

    struct Adapt;
    impl Adapter for Adapt {
        type Link = Link<Adapt>;
        type Value = My;
        type Pointer = NonNull<My>;
        type Size = usize;

        fn get_link(value: &Self::Value) -> *const Self::Link {
            &value.link
        }
    }

    let mut slist = SinglyLinkedList::new(Adapt);
    assert_eq!(slist.is_empty(), true);

    let mut d1 = My {
        data: 100,
        link: Link::unlinked(),
    };
    slist.push_front(&mut d1);
    assert_eq!(slist.iter().nth(0).unwrap().data, 100);

    let mut d2 = My {
        data: 200,
        link: Link::unlinked(),
    };
    slist.push_front(&mut d2);
    assert_eq!(slist.iter().nth(0).unwrap().data, 200);
    assert_eq!(slist.iter().nth(1).unwrap().data, 100);

    assert_eq!(slist.pop_front().unwrap().data, 200);
    assert_eq!(slist.pop_front().unwrap().data, 100);
}
