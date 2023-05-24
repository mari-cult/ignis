//#![deny(warnings)]

use crossterm::style::Stylize;

pub use crate::{args::Args, index::Index, milk::Milk};

pub(crate) mod index;
pub(crate) mod milk;

mod args;

fn main() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .thread_name(concat!(env!("CARGO_PKG_NAME"), "_worker"))
        .build()
        .unwrap()
        .block_on(run())
}

async fn run() {
    let args = Args::parse();
    let milk = Milk::new("/mocha");

    match args {
        Args::Add { packages } => {
            let index = milk.open_index().unwrap();
            let packages = match index.resolve(packages) {
                Ok(packages) => packages
                    .into_iter()
                    .filter(|package| !package.is_installed())
                    .collect::<Vec<_>>(),
                Err(unknown_packages) => {
                    let list = unknown_packages
                        .into_iter()
                        .map(|package| package.as_str().red().to_string())
                        .collect::<Vec<_>>()
                        .join(", ");

                    println!("Unknown packages: {list}");

                    return;
                }
            };

            if packages.is_empty() {
                println!("Nothing to do.");

                return;
            }

            let list = packages
                .iter()
                .map(|package| package.ident().as_str().blue().to_string())
                .collect::<Vec<_>>()
                .join(", ");

            println!("To be installed: {list}");

            for package in packages {
                /*let result = add(&ident, entry).await;

                match result {
                    Ok(()) => println!("Installed: {ident}"),
                    Err(error) => println!("Failed to install {ident}: {error}"),
                }*/
            }
        }
        Args::Del { packages } => {
            let index = milk.open_index().unwrap();
            let packages = match index.resolve(packages) {
                Ok(packages) => packages
                    .into_iter()
                    .filter(|package| package.is_installed())
                    .collect::<Vec<_>>(),
                Err(unknown_packages) => {
                    let list = unknown_packages
                        .into_iter()
                        .map(|package| package.as_str().red().to_string())
                        .collect::<Vec<_>>()
                        .join(", ");

                    println!("Unknown packages: {list}");

                    return;
                }
            };

            if packages.is_empty() {
                println!("Nothing to do.");

                return;
            }

            let list = packages
                .iter()
                .map(|package| package.ident().as_str().blue().to_string())
                .collect::<Vec<_>>()
                .join(", ");

            println!("To be uninstalled: {list}");

            for package in packages {
                let ident = package.ident();
                let prefix = ident.as_str().blue();

                if let Err(error) = package.uninstall() {
                    println!("error: Failed to uninstall {prefix}: {error}");
                }
            }
        }
        Args::Fmt { specs: _ } => {
            //
        }
        Args::Sync { repositories: _ } => {
            //
        }
    }
}

/*async fn add(package: &Package) -> io::Result<()> {
    let source_dir = Utf8Path::new("/mocha/sources").join(package);
    let image_dir = source_dir.join("image");
    let prefix = package.blue();

    // TODO: Figure out how to handle multiple sources.
    for source in &entry.serialized.sources {
        println!("{prefix}: Syncing {}.", format!("{source:?}").green());

        let start_time = Instant::now();
        let git = Command::new("/usr/bin/git")
            .current_dir(&source_dir)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .execution_policy((Category::SetUsers, Rule::Kill));

        let git = if !source_dir.join(".git").is_dir() {
            let _ = fs::remove_dir_all(&source_dir);

            fs::create_dir_all(&source_dir)?;

            git.arg("clone").arg("--depth=1").arg(source).arg(".")
        } else {
            git.arg("pull").arg("--depth=1")
        };

        git.spawn()?.wait().await?;

        let elapsed = start_time.elapsed();

        println!("{prefix}: Sync finished in {elapsed:.2?}.");
    }

    if !image_dir.is_dir() {
        let _ = fs::remove_dir_all(&image_dir);

        fs::create_dir_all(&image_dir)?;
    }

    let mut permissions = Permissions::empty();

    for part in &entry.serialized.parts {
        let start_time = Instant::now();

        match part {
            Part::Rust {
                features,
                depends: _,
                artifacts,
            } => {
                println!("{prefix}: Using {}.", "rust".green());

                if !features.is_empty() {
                    println!("{prefix}: With features:");

                    let features = features
                        .iter()
                        .map(|string| string.to_string().yellow().to_string())
                        .collect::<Vec<_>>()
                        .join(", ");

                    println!("{prefix}:   {features}");
                }

                println!("{prefix}: Produces artifacts:");

                let artifacts_list = artifacts
                    .iter()
                    .map(|string| string.to_string().yellow().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");

                println!("{prefix}:   {artifacts_list}");

                let target = "x86_64-gnu".parse().unwrap();
                let cargo = Cargo::new("/mari/.cargo/bin/cargo")?;
                let mut child = cargo
                    .build(&source_dir)
                    .features(features)
                    .target(target)
                    .spawn()?;

                let mut stdout = BufWriter::new(io::stdout());
                let mut status = (0, 0);
                let mut time = tokio::time::interval(std::time::Duration::from_millis(50));

                loop {
                    tokio::select! {
                        biased;

                        new_status = child.process() => {
                            if let Ok(Some(new_status)) = new_status {
                                status = new_status;
                            } else {
                                break;
                            }
                        }
                        _ = time.tick() => {
                            ProgressBars::new()
                                .add(package, status.0, status.1)
                                .auto_terminal_width()
                                .render(&mut stdout)?;
                        }
                    }
                }

                println!();

                for artifact in artifacts {
                    if let Some((link_name, source)) = artifact.split_once(" -> ") {
                        use std::os::unix::fs::symlink;

                        symlink(source, image_dir.join(link_name))?;

                        println!(
                            "{prefix}: Symlink {link_name} -> {source}",
                            link_name = link_name.cyan(),
                            source = source.green(),
                        );
                    } else {
                        let source = source_dir
                            .join("target")
                            .join(target.rust_triple())
                            .join("release")
                            .join(artifact);

                        fs::copy(source, image_dir.join(artifact))?;

                        permissions.insert(Permissions::EXECUTE);

                        println!("{prefix}: Binary {binary}", binary = artifact.green());
                    }
                }
            }
            _ => {}
        }

        let elapsed = start_time.elapsed();

        println!("{prefix}: Produced artifacts in {elapsed:.2?}");
    }

    let _ = fs::create_dir_all("/mocha/images");
    let _ = fs::create_dir_all(&system_path);

    mocha_image::brew_mocha(image_dir, &mocha_path, permissions).await?;
    mocha_image::drink_mocha(mocha_path, system_path)?;

    Ok(())
}
*/
