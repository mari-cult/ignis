use {
    mocha_fs::Utf8Path,
    mocha_utils::{Category, Command, Rule, Stdio},
    std::io::{self, Write},
};

fn main() {
    let current_dir = "/mari/wip/musl";

    Command::new("/opt/zig/zig")
        .arg("build")
        .current_dir(current_dir)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn_blocking()
        .unwrap()
        .wait()
        .unwrap();
}

const PRELUDE: &str = r#"const std = @import("std");

pub fn build(b: *std.Build) void {
    const optimize = b.standardOptimizeOption(.{});
    const target = b.startTargetOption(.{});

"#;

pub struct Item {
    pub name: String,
    pub c_flags: String,
    pub cxx_flags: String,
    pub includes: Vec<String>,
    pub link_libc: bool,
    pub sources: Vec<String>,
}

fn write_build<P: AsRef<Utf8Path>>(path: P, items: Vec<Item>) -> io::Result<()> {
    let mut build = mocha_fs::create_new_buffered(path)?;

    write!(&mut build, "{PRELUDE}")?;

    for item in items {
        if let Some(name) = item.name.strip_prefix("lib") {
            if let Some(name) = name.strip_suffix(".a") {
                (name, "addStaticLibrary")
            } else if let Some(name) = name.strip_suffix(".so") {
                (name, "addSharedLibrary")
            }
        }
    }

    write!(&mut build, "}}")?;

    build.flush()?;

    Ok(())
}
