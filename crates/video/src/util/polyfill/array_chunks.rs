use core::{
    iter::FusedIterator,
    marker::PhantomData,
    slice::{ChunksExact},
};

/// An iterator over a slice in (non-overlapping) chunks (`N` elements at a
/// time), starting at the beginning of the slice.
///
/// When the slice len is not evenly divided by the chunk size, the last
/// up to `N-1` elements will be omitted but can be retrieved from
/// the [`remainder`] function from the iterator.
///
/// This struct is created by the [`array_chunks`] method on [slices].
///
/// [`array_chunks`]: slice::array_chunks
/// [`remainder`]: ArrayChunks::remainder
/// [slices]: slice
#[derive(Debug)]
pub struct ArrayChunks<'a, T, const N: usize> {
    iter: ChunksExact<'a, T>,
    _chunk: PhantomData<&'a [T; N]>,
}

impl<'a, T, const N: usize> ArrayChunks<'a, T, N> {
    #[inline]
    pub(crate) fn new(slice: &'a [T]) -> Self {
        Self {
            iter: slice.chunks_exact(N),
            _chunk: PhantomData,
        }
    }

    /// Returns the remainder of the original slice that is not going to be
    /// returned by the iterator. The returned slice has at most `N-1`
    /// elements.
    #[inline]
    #[must_use]
    pub fn remainder(&self) -> &'a [T] {
        self.iter.remainder()
    }
}

impl<'a, T, const N: usize> Iterator for ArrayChunks<'a, T, N> {
    type Item = &'a [T; N];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|slice| unsafe { as_array(slice) })
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
        self.iter.nth(n).map(|slice| unsafe { as_array(slice) })
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        self.iter.last().map(|slice| unsafe { as_array(slice) })
    }
}

impl<'a, T, const N: usize> DoubleEndedIterator for ArrayChunks<'a, T, N> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter
            .next_back()
            .map(|slice| unsafe { as_array(slice) })
    }

    #[inline]
    fn nth_back(&mut self, _n: usize) -> Option<Self::Item> {
        self.iter
            .next_back()
            .map(|slice| unsafe { as_array(slice) })
    }
}

impl<'a, T, const N: usize> ExactSizeIterator for ArrayChunks<'a, T, N> {
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a, T, const N: usize> FusedIterator for ArrayChunks<'a, T, N> {}

/// Convert the provided slice of length `N` to an array.
///
/// # Safety
///
/// The length of the provided slice must match `N`.
#[inline(always)]
unsafe fn as_array<T, const N: usize>(slice: &[T]) -> &[T; N] {
    debug_assert_eq!(slice.len(), N, "slice length is not N");

    TryFrom::try_from(slice).unwrap_unchecked()
}
