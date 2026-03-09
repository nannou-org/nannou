# nannou_webcam

**The webcam API for** [**nannou**](https://nannou.cc)**, the creative coding
framework.**

**nannou_webcam** uses [**nokhwa**](https://crates.io/crates/nokhwa) on native
and `MediaStreamTrackProcessor` on wasm for cross-platform webcam capture, with
inspiration from [**bevy_webcam**](https://github.com/mosure/bevy_webcam).
Cameras are automatically discovered as entities and frames are streamed as Bevy
`Image` assets.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
