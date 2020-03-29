# nannou_osc [![Build Status](https://travis-ci.org/nannou-org/nannou_osc.svg?branch=master)](https://travis-ci.org/nannou-org/nannou_osc) [![Crates.io](https://img.shields.io/crates/v/nannou_osc.svg)](https://crates.io/crates/nannou_osc) [![Crates.io](https://img.shields.io/crates/l/nannou_osc.svg)](https://github.com/nannou-org/nannou_osc/blob/master/LICENSE-MIT) [![docs.rs](https://docs.rs/nannou_osc/badge.svg)](https://docs.rs/nannou_osc/)

**The OSC API for** [**nannou**](https://nannou.cc)**, the creative coding
framework.**

Please see [**the nannou guide**](https://guide.nannou.cc) for more information
on how to get started with nannou!

## Features

Some of the features of this API include:

- [x] Simple OSC `Sender` and `Receiver` API around the raw UDP socket and OSC
  protocol.
- [x] Reasonable defaults for sender and receiver binding UDP addresses.
- [x] Type-safe distinction between "connected" and "unconnected" senders and
  receivers.
- [x] Blocking and non-blocking `Iterator` APIs for `Receiver` type.

**nannou_osc** uses the [**rosc**](https://crates.io/crates/rosc) crate - a
pure-Rust, cross-platform OSC library for handling the low-level protocol
encoding and decoding under the hood. `Sender`s and `Receiver`s are thin,
zero-cost abstractions around the `std::net::UdpSocket` type.

## Examples

You can find examples of **nannou_osc** in action at the [nannou
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
