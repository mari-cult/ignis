use {
    rustix::ioctl,
    std::{
        io,
        os::unix::io::{AsFd, AsRawFd},
    },
};

pub mod c;

/// `/dev/loop{index}` options.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DeviceOptions {
    pub offset: u64,
}

/// Obtain the next free device index.
pub fn free_device_index<Fd: AsFd>(fd: Fd) -> io::Result<u32> {
    unsafe {
        let ctl = ioctl::Getter::<ioctl::BadOpcode<{ c::LOOP_CTL_GET_FREE }>, u32>::new();

        ioctl::ioctl(fd, ctl).map_err(Into::into)
    }
}

/// Configure the device.
pub fn configure_device<Fd: AsFd, BackingFd: AsFd>(
    fd: Fd,
    backing_fd: BackingFd,
    options: DeviceOptions,
) -> io::Result<()> {
    let config = c::loop_config {
        fd: backing_fd.as_fd().as_raw_fd(),
        block_size: 4096,
        info: c::loop_info64 {
            lo_offset: options.offset,
            //lo_flags: options.flags.bits(),
            ..Default::default()
        },
        ..Default::default()
    };

    unsafe {
        let ctl =
            ioctl::Setter::<ioctl::BadOpcode<{ c::LOOP_CONFIGURE }>, *const c::loop_config>::new(
                &config,
            );

        ioctl::ioctl(fd, ctl)?;
    }

    Ok(())
}
