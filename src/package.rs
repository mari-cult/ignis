use {
    super::{Artifact, Error, Result},
    camino::Utf8Path,
    milk_cargo::{Cargo, Status},
    milk_progress::ProgressBars,
    milk_target::Target,
    serde::{Deserialize, Serialize},
    std::{
        fs,
        io::{self, BufWriter},
        os::unix,
        process::{Command, Stdio},
        time::Instant,
    },
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    features: Vec<String>,
    artifacts: Vec<Artifact>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    beta_artifacts: Vec<(String, Vec<String>)>,
}

impl Package {
    pub fn from_path<P: AsRef<Utf8Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let name = path.file_stem().unwrap().into();

        let content = fs::read_to_string(path).unwrap();
        let serialized: Serialized = serde_yaml::from_str(&content)
            .map_err(|error| Error::deserialize_spec(Utf8Path::new(&name), &content, error))?;

        Ok(Self { name, serialized })
    }

    pub fn format<P: AsRef<Utf8Path>>(path: P) -> Result<()> {
        let path = path.as_ref();
        let name = path.file_stem().unwrap();

        let content = fs::read_to_string(path).unwrap();
        let serialized: Serialized = serde_yaml::from_str(&content)
            .map_err(|error| Error::deserialize_spec(Utf8Path::new(&name), &content, error))?;

        let serialized = serde_yaml::to_string(&serialized).unwrap();

        fs::write(path, serialized).unwrap();

        Ok(())
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

    pub async fn install(&self, target: Target) -> io::Result<()> {
        let rust_triple = target.rust_triple();
        let root_dir = Utf8Path::new("/mocha");
        let source_dir = root_dir.join("src").join(self.name());
        let target_dir = source_dir.join(format!("target/{rust_triple}/release"));
        let binary_dir = root_dir.join("bin");

        print!(" sync {}.. ", self.name());

        let instant = Instant::now();
        let mut command = Command::new("git");

        if source_dir.exists() {
            command.arg("fetch").args(&["--depth", "1"]);
        } else {
            fs::create_dir(&source_dir)?;

            command
                .arg("clone")
                .args(&["--depth", "1"])
                .arg(self.source())
                .arg(".");
        }

        command
            .current_dir(&source_dir)
            /*.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())*/
            ;

        println!("!!! {command:?}");

        command.spawn()?.wait()?;

        println!("done! took {:.2?}", instant.elapsed());

        let instant = Instant::now();

        print!(" build {}.. ", self.name());

        use std::io::Write;

        let mut build = std::fs::File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .open(source_dir.join("build.zig"))?;

        writeln!(&mut build, "const std = @import(\"std\");")?;
        writeln!(&mut build)?;
        writeln!(&mut build, "pub fn build(b: *std.Build) void {{")?;
        writeln!(
            &mut build,
            "    const optimize = b.standardOptimizeOption(.{{}});"
        )?;
        writeln!(
            &mut build,
            "    const target = b.standardTargetOptions(.{{}});"
        )?;
        writeln!(&mut build)?;

        for (artifact, sources) in self.serialized.beta_artifacts.iter() {
            let sources = sources
                .iter()
                .map(|source| format!("\"{source}\""))
                .collect::<Vec<_>>()
                .join(",\n        ");

            if let Some(artifact) = artifact.strip_prefix("lib ") {
                writeln!(
                    &mut build,
                    "    const lib{artifact} = b.addStaticLibrary(.{{"
                )?;
                writeln!(&mut build, "        .link_libc = true,")?;
                writeln!(&mut build, "        .name = \"{artifact}\",")?;
                writeln!(&mut build, "        .optimize = optimize,")?;
                writeln!(&mut build, "        .target = target,")?;
                writeln!(&mut build, "    }});")?;
                writeln!(&mut build)?;
                writeln!(&mut build, "    lib{artifact}.addCSourceFiles(&.{{")?;
                writeln!(&mut build, "        {sources}")?;
                writeln!(&mut build, "        }},")?;
                writeln!(&mut build, "        &[_][]const u8{{}},")?;
                writeln!(&mut build, "    );")?;
                writeln!(&mut build, "    lib{artifact}.addIncludePath(\"lib\");")?;
                writeln!(
                    &mut build,
                    "    lib{artifact}.addIncludePath(\"lib/common\");"
                )?;
                writeln!(&mut build)?;
                writeln!(&mut build, "    b.installArtifact(lib{artifact});")?;
                writeln!(&mut build)?;
            }

            if let Some(artifact) = artifact.strip_prefix("bin ") {
                writeln!(&mut build, "    const {artifact} = b.addExecutable(.{{")?;
                writeln!(&mut build, "        .link_libc = true,")?;
                writeln!(&mut build, "        .name = \"{artifact}\",")?;
                writeln!(&mut build, "        .optimize = optimize,")?;
                writeln!(&mut build, "        .target = target,")?;
                writeln!(&mut build, "    }});")?;
                writeln!(&mut build)?;
                writeln!(&mut build, "    {artifact}.addCSourceFiles(&.{{")?;
                writeln!(&mut build, "        {sources}")?;
                writeln!(&mut build, "        }},")?;
                writeln!(&mut build, "        &[_][]const u8{{}},")?;
                writeln!(&mut build, "    );")?;
                writeln!(&mut build, "    {artifact}.addIncludePath(\"lib\");")?;
                writeln!(&mut build, "    {artifact}.addIncludePath(\"lib/common\");")?;
                writeln!(
                    &mut build,
                    "    {artifact}.addObjectFile(\"zig-out/lib/libzstd.a\");"
                )?;
                writeln!(&mut build)?;
                writeln!(&mut build, "    b.installArtifact({artifact});")?;
                writeln!(&mut build)?;
            }
        }

        writeln!(&mut build, "}}")?;

        let mut command = Command::new("zig");

        command
            .arg("build")
            .arg("-Doptimize=ReleaseFast")
            .arg("-Dtarget=x86_64-linux-musl")
            .current_dir(&source_dir);

        println!("!!! {command:?}");

        command.spawn()?.wait()?;

        let cargo = Cargo::new("~/.cargo/bin/cargo")?;
        let mut child = cargo
            .build(&source_dir)
            .features(self.features())
            .target(target)
            .spawn()?;

        let mut stdout = BufWriter::new(io::stdout());

        while let Some(Status { completed, total }) = child.process().await? {
            ProgressBars::new()
                .add(self.name(), completed, total)
                .render(&mut stdout)?;
        }

        println!("done! took {:.2?}", instant.elapsed());

        for arifact in self.artifacts() {
            match arifact {
                Artifact::Bin { name, rename_to } => {
                    let src_name: &str = name;
                    let src_path = target_dir.join(src_name);
                    let dst_name = rename_to.as_deref().unwrap_or(src_name);
                    let dst_path = binary_dir.join(dst_name);

                    let _ = fs::remove_file(&dst_path);
                    fs::copy(src_path, dst_path)?;

                    artifact_log("bin", src_name, rename_to.as_deref());
                }

                Artifact::Sym { name, points_to } => {
                    let src_name: &str = points_to;
                    let dst_name: &str = name;
                    let dst_path = binary_dir.join(dst_name);

                    let _ = fs::remove_file(&dst_path);
                    unix::fs::symlink(src_name, dst_path)?;

                    artifact_log("sym", src_name, Some(dst_name));
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
