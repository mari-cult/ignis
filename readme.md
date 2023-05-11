### Installation

Milk is written in rust, and heavily depends on the
zig toolchain. Get rust from [https://rustup.rs](https://rustup.rs).
As it is installed to your home, if your system
provides it, say yes to it's complaint. Get the 
master build of zig from [https://ziglang.org/download](https://ziglang.org/download).
Extract the tarball and export PATH to provide the `zig` binary.

Install the musl target.

```bash
rustup target add x86_64-unknown-linux-musl
````

Install the following cargo extension that uses Zig.

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
cargo zigbulild --target=x86_64-unknown-linux-musl
```

Use Milk to build & install Milk.
  
```bash
target/x86_64-unknown-linux-musl/release/milk add milk
````