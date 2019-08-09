# Unreleased

- Add an example that demonstrates using the Draw API with multiple windows.
- Fix a bug where `Draw::to_frame` would `panic!` when used between multiple
  windows.
- Add lyon for 2D tessellation.
- Add `geom::path` module as a nannou-friendly abstraction around lyon's `Path`
  type API. Adds the ability for bezier curves and paths with more options for
  line joins and caps.
- Fix all known polyline bugs by switching to lyon polyline tessellation.
- Add `draw.path()` with methods for specifying line caps, line joins, miter
  limits, approximation tolerance, and closing end and start points. Can receive
  both raw path events as well as a polyline described by a sequence of points.
- Re-implement `draw.line()` in terms of `draw.path()`.
- Add `.stroke(color)` method to ellipse for drawing its outline.
- Remove the `geom::line` module in favour of `geom::path`.

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
