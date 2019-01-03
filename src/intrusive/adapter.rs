use ptr::Pointer;
use intrusive::size::Size;

pub trait Adapter {
    type Link;
    type Value;
    type Pointer: Pointer<Self::Value>;
    type Size: Size;

    fn get_link(value: &Self::Value) -> *const Self::Link;
}
