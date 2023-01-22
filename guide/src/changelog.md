# Changelog

All notable changes to nannou should be documented below! From the most recent,
back to the origins.

---

# Unreleased

- Handle `winit::window::WindowEvent::Occluded` events. 
- Update winit to `0.27`. 
- Add GL backend to default backends for better WASM support.
- Add CI for testing the `wasm32-unknown-unknown` target.
- Enable `wgpu/webgl` when `wasm` feature is enabled.
- Update minimum wgpu version to `0.11.1`, update winit to `0.26`.
- Merge the `nannou_egui` repo into the main `nannou` repo.
- Move `nannou_conrod` and `nannou_timeline` into a new repository:
  https://github.com/nannou-org/nannou_conrod. Both crates are deprecated in
  favour of `nannou_egui`.

---

# `nannou` 0.18.1 (2021-12-17)

- Expose missing `begin()` method in `geom::path::Builder`.
- Set window class for X11 running apps.
- Ensure wakeup calls (`UserEvent`s) provide updates in `Wait` mode.

---

# Version 0.18.0 (2021-11-15)

### wgpu 0.9 -> 0.11

**Note:** As of wgpu 0.10, all nannou projects now require either:

1. The following line in their top-level cargo manifest:
   ```toml
   resolver = "2"
   ```
   OR
2. All packages must use the 2021 edition (or later) of Rust, e.g.
   ```toml
   edition = "2021"
   ```

This requirement is due to wgpu 0.10's reliance on the new version of cargo's
dependency resolver and how it unifies features. Without either of the above
amendments, you will likely run into strange upstream compilation errors. You
can read more about the cargo dependency resolver versions here:

https://doc.rust-lang.org/cargo/reference/resolver.html#resolver-versions

- As of wgpu 0.10, it is now pure Rust! No more SPIR-V cross.
- The concept of the wgpu `SwapChain` has been removed by merging it into the
  `Surface`, simplifying internals and improving approachability.
- WGSL is now wgpu's default shader format. All internal shaders and examples
  have been updated to use WGSL instead of SPIR-V.
- A new `spirv` feature has been added that enables the old behaviour of
  accepting SPIR-V. This is disabled by default to try and keep build times low.
- `TextureUsage`, `BufferUsage`, `ColorWrite` are renamed to plural.
- Renamed `TextureUsage` consts: `SAMPLED` -> `TEXTURE_BINDING`, `STORAGE` ->
  `STORAGE_BINDING`.
- Renamed `InputStepMode` to `VertexStepMode`.
- Readable storage textures are no longer a part of the base API. They are now
  exposed via format-specific features, non-portably.
- Added limits for binding sizes, vertex data, per-stage bindings, and others.
- Adds ability to request a software (fallback) adapter.

For more details, see the wgpu CHANGELOG:

- 0.10: https://github.com/gfx-rs/wgpu/blob/master/CHANGELOG.md#v010-2021-08-18
- 0.11: https://github.com/gfx-rs/wgpu/blob/master/CHANGELOG.md#wgpu-011-2021-10-07

### General

- Update `lyon` to version `0.17`.
- Refactor the `nannou::mesh` module into a `nannou_mesh` crate, re-exported to
  maintain the same API.
- Refactor the `nannou::wgpu` module into a `nannou_wgpu` crate, re-exported to
  maintain the same API.
- Remove the `nannou::ui` module in favour of providing a `nannou_conrod` crate.
  See the updated `examples/ui/conrod` examples to find out how to update. Note
  that input must now be manually submitted to the `Ui` and is no longer done
  automatically by the nannou `App`. The easiest approach is to register a
  `raw_event` function with the `Ui`'s window. Refer to the updated examples for
  demonstration.
- Add community tutorials section to the guide.
- Provided an user-friendly way to get the character value of a pressed keyboard key.
- Clear surface background automatically when re-allocated. Can set the clear
  color via `WindowBuilder::clear_color()`.
