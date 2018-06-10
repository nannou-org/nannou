# Unpublished

- Add better `panic!` message to `map_range` if cast fails.
- Add many items to prelude (audio, io, math, osc, ui, window).
- Change event positioning types to use DefaultScalar.
- Implement `draw.polygon()`
- Update internal `IntoDrawn` API to support a dynamic number of arbitrary
  vertices.
- Update `Drawing` API to allow builders to produce new `Drawing` types.

# Version 0.6.0 (2017-06-07)

- Add beginnings of Nature of Code and Generative Gestaltung examples.
- Add `App::elapsed_frames` method.
- Remove `app.window.id` field in favour of more reliable `app.window_id`
  method.
- Change `ui::Builder` so that it no longer requires `window::Id`. Now defaults
  to focused window.
- Fix several HiDPI related bugs introduced in the last winit update.
- Add support for rotation and orientation to `draw` API.

# Version 0.5.2 (2017-04-28)

- Improve efficiency of the `App` proxy by only making OS calls when needed.

# Version 0.5.1 (2017-04-26)

- Add `Ui::draw_to_frame_if_changed` method which only draws if necessary.
- Add README to nannou-package.
- Add missing `Cargo.toml` details to nannou-package.
- Add an `io` module with some helper functions simplifying `std::io`.
- Add `fmod` function to `math` module.

# Version 0.5.0 (2017-04-17)

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

# Version 0.4.0 (2017-03-25)

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

# Version 0.3.0 (2017-02-18)

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
