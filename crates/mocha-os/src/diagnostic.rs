use std::{
    borrow::Cow,
    fmt,
    fs::File,
    io::{self, Write},
};

/// The kernel's diagnostic message buffer (`/dev/kmsg`).
pub struct Diagnostic {
    file: File,
}

impl Diagnostic {
    pub fn open() -> io::Result<Self> {
        let file = File::options().write(true).open("/dev/kmsg")?;

        Ok(Self { file })
    }

    pub fn write_fmt(&self, args: fmt::Arguments<'_>) -> io::Result<()> {
        let mut string = args
            .as_str()
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Owned(args.to_string()));

        if !string.ends_with('\n') {
            string.to_mut().push('\n');
        }

        let mut file = &self.file;

        file.write(string.as_bytes())?;

        Ok(())
    }
}
