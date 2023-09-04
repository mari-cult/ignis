use {
    crate::util,
    nix::kmod::{self, DeleteModuleFlags, ModuleInitFlags},
    rustix::path::Arg,
    std::{ffi::OsStr, fs::File, io, path::Path},
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Ignore {
    pub version: bool,
    pub magic: bool,
}

pub fn load_module<P: AsRef<Path>>(path: P, options: String, ignore: Ignore) -> io::Result<()> {
    let file = File::open(path)?;
    let mut flags = ModuleInitFlags::empty();

    flags.set(
        ModuleInitFlags::MODULE_INIT_IGNORE_MODVERSIONS,
        ignore.version,
    );

    flags.set(ModuleInitFlags::MODULE_INIT_IGNORE_VERMAGIC, ignore.magic);

    options.into_with_c_str(|options| {
        kmod::finit_module(&file, options, flags).map_err(util::nix_to_rustix)
    })?;

    Ok(())
}

pub fn unload_module<S: AsRef<OsStr>>(name: S, force: bool) -> io::Result<()> {
    let name = name.as_ref();
    let mut flags = DeleteModuleFlags::O_NONBLOCK;

    if force {
        flags.insert(DeleteModuleFlags::O_TRUNC);
    }

    name.into_with_c_str(|name| kmod::delete_module(name, flags).map_err(util::nix_to_rustix))?;

    Ok(())
}
