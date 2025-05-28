use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    println!("cargo:rustc-link-arg=-Tkernel/src/linker-{arch}.ld");
    println!("cargo:rerun-if-changed=kernel/src/linker-{arch}.ld");
}
