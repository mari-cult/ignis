use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

fn create_dir_all(dir: impl AsRef<Path>) -> Result<(), String> {
    let dir = dir.as_ref();

    fs::create_dir_all(dir).map_err(|error| {
        let dir = dir.display();

        format!("create_dir_all: {dir}: {error}")
    })
}

fn copy(source: impl AsRef<Path>, destination: impl AsRef<Path>) -> Result<(), String> {
    let source = source.as_ref();
    let destination = destination.as_ref();

    let source = source.canonicalize().map_err(|error| {
        let source = source.display();

        format!("copy: failed to resolve source={source}: {error}")
    })?;

    let source = source.as_path();

    fs::copy(source, destination).map_err(|error| {
        let source = source.display();
        let destination = destination.display();

        format!("copy: source={source} destination={destination}: {error}")
    })?;

    Ok(())
}

fn cargo_build(package: &str, target: &str) -> Result<(), String> {
    let status = Command::new("cargo")
        .arg("build")
        .arg("--package")
        .arg(package)
        .arg("--target")
        .arg(target)
        .spawn()
        .map_err(|error| format!("cargo build {package} for {target}: {error}"))?
        .wait()
        .map_err(|error| format!("cargo build {package} for {target}: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err("failed to build".to_string())
    }
}

fn create_iso(
    bios_cd: impl AsRef<Path>,
    uefi_cd: impl AsRef<Path>,
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
) -> Result<(), String> {
    let bios_cd = bios_cd.as_ref();
    let uefi_cd = uefi_cd.as_ref();
    let input = input.as_ref();
    let output = output.as_ref();

    let status = Command::new("xorriso")
        .args(["-as", "mkisofs"])
        .arg("-b")
        .arg(bios_cd)
        .arg("-no-emul-boot")
        .args(["-boot-load-size", "4"])
        .arg("-boot-info-table")
        .arg("--efi-boot")
        .arg(uefi_cd)
        .arg("-efi-boot-part")
        .arg("--efi-boot-image")
        .arg("--protective-msdos-label")
        .arg(input)
        .arg("-o")
        .arg(output)
        .spawn()
        .map_err(|error| format!("create_iso: {error}"))?
        .wait()
        .map_err(|error| format!("create_iso: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err("failed to build".to_string())
    }
}

fn run_qemu(
    ovmf_code: impl AsRef<Path>,
    ovmf_vars: impl AsRef<Path>,
    iso: impl AsRef<Path>,
) -> Result<(), String> {
    let ovmf_code = ovmf_code.as_ref().display();
    let ovmf_vars = ovmf_vars.as_ref().display();
    let iso = iso.as_ref();

    let status = Command::new("qemu-system-x86_64")
        .args(["-M", "q35"])
        .args([
            "-drive",
            &format!("if=pflash,unit=0,format=raw,file={ovmf_code},readonly=on"),
        ])
        .args([
            "-drive",
            &format!("if=pflash,unit=1,format=raw,file={ovmf_vars}"),
        ])
        .arg("-cdrom")
        .arg(iso)
        .args(["-m", "2G"])
        .spawn()
        .map_err(|error| format!("qemu: {error}"))?
        .wait()
        .map_err(|error| format!("qemu: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err("failed to build".to_string())
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("xtask: {error}");
    }
}

fn run() -> Result<(), String> {
    let Some(root_dir) = env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .as_deref()
        .and_then(Path::parent)
        .map(PathBuf::from)
    else {
        return Err("xtask must be executed within the ignis project".to_string());
    };

    let target_dir = env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root_dir.join("target"));

    let external_limine = root_dir.join("external/boot/limine");

    let iso = target_dir.join("ignis.iso");

    let iso_dir = target_dir.join("iso");
    let iso_limine = iso_dir.join("boot/limine");
    let iso_efi = iso_dir.join("EFI/BOOT");

    let ovmf_dir = target_dir.join("ovmf");
    let ovmf_code = ovmf_dir.join("ovmf-code-x86_64.fd");
    let ovmf_vars = ovmf_dir.join("ovmf-vars-x86_64.fd");

    create_dir_all(&iso_limine)?;
    create_dir_all(&iso_efi)?;
    create_dir_all(&ovmf_dir)?;

    copy("/usr/share/edk2/x64/OVMF_CODE.4m.fd", &ovmf_code)?;
    copy("/usr/share/edk2/x64/OVMF_VARS.4m.fd", &ovmf_vars)?;

    cargo_build("kernel", "x86_64-unknown-none")?;

    copy(
        target_dir.join("x86_64-unknown-none/debug/kernel"),
        iso_limine.join("ignis.elf"),
    )?;

    copy(
        root_dir.join("boot/limine.conf"),
        iso_limine.join("limine.conf"),
    )?;

    for file in [
        "limine-bios.sys",
        "limine-bios-cd.bin",
        "limine-uefi-cd.bin",
    ] {
        copy(external_limine.join(file), iso_limine.join(file))?;
    }

    for file in ["BOOTIA32.EFI", "BOOTX64.EFI"] {
        copy(external_limine.join(file), iso_efi.join(file))?;
    }

    create_iso(
        "boot/limine/limine-bios-cd.bin",
        "boot/limine/limine-uefi-cd.bin",
        iso_dir,
        &iso,
    )?;

    run_qemu(ovmf_code, ovmf_vars, iso)?;

    Ok(())
}
