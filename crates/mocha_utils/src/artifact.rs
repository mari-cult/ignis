use {
    mocha_ident::{spec::ArtifactIdent, TryFromStrError},
    std::str,
    thiserror::Error,
};

#[derive(Debug, Error)]
pub enum ArtifactError {
    #[error("{0}")]
    InvalidItem(#[from] TryFromStrError),

    #[error("the arifact \"{0}\" is not supported")]
    Unsupported(Box<str>),
}

/// An artifact.
///
/// ```rust
/// "name" -> Binary("name")
/// "libname.a" -> StaticLibrary("name")
/// "libname.so" -> DynamicLibrary("name")
/// "name" -> "libname.so" -> Link(Binary("name"), Library("name"))
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Artifact {
    Item(Item),
    Link(Item, Item),
}

/// A single artifact item.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Item {
    Binary(ArtifactIdent),
    DynamicLibrary(ArtifactIdent),
    StaticLibrary(ArtifactIdent),
}

impl str::FromStr for Artifact {
    type Err = ArtifactError;

    fn from_str(artifact: &str) -> Result<Self, Self::Err> {
        if let Some((name, link)) = artifact.split_once(" -> ") {
            Ok(Self::Link(name.parse()?, link.parse()?))
        } else {
            Ok(Self::Item(artifact.parse()?))
        }
    }
}

impl str::FromStr for Item {
    type Err = ArtifactError;

    fn from_str(artifact: &str) -> Result<Self, Self::Err> {
        if let Some(artifact) = artifact.strip_prefix("lib") {
            if let Some(ident) = artifact.strip_suffix(".so") {
                Ok(Self::DynamicLibrary(ident.parse()?))
            } else if let Some(ident) = artifact.strip_suffix(".a") {
                Ok(Self::StaticLibrary(ident.parse()?))
            } else {
                Err(ArtifactError::Unsupported(Box::from(artifact)))
            }
        } else {
            Ok(Self::Binary(artifact.parse()?))
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse() {
        assert_eq!("name".parse(), Ok(Artifact::Item(Item::Binary)));
    }
}
