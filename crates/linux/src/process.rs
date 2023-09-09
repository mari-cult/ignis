use {
    crate::syscall::Error,
    nix::{
        libc,
        sys::uio::{self, RemoteIoVec},
        unistd::Pid,
    },
    std::{
        io::{self, IoSlice, IoSliceMut, Read, Seek, SeekFrom, Write},
        slice,
    },
};

pub struct Process {
    process_id: u32,
    position: usize,
}

impl Process {
    #[inline]
    #[must_use]
    pub fn new(process_id: u32, position: usize) -> Self {
        Self {
            process_id,
            position,
        }
    }
}

impl Read for Process {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.read_vectored(slice::from_mut(&mut IoSliceMut::new(buf)))
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        let len = bufs.iter().map(|buf| buf.len()).sum();
        let amount = uio::process_vm_readv(
            Pid::from_raw(self.process_id as libc::pid_t),
            bufs,
            slice::from_ref(&RemoteIoVec {
                base: self.position,
                len,
            }),
        )?;

        self.position += amount;

        Ok(amount)
    }
}

impl Seek for Process {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        self.position = match position {
            SeekFrom::Start(position) => position.try_into().map_err(|_error| invalid_input())?,
            SeekFrom::Current(offset) => {
                let offset = offset.try_into().map_err(|_error| invalid_input())?;

                self.position
                    .checked_add_signed(offset)
                    .ok_or_else(invalid_input)?
            }
            SeekFrom::End(_offset) => {
                panic!("unable to determine end offset");
            }
        };

        Ok(self.position as u64)
    }
}

impl Write for Process {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write_vectored(slice::from_ref(&IoSlice::new(buf)))
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        let len = bufs.iter().map(|buf| buf.len()).sum();
        let amount = uio::process_vm_writev(
            Pid::from_raw(self.process_id as libc::pid_t),
            bufs,
            slice::from_ref(&RemoteIoVec {
                base: self.position,
                len,
            }),
        )?;

        self.position += amount;

        Ok(amount)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn invalid_input() -> io::Error {
    Error::InvalidInput.into()
}
