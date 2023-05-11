### Installation

Milk is written in rust, and heavily depends on the
Zig toolchain.

Obtain Rust from [https://rustup.rs](https://rustup.rs).
If it complains that rust is already installed, say `yes`,
it will not interfere with your system. And your system
version is unsupported.

Obtain the master build of Zig from [https://ziglang.org/download](https://ziglang.org/download).
Extract the tarball somewhere, update `PATH` to provide the `zig` binary.

Install the musl target.

```bash
rustup target add x86_64-unknown-linux-musl
````

Install the following cargo extension that uses Zig for builds.

```bash
$ cargo install cargo-zigbuild
```

Create Mocha's filesystem layout (does not interfere with existing systems).

```bash

mkdir -pv /mocha/{bin,src,repos}
```

Fetch the main repository.

```bash
git clone https://github.com/ka1mari/mocha-main /mocha/repos/main
```

Build Milk to build Milk :trolleyzoom:

```bash
cargo zigbuild --target=x86_64-unknown-linux-musl
```

Use Milk to build & install Milk.
  
```bash
target/x86_64-unknown-linux-musl/release/milk add milk
````