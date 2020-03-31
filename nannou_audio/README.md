# nannou_audio [![Crates.io](https://img.shields.io/crates/v/nannou_audio.svg)](https://crates.io/crates/nannou_audio) [![Crates.io](https://img.shields.io/crates/l/nannou_audio.svg)](https://github.com/nannou-org/nannou_audio/blob/master/LICENSE-MIT) [![docs.rs](https://docs.rs/nannou_audio/badge.svg)](https://docs.rs/nannou_audio/)

**The audio API for** [**nannou**](https://nannou.cc)**, the creative coding
framework.**

Please see [**the nannou guide**](https://guide.nannou.cc) for more information
on how to get started with nannou!

## Features

Some of the features of this API include:

- [x] Access to the available audio devices on the system.
- [x] Spawn any number of input and output audio streams.
- [x] Simple builder API for establishing streams with reasonable defaults.
- [x] Requesting consistent buffer sizes for input and output streams no matter
      the back-end.
- [x] An easy-to-use, non-blocking API.

**nannou_audio** uses and contributes to
[**cpal**](https://github.com/tomaka/cpal) - a pure-Rust, cross-platform audio
library for handling the low-level cross-platform stuff under the hood.

## Examples

You can find examples of **nannou_audio** in action at the [nannou
repository](git@github.com:nannou-org/nannou.git) in the
[examples](https://github.com/nannou-org/nannou/tree/master/examples) directory.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

**Contributions**

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
