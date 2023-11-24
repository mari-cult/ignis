use {
    super::Tool,
    clap::{Parser, ValueHint},
    std::{
        env,
        ffi::OsString,
        io,
        os::unix::{self, process::CommandExt},
        path::PathBuf,
        process::Command,
    },
};

/// Change the root directory for the specified program.
#[derive(Debug, Clone, Parser)]
pub struct Chroot {
    #[arg(value_hint = ValueHint::DirPath)]
    pub root_dir: PathBuf,

    #[arg(default_value = "/bin/sh")]
    #[arg(trailing_var_arg = true)]
    pub program: OsString,

    #[arg(trailing_var_arg = true)]
    pub args: Vec<OsString>,
}

impl Tool for Chroot {
    fn run(self) -> io::Result<()> {
        let Self {
            root_dir,
            program,
            args,
        } = self;

        let mut command = Command::new(program);

        // SAFETY: Only changing the commnd's root directory.
        unsafe {
            command.pre_exec(move || {
                unix::fs::chroot(&root_dir)?;
                env::set_current_dir("/")?;

                Ok(())
            });
        }

        Err(command.args(args).exec())
    }
}
