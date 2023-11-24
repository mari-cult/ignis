use {clap::Parser, std::io};

mod chroot;
mod echo;
mod env;
mod generate_completion;
mod util;
mod yes;

pub trait Tool {
    fn run(self) -> io::Result<()>;
}

#[derive(Debug, Clone, Parser)]
pub enum Args {
    Chroot(chroot::Chroot),
    Echo(echo::Echo),
    Env(env::Env),
    GenerateCompletion(generate_completion::GenerateCompletion),
    Yes(yes::Yes),
}

impl Args {
    fn run(self) -> io::Result<()> {
        match self {
            Self::Chroot(tool) => tool.run(),
            Self::Echo(tool) => tool.run(),
            Self::Env(tool) => tool.run(),
            Self::GenerateCompletion(tool) => tool.run(),
            Self::Yes(tool) => tool.run(),
        }
    }
}

fn main() -> io::Result<()> {
    dbg!(Args::parse()).run()
}
