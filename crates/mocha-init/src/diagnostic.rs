use {
    mocha_os::diagnostic::Diagnostic,
    std::{borrow::Cow, fmt, io, sync::OnceLock},
};

static DIAGNOSTIC: OnceLock<Diagnostic> = OnceLock::new();

#[doc(hidden)]
pub fn info(args: fmt::Arguments<'_>) -> io::Result<()> {
    let diagnostic = if let Some(diagnostic) = DIAGNOSTIC.get() {
        diagnostic
    } else {
        let diagnostic = Diagnostic::open()?;

        DIAGNOSTIC.get_or_init(|| diagnostic)
    };

    let mut string = args
        .as_str()
        .map(Cow::Borrowed)
        .unwrap_or_else(|| Cow::Owned(args.to_string()));

    if !string.ends_with('\n') {
        string.to_mut().push('\n');
    }

    for line in string.lines() {
        diagnostic.write_fmt(format_args!("{}: {line}", env!("CARGO_PKG_NAME")))?;
    }

    Ok(())
}

#[allow_internal_unstable(format_args_nl)]
#[macro_export]
macro_rules! info {
    ($fmt:expr) => {
        $crate::diagnostic::info(format_args_nl!($fmt))
    };
    ($fmt:expr, $($args:tt)*) => {
        $crate::diagnostic::info(format_args_nl!($fmt, $($args)*))
    };
}
