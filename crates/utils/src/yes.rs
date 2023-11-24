use {
    super::{util, Tool},
    clap::Parser,
    std::{
        borrow::Cow,
        ffi::{OsStr, OsString},
        io,
    },
};

/// Repeatedly output a line of text.
#[derive(Debug, Clone, Parser)]
pub struct Yes {
    #[arg(trailing_var_arg = true)]
    pub text: Vec<OsString>,
}

impl Tool for Yes {
    fn run(self) -> io::Result<()> {
        let Self { text } = self;
        let line = if text.is_empty() {
            Cow::Borrowed(OsStr::new("yes\n"))
        } else {
            Cow::Owned(util::join_line(text))
        };

        util::stdout_write_repeated(line.as_encoded_bytes())
    }
}
