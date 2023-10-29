use {
    super::vec::ArrayVec,
    std::{
        borrow::{self, Cow},
        cmp,
        ffi::{OsStr, OsString},
        fmt, hash, ops,
        path::{Path, PathBuf},
    },
};

/// A type that can represent owned, mutable platform-native strings, but is
/// cheaply inter-convertible with Rust strings.
pub struct ArrayOsString<const N: usize> {
    inner: ArrayVec<u8, N>,
}

impl<const N: usize> ArrayOsString<N> {
    /// Constructs a new empty `ArrayOsString`.
    ///
    /// # Examples
    ///
    /// ```
    /// use mocha_std::array::os_string::ArrayOsString;
    ///
    /// let os_string = ArrayOsString::<3>::new();
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Self {
            inner: ArrayVec::new(),
        }
    }

    /// Converts to an [`OsStr`] slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use mocha_std::array::os_string::ArrayOsString;
    /// use std::ffi::OsStr;
    ///
    /// let mut os_string = ArrayOsString::<3>::new();
    /// let os_str = OsStr::new("foo");
    ///
    /// os_string.push("foo").unwrap();
    ///
    /// assert_eq!(os_string.as_os_str(), os_str);
    /// ```
    #[inline]
    pub fn as_os_str(&self) -> &OsStr {
        self
    }

    /// Converts the `OsString` into a byte slice.  To convert the byte slice back into an
    /// `OsString`, use the [`OsStr::from_encoded_bytes_unchecked`] function.
    #[inline]
    pub fn into_encoded_bytes(self) -> ArrayVec<u8, N> {
        self.inner
    }

    /// Extends the string with the given <code>&[OsStr]</code> slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use mocha_std::array::os_string::ArrayOsString;
    /// use std::ffi::OsStr;
    ///
    /// let mut os_string = ArrayOsString::<6>::try_from("foo").unwrap();
    ///
    /// os_string.push("bar").unwrap();
    ///
    /// assert_eq!(&os_string, "foobar");
    /// ```
    #[inline]
    pub fn push<T: AsRef<OsStr>>(&mut self, string: T) -> Result<(), ()> {
        self.inner
            .extend_from_slice(string.as_ref().as_encoded_bytes())
    }

    /// Truncates the `ArrayOsString` to zero length.
    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Returns the capacity this `ArrayOsString` can hold.
    #[inline]
    pub fn capacity(&self) -> usize {
        N
    }
}

impl<const N: usize> AsRef<OsStr> for ArrayOsString<N> {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        self
    }
}

impl<const N: usize> AsRef<Path> for ArrayOsString<N> {
    #[inline]
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl<const N: usize> Default for ArrayOsString<N> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Eq for ArrayOsString<N> {}

impl<const N: usize> Ord for ArrayOsString<N> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        (&**self).cmp(&**other)
    }
}

impl<const N: usize> PartialEq for ArrayOsString<N> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        (&**self) == (&**other)
    }
}

impl<const N: usize> PartialOrd for ArrayOsString<N> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        (&**self).partial_cmp(&**other)
    }
}

macro_rules! impl_cmp {
    ($($ty:ty),* $(,)?) => {$(
        impl<const N: usize> PartialEq<$ty> for ArrayOsString<N> {
            #[inline]
            fn eq(&self, other: &$ty) -> bool {
                (&**self) == other
            }
        }

        impl<const N: usize> PartialOrd<$ty> for ArrayOsString<N> {
            #[inline]
            fn partial_cmp(&self, other: &$ty) -> Option<cmp::Ordering> {
                (&**self).partial_cmp(other)
            }
        }

        impl<const N: usize> PartialEq<ArrayOsString<N>> for $ty {
            #[inline]
            fn eq(&self, other: &ArrayOsString<N>) -> bool {
                (&**other) == self
            }
        }

        impl<const N: usize> PartialOrd<ArrayOsString<N>> for $ty {
            #[inline]
            fn partial_cmp(&self, other: &ArrayOsString<N>) -> Option<cmp::Ordering> {
                (&**other).partial_cmp(self)
            }
        }
    )*};
}

impl_cmp! {
    OsStr,
    OsString,
    Path,
    PathBuf,
    Cow<'_, OsStr>,
    Cow<'_, Path>,
}

impl<const N: usize> borrow::Borrow<OsStr> for ArrayOsString<N> {
    #[inline]
    fn borrow(&self) -> &OsStr {
        self
    }
}

impl<const N: usize> fmt::Debug for ArrayOsString<N> {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, fmt)
    }
}

impl<const N: usize> fmt::Write for ArrayOsString<N> {
    #[inline]
    fn write_str(&mut self, string: &str) -> fmt::Result {
        match self.push(string) {
            Ok(()) => Ok(()),
            Err(()) => Err(fmt::Error),
        }
    }
}

impl<const N: usize> hash::Hash for ArrayOsString<N> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        (&**self).hash(state)
    }
}

impl<const N: usize> ops::Deref for ArrayOsString<N> {
    type Target = OsStr;

    #[inline]
    fn deref(&self) -> &Self::Target {
        // SAFETY: `self.inner` contains encoded bytes from an `OsStr`.
        unsafe { OsStr::from_encoded_bytes_unchecked(&self.inner) }
    }
}

impl<const N: usize> ops::DerefMut for ArrayOsString<N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY:
        // - `self.inner` contains encoded bytes from an `OsStr`.
        // - There is no other way to construct a mutable `&mut OsStr`.
        #[allow(invalid_reference_casting)]
        unsafe {
            &mut *(self.as_os_str() as *const OsStr as *mut OsStr)
        }
    }
}

impl<const N: usize> ops::Index<ops::RangeFull> for ArrayOsString<N> {
    type Output = OsStr;

    #[inline]
    fn index(&self, index: ops::RangeFull) -> &Self::Output {
        let _index = index;

        self
    }
}

impl<const N: usize> ops::IndexMut<ops::RangeFull> for ArrayOsString<N> {
    #[inline]
    fn index_mut(&mut self, index: ops::RangeFull) -> &mut Self::Output {
        let _index = index;

        self
    }
}
