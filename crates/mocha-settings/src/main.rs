use camino::Utf8Path;
use codespan_reporting::{diagnostic::Diagnostic, term::termcolor::StandardStream};

use {
    ipnetwork::IpNetwork,
    serde::{Deserialize, Serialize},
    std::{
        error,
        fs::File,
        io::{self, Read},
        net::IpAddr,
        panic, process,
    },
};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Network {
    pub address: IpNetwork,
    pub gateway: IpAddr,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Settings {
    pub network: Vec<Network>,
}

/// Hard 8KB for configuration files.
const MAX_SIZE: usize = 8 * 1024;

fn main() -> io::Result<()> {
    panic::set_hook(Box::new(|info| {
        println!("This shouldn't ever happen.");
        println!();
        println!("{info}");
        println!();
        println!("Report an issue at https://github.com/kalmari246/mocha/issues");
    }));

    let mut file = File::open("/system/settings.toml")?;
    let len = file.metadata()?.len();

    if len > MAX_SIZE as u64 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "settings.toml above 8KB limit",
        ));
    }

    let mut string = String::with_capacity(len as usize);

    file.read_to_string(&mut string)?;

    let diagnostic = Diagnostic::error().with_message("foo");
    let mut writer =
        StandardStream::stderr(codespan_reporting::term::termcolor::ColorChoice::Always);

    codespan_reporting::term::emit(
        &mut writer,
        &codespan_reporting::term::Config::default(),
        &codespan_reporting::files::SimpleFile::new("settings.toml", "on god"),
        &diagnostic,
    )
    .unwrap();

    let settings = toml::from_str::<Settings>(&string).unwrap();

    println!("{settings:?}");

    Ok(())
}

/*/// Reads the entire contents of a file into a string.
///
/// This function is similar to [`std::fs::read_to_string`], allowing only UTF-8 paths.
/// It also provides an optional `limit` parameter to reject files larger than the specified size in bytes.
#[inline]
pub fn read_to_string<P: AsRef<Utf8Path>>(path: P, limit: Option<u64>) -> io::Result<String> {
    fn inner(path: &Utf8Path, limit: Option<u64>) -> io::Result<String> {
        let Some(limit) = limit else {
            return fs::read_to_string(path);
        };

        let mut file = File::open(path)?;
        let len = file.metadata()?.len();
    }

    inner(path.as_ref(), limit)
}*/
