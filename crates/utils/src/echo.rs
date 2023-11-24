use {
    super::{util, Tool},
    clap::Parser,
    std::{
        ffi::OsString,
        io::{self, Write},
    },
};

/// Output a line of text.
#[derive(Debug, Clone, Parser)]
pub struct Echo {
    #[arg(trailing_var_arg = true)]
    pub text: Vec<OsString>,
}

impl Tool for Echo {
    fn run(self) -> io::Result<()> {
        let Self { text } = self;

        let line = util::join_line(text);
        let mut stdout = io::stdout().lock();

        stdout.write_all(line.as_encoded_bytes())?;
        stdout.flush()?;

        Ok(())
    }
}
