use {
    crate::{artifact::Artifact, error::Error},
    clap::Parser,
    std::{env, io},
};

mod artifact;
mod error;
mod package;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Parser)]
pub struct Args {
    atoms: Vec<String>,
}

fn main() {
    let packages = walkdir::WalkDir::new("/mocha/repos")
        .max_depth(2)
        .min_depth(2)
        .sort_by_file_name()
        .into_iter()
        .flatten()
        .flat_map(|entry| {
            let path = camino::Utf8Path::from_path(entry.path())?;
            let repository = path.parent()?.as_str();
            let spec = path.file_name()?;

            match package::Package::from_path(path) {
                Ok(package) => Some(package),
                Err(error) => {
                    error.emit();
                    std::process::exit(0);
                }
            }
        })
        .collect::<Vec<_>>();

    let args = Args::parse();

    if args.atoms.is_empty() {
        for package in packages {
            println!("{}", package.name());
            println!("  {}", package.source());
            println!("  {:?}", package.features());
            println!("  {:?}", package.artifacts());
            println!("  {:?}", package.dependencies());
            println!();
        }
    } else {
        for atom in args.atoms {
            let package = packages.iter().find(|package| package.name() == atom);

            if let Some(package) = package {
                package.install().expect("lol");
            }
        }
    }
}
