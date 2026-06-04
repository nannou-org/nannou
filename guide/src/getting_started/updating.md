# Updating nannou

You can update to a new version of nannou by editing your `Cargo.toml` file to
use the new crate. For version 0.19 add the line

```toml
nannou = "0.19"
```

Then within the nannou directory run the following to update all dependencies:

```bash
cargo update
```

## Updating Rust.

From time to time, a nannou update might require features from a newer version
of rustc. For example, nannou 0.19 uses the Rust 2024 edition and requires a
recent stable toolchain. In these cases, you can update your rust toolchain to
the latest version by running the following:

```bash
rustup update
```
