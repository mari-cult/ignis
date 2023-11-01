//! Polyfills for unstable stdlib features.

pub use self::{array_chunks::ArrayChunks, array_chunks_mut::ArrayChunksMut};

mod array_chunks;
mod array_chunks_mut;

/// Polyfill trait for unstable slice features.
pub trait SlicePolyfill<T> {
    /// Returns an iterator over `N` elements of the slice at a time, starting at the
    /// beginning of the slice.
    ///
    /// The chunks are array references and do not overlap. If `N` does not divide the
    /// length of the slice, then the last up to `N-1` elements will be omitted and can be
    /// retrieved from the `remainder` function of the iterator.
    ///
    /// This method is the const generic equivalent of [`chunks_exact`]([T]::chunks_exact).
    ///
    /// # Panics
    ///
    /// Panics if `N` is 0. This check will most probably get changed to a compile time
    /// error before this method gets stabilized.
    fn array_chunks<const N: usize>(&self) -> ArrayChunks<'_, T, N>;

    /// Returns an iterator over `N` elements of the slice at a time, starting at the
    /// beginning of the slice.
    ///
    /// The chunks are mutable array references and do not overlap. If `N` does not divide
    /// the length of the slice, then the last up to `N-1` elements will be omitted and
    /// can be retrieved from the `into_remainder` function of the iterator.
    ///
    /// This method is the const generic equivalent of [`chunks_exact_mut`]([T]::chunks_exact_mut).
    ///
    /// # Panics
    ///
    /// Panics if `N` is 0. This check will most probably get changed to a compile time
    /// error before this method gets stabilized.
    fn array_chunks_mut<const N: usize>(&mut self) -> ArrayChunksMut<'_, T, N>;
}

impl<T> SlicePolyfill<T> for [T] {
    #[inline]
    #[track_caller]
    fn array_chunks<const N: usize>(&self) -> ArrayChunks<'_, T, N> {
        assert!(N != 0, "chunk size must be non-zero");

        ArrayChunks::new(self)
    }

    #[inline]
    #[track_caller]
    fn array_chunks_mut<const N: usize>(&mut self) -> ArrayChunksMut<'_, T, N> {
        assert!(N != 0, "chunk size must be non-zero");

        ArrayChunksMut::new(self)
    }
}
