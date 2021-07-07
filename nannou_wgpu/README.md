# `nannou_wgpu`

Items related to wgpu and its integration in nannou!

**WebGPU** is the portable graphics specification that nannou targets allowing
us to write code that is both fast and allows us to target a wide range of
platforms. **wgpu** is the name of the crate we use that implements this
specification.

The `nannou_wgpu` crate re-exports the entire `wgpu` crate along with all of its
documentation while also adding some additional items that makes `wgpu` easier
to use alongside `nannou`.

The `image` feature enables easier interoperation with the `image` crate,
including functions for uploading textures from image files.

The `capturer` feature provides the `wgpu::TextureCapturer` API that aims to
simplify the process of downloading textures from the GPU and easily save them
as image files. As an example, this is particularly useful for recording the
contents of a window or sketch.

Note that when using `nannou_wgpu` via `nannou::wgpu`, both features are enabled
by default.

Useful links:

- An awesome [guide for wgpu-rs](https://sotrh.github.io/learn-wgpu/#what-is-wgpu). Highly
  recommended reading if you would like to work more closely with the GPU in nannou!
- The [wgpu-rs repository](https://github.com/gfx-rs/wgpu-rs).
- The [WebGPU specification](https://gpuweb.github.io/gpuweb/).
- WebGPU [on wikipedia](https://en.wikipedia.org/wiki/WebGPU).
