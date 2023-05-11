use {
    crate::{artifact::Artifact, atom::Atom, error::Error},
    camino::Utf8PathBuf,
    clap::Parser,
};

mod artifact;
mod atom;
mod error;
mod package;

type Result<T> = std::result::Result<T, Error>;

/// Mocha's package manager.
#[derive(Debug, Parser)]
pub enum Args {
    /// Install packages.
    Add(Add),

    /// Format package specifications.
    Fmt(Fmt),

    /// Sync repositories.
    Sync,
}

/// Install packages.
#[derive(Debug, Parser)]
pub struct Add {
    /// <package>@<target>.
    atoms: Vec<Atom>,
}

/// Format package specifications.
#[derive(Debug, Parser)]
pub struct Fmt {
    /// <package>.spec.
    specs: Vec<Utf8PathBuf>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();

    match args {
        Args::Add(Add { atoms }) => {
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
        Args::Fmt(Fmt { specs }) => {
            for spec in specs {
                if let Err(error) = package::Package::format(spec) {
                    error.emit();
                }
            }
        }
        Args::Sync => println!("sunch"),
    }
}
