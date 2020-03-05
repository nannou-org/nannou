# Unreleased


# Version 0.13.1 (2020-03-05)

- Add `Texture::inner` producing a reference to the inner texture handle.
- Add `Texture::into_inner` producing the inner texture handle.
- Add `Into<TextureHandle>` impl for `Texture`
- Add `Texture::into_ui_image`.

# Version 0.13.0 (2020-03-05)

- Transition from `vulkano` to `wgpu` for all graphics handling!
    - Fixes llooooooooooottss of macOS bugs.
    - The `draw` and `ui` APIs now render via wgpu.
    - Replace `vk` module with `wgpu` module.
    - Replace `examples/vulkan` with `examples/wgpu`.
    - Big step towards enabling web target.
    - Add `wgpu::TextureBuilder` to simplify texture building process.
    - Add `wgpu::TextureReshaper` for writing a texture to another of differing
      size, format and sample_count.
    - Add `wgpu::TextureCapturer` for reading textures onto CPU as images.
- Update to `winit` 0.21. Another big step towards enabling web target. Also
  includes an overhaul of the application loop which should be significantly
  simpler.
- Update `view` function API to take `Frame` by value rather than by reference.
  For example, rather than:
  ```rust
  fn view(app: &App, model: &Model, frame: &Frame) {}
  ```
  the `view` function signature now must look like:
  ```rust
  fn view(app: &App, model: &Model, frame: Frame) {}
  ```
  This was necessary to enable ergonomic texture capturing.
- `frame.submit()` can now be used to submit the frame to the GPU before the end
  of the `view` function.
