use super::{Artifact, Error, Result};
use camino::Utf8Path;
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    os::unix,
    process::{Command, Stdio},
    time::Instant,
};

#[derive(Debug)]
pub struct Package {
    name: String,
    serialized: Serialized,
}

#[derive(Debug, Deserialize, Serialize)]
struct Serialized {
    source: String,
    dependencies: Vec<String>,
    #[serde(default)]
    features: Vec<String>,
    artifacts: Vec<Artifact>,
}

impl Package {
    pub fn from_path<P: AsRef<Utf8Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let name = path.file_stem().unwrap().into();

        let content = fs::read_to_string(path).unwrap();
        let serialized = serde_yaml::from_str(&content)
            .map_err(|error| Error::deserialize_spec(Utf8Path::new(&name), &content, error))?;

        Ok(Self { name, serialized })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn source(&self) -> &str {
        &self.serialized.source
    }

    pub fn dependencies(&self) -> &[String] {
        &self.serialized.dependencies
    }

    pub fn features(&self) -> &[String] {
        &self.serialized.features
    }

    pub fn artifacts(&self) -> &[Artifact] {
        &self.serialized.artifacts
    }

    pub fn install(&self) -> io::Result<()> {
        let root_dir = Utf8Path::new("/mocha");
        let source_dir = root_dir.join("src").join(self.name());
        let target_dir = source_dir.join("target/x86_64-unknown-linux-musl/release");
        let binary_dir = root_dir.join("bin");

        print!(" sync {}.. ", self.name());

        let mut instant = Instant::now();
        let mut command = Command::new("gix");

        if source_dir.exists() {
            command.arg("fetch").args(&["--depth", "1"]);
        } else {
            fs::create_dir(&source_dir)?;

            command
                .arg("clone")
                .args(&["--depth", "1"])
                .arg("--no-tags")
                .arg(self.source())
                .arg(".");
        }

        command
            .current_dir(&source_dir)
            /*.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())*/
            .spawn()?
            .wait()?;

        println!("done! took {:.2?}", instant.elapsed());

        let mut instant = Instant::now();

        print!(" build {}.. ", self.name());

        let mut command = Command::new("cargo");
        let features = self.features().join(",");

        command
            .arg("+nightly")
            .arg("zigbuild")
            .arg(format!("--features={features}"))
            .arg("--no-default-features")
            .arg("--target=x86_64-unknown-linux-musl")
            .arg("--release")
            .current_dir(source_dir)
            /*.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())*/
            .spawn()?
            .wait()?;

        println!("done! took {:.2?}", instant.elapsed());

        for arifact in self.artifacts() {
            match arifact {
                Artifact::Bin { name, rename_to } => {
                    let src_name: &str = name;
                    let src_path = target_dir.join(src_name);
                    let dst_name = rename_to.as_deref().unwrap_or(src_name);
                    let dst_path = binary_dir.join(dst_name);

                    artifact_log("bin", src_name, rename_to.as_deref());

                    let _ = fs::remove_file(&dst_path);
                    fs::copy(src_path, dst_path)?;
                }
                Artifact::Sym { name, points_to } => {
                    let src_name: &str = points_to;
                    let dst_name: &str = name;
                    let dst_path = binary_dir.join(dst_name);

                    artifact_log("sym", src_name, Some(dst_name));

                    let _ = fs::remove_file(&dst_path);
                    unix::fs::symlink(src_name, dst_path)?;
                }
            }
        }

        Ok(())
    }
}

fn artifact_log(kind: &'static str, source_name: &str, destination_name: Option<&str>) {
    use yansi::{Color, Style};

    let kind_style = Style::new(Color::Black).bg(Color::Green);
    let kind = kind_style.paint(format!(" {kind} "));

    if let Some(destination_name) = destination_name {
        println!(" {kind} {source_name} -> {destination_name}");
    } else {
        println!(" {kind} {source_name}");
    }
}
