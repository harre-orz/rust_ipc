
pub trait Data {
    fn data_move(&mut self, other: &mut Self);
}

impl<T: Copy> Data for T {
    fn data_move(&mut self, other: &mut Self) {
        *self = *other;
    }
}

pub mod slist;
