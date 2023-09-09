use {
    std::{
        borrow, cmp::Ordering, ffi, fmt, hash, io, mem::MaybeUninit, num::NonZeroU8, ops, path,
        ptr, slice, str,
    },
    thiserror::Error,
};

mod macros;

#[derive(Debug, Error)]
pub enum IdentError {
    #[error("an identifier cannot be empty")]
    Empty,

    #[error("exceeds the capacity of the identifier")]
    ExceedsCapacity,

    #[error("unexpected character {character} at index {position}")]
    UnexpectedCharacter { position: usize, character: char },

    #[error("invalid source url: {error}")]
    Source {
        #[from]
        error: SourceError,
    },
}

#[derive(Debug, Error)]
pub enum SourceError {
    #[error("parse: {error}")]
    Parse {
        #[from]
        error: url::ParseError,
    },

    #[error("expected \"github:/{{user}}/{{repository}}\"")]
    ExpectedGithub,

    #[error("unsupported source: {url}")]
    Unsupported { url: Box<str> },
}

impl From<IdentError> for io::Error {
    #[inline]
    fn from(error: IdentError) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, error)
    }
}

/// Whether the character `character` is ASCII lowercase alphanumeric, or an underscore.
#[inline]
const fn is_package_ident(character: char) -> bool {
    matches!(character, 'a'..='z' | '0'..='9' | '_')
}

/// Whether the character `character` is ASCII alphanumeric, an underscore, a hyphen, or a backslash.
#[inline]
const fn is_cargo_feature(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '/')
}

/// Check ceneric identifier requirements, non-empty, capacity constraints.
fn generic_check(ident: &str, capacity: usize) -> Result<NonZeroU8, IdentError> {
    assert!(capacity != 0, "capacity is zero");
    assert!(capacity < 254, "capacity is greater than 254");

    let len = ident.len();

    if len > capacity {
        Err(IdentError::ExceedsCapacity)
    } else if let Some(len) = NonZeroU8::new(len as u8) {
        Ok(len)
    } else {
        Err(IdentError::Empty)
    }
}

/// Check whether the input is valid using the given per-character validator.
fn per_char<F>(input: &str, mut is_valid: F) -> Result<(), IdentError>
where
    F: FnMut(char) -> bool,
{
    let unexpected_character = input
        .chars()
        .enumerate()
        .find(|(_position, character)| !(is_valid)(*character));

    match unexpected_character {
        Some((position, character)) => Err(IdentError::UnexpectedCharacter {
            position,
            character,
        }),
        None => Ok(()),
    }
}

/// Check whether the input is a package identifier.
fn check_package_ident(input: &str) -> Result<(), IdentError> {
    per_char(input, is_package_ident)
}

/// Check whether the input is a cargo feature.
fn check_cargo_feature(input: &str) -> Result<(), IdentError> {
    per_char(input, is_cargo_feature)
}

/// Check whether the input is a valid source URL.
fn parse_source_url(input: &str) -> Result<String, IdentError> {
    let url = input.parse::<url::Url>().map_err(SourceError::from)?;
    let scheme = url.scheme();
    let host = url.host_str();
    let mut path = url
        .path_segments()
        .into_iter()
        .flatten()
        .map(|segment| dbg!(segment));
    let sanitized = match (scheme, host) {
        ("github", None) | ("http" | "https", Some("github.com")) => {
            let (user, repository) = path
                .next()
                .and_then(|user| path.next().map(|repository| (user, repository)))
                .ok_or(SourceError::ExpectedGithub)?;

            format!("github:{user}/{repository}")
        }
        _ => Err(SourceError::Unsupported {
            url: Box::from(url.as_str()),
        })?,
    };

    Ok(sanitized)
}

macros::ident! {
    /// An artifact.
    ///
    /// Cannot be empty, and cannot be greater than 127 characters.
    /// Permits ASCII alphanumeric characters, an underscore, or a hyphen.
    pub struct Artifact: 64;

    /// An identifier of a package.
    ///
    /// Cannot be empty, and cannot be greater than 63 characters.
    /// Permits ASCII lowercase alphanumeric characters, or an underscore.
    pub struct PackageIdent: 64;

    /// An identifier of a repository.
    ///
    /// Cannot be empty, and cannot be greater than 31 characters.
    /// Permits ASCII lowercase alphanumeric characters, or an underscore.
    pub struct RepositoryIdent: 32;

    /// A cargo feature.
    ///
    /// Cannot be empty, and cannot be greater than 127 characters.
    /// Permits ASCII alphanumeric characters, an underscore, a hyphen, or a backslash.
    pub struct CargoFeature: 128;

    /// A source URL.
    ///
    /// Cannot be empty, and cannot be greater than 127 characters.
    /// Permits ASCII alphanumeric characters, an underscore, or a hyphen.
    pub struct Source: 128;
}

macros::impl_common_traits! {
    Artifact,
    PackageIdent,
    RepositoryIdent,
    CargoFeature,
    Source,
}

macros::impl_generic_from_str! {
    Artifact: check_cargo_feature,
    PackageIdent: check_package_ident,
    RepositoryIdent: check_package_ident,
    CargoFeature: check_cargo_feature,
}

macros::impl_serde! {
    Artifact,
    PackageIdent,
    RepositoryIdent,
    CargoFeature,
    Source,
}

impl str::FromStr for Source {
    type Err = IdentError;

    fn from_str(url: &str) -> Result<Self, Self::Err> {
        let len = generic_check(url, Self::CAPACITY)?;
        let url = parse_source_url(url)?;
        let url = url.as_str();
        let mut data = [MaybeUninit::uninit(); Self::CAPACITY];

        unsafe {
            ptr::copy_nonoverlapping(url.as_ptr().cast(), data.as_mut_ptr(), len.get() as usize);
        }

        Ok(Self { data, len })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_ident() {
        assert!(matches!("".parse::<PackageIdent>(), Err(IdentError::Empty)));

        assert!(matches!(
            " ".repeat(64).parse::<PackageIdent>(),
            Err(IdentError::ExceedsCapacity)
        ));

        assert!(matches!(
            "-".parse::<PackageIdent>(),
            Err(IdentError::UnexpectedCharacter {
                position: 0,
                character: '-'
            })
        ));

        assert_eq!("mocha".parse::<PackageIdent>().unwrap(), "mocha");
    }

    #[test]
    fn cargo_feature() {
        assert!(matches!("".parse::<CargoFeature>(), Err(IdentError::Empty)));

        assert!(matches!(
            " ".repeat(128).parse::<CargoFeature>(),
            Err(IdentError::ExceedsCapacity)
        ));

        assert!(matches!(
            "!".parse::<CargoFeature>(),
            Err(IdentError::UnexpectedCharacter {
                position: 0,
                character: '!'
            })
        ));

        assert_eq!("Mocha_-246".parse::<CargoFeature>().unwrap(), "Mocha_-246");
        assert_eq!(
            "mocha/Mocha_-246".parse::<CargoFeature>().unwrap(),
            "mocha/Mocha_-246"
        );
    }

    #[test]
    fn source() {
        assert!(matches!("".parse::<Source>(), Err(IdentError::Empty)));

        assert!(matches!(
            " ".repeat(128).parse::<Source>(),
            Err(IdentError::ExceedsCapacity)
        ));

        assert!("file://invalid.com".parse::<Source>().is_err());

        assert_eq!(
            "github:ka1mari/mocha-milk".parse::<Source>().unwrap(),
            "https://github.com/ka1mari/mocha-milk"
                .parse::<Source>()
                .unwrap(),
        );
    }
}
