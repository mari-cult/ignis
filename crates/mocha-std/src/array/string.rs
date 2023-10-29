use super::vec::ArrayVec;

pub struct ArrayString<const N: usize> {
    inner: ArrayVec<u8, N>,
}

impl<const N: usize> ArrayString<N> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            inner: ArrayVec::new(),
        }
    }
}