- Provided a method which allows users to specify an initial default `LoopMode`
  in both `SketchBuilder` and `Builder`. Updated the relevant example to
  showcase the new functionality.
- Remove unused `daggy` dependency.
- Avoids calling `update` under `LoopMode::Wait` if no events were received.

---

### `nannou` 0.17.1 (2021-07-02)

- Fix some edge-cases in the behaviour of `io::safe_file_save`.

---

# Version 0.17.0 (2021-06-20)

**Upgrade WGPU to 0.9**

Most changes have been about renaming Blend-related data structres and fixing shaders to avoid sampling textures inside of conditionals (wgpu validation layer found this one).
- Item Name changes:
    - `BlendState` -> `BlendComponent`
    - `wgpu::Extend3d::depth` -> `wgpu::Extend3d::depth_of_array_layers`
    - Float tpes are now typed more descripively. E.g., `Float2` -> `Float32x2`

**Refactor core of `nannou` into `nannou_core` crate**

- Add a new `nannou_core` crate, targeted towards headless or embedded apps,
  libraries and rust-gpu.
- Move the `color`, `geom`, `math` and `rand` crates into `nannou_core`.
- Remove the `geom::Graph` type due to lack of strong use-case and no reports of
  use.
- Remove generic scalar param from `Draw` API in favour of using `f32`
  generally.
- Remove `cgmath` computer graphics linear algebra lib in favour of `glam` for
  faster compile times, simpler API, easier documentation, `no_std` support and
  more.
- Refactor the `Rect` and `Cuboid` method implementations that expose `Point`
  and `Vec` to avoid breakage. Previously, our `Point` and `Vector` types were
  generic, however as of switching to `glam` this is no longer the case.
  Instead, methods that used these types are now implemented independently for
  `Rect<f32>` and `Rect<f64>` (likewise for `Cuboid`).

**General**

- Fix a bug in `text::line::Infos` iterator where reported character index was
  incorrect.
- Fix `glyph_colors` miscoloring on resize.
- Enable serializing of color types.
- Enable `nannou_laser` features for documentation build.
- Update dependencies:
    - `conrod_*` from 0.73 to 0.74.
    - `noise` from 0.6 to 0.7 (`image` feature no longer enabled).
    - `rand` from 0.7 to 0.8 (changes `gen_range(a,b)` to `gen_range(a..b)`)
---

# Version 0.16.0 (2021-04-21)

- Add ability to color characters individually to `Draw` API, i.e.
  `draw.text().glyph_colors(color_iter)`.
- Add a `app.quit()` method to allow for quitting the application without user
  input.
- Use the `instant` crate rather than `std::time::Instant` in preparation for
  wasm support.
- Fix a major memory leak and various resize crashes - thanks danwilhelm!
- Fix non-uniform scaling in `Draw` API.

**Update to wgpu 0.7**

These changes mostly involved renaming of items, though also included some
significant refactoring of the `wgpu::RenderPipeline`.

- The `wgpu::RenderPipelineBuilder` had some methods added, some removed in
  order to more closely match the newly refactored `wpgu::RenderPipeline`.
  Documentation of `RenderPipelineBuilder` methods has been added to match
  the upstream wgpu docs of the associated fields.
- The `Sampler` binding type now requires specifying whether or not it uses the
  `Linear` option for any of its minify, magnify or mipmap filters. A
  `wgpu::sampler_filtering` function was added to make it easier to retrieve
  this bool from the `SamplerDescriptor`.
- The vertex buffer `IndexFormat` is now specified while setting the index
  buffer in the render pass command, rather than in the render pipeline
  descriptor.
- Item name changes include:
    - `PowerPreference::Default` -> `PowerPreference::LowPower`
    - `TextureUsage::OUTPUT_ATTACHMENT` -> `TextureUsage::RENDER_ATTACHMENT`
    - `TextureComponentType` -> `TextureSampleType`
    - `component_type` -> `sample_type` (for textures)
    - `BlendDescriptor` -> `BlendState`
    - `VertexAttributeDescriptor` -> `VertexAttribute`
    - `BindingType::SampledTexture` -> `BindingType::Texture`
    - `ColorStateDescriptor` -> `ColorTargetState`
    - `DepthStencilStateDescriptor` -> `DepthStencilState`
    - `VertexBufferDescriptor` -> `VertexBufferLayout`
