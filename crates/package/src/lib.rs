use {
    binrw::{BinRead, BinWrite},
    std::{
        fs::{self, File},
        io::{self, BufWriter, Write},
        path::Path,
        process::Command,
    },
};

#[binrw::binrw]
#[derive(Debug)]
#[brw(magic = b"MOCHA", little)]
struct Metadata {
    #[brw(align_after = 1024)]
    pub size: u64,
}

fn main() {
    brew_mocha("uutils", "package").unwrap();
    drink_mocha("uutils", "package").unwrap();
}

pub fn brew_mocha<P>(name: &str, directory: P) -> io::Result<()>
where
    P: AsRef<Path>,
{
    let directory = directory.as_ref();
    let erofs_name = format!("{name}.erofs");
    let mocha_name = format!("{name}.mocha");

    // Generate erofs from a directory.
    Command::new("mkfs.erofs")
        .arg("-T0")
        .arg("--force-gid=9")
        .arg("--force-uid=9")
        .arg(&erofs_name)
        .arg(&directory)
        .spawn()?
        .wait()?;

    // Create the mocha.
    let mocha = File::options()
        .create_new(true)
        .write(true)
        .open(&mocha_name)?;

    let mut mocha = BufWriter::new(mocha);

    let metadata = Metadata { size: 5 };

    metadata
        .write(&mut mocha)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;

    let mut erofs = File::open(&erofs_name)?;

    io::copy(&mut erofs, &mut mocha)?;

    // Ensure everything is written.
    mocha.flush()?;
    drop(mocha);

    // Clean up the left-over erofs.
    fs::remove_file(erofs_name)?;

    Ok(())
}

pub fn drink_mocha<P>(name: &str, directory: P) -> io::Result<()>
where
    P: AsRef<Path>,
{
    let directory = directory.as_ref();
    let mocha_name = format!("{name}.mocha");
    let mut mocha = File::open(&mocha_name)?;
    let metadata = Metadata::read(&mut mocha)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;

    println!("{metadata:?}");

    loopy::Loop::options()
        .offset(1024)
        .read_only(true)
        .open(mocha_name)?;

    Ok(())
}