- `nannou::sketch` now returns a `SketchBuilder`. This allows for specifying the
  sketch `.size(w, h)`, but now requires that `.run()` is called (or the sketch
  won't do anything!).
- A `.size(w, h)` builder has been added to the `app::Builder` type that allows
  for specifying a default window size.
- Add `window.capture_frame(path)` method for capturing the next frame to an
  image file at the given file path.
- Add a `simple_capture.rs` example.
- Add a `capture_hi_res.rs` example.
- `sketch`'s now need a call to `.run()` to do anything.
- `sktech`'s now support a `.size(width, height)` builder method for setting
  window size.
- The `app::Builder` now also supports a `.size(width, height)` method for
  specifying the default window width and height.
- `LoopMode`s have been simplified:
    - `Wait` no longer requires `update_following_event` or `update_interval`
    - `NTimes` no longer requires `update_interval`
    - `Refresh` no longer requires `minimum_update_interval` or `windows`

# Version 0.12.0 (2019-11-03)

- Update vulkano dependencies to 0.16 in order to address `metal` related bug on
  macOS.
- Update conrod dependencies to 0.68 for vulkano patch. New version includes
  copy/paste, double-click select and shift-click select support for the
  `TextEdit` widget.
- [Breaking] Small change to Vulkan debug items.
- [Breaking] New fields have been added to `DynamicState`.
- Update shade_runner to 0.3 for vulkano patch.
- Frame command buffer builder no longer provides access to unrelated
  `secondary` buffer methods.

# Version 0.11.0 (2019-09-17)

- Update vulkano and shaderc dependencies to fix linux build issues.
- Add an example that demonstrates using the Draw API with multiple windows.
- Fix a bug where `Draw::to_frame` would `panic!` when used between multiple
  windows.
- Add lyon for 2D tessellation.
- A new `geom::path()` API has been added that allows for building 2D vector
  graphics paths as an iterator yielding `lyon::path::PathEvent`s. This adds
  support for curves, arcs, sub-paths and more.
- A `draw.path()` API has been added to allow for taking advantage of paths via
  the `Draw` API. `draw.path().stroke()` produces a path that will be rendered
  via stroke tessellation, `draw.path().fill()` produces a path that will be
  rendered via fill tessellation.
- The `draw.polyline()` and `draw.line()` APIs are now implemented in terms of
  `draw.path().stoke()`.
- All known polyline bugs should be fixed.
- `draw.polygon()` has been updated to use lyon's `FillTessellator` allowing for
  concave shapes.
- `draw.polygon()` now supports optional stroke tessellation of its outline and
  includes a suite of stroke option builder methods including line join types,
  stroke weight, stroke color, etc. See the `SetStroke` method docs to find all
  new methods now available.
- `.no_fill()` and `.stroke(color)` can be called on all polygon types to
  indicate that no fill tessellation is required or to specify stroke color
  respectively.
- All other `draw` API polygons (`rect`, `quad`, `tri`, `ellipse`) have been
  implemented in terms of `draw.polygon()`, allowing them to take advantage of
  the same stroke tessellation options.
- The line `thickness` methods have been replaced with `stroke_weight` and
  `weight` methods.
- Fixes a pretty severe bug where any draw primitives that use the intermediary
  mesh would produce incorrect triangulation indices if they weren't the first
  instance to be created.
- `draw.polygon()` will temporarily lose support for individually colored
  vertices. This is due to limitations with lyon's `FillTessellator`, however
  these are in the process of being addressed.
- `draw.tri()` and `draw.quad()` now expect `Point2`s instead of `Point3`s. This
  was a trade-off in order to take advantage of the lyon tessellators which only
  support 2D geometry. Currently, the draw API's 3D story is very limited
  anyway, and this can likely be revisited as a part of a larger 3D tessellation
  overhaul. For now, `draw.mesh()` can still be used for drawing arbitrary 3D
  via the `draw` API.
- Introduce notosans crate for guaranteed default fallback font. Can be disabled
  by disabling default-features.
- Refactor default font out of ui module into app module.
- Add `text` module along with `text::Builder` and `Text` APIs. Allows for
  laying out multi-line, justified, auto-wrapping text.
- Add `draw.text("foo")` API. Currently quite slow as it uses the `draw.path()`
  API internally, but this can be improved in the future by adopting a glyph
  cache.
- Add `simple_text.rs` and `simple_text_path.rs` examples.

# Version 0.10.0 (2019-07-21)

- Change the `view` function signature to take `Frame` by reference rather than
  by value.
- Remove depth format constants in favour of querying supported formats.
- Update from palette 0.2 to 0.4.
- Add shorthand color constructors to the `color` module.
- Remove nannou named colors in favour of using palette's.
- Added a `named_color_reference.rs` example for finding suitable colors and to
  act as a test of color accuracy.
- Change the `Frame` image type from the swapchain color format (non-linear
  sRGB) to a linear sRGB format for better consistency across platforms.
- Add Window::rect method.
- Add `simple_audio_file.rs` playback example.
- Add a new `NTimes` loop mode.
- Separate the OSC API out into a `nannou_osc` crate.
- Separate the audio API out into a `nannou_audio` crate.
- Update laser examples for switch from `lasy` to `nannou_laser`.
- Update deps:
  - rand 0.7
  - conrod 0.66
  - vulkano 0.13

# Version 0.9.0 (2019-05-28)

- Change graphics rendering backend from glium to vulkano! This affects a wide
  range of nannou's API including:
  - Window creation and methods. Each window now has it's own associated Vulkan
    swapchain and related methods.
  - The `Frame` API now wraps a single swapchain image and a vulkan command
    buffer builder.
  - The `draw` API's renderer now renders via a vulkan pipeline.
  - The `Ui` API's renderer now renders via a vulkan pipeline.
  - The `App` includes methods for accessing the vulkan instance.
  - The `App` can be built with a custom vulkan instance and custom debug
    callback function.
  - A suite of examples demonstrating low-level vulkano access have been added.
- Improve the clarity of the `App` creation process by introducing an
  `app::Builder` type. Examples have been updated accordingly.
- The `view` function is now called separately for each frame for each window,
  rather than a single frame for all windows at once. The window a frame is
  associated with can be determined via `Frame::window_id`.
- A suite of new event handling functions have been added as an alternative to
  matching on the raw `Event` type. This has simplified a lot of the examples.
  See the `app::Builder` and `window::Builder` docs for the newly available
  methods and more documentation.
- Add `Window::grab_cursor` and `Window::hide_cursor` methods.
- Add `window::SwapchainFramebuffers` helper type.
- Add `vk::Framebuffer` to simplify framebuffer management.
- Remove the `state::time::Duration` type in favour of a `DurationF64` trait.
- Prefer sRGB colour formats when building swapchain.
- Update deps:
  - conrod crates 0.65
  - image 0.21
  - noise 0.5
  - pennereq 0.3
  - rand 0.6
  - sample 0.10
  - winit 0.19
- Fix mouse positioning on HiDPI macOS displays.
- Draw to an intermediary frame before resolving to the swapchain to simplify
  MSAA and keeping the image consistent between frames.
- Add some laser streaming examples using the `nannou-org/lasy` crate.

# Version 0.8.0 (2018-07-19)

- Update deps: glium 0.22, image 0.19.
- Change `random_range` to check that `min` is smaller than `max`, swapping the
  two if not. This avoids some common `panic!`s.
- Add expanding conversion implementations that vector types.
- Add custom `Vector` types - replaces the use of `cgmath::{VectorN, PointN}`
  types.
- Update `rand` to version `0.5`.
- Add `geom::scalar` module. Move `DefaultScalar` to `scalar::Default`.
- Fix the order of `geom::line` vertices.
- Add a `draw.polygon()` API.
- Remove `geom::polyline` module.
- Add `geom::line::join` module with `miter` submodule implementation.

# Version 0.7.0 (2018-06-13)

- Add better `panic!` message to `map_range` if cast fails.
- Add many items to prelude (audio, io, math, osc, ui, window).
- Change event positioning types to use DefaultScalar.
- Implement `draw.polygon()`
- Implement `draw.mesh()`
- Update internal `IntoDrawn` API to support a dynamic number of arbitrary
  vertices.
- Update `Drawing` API to allow builders to produce new `Drawing` types.

# Version 0.6.0 (2018-06-07)

- Add beginnings of Nature of Code and Generative Gestaltung examples.
- Add `App::elapsed_frames` method.
- Remove `app.window.id` field in favour of more reliable `app.window_id`
  method.
- Change `ui::Builder` so that it no longer requires `window::Id`. Now defaults
  to focused window.
- Fix several HiDPI related bugs introduced in the last winit update.
- Add support for rotation and orientation to `draw` API.

# Version 0.5.2 (2018-04-28)

- Improve efficiency of the `App` proxy by only making OS calls when needed.

# Version 0.5.1 (2018-04-26)

- Add `Ui::draw_to_frame_if_changed` method which only draws if necessary.
- Add README to nannou-package.
- Add missing `Cargo.toml` details to nannou-package.
- Add an `io` module with some helper functions simplifying `std::io`.
- Add `fmod` function to `math` module.

# Version 0.5.0 (2018-04-17)

- Add simple accessor field for getting the time since app start in secs.
- Add ability to adjust glyph cache size for text (ui).
- Update to glium 0.21 and conrod 0.59.
- Remove `app.window.*` fields in favour of `app.window_rect()` method.
- Enable vsync and 4x multisampling by default.
- Add fullscreen toggle keyboard shortcuts.
- Add `nannou-new` and `nannou-package` tools.
- Add `Draw::line` along with custom line builders to `Drawing`.
- Change `draw::Background` coloring API to match the `SetColor` API.
- Change OSC default binding address from `127.0.0.1` to `0.0.0.0`.
- Add many new items to prelude.
- Add more `Rect` constructors.
- Add `Range::lerp` method.
- Window name defaults to "nannou - exe_name" if no name is given.
- Correct existing and add missing geometry scalar default types.

# Version 0.4.0 (2018-03-25)

- Add hsv (aka hsb) color builder methods to Draw API.
- Add nicer panic message for when `max_supported_input/output_channels` methods
  fail.
- Add `Ellipse::triangle_indices` method.
- Improve efficiency of `geom::graph::node::Transform`.
- Add a `Duration` wrapper with simpler access methods (`secs`, `ms`, etc).
- Add `quad`, `rect` and `tri` methods to `Draw` API.
- Add `draw::mesh::vertex::IntoPoint` trait with many impls.
- Add `geom::centroid` function.
- Add `Quad::bounding_rect` and `bounding_cuboid` methods.
- Add more `geom::Vertex` impls.
- Add `Drawing<Ellipse>::radius` method.
- Fix bug in audio input stream.
- Add simpler `Frame` clear methods.
- Add simpler `App` constructors.
- Fix bug where mesh types would not clear properly.
- Remove `color` module from prelude to avoid `ui` module conflicts.
- Add named colors.
- Add `draw` module. A high-level, simple, expressive graphics API.
- Add `mesh` module. Supports meshes with custom channels and layouts.
- Add `geom::Graph` for composing together geometric primitives.
- Add new triangles iterators to `geom::quad` and `geom::rect` modules.
- Add `geom::cuboid` module.
- Add `geom::polyline` module.
- Add `geom::line` module.

# Version 0.3.0 (2018-02-18)

- Add `audio::Stream::id` method.
- Add `ExactSize` and `DoubleEnded` iterator implementations for
  `audio::Buffer`.
- Update for input stream support.
- Add support for audio input devices and streams
- Expose helper Vector and Point constructors in prelude.
- Add `state` module for tracking mouse, keyboard and window state.
- Add `geom` module. Includes basic 2D primitives (lines/text).
- Add `ease` module which re-exports the `pennereq` crate.
- Add `map_range`, `partial_max`, `min`, `clamp` math functions
- Begin implementation of  tutorial `basics` examples.

# Version 0.2.0 (2017-12-12)

- Add support for audio output device and streams.
- Add OSC support.

BEGINNING OF CHANGELOG
