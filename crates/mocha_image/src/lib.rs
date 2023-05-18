use {
    binrw::{BinRead, BinWrite},
    camino::Utf8Path,
    mocha_utils::{Category, Command, Rule},
    std::{
        fs::{self, File},
        io::{self, BufWriter, Write},
    },
};

#[binrw::binrw]
#[derive(Debug)]
#[brw(magic = b"MOCHA", little)]
struct Metadata {
    #[brw(align_after = 1024)]
    pub size: u64,
}

/// Generate a Mocha image.
pub async fn brew_mocha<S, D>(source: S, destination: D) -> io::Result<()>
where
    S: AsRef<Utf8Path>,
    D: AsRef<Utf8Path>,
{
    let source = source.as_ref();
    let destination = destination.as_ref();
    let erofs_name = destination.with_extension("erofs");
    let mocha_name = destination.with_extension("mocha");

    // Generate erofs from a directory.
    Command::new("/usr/bin/mkfs.erofs")
        .arg("-T0")
        .arg("--force-gid=1")
        .arg("--force-uid=1")
        .arg(erofs_name.as_str())
        .arg(source.as_str())
        .execution_policy((Category::Network, Rule::Kill))
        .execution_policy((Category::SetUsers, Rule::Kill))
        .spawn()?
        .wait()
        .await?;

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

/// Mount a Mocha image.
pub fn drink_mocha<P>(name: &str, directory: P) -> io::Result<()>
where
    P: AsRef<Utf8Path>,
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
