# Upgrading nannou

You can upgrade to a new version of nannou by editing your `Cargo.toml` file to use the new
crate. For v0.12 use the line `nannou = "0.12"`. Run
```bash
cargo update
```
inside the nannou directory to upgrade all dependencies.

Building Nannou examples might still fail. This is most likely due to new language features.
Nannou 0.12 for example requires rustc 1.35.0.
You can upgrade your (local) rust toolchain by executing
```bash
rustup update
```
