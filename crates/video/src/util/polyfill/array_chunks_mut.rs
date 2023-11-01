use core::{
    iter::FusedIterator,
    marker::PhantomData,
    slice::{ChunksExactMut},
};

/// An iterator over a slice in (non-overlapping) mutable chunks (`N` elements
/// at a time), starting at the beginning of the slice.
///
/// When the slice len is not evenly divided by the chunk size, the last
/// up to `N-1` elements will be omitted but can be retrieved from
/// the [`into_remainder`] function from the iterator.
///
/// This struct is created by the [`array_chunks_mut`] method on [slices].
///
/// [`array_chunks_mut`]: slice::array_chunks_mut
/// [`into_remainder`]: ArrayChunksMut::into_remainder
/// [slices]: slice
#[derive(Debug)]
pub struct ArrayChunksMut<'a, T, const N: usize> {
    iter: ChunksExactMut<'a, T>,
    _chunk: PhantomData<&'a mut [T; N]>,
}

impl<'a, T, const N: usize> ArrayChunksMut<'a, T, N> {
    #[inline]
    pub(crate) fn new(slice: &'a mut [T]) -> Self {
        Self {
            iter: slice.chunks_exact_mut(N),
            _chunk: PhantomData,
        }
    }

    /// Returns the remainder of the original slice that is not going to be
    /// returned by the iterator. The returned slice has at most `N-1`
    /// elements.
    #[inline]
    #[must_use]
    pub fn into_remainder(self) -> &'a mut [T] {
        self.iter.into_remainder()
    }
}

impl<'a, T, const N: usize> Iterator for ArrayChunksMut<'a, T, N> {
    type Item = &'a mut [T; N];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|slice| unsafe { as_array_mut(slice) })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.iter.count()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth(n).map(|slice| unsafe { as_array_mut(slice) })
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        self.iter.last().map(|slice| unsafe { as_array_mut(slice) })
    }
}

impl<'a, T, const N: usize> DoubleEndedIterator for ArrayChunksMut<'a, T, N> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter
            .next_back()
            .map(|slice| unsafe { as_array_mut(slice) })
    }

    #[inline]
    fn nth_back(&mut self, _n: usize) -> Option<Self::Item> {
        self.iter
            .next_back()
            .map(|slice| unsafe { as_array_mut(slice) })
    }
}

impl<'a, T, const N: usize> ExactSizeIterator for ArrayChunksMut<'a, T, N> {
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a, T, const N: usize> FusedIterator for ArrayChunksMut<'a, T, N> {}

/// Convert the provided slice of length `N` to an array.
///
/// # Safety
///
/// The length of the provided slice must match `N`.
#[inline(always)]
unsafe fn as_array_mut<T, const N: usize>(slice: &mut [T]) -> &mut [T; N] {
    debug_assert_eq!(slice.len(), N, "slice length is not N");

    TryFrom::try_from(slice).unwrap_unchecked()
}
