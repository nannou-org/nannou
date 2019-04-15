# lasy [![Build Status](https://travis-ci.org/nannou-org/lasy.svg?branch=master)](https://travis-ci.org/nannou-org/lasy) [![Crates.io](https://img.shields.io/crates/v/lasy.svg)](https://crates.io/crates/lasy) [![Crates.io](https://img.shields.io/crates/l/lasy.svg)](https://github.com/nannou-org/lasy/blob/master/LICENSE-MIT) [![docs.rs](https://docs.rs/lasy/badge.svg)](https://docs.rs/lasy/)

A cross-platform laser DAC detection and streaming API.

**lasy** aims to be a higher-level API around a variety of laser protocols
providing a unified interface for detecting DACs and streaming data to them.

## Features

- **DAC Detection**: Detect all DACs available to the system.
- **Specify maximum latency**: Choose how much latency you wish to allow for
  achieving the right balance between stream stability and low-latency to suit
  the DAC.
- **Frame Streams**: Stream data to the DAC as a sequence of 2D vector images
  without worrying about details like path optimisation, etc.
- **Raw Streams**: While frame streams are convenient, sometimes direct access
  to the lower-level raw DAC stream is required (e.g. when visualising a raw
  audio stream). This can be accessed via the **RawStream** API.
- **Frame Optimisation**: **lasy** implements the full suite of optimisations
  covered in *Accurate and Efficient Drawing Method for Laser Projection* by
  Purkhet Abderyim et al. These include Graph optimisation, draw order
  optimisation, blanking delays and sharp angle delays. See [the
  paper](https://art-science.org/journal/v7n4/v7n4pp155/artsci-v7n4pp155.pdf)
  for more details.
- **Custom frame rate**: Choose the rate at which you wish to present frames.
  **lasy** will determine the number of points used to draw each frame using the
  connected DAC's points-per-second.

*Note: Higher level features like pattern generators and frame graphs are out of
scope for lasy, though could be built downstream. The priority for this crate is
easy laser DAC detection and high-quality, high-performance data streams.*

## Supported Protocols

Currently, **lasy** only supports the open source [Ether Dream
DAC](https://ether-dream.com/) protocol. The plan is to progressively add
support for more protocols as they are needed by ourselves and users throughout
the lifetime of the project.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

**Contributions**

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
