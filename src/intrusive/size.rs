use std::iter::Iterator;

pub trait Size {
    fn new() -> Self;

    fn count<T>(&self, it: T) -> usize
        where T: Iterator;

    fn add(&mut self, count: usize);

    fn sub(&mut self, count: usize);

    fn inc(&mut self) {
        self.add(1)
    }

    fn dec(&mut self) {
        self.sub(1)
    }
}

impl Size for usize {
    fn new() -> Self {
        0
    }

    fn count<T>(&self, _: T) -> usize
        where T: Iterator,
    {
        *self
    }

    fn add(&mut self, count: usize) {
        *self += count;
    }

    fn sub(&mut self, count: usize) {
        *self -= count;
    }
}


pub struct NotIncludeSize;
impl Size for NotIncludeSize {
    fn new() -> Self {
        NotIncludeSize
    }

    fn count<T>(&self, it: T) -> usize
        where T: Iterator,
    {
        it.count()
    }

    fn add(&mut self, _: usize) {
    }

    fn sub(&mut self, _: usize) {
    }
}
