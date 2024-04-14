//! nannou's core abstractions.
//!
//! This crate aims to be a stripped-down foundation for nannou projects that don't require
//! windowing or wgpu graphics. These might include:
//!
//! - Headless applications, e.g. for LASER or lighting control.
//! - Embedded applications, e.g. driving motors or LEDs in an art installation.
//! - `rust-gpu` shaders which have strict requirements beyond the limitations of `no_std`.
//! - Hot-loaded dynamic libraries that benefit from faster compilation times.
//!
//! The crate includes nannou's color, math, geometry and noise abstractions without the deep stack
//! of crates required to establish an event loop, interoperate with wgpu, etc. Another way of
//! describing this crate might be "nannou without the I/O".
//!
//! ## Crate `[features]`
//!
//! The primary feature of this crate is support for `#![no_std]`. This means we can use the crate
//! for embedded applications and in some cases rust-gpu shaders.
//!
//! By default, the `std` feature is enabled. For compatibility with a `#![no_std]` environment be
//! sure to disable default features (i.e. `default-features = false`) and enable the `libm`
//! feature. The `libm` feature provides some core functionality required by crates
//!
//! - `std`: Enabled by default, enables the Rust std library. One of the primary features of this
//!   crate is support for `#![no_std]`. This means we can use the crate for embedded applications
//!   and in some cases rust-gpu shaders. For compatibility with a `#![no_std]` environment be sure
//!   to disable default features (i.e. `default-features = false`) and enable the `libm` feature.
//! - `libm`: provides some core math support in the case that `std` is not enabled. This feature
//!   must be enabled if `std` is disabled.
//! - `serde`: enables the associated serde serialization/deserialization features in `glam`,
//!   `palette` and `rand`.

#![no_std]

pub mod geom;
pub mod math;
pub mod prelude;
pub mod rand;

/// Re-export `glam` - linear algebra lib for graphics.
pub use glam;
