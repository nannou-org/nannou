# nannou_laser [![Crates.io](https://img.shields.io/crates/v/nannou_laser.svg)](https://crates.io/crates/nannou_laser) [![Crates.io](https://img.shields.io/crates/l/nannou_laser.svg)](https://github.com/nannou-org/nannou_laser/blob/master/LICENSE-MIT) [![docs.rs](https://docs.rs/nannou_laser/badge.svg)](https://docs.rs/nannou_laser/)

A cross-platform laser DAC detection and streaming API.

**nannou_laser** aims to be a higher-level API around a variety of laser protocols
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
- **Frame Optimisation**: **nannou_laser** implements the full suite of
  optimisations covered in *Accurate and Efficient Drawing Method for Laser
  Projection* by Purkhet Abderyim et al. These include Graph optimisation, draw
  order optimisation, blanking delays and sharp angle delays. See [the
  paper](https://art-science.org/journal/v7n4/v7n4pp155/artsci-v7n4pp155.pdf)
  for more details.
- **Custom frame rate**: Choose the rate at which you wish to present frames.
  **nannou_laser** will determine the number of points used to draw each frame
  using the connected DAC's points-per-second.

*Note: Higher level features like pattern generators and frame graphs are out of
scope for nannou_laser, though could be built downstream. The priority for this
crate is easy laser DAC detection and high-quality, high-performance data
streams.*

## Supported Protocols

Currently, **nannou_laser** supports the [Ether Dream](https://ether-dream.com/) and [Helios](https://bitlasers.com/helios-laser-dac/) open-source DAC protocols.

When creating a new Frame/Raw Stream the type of DAC to be detected can be specified using the `Builder::dac_variant()` method. If this is not specified the Ether dream variant is selected by default.

```
let _laser_api = laser::Api::new();
let laser_stream = _laser_api
    .new_raw_stream(laser_model, laser)
    .dac_variant(laser::DacVariant::DacVariantHelios)
    .build()
    .unwrap();
```

The plan is to progressively add
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