- Also updates related dependencies:
    - `conrod_derive` and `conrod_core` to `0.72`.

**Update to wgpu 0.6**

For the most part, these changes will affect users of the `nannou::wgpu` module,
but not so much the users of the `draw` or `ui` APIs. *Find the relevant wgpu
changelog entry
[here](https://github.com/gfx-rs/wgpu/blob/master/CHANGELOG.md#v06-2020-08-17).*

- `Window::current_monitor` now returns a result.
- `wgpu::Device::create_buffer_with_data` has been removed in favour of
  a new `wgpu::DeviceExt::create_buffer_init` trait method that takes a
  `wgpu::BufferInitDescripor` as an argument.
- `wgpu::BufferDescriptor` now requires specifying whether or not the buffer
  should be mapped (accessible via CPU) upon creation.
- The swap chain queue `submit` method now takes an iterator yielding commands
  rather than a slice.
- The async API for mapped reads/writes has changed.
- `wgpu::Buffer`s can now be sliced.
- `wgpu::PipelineLayoutDescriptor` requires specifying `push_constant_ranges`.
- The render pass `set_vertex_buffer` method now takes a buffer slice directly,
  rather than a range.
- A new `wgpu::TextureViewInfo` type was added. It represents the
  `wgpu::TextureViewDescriptor` parameters that were supplied to build a
  `wgpu::TextureView`.
- A top-level `wgpu::Instance` type has been introduced.
- Load and store ops have been consolidated into a `wgpu::Operations` type.
- `wgpu::Binding` was renamed to `wgpu::BindGroupEntry`.
- A `RowPaddedBuffer` abstraction was added to more gracefully/safely handle
  conversions between `wgpu::Buffer`s and `wgpu::Texture`s.
- Updates some dependencies:
    - `audrey` to 0.3.
    - `winit` to 0.24.
    - `conrod_derive` and `conrod_core` to 0.71 (`nannou_timeline` only).

### nannou_audio

- Update to CPAL 0.13.1 and from `sample` to `dasp_sample`.
- Add the ability to specify a device buffer size.
- Allow fallback to device default sample rate.
- Add builder method for specifying a stream error callback. This replaces the
  `render_result/capture_result` functions.
- Switch from `failure` to `thiserror` for error handling.
- Rename `format` to `config` throughout to match cpal 0.12.
- Fix bug where stream channel count could be greater than specified.

---

# Version 0.15.0 (2020-10-04)

**Update to WGPU 0.5**

For the most part, these changes will affect users of the `nannou::wgpu` module,
but not so much the users of the `draw` or `ui` APIs. *Find the relevant wgpu
changelog entry
[here](https://github.com/gfx-rs/wgpu/blob/master/CHANGELOG.md#v05-06-04-2020).*

- The *y* axis has been inverted for wgpu pipelines, meaning the y axis now
  increases upwards in NDC (normalised device coordinates). This does not affect
  the `Draw` or `Ui` APIs, but does affect users creating custom render
  pipelines. The updated wgpu examples should demonstrate how to deal with this
  change.
- `wgpu::Device` no longer offers a generic buffer creation method, instead
  requiring that users upload all data as slices of bytes. This required adding
  some small `unsafe` functions for converting data (most often vertices,
  indices and uniforms) to bytes ready for upload to the GPU. See the new
  `wgpu::bytes` module docs for details.
- `wgpu::VertexDescriptor` trait has been removed in favour of a new
  `wgpu::vertex_attr_array!` macro. See updated wgpu examples for a usage
  demonstration.
- `wgpu::BindGroupLayout` now requires the texture component type for sampled
  texture binding types. This means wgpu users may now need to switch between
  render pipelines if dynamically switching between textures at runtime.
- `wgpu::Texture` and `wgpu::TextureView` now expose a `component_type` method,
  allowing for easy retrieval of the `wgpu::TextureComponentType` associated
  with their `wgpu::TextureFormat`.
- Fixes a bug where sometimes `Draw` items could be drawn with an incorrect
  blend mode, primitive topology or bind group.
- `wgpu::Queue`s no longer requires mutable access when submitting command
  buffers. This allowed for the removal of the awkward `Mutex` that was exposed
  when providing access to the window's swapchain queue.
- The `Frame::TEXTURE_FORMAT` has changed from `Rgba16Unorm` to `Rgba16Float`,
  as the `Rgba16Unorm` format was removed from the WGPU spec due to lack of
  consistent cross-platform support. Similarly, `image::ColorType`'s that were
  previously mapped to `Unorm` formats are now mapped their respective `Uint`
  formats instead.
- Update to conrod 0.70 to match wgpu dependency version.
- Remove `threadpool` crate in favour of `futures` crate thread pool feature.
  This is necessary in order to run `TextureCapturer` futures now that they take
  advantage of rust's async futures API.
- Adds the `num_cpus` dependency for selecting a default number of worker
  threads for the `TextureCapturer`'s inner thread pool.

**Texture Capturing Fixes and Improvements**

- Fixes the issue where `TextureCapturer` could spawn more user callbacks than
  there where worker threads to process them.
- Fixes the issue where an application might exit before all
  `window.capture_frames(path)` snapshots have completed.
- Provides a `TextureCapturer::await_active_snapshots(device)` method that
  allows to properly await the completion of all snapshot read futures by
  polling the device as necessary.
- `TextureCapturer::new` now takes the number of workers and optional timeout
  duration as an argument.
- `Snapshot::read_threaded` has been removed in favour of a single
  `Snapshot::read` method that is threaded by default. The old synchronous
  behaviour can be emulated by creating the `TextureCapturer` with a single
  worker. Likewise, `window.capture_frame_threaded` has been removed in favour
  of `window.capture_frame` for the same reason.
- New `app::Builder` and `window::Builder` `max_capture_frame_jobs` and
  `capture_frame_timeout` methods have been added to allow for specifying the
  number of worker threads and optional timeout duration for the windows' inner
  `TextureCapturer`s.

---

### `nannou_laser` 0.14.3 (2020-05-19)

- Add support for enabling/disabling draw path reordering.

---

### `nannou_laser` 0.14.2 (2020-05-15)

- Update `lasy` to 0.4. Adds better support for points and improved euler
  circuit interpolation.

---

### `nannou_laser` 0.14.1 (2020-05-06)

- Add `ilda-idtf` feature to `nannou_laser`.

---

### `nannou` 0.14.1 (2020-05-06)

- Fix bug where `draw::Renderer` was initialised with an incorrect scale factor.
- Fix `Vector::angle_between` implementation.

---

# Version 0.14.0 (2020-04-24)

- Relax trait bounds on many mesh types.
- Add Rgb8 and Rgba8 type aliases.
- Add `vec2.rotate(radians)`, `Vector2::from_angle`, `a.angle_between(b)`,
  `vec.magnitude()` and `vec.magnitude2()` inherent methods.
- Add `rgb_u32` for creating color from hex literals.
- Fix `.z_radians()` behaviour.
- Simplify the fullscreen API.
  [#521](https://github.com/nannou-org/nannou/pull/521).
- Adds a `set_version` script for synchronising version updates across nannou
  crates.
- Add a `random_ascii()` function.
- Add many more "Generative Design" and "Nature of Code" examples.
- Fix bug where `capture_frame_threaded` was not actually threaded.

**The Great Repository Refactor**

- Move nannou src and tests into a `nannou` subdirectory.
- Move `nannou_audio` into the nannou repo.
- Move `nannou_isf` into the nannou repo.
- Move `nannou_laser` into the nannou repo.
- Move `nannou_osc` into the nannou repo.
- Move `nannou_timeline` into the nannou repo.
- Move guide into the nannou repo.
- Add all crates under a single workspace.
- Update README.md with a repository overview.
- Move `run_all_examples.rs` test into a new `scripts/` directory. Add
  ability to run examples of specific packages in the workspace.
- Move `nature_of_code` examples
- Move CHANGELOG into the guide.
- Switch from travis CI to a github workflow.

**Guide**

- Add a "Sketch vs App" tutorial.
- Add a "Window Coordinates" tutorial.
- Add "OSC Introduction" and "OSC Sender" tutorials.
- Ensure the "Drawing 2D Shapes" tutorial code is tested.
- Add automated testing of all guide code on each PR to nannou repo.

**Draw API**

- Update to lyon 0.15.
- Re-add support for colored vertices on polygons.
- Added blend modes, e.g. `draw.blend(blend_desc)`.
- Added scissor, e.g. `draw.scissor(rect)`
- Added transforms, e.g. `draw.scale(s)`, `draw.rotate(r)`.
- Removed many relative positioning methods in favour of draw transforms.
- Add `draw.texture` API.
- Rename all APIs taking `points`, `points_colored` and `points_textured` to
  take iterators yielding tuples, e.g. `point`, `(point, color)`, `(point,
  tex_coords)`.
- Add support for `.points_textured(tex, pts)` to `draw.mesh()`,
  `draw.path()` and `draw.polygon()`.
- Add support for `draw.sampler(sampler_desc)`, for specifying a draw
  context with a custom texture sampler.
- Add `draw.triangle_mode()`, `draw.line_mode()` and `draw.point_mode()` for
  switching between render pipeline primitive topology.
- Add GPU caching of glyphs for text drawn via `draw.text()`. Allows for
  much higher-performance text rendering.
- Add `draw.arrow()` API.
- New examples of the new APIs in the `nannou/examples/draw/` directory.

**WGPU API**

- Simplified texture loading, e.g. `Texture::from_path` and
  `Texture::from_image`.
- Simplified `TextureView` creation via `Texture::view` builder.
- Add `RenderPipelineBuilder`.
- Add `BindGroupLayoutBuilder`.
- Add `BindGroupBuilder`.
- Add `RenderPassBuilder`.
- Add `wgpu::shader_from_spirv_bytes` helper function.

**LASER API**

- Add non-blocking LASER DAC detection via `Api::detect_dacs_async`.
- Add error callback to LASER stream API.
- Expose some missing setters on the LASER stream handles.
- Add nicer socket address and error handling to `nannou_laser::ffi`.

---

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
  ```rust,ignore
  fn view(app: &App, model: &Model, frame: &Frame) {}
  ```
  the `view` function signature now must look like:
  ```rust,ignore
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

---

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

---

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

---

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

---

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

---

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

---

# Version 0.7.0 (2018-06-13)

- Add better `panic!` message to `map_range` if cast fails.
- Add many items to prelude (audio, io, math, osc, ui, window).
- Change event positioning types to use DefaultScalar.
- Implement `draw.polygon()`
- Implement `draw.mesh()`
- Update internal `IntoDrawn` API to support a dynamic number of arbitrary
  vertices.
- Update `Drawing` API to allow builders to produce new `Drawing` types.

---

# Version 0.6.0 (2018-06-07)

- Add beginnings of Nature of Code and Generative Gestaltung examples.
- Add `App::elapsed_frames` method.
- Remove `app.window.id` field in favour of more reliable `app.window_id`
  method.
- Change `ui::Builder` so that it no longer requires `window::Id`. Now defaults
  to focused window.
- Fix several HiDPI related bugs introduced in the last winit update.
- Add support for rotation and orientation to `draw` API.

---

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

---

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

---

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

---

# Version 0.2.0 (2017-12-12)

- Add support for audio output device and streams.
- Add OSC support.

---
<br>
<center>BEGINNING OF CHANGELOG</center>
