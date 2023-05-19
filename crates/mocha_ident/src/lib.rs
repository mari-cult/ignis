use {
    camino::Utf8Path,
    std::{
        borrow, cmp::Ordering, convert, error, fmt, hash, mem::MaybeUninit, ops, ptr, slice, str,
    },
};

pub mod spec;

/// A stack-allocated read-only string.
#[derive(Clone, Copy)]
pub struct Ident<const CAPACITY: usize> {
    data: [MaybeUninit<u8>; CAPACITY],
    len: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct TryFromStrError(());

impl<const CAPACITY: usize> Ident<CAPACITY> {
    #[inline]
    pub const fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr().cast()
    }

    #[inline]
    pub const fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.as_ptr(), self.len()) }
    }

    #[inline]
    pub const fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(self.as_bytes()) }
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.len as usize
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<const CAPACITY: usize> AsRef<str> for Ident<CAPACITY> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<const CAPACITY: usize> AsRef<[u8]> for Ident<CAPACITY> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<const CAPACITY: usize> AsRef<Utf8Path> for Ident<CAPACITY> {
    #[inline]
    fn as_ref(&self) -> &Utf8Path {
        Utf8Path::new(self.as_str())
    }
}

impl<const CAPACITY: usize> Eq for Ident<CAPACITY> {}

impl<const CAPACITY: usize> Ord for Ident<CAPACITY> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl<const CAPACITY: usize> PartialEq for Ident<CAPACITY> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl<const CAPACITY: usize> PartialOrd for Ident<CAPACITY> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl<const CAPACITY: usize> From<Ident<CAPACITY>> for String {
    #[inline]
    fn from(ident: Ident<CAPACITY>) -> Self {
        ident.as_str().into()
    }
}

impl<const CAPACITY: usize> From<&Ident<CAPACITY>> for String {
    #[inline]
    fn from(ident: &Ident<CAPACITY>) -> Self {
        (*ident).into()
    }
}

impl<const CAPACITY: usize> From<Ident<CAPACITY>> for Vec<u8> {
    #[inline]
    fn from(ident: Ident<CAPACITY>) -> Self {
        ident.as_str().into()
    }
}

impl<const CAPACITY: usize> From<&Ident<CAPACITY>> for Vec<u8> {
    #[inline]
    fn from(ident: &Ident<CAPACITY>) -> Self {
        (*ident).into()
    }
}

impl<const CAPACITY: usize> borrow::Borrow<str> for Ident<CAPACITY> {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<const CAPACITY: usize> ops::Deref for Ident<CAPACITY> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl<const CAPACITY: usize> fmt::Debug for Ident<CAPACITY> {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), fmt)
    }
}

impl<const CAPACITY: usize> fmt::Display for Ident<CAPACITY> {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), fmt)
    }
}

impl<const CAPACITY: usize> hash::Hash for Ident<CAPACITY> {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

impl<const CAPACITY: usize> str::FromStr for Ident<CAPACITY> {
    type Err = TryFromStrError;

    #[inline]
    fn from_str(ident: &str) -> Result<Self, Self::Err> {
        ident.try_into()
    }
}

impl<const CAPACITY: usize> convert::TryFrom<&str> for Ident<CAPACITY> {
    type Error = TryFromStrError;

    fn try_from(ident: &str) -> Result<Self, Self::Error> {
        assert!(
            CAPACITY < 255,
            "ident capacity must be below 255 as the length is stored in a byte"
        );

        if ident.len() < CAPACITY {
            let mut data = [MaybeUninit::uninit(); CAPACITY];

            unsafe {
                ptr::copy_nonoverlapping(ident.as_ptr().cast(), data.as_mut_ptr(), ident.len());
            }

            Ok(Self {
                data,
                len: ident.len() as u8,
            })
        } else {
            Err(TryFromStrError(()))
        }
    }
}

impl fmt::Display for TryFromStrError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str("str is too long to fit in ident")
    }
}

impl error::Error for TryFromStrError {}

impl<const CAPACITY: usize> serde::Serialize for Ident<CAPACITY> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de, const CAPACITY: usize> serde::Deserialize<'de> for Ident<CAPACITY> {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de;

        struct IdentVisitor<const CAPACITY: usize>;

        impl<'de, const CAPACITY: usize> de::Visitor<'de> for IdentVisitor<CAPACITY> {
            type Value = Ident<CAPACITY>;

            #[inline]
            fn expecting(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt.write_str("an identifier")
            }

            #[inline]
            fn visit_str<E: de::Error>(self, ident: &str) -> Result<Self::Value, E> {
                ident.parse().map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_str(IdentVisitor::<CAPACITY>)
    }
}
