# nannou [![Actions Status](https://github.com/nannou-org/nannou/workflows/nannou/badge.svg)](https://github.com/nannou-org/nannou/actions) [![Backers on Open Collective](https://opencollective.com/nannou/backers/badge.svg)](https://guide.nannou.cc/contributors.html#backers) [![Sponsors on Open Collective](https://opencollective.com/nannou/sponsors/badge.svg)](https://guide.nannou.cc/contributors.html#sponsors)

![nannou_logo](https://i.imgur.com/1ldLFfj.png)

An open-source creative-coding toolkit for Rust.

**nannou** is a collection of code aimed at making it easy for artists to
express themselves with simple, fast, reliable, portable code.  Whether working
on a 12-month installation or a 5 minute sketch, this framework aims to
give artists easy access to the tools they need.

The project was started out of a desire for a creative coding framework inspired
by Processing, OpenFrameworks and Cinder, but for Rust. <sup>Named after
[this](https://www.youtube.com/watch?v=A-Pkx37kYf4).</sup>

nannou is built on the [**Bevy**](https://bevyengine.org) game engine. Its
crates are exposed as a set of Bevy plugins, so nannou's familiar sketch/app
API and `Draw` interface can be used on their own or dropped into any Bevy
application.

|     |     |     |
| --- |:---:| ---:|
| [![1](https://i.imgur.com/kPn91tW.gif)](https://github.com/nannou-org/nannou/blob/master/examples/draw/draw_polygon.rs) | [![2](https://i.imgur.com/gaiWHZX.gif)](https://github.com/nannou-org/nannou/blob/master/examples/ui/egui/simple_ui.rs) | [![3](https://i.imgur.com/lm4RI4N.gif)](https://github.com/nannou-org/nannou/blob/master/examples/draw/draw_polyline.rs) |

### A Quick Note

It is still early days and there is a lot of work to be done. Feel free to help out!

## The Guide

- [**Welcome!**](https://www.guide.nannou.cc/)
- [**Why Nannou?**](https://www.guide.nannou.cc/why_nannou.html)
  - [**Goals**](https://www.guide.nannou.cc/why_nannou.html#goals)
  - [**Why Rust?**](https://www.guide.nannou.cc/why_nannou.html#why-rust)
  - [**FOSS Licensing**](https://guide.nannou.cc/why_nannou.html#why-the-apachemit-dual-licensing)
- [**Getting Started**](https://www.guide.nannou.cc/getting_started.html)
  - [**Platform-specific Setup**](https://www.guide.nannou.cc/getting_started/platform-specific_setup.html)
  - [**Installing Rust**](https://www.guide.nannou.cc/getting_started/installing_rust.html)
  - [**Editor Setup**](https://www.guide.nannou.cc/getting_started/editor_setup.html)
  - [**Running Examples**](https://www.guide.nannou.cc/getting_started/running_examples.html)
  - [**Create A Project**](https://www.guide.nannou.cc/getting_started/create_a_project.html)
  - [**Updating Nannou and Rust**](https://guide.nannou.cc/getting_started/updating.html)
- [**Tutorials**](https://www.guide.nannou.cc/tutorials.html)
- [**Community Tutorials**](https://www.guide.nannou.cc/community_tutorials.html)
- [**Developer Reference**](https://www.guide.nannou.cc/developer_reference.html)
- [**API Reference**](https://www.guide.nannou.cc/api_reference.html)
- [**Showcases**](https://www.guide.nannou.cc/showcases.html)
- [**Changelog**](https://www.guide.nannou.cc/changelog.html)
- [**Contributors**](https://www.guide.nannou.cc/contributors.html)
- [**Code of Conduct**](https://guide.nannou.cc/code_of_conduct.html)

## Examples

The following collection of **examples** are a great way to get familiar with nannou.

| **Directory** | **Description** |
| --- | --- |
| [**`examples/`**](./examples) | A collection of examples demonstrating how to use nannou! |
| [**`generative_design/`**](./generative_design) | Examples from [Generative Gestaltung](http://www.generative-gestaltung.de/), ported from p5.js to nannou. |
| [**`nature_of_code/`**](./nature_of_code) | Examples from [Nature of Code](https://natureofcode.com/), ported from Processing to nannou. |

If you spot an example that interests you, you may run it with the following:

```
cargo run --release --example <example_name>
```

where `<example_name>` is the example's file name without the `.rs`. Note that
the first run might take a while in order to build nannou first, but consecutive
runs should be much quicker.

## Libraries

The following nannou **libraries** are included within this repository.

| **Library** | **Links** | **Description** |
| --- | --- | --- |
| [**`nannou`**](./nannou) | [![Crates.io](https://img.shields.io/crates/v/nannou.svg)](https://crates.io/crates/nannou) [![docs.rs](https://docs.rs/nannou/badge.svg)](https://docs.rs/nannou/) | App, sketching, graphics, windowing and UI. |
| [**`nannou_audio`**](./nannou_audio) | [![Crates.io](https://img.shields.io/crates/v/nannou_audio.svg)](https://crates.io/crates/nannou_audio) [![docs.rs](https://docs.rs/nannou_audio/badge.svg)](https://docs.rs/nannou_audio/) | Audio hosts, devices and streams. |
| [**`nannou_core`**](./nannou_core) | [![Crates.io](https://img.shields.io/crates/v/nannou_core.svg)](https://crates.io/crates/nannou_core) [![docs.rs](https://docs.rs/nannou_core/badge.svg)](https://docs.rs/nannou_core/) | Just-the-core for headless, embedded and libraries. |
| [**`nannou_draw`**](./nannou_draw) | [![Crates.io](https://img.shields.io/crates/v/nannou_draw.svg)](https://crates.io/crates/nannou_draw) [![docs.rs](https://docs.rs/nannou_draw/badge.svg)](https://docs.rs/nannou_draw/) | The `Draw` API for 2D and 3D graphics. |
| [**`nannou_isf`**](./nannou_isf) | [![Crates.io](https://img.shields.io/crates/v/nannou_isf.svg)](https://crates.io/crates/nannou_isf) [![docs.rs](https://docs.rs/nannou_isf/badge.svg)](https://docs.rs/nannou_isf/) | An Interactive Shader Format pipeline. |
| [**`nannou_laser`**](./nannou_laser) | [![Crates.io](https://img.shields.io/crates/v/nannou_laser.svg)](https://crates.io/crates/nannou_laser) [![docs.rs](https://docs.rs/nannou_laser/badge.svg)](https://docs.rs/nannou_laser/) | LASER devices, streams and path optimisation. |
| [**`nannou_midi`**](./nannou_midi) | [![Crates.io](https://img.shields.io/crates/v/nannou_midi.svg)](https://crates.io/crates/nannou_midi) [![docs.rs](https://docs.rs/nannou_midi/badge.svg)](https://docs.rs/nannou_midi/) | Simple MIDI input and output. |
| [**`nannou_osc`**](./nannou_osc) | [![Crates.io](https://img.shields.io/crates/v/nannou_osc.svg)](https://crates.io/crates/nannou_osc) [![docs.rs](https://docs.rs/nannou_osc/badge.svg)](https://docs.rs/nannou_osc/) | Simple OSC sender and receiver. |
| [**`nannou_video`**](./nannou_video) | [![Crates.io](https://img.shields.io/crates/v/nannou_video.svg)](https://crates.io/crates/nannou_video) [![docs.rs](https://docs.rs/nannou_video/badge.svg)](https://docs.rs/nannou_video/) | Video decoding and playback. |
| [**`nannou_webcam`**](./nannou_webcam) | [![Crates.io](https://img.shields.io/crates/v/nannou_webcam.svg)](https://crates.io/crates/nannou_webcam) [![docs.rs](https://docs.rs/nannou_webcam/badge.svg)](https://docs.rs/nannou_webcam/) | Cross-platform webcam capture. |
| [**`nannou_wgpu`**](./nannou_wgpu) | [![Crates.io](https://img.shields.io/crates/v/nannou_wgpu.svg)](https://crates.io/crates/nannou_wgpu) [![docs.rs](https://docs.rs/nannou_wgpu/badge.svg)](https://docs.rs/nannou_wgpu/) | WGPU helpers and extensions. |

## Links

- [Website](https://www.nannou.cc/)
- [Guide](https://www.guide.nannou.cc/)
- [Slack](https://communityinviter.com/apps/nannou/join-nannou-slack) / [Matrix](https://matrix.to/#/+nannou:matrix.org)
- [Support nannou!](https://opencollective.com/nannou)
