# Why Nannou?

**nannou** is a collection of code aimed at making it easy for artists to
express themselves with simple, fast, reliable, portable code.  Whether working
on a 12-month installation or a 5 minute sketch, this framework aims to give
artists easy access to the tools they need.

The project was started out of a desire for a creative coding framework inspired
by Processing, OpenFrameworks and Cinder, but for Rust. <sup>Named after
[this](https://www.youtube.com/watch?v=A-Pkx37kYf4).</sup>

## Goals

Nannou aims to provide easy, cross-platform access to the things that artists
need:

- [x] **Windowing & Events** via [winit](https://crates.io/crates/winit).
- [x] **Audio** via [CPAL](https://crates.io/crates/cpal). *Input and
  output streams. Duplex are not yet supported.*
- [ ] **Video** input, playback and processing (*would love suggestions and
  ideas*).
- [x] **GUI** via [conrod](https://crates.io/crates/conrod). *May switch to a
  custom nannou solution [in the
  future](https://github.com/nannou-org/nannou/issues/383)*.
- **Geometry** with functions and iterators for producing vertices and indices:
  - [x] 1D - `Scalar`, `Range`.
  - [x] 2D - `Path`, `Polyline`, `Polygon`, `Rect`, `Line`, `Ellipse`, `Quad`,
    `Tri`.
  - [x] 3D - `Cuboid`.
  - [ ] 3D TODO - `Ellipsoid`, `Cube`, Prisms, Pyramids, *Hedrons, Camera, etc.
  - [x] Vertex & index iterators.
  - [x] [Graph](https://docs.rs/nannou/latest/nannou/geom/graph/index.html) for
    composing geometry.
- **Graphics** via WGPU (via [wgpu-rs](https://github.com/gfx-rs/wgpu-rs)):
  - [x] [Draw](https://docs.rs/nannou/latest/nannou/draw/index.html) API. E.g.
    `draw.ellipse().w_h(20.0, 20.0).color(RED)`.
  - [x] [Mesh](https://docs.rs/nannou/latest/nannou/mesh/index.html) API.
  - [x] [Image](https://docs.rs/nannou/latest/nannou/image/index.html) API.
  - [x] [Texture](https://docs.rs/nannou/latest/nannou/wgpu/struct.Texture.html) API.
  - [x] [WGPU](https://docs.rs/nannou/0.13.1/nannou/wgpu/index.html) API.
  - [x] [Text](https://docs.rs/nannou/0.13.1/nannou/text/index.html) API.
  - [ ] Shader hot-loading. *See
    [hotglsl](https://github.com/nannou-org/hotglsl) and [nannou_isf
    WIP](https://github.com/nannou-org/nannou/tree/master/nannou_isf)*.
- **Protocols**:
  - [x] [OSC](https://docs.rs/nannou_osc) - Open Sound
    Control.
  - [x] [ISF](https://github.com/nannou-org/isf) - Interactive Shader Format.
  - [x] [CITP](https://github.com/nannou-org/citp) - Controller Interface
    Transport Protocol (network implementation is in progress).
  - [x] [Ether-Dream](https://github.com/nannou-org/ether-dream) Laser DAC
    protocol and network implementation.
  - [x] [DMX via sACN](https://github.com/lschmierer/sacn) - commonly used for
    lighting and effects.
  - [x] [Serial](https://crates.io/crates/serial) - commonly used for
    interfacing with LEDs and other hardware.
  - [x] [MIDI](https://crates.io/crates/midir) - Musical Instrument Digital
    Interface.
  - [x] [UDP](https://doc.rust-lang.org/std/net/struct.UdpSocket.html) via
    std.
  - [x] TCP
    [streams](https://doc.rust-lang.org/std/net/struct.TcpStream.html) and
    [listeners](https://doc.rust-lang.org/std/net/struct.TcpListener.html)
    via std.
- **Device & I/O stream APIs**:
  - [x] Windowing.
  - [x] Application events.
  - [x] [Audio](https://docs.rs/nannou/latest/nannou/app/struct.Audio.html).
  - [ ] Video.
  - [x] [Lasers](https://github.com/nannou-org/nannou/tree/master/nannou_laser).
  - [ ] Lights. *For now, we recommend DMX via the [sacn crate](https://docs.rs/sacn/0.4.4/sacn/).*
  - [ ] LEDs. *For now, we recommend DMX via the [sacn crate](https://docs.rs/sacn/0.4.4/sacn/).*
- [ ] **Graphical Node Graph** via [gantz](https://github.com/nannou-org/gantz).
- [ ] **GUI Editor**.

Nannou aims to **use only pure-rust libraries**. As a new user you should
require nothing more than `cargo build` to get going. Falling back to C-bindings
will be considered as a temporary solution in the case that there are no Rust
alternatives yet in development. We prefer to drive forward development of less
mature rust-alternatives than depend on bindings to C code. This should make it
easier for nannou *users* to become nannou *contributors* as they do not have to
learn a second language in order to contribute upstream.

Nannou **will not contain `unsafe` code** with the exception of bindings to
operating systems or hardware APIs if necessary.

Nannou wishes to **remove the need to decide between lots of different backends
that provide access to the same hardware**. Instead, we want to focus on a
specific set of backends and make sure that they work well.

## Why Rust?

Rust is a language that is both highly expressive and blazingly fast. Here are
some of the reasons why we choose to use it:

- **Super fast**, as in [C and
  C++ fast](https://benchmarksgame-team.pages.debian.net/benchmarksgame/fastest/rust-gpp.html).
- [**A standard package manager**](https://crates.io/) that makes it very
  easy to handle dependencies and share your own projects in seconds.
- **Highly portable.** Easily build for MacOS, Linux, Windows, Android, iOS and
  [so many others](https://forge.rust-lang.org/platform-support.html).
- **No header files** and no weird linking errors.
- **Sum Types and Pattern Matching** and no `NULL`.
- **Local type inference**. Only write types where it matters, no need to repeat
  yourself.
- A more modern, **ƒunctional and expressive style**.
- **Memory safe and data-race-free!** Get your ideas down without the fear of
  creating pointer spaghetti or segfault time-sinks.
- **Immutability by default.** Easily distinguish between variables that can
  change and those that can't at a glance.
- **Module system** resulting in very clean and concise name spaces.
- One of the kindest internet communities we've come across. Please visit
  mozilla's #rust or /r/rust if you're starting out and need any pointers.

## Why the Apache/MIT dual licensing?

For the most part, nannou is trying to maintain as much flexibility and compatibility
with the licensing of Rust itself, which is also [dual licensed](https://www.rust-lang.org/policies/licenses).

The Apache 2.0 and MIT license are very similar, but have a few key differences.
Using the Apache 2.0 license for contributions triggers the Apache 2.0 patent grant.
This grant is designed to protect against leveraging the patent law system to bypass
(some) terms of the license. If the contribution is under the Apache 2.0 license, the
contributor assures that they will not claim a violation of (their own) patents. If
someone makes a work based on Apache 2.0 licensed code, they in turn also vow to
not sue their users (for patent infringement).
The MIT license provides compatibility with a lot of other FLOSS licenses.

Further reading:

* [Apache License, Version 2.0](https://opensource.org/licenses/Apache-2.0)
* [MIT License](https://opensource.org/licenses/MIT)
* [Please read: Rust license changing (very slightly)](https://mail.mozilla.org/pipermail/rust-dev/2012-November/002664.html)
* [Rationale of Apache dual licensing](https://internals.rust-lang.org/t/rationale-of-apache-dual-licensing/8952)
* [Against what does the Apache 2.0 patent clause protect?](https://opensource.stackexchange.com/questions/1881/against-what-does-the-apache-2-0-patent-clause-protect)
* [GPLv2 Combination Exception for the Apache 2 License](https://blog.gerv.net/2016/09/gplv2-combination-exception-for-the-apache-2-license/)
