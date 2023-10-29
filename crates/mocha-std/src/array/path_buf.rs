use {
    super::os_string::ArrayOsString,
    std::{fmt, ops, path::Path},
};

/// An owned, mutable path (akin to [`ArrayString`]).
pub struct ArrayPathBuf<const CAPACITY: usize> {
    inner: ArrayOsString<CAPACITY>,
}

impl<const CAPACITY: usize> ArrayPathBuf<CAPACITY> {
    /// Creates an empty `ArrayPathBuf`.
    #[inline]
    pub const fn new() -> Self {
        Self {
            inner: ArrayOsString::new(),
        }
    }

    /// Coerces to a [`Path`] slice.
    #[inline]
    pub fn as_path(&self) -> &Path {
        self
    }

    /// Yields a mutable reference to the underlying [`ArrayOsString`] instance.
    #[inline]
    pub fn as_mut_os_string(&mut self) -> &mut ArrayOsString<CAPACITY> {
        &mut self.inner
    }

    /// Consumes the `PathBuf`, yielding its internal [`ArrayOsString`] storage.
    #[inline]
    pub fn into_os_string(self) -> ArrayOsString<CAPACITY> {
        self.inner
    }

    /// Invokes [`clear`] on the underlying instance of [`ArrayOsString`].
    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear();
    }
}

impl<const CAPACITY: usize> fmt::Debug for ArrayPathBuf<CAPACITY> {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_path(), fmt)
    }
}

impl<const CAPACITY: usize> ops::Deref for ArrayPathBuf<CAPACITY> {
    type Target = Path;

    #[inline]
    fn deref(&self) -> &Self::Target {
        Path::new(&self.inner)
    }
}

impl<const CAPACITY: usize> ops::DerefMut for ArrayPathBuf<CAPACITY> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: guh
        #[allow(invalid_reference_casting)]
        unsafe {
            &mut *(self.as_path() as *const Path as *mut Path)
        }
    }
}
