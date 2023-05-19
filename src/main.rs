use camino::Utf8Path;

use {
    crate::{
        args::Args,
        index::{Entry, Index, Part},
    },
    crossterm::style::Stylize,
    mocha_cargo::Cargo,
    mocha_progress::ProgressBars,
    mocha_utils::{Category, Command, Rule, Stdio},
    std::{
        fs,
        io::{self, BufWriter},
        time::Instant,
    },
};

mod args;
mod index;

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

    match args {
        Args::Add { packages } => {
            let start_time = Instant::now();
            let index = Index::open().unwrap();
            let elapsed = start_time.elapsed();

            println!(
                "Package index loaded in {elapsed:.2?} ({} {}).",
                index.len(),
                if index.len() == 1 { "entry" } else { "entries" }
            );

            let (packages, unknown_packages): (Vec<_>, Vec<_>) = packages
                .into_iter()
                .map(|package| {
                    let entry = index.index.get(&package);

                    (package, entry)
                })
                .partition(|(_package, maybe_entry)| maybe_entry.is_some());

            let packages = packages
                .into_iter()
                .flat_map(|(package, maybe_entry)| maybe_entry.map(|entry| (package, entry)))
                .filter(|(_package, entry)| !entry.installed)
                .collect::<Vec<_>>();

            let unknown_packages = unknown_packages
                .into_iter()
                .map(|(package, _maybe_entry)| package)
                .collect::<Vec<_>>();

            if !unknown_packages.is_empty() {
                let package_list = unknown_packages
                    .into_iter()
                    .map(|package| package.as_str().red().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");

                println!("Unknown packages: {package_list}");
                return;
            }

            if packages.is_empty() {
                println!("Nothing to do.");
                return;
            }

            let package_list = packages
                .iter()
                .map(|(package, _entry)| package.as_str().blue().to_string())
                .collect::<Vec<_>>()
                .join(", ");

            println!("To be installed: {package_list}");

            for (package, entry) in packages {
                let result = add(&package, entry).await;

                match result {
                    Ok(()) => println!("Installed: {package}"),
                    Err(error) => println!("Failed to install {package}: {error}"),
                }
            }
        }
        Args::Fmt { specs } => {
            //
        }
        Args::Sync { repositories } => {
            //
        }
    }
}

async fn add(package: &str, entry: &Entry) -> io::Result<()> {
    let source_dir = Utf8Path::new("/mocha/sources").join(&package);
    let image_dir = source_dir.join("image");

    // TODO: Figure out how to handle multiple sources.
    for source in &entry.serialized.sources {
        println!(
            "{}: Syncing {}.",
            package.blue(),
            format!("{source:?}").green()
        );

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

        println!("{}: Sync finished in {elapsed:.2?}.", package.blue());
    }

    if !image_dir.is_dir() {
        let _ = fs::remove_dir_all(&image_dir);

        fs::create_dir_all(&image_dir)?;
    }

    for part in &entry.serialized.parts {
        let start_time = Instant::now();

        match part {
            Part::Rust {
                features,
                depends,
                artifacts,
            } => {
                println!("{}: Using {}.", package.blue(), "rust".green());

                if !features.is_empty() {
                    println!("{}: With features:", package.blue());

                    let features = features
                        .into_iter()
                        .map(|string| string.to_string().yellow().to_string())
                        .collect::<Vec<_>>()
                        .join(", ");

                    println!("{}:   {features}", package.blue());
                }

                println!("{}: Produces artifacts:", package.blue());

                let artifacts_list = artifacts
                    .into_iter()
                    .map(|string| string.to_string().yellow().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");

                println!("{}:   {artifacts_list}", package.blue());

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
                                .add(&package, status.0, status.1)
                                .auto_terminal_width()
                                .render(&mut stdout)?;
                        }
                    }
                }

                println!();

                for artifact in artifacts {
                    let source = source_dir
                        .join("target")
                        .join(target.rust_triple())
                        .join("release")
                        .join(artifact);

                    fs::copy(source, image_dir.join(artifact))?;
                }
            }
            _ => {}
        }

        let elapsed = start_time.elapsed();

        println!("{}: Produced artifacts in {elapsed:.2?}", package.blue());
    }

    let _ = fs::create_dir_all("/mocha/images");

    mocha_image::brew_mocha(image_dir, format!("/mocha/images/{package}-unimplemented")).await?;

    Ok(())
}
