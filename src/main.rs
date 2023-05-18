use camino::Utf8Path;

use {
    crate::{
        args::Args,
        index::{Index, Part},
    },
    milk_progress::ProgressBars,
    mocha_cargo::Cargo,
    mocha_target::Target,
    mocha_utils::{Category, Command, Rule, Stdio},
    std::{
        fs,
        io::{self, BufWriter},
        time::Instant,
    },
};

mod args;
mod index;

#[tokio::main]
async fn main() {
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

            let packages = packages
                .into_iter()
                .flat_map(|package| index.index.get(&*package).map(|entry| (package, entry)))
                .filter(|(_package, entry)| !entry.installed)
                .collect::<Vec<_>>();

            println!("To be installed: {packages:?}");

            for (package, entry) in packages {
                let source_dir = Utf8Path::new("/mocha/sources").join(&package);

                // TODO: Figure out how to handle multiple sources.
                for source in &entry.serialized.sources {
                    println!("Syncing source: {source:?}");

                    let git = Command::new("/usr/bin/git")
                        .current_dir(&source_dir)
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit())
                        .execution_policy((Category::SetUsers, Rule::Kill));

                    let git = if !source_dir.join(".git").is_dir() {
                        let _ = fs::remove_dir_all(&source_dir);

                        fs::create_dir_all(&source_dir).unwrap();

                        git.arg("clone")
                            .arg("--depth=1")
                            .arg(source.clone())
                            .arg(".")
                    } else {
                        git.arg("pull").arg("--depth=1")
                    };

                    git.spawn().unwrap().wait().await.unwrap();
                }

                for part in &entry.serialized.parts {
                    println!("{part:?}");

                    match part {
                        Part::Rust {
                            features,
                            depends,
                            artifacts,
                        } => {
                            let cargo = Cargo::new("/mari/.cargo/bin/cargo").unwrap();
                            let mut child = cargo
                                .build(&source_dir)
                                .features(features)
                                .target("x86_64-gnu".parse().unwrap())
                                .spawn()
                                .unwrap();

                            let mut stdout = BufWriter::new(io::stdout());
                            let mut completed = 0;
                            let mut total = 0;
                            let mut time =
                                tokio::time::interval(std::time::Duration::from_millis(50));

                            loop {
                                tokio::select! {
                                    biased;

                                    status = child.process() => {
                                        if let Ok(Some(status)) = status {
                                            completed = status.0;
                                            total = status.1;
                                        } else {
                                            break;
                                        }
                                    }

                                    _ = time.tick() => {

                                let width = terminal_size::terminal_size()
                                    .map(|(width, height)| width.0 as usize)
                                    .unwrap_or(80);

                                ProgressBars::new()
                                    .add(&package, completed, total)
                                    .terminal_width(width)
                                    .render(&mut stdout)
                                    .unwrap();

                                    }
                                }
                            }
                        }
                        _ => {}
                    }
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
