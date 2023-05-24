#![deny(warnings)]

use {
    binrw::{BinRead, BinWrite},
    mocha_fs::Utf8Path,
    mocha_utils::process::{Category, Command, Rule},
    rustix::{cstr, fs::MountFlags},
    std::{
        fs::{self, File},
        io::{self, Error, ErrorKind, Write},
    },
};

/// Default mount flags for Mocha images.
const DEFAULT_FLAGS: MountFlags = MountFlags::NOATIME
    .union(MountFlags::NODEV)
    .union(MountFlags::NODIRATIME)
    .union(MountFlags::NOEXEC)
    .union(MountFlags::NOSUID)
    .union(MountFlags::RDONLY);

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct Permissions: u64 {
        const EXECUTE = 1 << 0;
        const SET_USERS = 1 << 1;
    }
}

#[binrw::binrw]
#[derive(Debug)]
#[brw(magic = b"MOCHA", little)]
pub struct Metadata {
    #[br(try_map = read_permissions)]
    #[bw(map = Permissions::bits)]
    permissions: Permissions,

    #[brw(align_after = 1024)]
    _alignment: (),
}

impl Metadata {
    #[inline]
    pub fn new(permissions: Permissions) -> Self {
        Self {
            permissions,
            _alignment: (),
        }
    }

    #[inline]
    pub fn permissions(&self) -> Permissions {
        self.permissions
    }
}

/// Generate a Mocha image.
pub async fn brew_mocha<S, D>(source: S, destination: D, permissions: Permissions) -> io::Result<()>
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

    let mut mocha = mocha_fs::create_new_buffered(&mocha_name)?;
    let metadata = Metadata::new(permissions);

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

/// Read a Mocha image's metadata.
pub fn mocha_metadata<P>(path: P) -> io::Result<Metadata>
where
    P: AsRef<Utf8Path>,
{
    let mut mocha = mocha_fs::open_buffered(path)?;
    let metadata =
        Metadata::read(&mut mocha).map_err(|error| Error::new(ErrorKind::InvalidData, error))?;

    Ok(metadata)
}

/// Mount a Mocha image.
pub fn drink_mocha<S, D>(source: S, destination: D) -> io::Result<()>
where
    S: AsRef<Utf8Path>,
    D: AsRef<Utf8Path>,
{
    let source = source.as_ref();
    let destination = destination.as_ref();
    let metadata = mocha_metadata(source)?;
    let device = loopy::Loop::options()
        .offset(1024)
        .read_only(true)
        .open(source)?;

    let permissions = metadata.permissions();
    let noexec = !permissions.contains(Permissions::EXECUTE);
    let nosuid = !permissions.contains(Permissions::SET_USERS);
    let mut flags = DEFAULT_FLAGS;

    flags.set(MountFlags::NOEXEC, noexec);
    flags.set(MountFlags::NOSUID, nosuid);

    rustix::fs::mount(
        device.path(),
        destination.as_std_path(),
        cstr!("erofs"),
        flags,
        cstr!(""),
    )?;

    Ok(())
}

/// Attempt to read a `u64` as `Permissions`.
#[inline]
fn read_permissions(permissions: u64) -> Result<Permissions, &'static str> {
    Permissions::from_bits(permissions).ok_or("invalid permissions")
}
