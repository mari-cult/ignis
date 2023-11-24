use {
    super::{util, Tool},
    clap::{
        builder::{OsStringValueParser, TypedValueParser},
        Parser, ValueHint,
    },
    std::{
        env,
        ffi::OsString,
        fs::File,
        io::{self, BufWriter, IsTerminal, Write},
        mem::ManuallyDrop,
        os::{
            fd::{AsRawFd, FromRawFd},
            unix::process::CommandExt,
        },
        process::Command,
    },
};

/// Repeatedly output a line of text.
#[derive(Debug, Clone, Parser)]
pub struct Env {
    #[arg(num_args = 0..)]
    #[arg(value_parser = OsStringValueParser::new().try_map(util::key_value))]
    pub vars: Vec<(OsString, OsString)>,

    #[arg(num_args = 0..)]
    #[arg(trailing_var_arg = true)]
    #[arg(value_hint = ValueHint::CommandWithArguments)]
    pub args: Vec<OsString>,
}

impl Tool for Env {
    fn run(self) -> io::Result<()> {
        let Self { vars, args } = self;
        let mut args = args.into_iter();

        if let Some(program) = args.next() {
            return Err(Command::new(program).args(args).envs(vars).exec());
        }

        // Obtain environment variables.
        let mut vars_os = env::vars_os().chain(vars).collect::<Vec<_>>();

        // Sort lexicographically.
        // Reduces cognitive load when reading.
        vars_os.sort_unstable();

        // Lock stdout.
        // Ensure nothing can write to stdout during the below code.
        let lock = io::stdout().lock();

        // Obtain a reference to stdout that isn't buffered.
        // Internal implementation of stdout is buffered by a `LineWriter`.
        // SAFETY: `ManuallyDrop` ensures the stdout handle isnt closed on drop.
        let stdout = unsafe { ManuallyDrop::new(File::from_raw_fd(lock.as_raw_fd())) };

        // Apply regular buffering to stdout.
        let mut stdout = BufWriter::new(&*stdout);

        if lock.is_terminal() {
            for (key, value) in vars_os {
                stdout.write_all(b"\x1b[31m")?;
                stdout.write_all(key.as_encoded_bytes())?;
                stdout.write_all(b"\x1b[m=\x1b[32m")?;
                stdout.write_all(value.as_encoded_bytes())?;
                stdout.write_all(b"\x1b[m\n")?;
            }
        } else {
            for (key, value) in vars_os {
                stdout.write_all(key.as_encoded_bytes())?;
                stdout.write_all(b"=")?;
                stdout.write_all(value.as_encoded_bytes())?;
                stdout.write_all(b"\n")?;
            }
        }

        stdout.flush()?;

        Ok(())
    }
}
