use {
    super::{artifact::Artifact, atom::Atom, error::Error, package, Result},
    camino::Utf8PathBuf,
    clap::{arg, Args, Parser},
};

/// Mocha's package manager.
#[derive(Debug, Parser)]
pub enum Milk {
    /// Install packages.
    Add(AddArgs),

    /// Format package specifications.
    Fmt(FmtArgs),

    /// Sync repositories.
    Sync,
}

impl Milk {
    pub async fn run() {
        match Self::parse() {
            Milk::Add(AddArgs { atoms, flags }) => {
                let packages = walkdir::WalkDir::new("/mocha/repos")
                    .max_depth(2)
                    .min_depth(2)
                    .sort_by_file_name()
                    .into_iter()
                    .flatten()
                    .flat_map(|entry| {
                        if entry.file_type().is_file() {
                            let path = camino::Utf8Path::from_path(entry.path())?;
                            let _repository = path.parent()?.as_str();
                            let _spec = path.file_name()?;

                            match package::Package::from_path(path) {
                                Ok(package) => Some(package),
                                Err(error) => error.emit(),
                            }
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                if atoms.is_empty() {
                    for package in packages {
                        println!("{}", package.name());
                        println!("  {}", package.source());
                        println!("  {:?}", package.features());
                        println!("  {:?}", package.artifacts());
                        println!("  {:?}", package.dependencies());
                        println!();
                    }
                } else {
                    for atom in atoms {
                        println!(" -> {atom}");

                        let package = packages
                            .iter()
                            .find(|package| package.name() == atom.package);

                        if let Some(package) = package {
                            package.install(atom.target).await.expect("lol");
                        }
                    }
                }
            }
            Milk::Fmt(FmtArgs { specs }) => {
                for spec in specs {
                    if let Err(error) = package::Package::format(spec) {
                        error.emit();
                    }
                }
            }
            Milk::Sync => println!("sunch"),
        }
    }
}

/// Install packages.
#[derive(Debug, Parser)]
pub struct AddArgs {
    // <package>@<target>
    atoms: Vec<Atom>,
    #[command(flatten)]
    flags: AddFlags,
}

// Build flags
#[derive(Debug, Clone, Args)]
pub struct AddFlags {
    #[arg(long, default_value = "~/.cargo/bin/cargo", help = "Cargo binary")]
    cargo_path: String,
    #[arg(long, default_value = "~/.zig/zig", help = "Zig binary")]
    zig_path: String,
}

/// Format package specifications.
#[derive(Debug, Parser)]
pub struct FmtArgs {
    /// <package>.spec.
    specs: Vec<Utf8PathBuf>,
}
