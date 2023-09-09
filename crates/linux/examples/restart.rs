use {
    clap::Parser,
    linux::power::{self, Restart},
    std::io,
};

#[derive(Parser)]
pub struct Args {
    /// Restart to firmware.
    #[arg(long)]
    firmware: bool,
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let restart = if args.firmware {
        Restart::Firmware
    } else {
        Restart::Normal
    };

    Err(power::restart(restart))
}
