#![deny(warnings)]

use {
    once_cell::sync::OnceCell,
    std::{
        fs::File,
        io,
        path::Path,
        sync::{Arc, Mutex},
    },
};

mod os;

static CONTROL: OnceCell<Arc<Mutex<os::LoopControl>>> = OnceCell::new();

/// A loop device.
pub struct Loop {
    _device: os::Loop,
    _file: File,
}

/// Options for opening a loop device.
#[derive(Default)]
pub struct LoopOptions {
    options: os::LoopOptions,
}

impl Loop {
    /// Create an empty set of options.
    #[inline]
    pub fn options() -> LoopOptions {
        LoopOptions::new()
    }
}

impl LoopOptions {
    /// Create an empty set of options.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn auto_clear(&mut self, auto_clear: bool) -> &mut Self {
        self.options
            .flags
            .set(os::LoopFlags::AUTO_CLEAR, auto_clear);

        self
    }

    #[inline]
    pub fn direct_io(&mut self, direct_io: bool) -> &mut Self {
        self.options.flags.set(os::LoopFlags::DIRECT_IO, direct_io);
        self
    }

    /// Set the byte offset within the backing file.
    #[inline]
    pub fn offset(&mut self, offset: u64) -> &mut Self {
        self.options.offset = offset;
        self
    }

    #[inline]
    pub fn part_scan(&mut self, part_scan: bool) -> &mut Self {
        self.options.flags.set(os::LoopFlags::PART_SCAN, part_scan);
        self
    }

    /// Force read-only mode.
    #[inline]
    pub fn read_only(&mut self, read_only: bool) -> &mut Self {
        self.options.flags.set(os::LoopFlags::READ_ONLY, read_only);
        self
    }

    /// Create a loop device with `path` as the backing file.
    #[inline]
    pub fn open<P: AsRef<Path>>(&self, path: P) -> io::Result<Loop> {
        /// Monomorphization codegen reduction.
        fn inner(options: &LoopOptions, path: &Path) -> io::Result<Loop> {
            let file = File::options()
                .read(true)
                .write(!options.options.flags.contains(os::LoopFlags::READ_ONLY))
                .open(path)?;

            let control = CONTROL
                .get_or_try_init(move || os::LoopControl::open().map(Mutex::new).map(Arc::new))?;

            let mut control = control
                .lock()
                .map_err(|_error| io::Error::new(io::ErrorKind::Other, "mutex poisoned"))?;

            let mut device = control.next()?;

            device.set_file(&file, options.options)?;

            Ok(Loop {
                _device: device,
                _file: file,
            })
        }

        inner(self, path.as_ref())
    }
}
