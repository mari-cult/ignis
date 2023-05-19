use {
    crate::{Ident, TryFromStrError},
    camino::Utf8Path,
    std::{convert, fmt, ops, str},
};

macro_rules! impl_spec {
    ($ident:ident<$capacity:literal>) => {
        #[derive(Clone, Copy, Hash, Eq, Ord, PartialEq, PartialOrd)]
        #[repr(align($capacity))]
        pub struct $ident(Ident<$capacity>);

        impl $ident {
            #[inline]
            pub const fn as_str(&self) -> &str {
                self.0.as_str()
            }
        }

        impl AsRef<str> for $ident {
            #[inline]
            fn as_ref(&self) -> &str {
                self.0.as_ref()
            }
        }

        impl AsRef<[u8]> for $ident {
            #[inline]
            fn as_ref(&self) -> &[u8] {
                self.0.as_ref()
            }
        }

        impl AsRef<Utf8Path> for $ident {
            #[inline]
            fn as_ref(&self) -> &Utf8Path {
                self.0.as_ref()
            }
        }

        impl From<$ident> for String {
            #[inline]
            fn from(ident: $ident) -> Self {
                ident.0.into()
            }
        }

        impl From<&$ident> for String {
            #[inline]
            fn from(ident: &$ident) -> Self {
                (*ident).into()
            }
        }

        impl From<$ident> for Vec<u8> {
            #[inline]
            fn from(ident: $ident) -> Self {
                ident.0.into()
            }
        }

        impl From<&$ident> for Vec<u8> {
            #[inline]
            fn from(ident: &$ident) -> Self {
                (*ident).into()
            }
        }

        impl convert::TryFrom<&str> for $ident {
            type Error = TryFromStrError;

            #[inline]
            fn try_from(ident: &str) -> Result<Self, Self::Error> {
                <Ident<$capacity> as convert::TryFrom<&str>>::try_from(ident).map($ident)
            }
        }

        impl fmt::Debug for $ident {
            #[inline]
            fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Debug::fmt(&self.0, fmt)
            }
        }

        impl fmt::Display for $ident {
            #[inline]
            fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(&self.0, fmt)
            }
        }

        impl str::FromStr for $ident {
            type Err = TryFromStrError;

            #[inline]
            fn from_str(ident: &str) -> Result<Self, Self::Err> {
                <Ident<$capacity> as str::FromStr>::from_str(ident).map($ident)
            }
        }

        impl ops::Deref for $ident {
            type Target = str;

            #[inline]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl serde::Serialize for $ident {
            #[inline]
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                <Ident<$capacity> as serde::Serialize>::serialize(&self.0, serializer)
            }
        }

        impl<'de> serde::Deserialize<'de> for $ident {
            #[inline]
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                <Ident<$capacity> as serde::Deserialize>::deserialize(deserializer).map($ident)
            }
        }
    };
}

impl_spec!(ArtifactIdent<64>);
impl_spec!(PackageIdent<64>);
impl_spec!(RepositoryIdent<32>);
impl_spec!(FeatureIdent<32>);
impl_spec!(SourceIdent<128>);
