use {mocha_std::device, std::io};

fn main() -> io::Result<()> {
    device::set_boot_to_firmware(false)?;

    //Err(device::restart())

    Ok(())
}
