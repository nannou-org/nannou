# Updating nannou

You can update to a new version of nannou by editing your `Cargo.toml` file to
use the new crate. For version 0.12 add the line

```toml
nannou = "0.12"
```

Then within the nannou directory run the following to update all dependencies:

```bash
cargo update
```

## Updating Rust.

From time to time, a nannou update might require features from a newer version
of rustc. For example, nannou 0.12 is known to require at least rustc 1.35.0. In
these cases, you can update your rust toolchain to the latest version by running
the following:

```bash
rustup update
```
