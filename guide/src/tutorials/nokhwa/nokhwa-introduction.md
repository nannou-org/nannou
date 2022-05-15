# An Intro to Nokhwa

**Tutorial Info**

- Author: [l1npengtul](https://l1n.pengtul.net)
- Required Knowledge:
    - [Anatomy of a nannou App](/tutorials/basics/anatomy-of-a-nannou-app.md)
- Reading Time: 5 minutes
---

## What is Nokhwa?

[Nokhwa](https://crates.io/crates/nokhwa) is a crate that allows for easy use of webcams that works on Linux and Windows (with some support for macOS)

## Setting up Nokhwa

To use Nokhwa in nannou, it is necessary to add the `nannou_nokhwa` crate as a dependency in your nannou project.

Open up your `Cargo.toml` file at the root of your nannou project and add the following line under the `[dependencies]` tag:

```toml
nannou_nokhwa = "0.1.0"
```

The value in the quotes is the version of the Nolhwa package. At the time of writing this, `"0.1.0"` is the latest version.

To get the latest version of the osc library, execute `cargo search nannou_nokhwa` on the command line and read the resulting version from there.

To use the crate in your nannou-projects you can add a use-statement at the top of your `main.rs` file.

```rust,no_run
# #![allow(unused_imports)]
use nannou_nokhwa as nokhwa;
# fn main() {}
```
