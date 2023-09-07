use {clap::Parser, mocha_std::device, std::io};

#[derive(Parser)]
pub struct Args {
    /// Restart to firmware.
    #[arg(long)]
    firmware: bool,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    device::set_boot_to_firmware(args.firmware)?;

    Err(device::restart())
}
