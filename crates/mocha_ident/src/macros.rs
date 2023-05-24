macro_rules! ident {
    ($(
        $(#[$meta:meta])*
        $vis:vis struct $ident:ident: $align:literal;
    )*) => {$(
        $(#[$meta])*
        #[derive(Clone, Copy)]
        #[repr(align($align))]
        $vis struct $ident {
            data: [MaybeUninit<u8>; { $align - 1 }],
            len: NonZeroU8,
        }

        impl $ident {
            const CAPACITY: usize = $align - 1;

            /// Returns a pointer to the first byte of the identifier.
            #[inline]
            pub const fn as_ptr(&self) -> *const u8 {
                self.data.as_ptr().cast()
            }

            /// Returns the length of the identifier.
            #[inline]
            pub const fn len(&self) -> usize {
                self.len.get() as usize
            }

            /// Always returns `false`.
            ///
            /// Identifiers are required to be non-empty.
            #[inline]
            pub const fn is_empty(&self) -> bool {
                false
            }

            /// Returns a byte slice of the contents of the identifier.
            #[inline]
            pub const fn as_bytes(&self) -> &[u8] {
                unsafe { slice::from_raw_parts(self.as_ptr(), self.len()) }
            }

            /// Returns a string representation of the identifier.
            #[inline]
            pub const fn as_str(&self) -> &str {
                unsafe { str::from_utf8_unchecked(self.as_bytes()) }
            }
        }
    )*};
}

macro_rules! impl_common_traits {
    ($($ident:ident,)*) => {$(
        impl AsRef<str> for $ident {
            #[inline]
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl AsRef<[u8]> for $ident {
            #[inline]
            fn as_ref(&self) -> &[u8] {
                self.as_bytes()
            }
        }

        impl AsRef<ffi::OsStr> for $ident {
            #[inline]
            fn as_ref(&self) -> &ffi::OsStr {
                self.as_str().as_ref()
            }
        }

        impl AsRef<mocha_fs::Utf8Path> for $ident {
            #[inline]
            fn as_ref(&self) -> &mocha_fs::Utf8Path {
                self.as_str().as_ref()
            }
        }

        impl AsRef<path::Path> for $ident {
            #[inline]
            fn as_ref(&self) -> &path::Path {
                self.as_str().as_ref()
            }
        }

        impl Eq for $ident {}

        impl From<$ident> for String {
            #[inline]
            fn from(ident: $ident) -> Self {
                ident.as_str().into()
            }
        }

        impl From<$ident> for Vec<u8> {
            #[inline]
            fn from(ident: $ident) -> Self {
                ident.as_str().into()
            }
        }

        impl From<$ident> for ffi::OsString {
            #[inline]
            fn from(ident: $ident) -> Self {
                ident.as_str().into()
            }
        }

        impl From<$ident> for mocha_fs::Utf8PathBuf {
            #[inline]
            fn from(ident: $ident) -> Self {
                ident.as_str().into()
            }
        }

        impl From<$ident> for path::PathBuf {
            #[inline]
            fn from(ident: $ident) -> Self {
                ident.as_str().into()
            }
        }

        impl Ord for $ident {
            #[inline]
            fn cmp(&self, other: &Self) -> Ordering {
                self.as_str().cmp(other.as_str())
            }
        }

        impl PartialEq for $ident {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                self.as_str() == other.as_str()
            }
        }

        impl PartialEq<str> for $ident {
            #[inline]
            fn eq(&self, other: &str) -> bool {
                self.as_str() == other
            }
        }

        impl PartialEq<&str> for $ident {
            #[inline]
            fn eq(&self, other: &&str) -> bool {
                self.as_str() == *other
            }
        }

        impl PartialEq<String> for $ident {
            #[inline]
            fn eq(&self, other: &String) -> bool {
                self.as_str() == other
            }
        }

        impl PartialEq<ffi::OsStr> for $ident {
            #[inline]
            fn eq(&self, other: &ffi::OsStr) -> bool {
                self.as_str() == other
            }
        }

        impl PartialOrd for $ident {
            #[inline]
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                self.as_str().partial_cmp(other.as_str())
            }
        }

        impl borrow::Borrow<str> for $ident {
            #[inline]
            fn borrow(&self) -> &str {
                self.as_str()
            }
        }

        impl fmt::Debug for $ident {
            #[inline]
            fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Debug::fmt(self.as_str(), fmt)
            }
        }

        impl fmt::Display for $ident {
            #[inline]
            fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(self.as_str(), fmt)
            }
        }

        impl hash::Hash for $ident {
            #[inline]
            fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
                self.as_str().hash(hasher)
            }
        }

        impl ops::Deref for $ident {
            type Target = str;

            #[inline]
            fn deref(&self) -> &str {
                self.as_str()
            }
        }
    )*};
}

macro_rules! impl_generic_from_str {
    ($($ident:ident: $check:ident,)*) => {$(
        impl str::FromStr for $ident {
            type Err = IdentError;

            fn from_str(ident: &str) -> Result<Self, Self::Err> {
                let len = generic_check(ident, Self::CAPACITY)?;

                $check(ident)?;

                let mut data = [MaybeUninit::uninit(); Self::CAPACITY];

                unsafe {
                    ptr::copy_nonoverlapping(
                        ident.as_ptr().cast(),
                        data.as_mut_ptr(),
                        len.get() as usize,
                    );
                }

                Ok(Self { data, len })
            }
        }
    )*};
}

macro_rules! impl_serde {
    ($($ident:ident,)*) => {$(
        impl serde::Serialize for $ident {
            #[inline]
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(self.as_str())
            }
        }

        impl<'de> serde::Deserialize<'de> for $ident {
            #[inline]
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                use serde::de;

                struct Visitor;

                impl<'de> de::Visitor<'de> for Visitor {
                    type Value = $ident;

                    #[inline]
                    fn expecting(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
                        fmt.write_str("an identifier")
                    }

                    #[inline]
                    fn visit_str<E: de::Error>(self, ident: &str) -> Result<Self::Value, E> {
                        ident.parse().map_err(de::Error::custom)
                    }
                }

                deserializer.deserialize_str(Visitor)
            }
        }
    )*};
}

pub(crate) use {ident, impl_common_traits, impl_generic_from_str, impl_serde};
