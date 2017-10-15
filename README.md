nannou
------

An open-source, creative-coding toolkit for Rust.

**nannou** is a collection of code aimed at making it easy for artists to
express themselves with simple, fast, reliable, portable code.  Whether working
on a 12-month laser installation or a 5 minute sketch, this framework aims to
give artists easy access to the tools they need.

The project was started out of a desire for a creative coding framework inspired
by Processing, OpenFrameworks and Cinder, but for Rust. <sup>Named after
[this](https://www.youtube.com/watch?v=A-Pkx37kYf4)</sup>

## A Quick Note

This project is brand new and there is a lot of work to be done. Feel free to
help out!

## Getting Started

- See what the code looks like by checking out [the examples]().
- If you're new to Rust, maybe check out [the official
  book](https://doc.rust-lang.org/book/)?
- Start your own project with:
  ```
  cargo new my_project
  cd my_project
  ```
- Add `nannou = "0.1"` under the `[dependencies]` line in your Cargo.toml.
  This is everything you need to use the framework in your own project or
  sketch. Rust's package manager *cargo* will automatically download and install
  everything you need!

## Goals

- Provide easy, cross-platform access to the things that artists need:
    - [ ] Graphics (via [glium](https://crates.io/crates/glium))
    - [ ] Audio (via [CPAL](https://crates.io/crates/cpal))
    - [ ] Video
    - [ ] Windowing (via [winit](https://crates.io/crates/winit) and
      [glutin](https://crates.io/crates/glutin))
    - [ ] Geometry
    - [ ] GUI
    - [ ] OSC (via [rosc](https://crates.io/crates/rosc))
    - [ ] Lighting & Lasers (DMX, ILDA)
- Use only pure-rust libraries. New users should require nothing more than
  `cargo add nannou` and `cargo build` to get going.
- No `unsafe` code with the exception of bindings to operating systems or
  hardware APIs.
- Remove the need to decide between lots of different backends that provide
  access to the same hardware. Instead, we want to focus on a specific set of
  backends and make sure that they work well.

## Why Rust?

Rust is a language that is both highly expressive and blazingly fast. Here are
some of the reasons why we choose to use it:

- **Super fast**, as in [C and
  C++ fast](https://benchmarksgame.alioth.debian.org/u64q/compare.php?lang=rust&lang2=gpp).
- [**A standard package manager**](https://crates.io/) that makes it very
  easy to handle dependencies and share your own projects in seconds.
- **Highly portable.** Easily move your code between MacOS, Linux or Windows.
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
