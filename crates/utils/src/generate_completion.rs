use {
    crate::{util, Tool},
    clap::{
        builder::{OsStringValueParser, TypedValueParser},
        CommandFactory, Parser,
    },
    clap_complete::Shell,
    std::io,
};

/// Generate shell completion.
#[derive(Debug, Clone, Parser)]
pub struct GenerateCompletion {
    #[arg(default_value_t = Shell::Bash)]
    #[arg(env)]
    #[arg(value_parser = OsStringValueParser::new().try_map(util::shell))]
    pub shell: Shell,
}

impl Tool for GenerateCompletion {
    fn run(self) -> io::Result<()> {
        let Self { shell } = self;
        let command = &mut super::Args::command();

        clap_complete::generate(
            shell,
            command,
            command.get_name().to_string(),
            &mut io::stdout(),
        );

        Ok(())
    }
}
