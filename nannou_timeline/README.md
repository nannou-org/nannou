# nannou_timeline [![Crates.io](https://img.shields.io/crates/v/nannou_timeline.svg)](https://crates.io/crates/nannou_timeline) [![Crates.io](https://img.shields.io/crates/l/nannou_timeline.svg)](https://github.com/nannou-org/nannou_timeline/blob/master/LICENSE-MIT) [![docs.rs](https://docs.rs/nannou_timeline/badge.svg)](https://docs.rs/nannou_timeline/)

A widget designed for controlling and viewing data over time. This crate was
developed for a generative music workstation but has abstracted for general use.

![nannou_timeline demo.rs example](https://i.imgur.com/IGnzfKy.png)

While this is designed and developed by the nannou organisation, this widget
should be compatible with all conrod GUI project.

Please see [**the nannou guide**](https://guide.nannou.cc) for more information
on how to get started with nannou!

## Features

- Continuous and discrete numeric automation.
- A set of readily available track types:
    - Piano roll.
    - Toggle automation.
    - Bang automation.
    - Numeric automation (continuous and discrete).
- Playhead widget.
- Easy-to-use API.
- Resizable tracks.
- Track pinning.
- Musical structure grid display (supports varying time signatures).
- Compatible with any conrod project.

## TODO

- [ ] Update to Rust 2018.
- [ ] Add support for free-form time (currently only supports musically
      structured time).
- [ ] Add ability to continuously scroll.
- [ ] Move tracks into a separate crate.
- [ ] Add example demonstrating how to create a custom track widget.
- [ ] Finish making toggle automation interactive.
- [ ] Add bezier curve support to numeric automation tracks.
- [ ] Smart cursor "snap-to-grid" functionality.
- [ ] Many track type ideas:
    - [ ] Plotter track (useful for waveforms / generic 1D data).
    - [ ] Audio waveform track.
    - [ ] Video preview track.
