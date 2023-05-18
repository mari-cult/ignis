use std::{borrow, fmt, ops};

/// A wrapper around `Box<str>` implementing `AsRef<[u8]>`.
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Key(Box<str>);

impl AsRef<[u8]> for Key {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl From<&str> for Key {
    #[inline]
    fn from(key: &str) -> Self {
        Self(Box::from(key))
    }
}

impl From<Key> for Box<str> {
    #[inline]
    fn from(key: Key) -> Self {
        key.0
    }
}

impl borrow::Borrow<str> for Key {
    fn borrow(&self) -> &str {
        &self
    }
}

impl fmt::Debug for Key {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, fmt)
    }
}

impl ops::Deref for Key {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
