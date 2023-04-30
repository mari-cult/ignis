use std::{fmt, str};

pub enum Artifact {
    Bin {
        name: Box<str>,
        rename_to: Option<Box<str>>,
    },
    Sym {
        name: Box<str>,
        points_to: Box<str>,
    },
}

impl str::FromStr for Artifact {
    type Err = &'static str;

    fn from_str(artifact: &str) -> std::result::Result<Self, Self::Err> {
        match split_once(artifact) {
            Some(("bin", name, to)) => Ok(Self::Bin {
                name: Box::from(name),
                rename_to: to.map(|to| Box::from(to)),
            }),
            Some(("sym", name, Some(to))) => Ok(Self::Sym {
                name: Box::from(name),
                points_to: Box::from(to),
            }),
            _ => Err("invalid artifact mapping"),
        }
    }
}

impl fmt::Debug for Artifact {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bin {
                name,
                rename_to: Some(rename_to),
            } => fmt.debug_tuple("Bin").field(name).field(rename_to).finish(),

            Self::Bin {
                name,
                rename_to: None,
            } => fmt.debug_tuple("Bin").field(name).finish(),

            Self::Sym { name, points_to } => {
                fmt.debug_tuple("Sym").field(name).field(points_to).finish()
            }
        }
    }
}

impl fmt::Display for Artifact {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bin {
                name,
                rename_to: Some(to),
            } => write!(fmt, "bin {name} -> {to}"),

            Self::Bin {
                name,
                rename_to: None,
            } => write!(fmt, "bin {name}"),

            Self::Sym { name, points_to } => write!(fmt, "sym {name} -> {points_to}"),
        }
    }
}

impl serde::Serialize for Artifact {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for Artifact {
    fn deserialize<D>(deserializer: D) -> Result<Artifact, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{Error, Unexpected, Visitor};

        struct ArtifactVisitor;

        impl<'de> Visitor<'de> for ArtifactVisitor {
            type Value = Artifact;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string representing an URL")
            }

            fn visit_str<E>(self, artifact: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                artifact.parse::<Artifact>().map_err(|error| {
                    let error = error.to_string();

                    Error::invalid_value(Unexpected::Str(artifact), &error.as_str())
                })
            }
        }

        deserializer.deserialize_str(ArtifactVisitor)
    }
}

fn split_once<'a>(artifact: &'a str) -> Option<(&'a str, &'a str, Option<&'a str>)> {
    let (kind, rest) = artifact.split_once(' ')?;
    let (name, to) = rest
        .split_once(" -> ")
        .map(|(name, to)| (name, Some(to)))
        .unwrap_or_else(|| (rest, None));

    Some((kind, name, to))
}
