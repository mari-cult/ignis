use {
    camino::Utf8Path,
    codespan_reporting::{
        diagnostic::{Diagnostic, Label},
        files::SimpleFile,
        term::{
            self,
            termcolor::{ColorChoice, StandardStream},
        },
    },
    std::{error, fmt, ops::Range, process},
};

pub enum Error {
    DeserializeSpec {
        source: Box<Utf8Path>,
        content: Box<str>,
        message: Box<str>,
        range: Range<usize>,
    },
}

impl Error {
    pub fn emit(self) -> ! {
        match self {
            Error::DeserializeSpec {
                source,
                content,
                message,
                range,
            } => {
                let diagnostic: Diagnostic<()> =
                    Diagnostic::error().with_labels(vec![Label::primary((), range)]);

                let diagnostic =
                    if message.starts_with("found character that cannot start any token") {
                        diagnostic
                            .with_message("unexpected character encountered")
                            .with_notes(vec![String::from("perhaps surround it in quotes?")])
                    } else {
                        diagnostic.with_message("failed to parse spec as yaml")
                    };

                let source = SimpleFile::new(source.as_str(), &content);
                let mut output = StandardStream::stderr(ColorChoice::Auto);

                term::emit(&mut output, &Default::default(), &source, &diagnostic)
                    .unwrap_or_else(|_| panic!("{message}"));
            }
        }

        process::exit(1)
    }

    pub(crate) fn deserialize_spec(
        path: &Utf8Path,
        content: &str,
        error: serde_yaml::Error,
    ) -> Self {
        let range = error
            .location()
            .and_then(|location| {
                let index = location.index();
                let rest = content.get(index..)?.trim();
                let word = rest.split_whitespace().next()?;

                Some(index..(index + word.len()))
            })
            .unwrap_or(0..content.len());

        let message = error.to_string().into_boxed_str();

        Self::DeserializeSpec {
            source: Box::from(path),
            content: Box::from(content),
            message,
            range,
        }
    }
}

impl error::Error for Error {}

impl fmt::Debug for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeserializeSpec { source, .. } => fmt
                .debug_struct("DeserializeSpec")
                .field("source", &source)
                .finish_non_exhaustive(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeserializeSpec { source, .. } => writeln!(fmt, "invalid yaml in {source}"),
        }
    }
}
