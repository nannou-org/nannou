# nannou [![Build Status](https://travis-ci.org/nannou-org/nannou.svg?branch=master)](https://travis-ci.org/nannou-org/nannou) [![Crates.io](https://img.shields.io/crates/v/nannou.svg)](https://crates.io/crates/nannou) [![Crates.io](https://img.shields.io/crates/l/nannou.svg)](https://github.com/nannou-org/nannou/blob/master/LICENSE-MIT) [![docs.rs](https://docs.rs/nannou/badge.svg)](https://docs.rs/nannou/)

An open-source creative-coding toolkit for Rust.

**nannou** is a collection of code aimed at making it easy for artists to
express themselves with simple, fast, reliable, portable code.  Whether working
on a 12-month laser installation or a 5 minute sketch, this framework aims to
give artists easy access to the tools they need.

The project was started out of a desire for a creative coding framework inspired
by Processing, OpenFrameworks and Cinder, but for Rust. <sup>Named after
[this](https://www.youtube.com/watch?v=A-Pkx37kYf4).</sup>

### A Quick Note

This project is brand new and there is a lot of work to be done. Feel free to
help out!

## Contents

- [**Goals**](#goals)
- [**Why Rust?**](#why-rust)
- [**Getting Started**](#getting-started)
  - [**Platform-specific Setup**](#platform-specific-setup)
  - [**Install Rust**](#install-rust)
  - [**IDE Setup**](#ide-setup)
  - [**Nannou Examples**](#nannou-examples)
  - [**More Resources**](#more-resources)
- [**License**](#license)

## Goals

Nannou aims to provide easy, cross-platform access to the things that artists need:

- [x] **Windowing & Events** via [winit](https://crates.io/crates/winit).
- [x] **Audio** via [CPAL](https://crates.io/crates/cpal). *Input and
  output streams. Duplex are not yet supported.*
- [ ] **Video** input, playback and processing (*would love suggestions and
  ideas*).
- [x] **GUI** via [conrod](https://crates.io/crates/conrod). *May switch to a
  custom nannou solution in the future*.
- **Geometry** with functions and iterators for producing vertices and indices:
  - [x] 1D - `Scalar`, `Range`.
  - [x] 2D - `Rect`, `Line`, `Ellipse`, `Polygon`, `Polyline`, `Quad`,
    `Tri`.
  - [x] 3D - `Cuboid`.
  - [ ] 3D TODO - `Ellipsoid`, `Cube`, Prisms, Pyramids, *Hedrons, etc.
  - [x] Vertex & index iterators.
  - [x] [Graph](https://docs.rs/nannou/latest/nannou/geom/graph/index.html) for
    composing geometry.
- **Graphics** via Vulkan (via [vulkano](https://github.com/vulkano-rs/vulkano)):
  - [x] [Draw](https://docs.rs/nannou/latest/nannou/draw/index.html) API. E.g.
    `draw.ellipse().w_h(20.0, 20.0).color(RED)`.
  - [x] [Mesh](https://docs.rs/nannou/latest/nannou/mesh/index.html) API.
  - [ ] Image API (currently only supported via GUI).
  - [ ] Framebuffer object API.
- **Protocols**:
  - [x] [OSC](https://docs.rs/nannou/latest/nannou/osc/index.html) - Open Sound
    Control.
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
  - [x] [Audio](https://docs.rs/nannou/latest/nannou/app/struct.Audio.html).
  - [ ] Video.
  - [ ] Lasers.
  - [ ] Lights.
  - [ ] LEDs.
- [ ] **Graphical Node Graph** via [gantz](https://github.com/nannou-org/gantz).
- [ ] **GUI Editor**.

Nannou aims to **use only pure-rust libraries**. New users should require
nothing more than `cargo build` to get going. Falling back to C-bindings will be
considered as a temporary solution in the case that there are no Rust
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
  C++ fast](https://benchmarksgame.alioth.debian.org/u64q/compare.php?lang=rust&lang2=gpp).
- [**A standard package manager**](https://crates.io/) that makes it very
  easy to handle dependencies and share your own projects in seconds.
- **Highly portable.** Easily build for MacOS, Linux, Windows, Android, iOS and
  [so many others](https://forge.rust-lang.org/platform-support.html).
- **No header files** (and no weird linking errors).
- **Sum Types and Pattern Matching** (and no `NULL`).
- **Local type inference**. Only write types where it matters, no need to repeat
  yourself.
- A more modern, **ƒunctional and expressive style**.
- **Memory safe and data-race-free!** Get your ideas down without the fear of
  creating pointer spaghetti or segfault time-sinks.
- **Immutability by default.** Easily distinguish between variables that can
  change and those that can't at a glance.
- **Module system** resulting in very clean and concise name spaces.
- One of the kindest internet communities we've come across (please visit
  mozilla's #rust or /r/rust if you're starting out and need any pointers)

## Getting Started

Nannou is a library written for the Rust programming language. Thus, the first
step step is to install Rust!

### Install Rust

To install Rust, open up your terminal, copy the text below, paste it into your
terminal and hit enter.

```bash
curl https://sh.rustup.rs -sSf | sh
```

Now Rust is installed! Next we will install some tools that help IDEs do fancy
things like auto-completion and go-to-definition.

```bash
rustup component add rust-src rustfmt-preview rust-analysis
```

Please see [this link](https://www.rust-lang.org/en-US/install.html) if you
would like more information on the Rust installation process.

### Platform-specific Setup

Depending on what OS you are running, you might require an extra step or two.

- **macOS**: Ensure that you have `xcode-tools` installed:
  ```
  xcode-select --install
  ```
  If you already have `xcode-tools` installed don't worry! This command will let
  you know.

- **linux** ensure you have the following system packages installed:
  - alsa dev package

    For Fedora users:
    `$ sudo dnf install alsa-lib-devel`

    For Debian/Ubuntu users:
    `$ sudo apt-get install libasound2-dev`

  - curl lib dev package

    Nannou depends on the `curl-sys` crate. Some Linux distributions use LibreSSL instead of OpenSSL (such as AlpineLinux, Voidlinux, possibly [others](https://en.wikipedia.org/wiki/LibreSSL#Adoption) if manually installed).


### IDE Setup

**VS Code**

For new Rust users we recommend using VS-Code as your editor and IDE for Nannou
development. Currently it seems to have the best support for the Rust language
including syntax highlighting, auto-complete, code formatting, etc. It also
comes with an integrated unix terminal and file navigation system. Below are the
steps we recommend for getting started with Nannou development using VS-Code.

1. [Download VS-Code](https://code.visualstudio.com/download) for your OS.
2. In VS code user settings, set `"rust-client.channel": "stable"`.
3. [Install
   RLS](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust) (the
   Rust Language Server) plugin for VS-Code.
4. Click on the 'view' menu and select 'integrated terminal'.

**Other IDEs**

Although we recommend VS-Code, it is also possible to configure the following
development environments.

1. [Sublime Text](https://packagecontrol.io/packages/Rust%20Enhanced)
2. [Atom](https://atom.io/packages/language-rust)
3. [Intellij IDEA](https://intellij-rust.github.io)
4. [Vim](https://github.com/rust-lang/rust.vim)
5. [Emacs](https://github.com/rust-lang/rust-mode)
6. [Visual Studio](https://github.com/PistonDevelopers/VisualRust)
7. [Eclipse](https://github.com/RustDT/RustDT) (No longer maintained)

### Nannou Examples

The easiest way to get familiar with Nannou is to explore the examples. To get
the examples we just need to clone this repository.

```
git clone https://github.com/nannou-org/nannou
```

If you do not have `git` installed you can press the "Clone or download" button
at the top of this page and then press "Download .zip".

Now, change the current directory to `nannou`.

```
cd nannou
```

Run the example using cargo.

```
cargo run --release --example simple_draw
```

The `--release` flag means we want to build with optimisations enabled.

If you are compiling nannou for the first time you will see cargo download and build all the necessary dependencies.

![Alt Text](https://thumbs.gfycat.com/ShabbyWildGermanspitz-size_restricted.gif)

Once the example compiles you should see the following window appear.

<img src="https://thumbs.gfycat.com/MalePracticalIberianchiffchaff-size_restricted.gif" width="600" height="400" />


To run any of the other examples, replace `simple_draw` with the name of the
desired example.

### More Resources

- [Official Rust Book](https://doc.rust-lang.org/book/second-edition/index.html)
- [Rust by Example](https://rustbyexample.com/)
- [Porting C++ projects to Rust GitHub Book](https://locka99.gitbooks.io/a-guide-to-porting-c-to-rust/content/)
- [#rust-beginners IRC](https://chat.mibbit.com/?server=irc.mozilla.org&channel=%23rust-beginners)
- [Udemy Rust Course](https://www.udemy.com/rust-lang/) (paid)
- [Nannou Website](http://nannou.cc)
- [Nannou Forum](http://forum.nannou.cc)
- [Nannou Slack](https://communityinviter.com/apps/nannou/nannou-slack)

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

**Contributions**

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
