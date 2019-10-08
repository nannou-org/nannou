//! The User Interface API. Instantiate a [**Ui**](struct.Ui.html) via `app.new_ui()`.

pub use self::conrod_core::event::Input;
pub use self::conrod_core::{
    color, cursor, event, graph, image, input, position, scroll, text, theme, utils, widget,
};
pub use self::conrod_core::{
    Borderable, Bordering, Color, Colorable, Dimensions, FontSize, Labelable, Point, Positionable,
    Range, Rect, Scalar, Sizeable, Theme, UiCell, Widget,
};
pub use crate::conrod_core;
pub use crate::conrod_vulkano;
pub use crate::conrod_winit;

/// Simplify inclusion of common traits with a `nannou::ui::prelude` module.
pub mod prelude {
    // Traits.
    pub use super::{Borderable, Colorable, Labelable, Positionable, Sizeable, Widget};
    // Types.
    pub use super::{
        Bordering, Dimensions, FontSize, Input, Point, Range, Rect, Scalar, Theme, Ui, UiCell,
    };
    // Modules.
    pub use super::{color, image, position, text, widget};
}

use self::conrod_core::text::rt::gpu_cache::CacheWriteErr;
use self::conrod_vulkano::RendererCreationError;
use crate::frame::{Frame, ViewFbo};
use crate::text::{font, Font};
use crate::window::{self, Window};
use crate::{vk, App};
use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error as StdError;
use std::fmt;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use winit;

/// Owned by the `App`, the `Arrangement` handles the mapping between `Ui`s and their associated
/// windows.
pub(crate) struct Arrangement {
    pub(super) windows: RefCell<HashMap<window::Id, Vec<Handle>>>,
}

/// A handle to the `Ui` owned by the `App`.
pub(crate) struct Handle {
    /// A channel used for automatically submitting `Input` to the associated `Ui`.
    pub(crate) input_tx: Option<mpsc::SyncSender<Input>>,
}

/// A handle to the `Ui` for a specific window.
pub struct Ui {
    /// The `Id` of the window upon which this `Ui` is instantiated.
    window_id: window::Id,
    ui: conrod_core::Ui,
    input_rx: Option<mpsc::Receiver<Input>>,
    pub image_map: ImageMap,
    renderer: Mutex<conrod_vulkano::Renderer>,
    render_mode: Mutex<RenderMode>,
}

// The mode in which the `Ui` is to be rendered.
enum RenderMode {
    // The `Ui` is to be rendered as a subpass within an existing render pass.
    //
    // This subpass was specified when building the `Ui`.
    Subpass,
    // The `Ui` has its own render pass and in turn also owns it's own buffers.
    //
    // This mode is necessary for `draw_to_frame` to work.
    OwnedRenderTarget(RenderTarget),
}

// The render pass in which the `Ui` will be rendered along with the owned buffers.
struct RenderTarget {
    render_pass: Arc<dyn vk::RenderPassAbstract + Send + Sync>,
    images: RenderPassImages,
    view_fbo: ViewFbo,
}

// The buffers associated with a render target.
struct RenderPassImages {
    depth: Arc<vk::AttachmentImage<vk::Format>>,
}

/// A type used for building a new `Ui`.
pub struct Builder<'a> {
    app: &'a App,
    window_id: Option<window::Id>,
    dimensions: Option<[Scalar; 2]>,
    theme: Option<Theme>,
    automatically_handle_input: bool,
    pending_input_limit: usize,
    default_font_path: Option<PathBuf>,
    glyph_cache_dimensions: Option<(u32, u32)>,
    render_pass_subpass: Option<Subpass>,
}

/// Failed to build the `Ui`.
#[derive(Debug)]
pub enum BuildError {
    /// Either the given window `Id` is not associated with any open windows or the window was
    /// closed during the build process.
    InvalidWindow,
    RendererCreation(RendererCreationError),
    RenderTargetCreation(RenderTargetCreationError),
    FailedToLoadFont(text::font::Error),
}

/// Failed to create the custom render target for the `Ui`.
#[derive(Debug)]
pub enum RenderTargetCreationError {
    RenderPassCreation(vk::RenderPassCreationError),
    ImageCreation(vk::ImageCreationError),
    NoSupportedDepthFormat,
}

/// An error that might occur while drawing to a `Frame`.
#[derive(Debug)]
pub enum DrawToFrameError {
    InvalidWindow,
    RendererPoisoned,
    RenderModePoisoned,
    InvalidRenderMode,
    ImageCreation(vk::ImageCreationError),
    FramebufferCreation(vk::FramebufferCreationError),
    RendererFill(CacheWriteErr),
    GlyphPixelDataUpload(vk::memory::DeviceMemoryAllocError),
    CopyBufferImageCommand(vk::command_buffer::CopyBufferImageError),
    RendererDraw(conrod_vulkano::DrawError),
    DrawCommand(vk::command_buffer::DrawError),
}

/// The subpass type to which the `Ui` may be rendered.
pub type Subpass = vk::framebuffer::Subpass<Arc<dyn vk::RenderPassAbstract + Send + Sync>>;

/// The image type compatible with nannou's UI image map.
///
/// The `vk::Format` format type allows for specifying dynamically determined image formats.
pub type Image = Arc<vk::ImmutableImage<vk::Format>>;

/// A map from `image::Id`s to their associated `Texture2d`.
pub type ImageMap = conrod_core::image::Map<conrod_vulkano::Image>;

impl conrod_winit::WinitWindow for Window {
    fn get_inner_size(&self) -> Option<(u32, u32)> {
        let (w, h) = self.inner_size_points();
        Some((w as _, h as _))
    }
    fn hidpi_factor(&self) -> f32 {
        self.hidpi_factor()
    }
}

impl Arrangement {
    /// Initialise a new UI **Arrangement** (used by the App).
    pub(crate) fn new() -> Self {
        let windows = RefCell::new(HashMap::new());
        Arrangement { windows }
    }
}

impl<'a> Builder<'a> {
    /// Begin building a new `Ui`.
    pub(super) fn new(app: &'a App) -> Self {
        Builder {
            app,
            window_id: None,
            dimensions: None,
            theme: None,
            automatically_handle_input: true,
            pending_input_limit: Ui::DEFAULT_PENDING_INPUT_LIMIT,
            default_font_path: None,
            glyph_cache_dimensions: None,
            render_pass_subpass: None,
        }
    }

    /// Specify the window on which the **Ui** will be instantiated.
    ///
    /// By default, this is the currently focused window, aka the window returned via
    /// **App::window_id**.
    pub fn window(mut self, window_id: window::Id) -> Self {
        self.window_id = Some(window_id);
        self
    }

    /// Build the `Ui` with the given dimensions.
    ///
    /// By default, the `Ui` will have  the dimensions of the specified window.
    pub fn with_dimensions(mut self, dimensions: [Scalar; 2]) -> Self {
        self.dimensions = Some(dimensions);
        self
    }

    /// Build the `Ui` with the given theme.
    ///
    /// By default, nannou uses conrod's default theme.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Whether or not the `App` should automatically submit input to the `Ui`.
    ///
    /// When enabled, events that can be interpreted as UI `Input` will be passed to the `Ui` via
    /// the `conrod::Ui::handle_input` method.
    ///
    /// Note that `Input`s are not immediately submitted to the `Ui` when received by the `App`.
    /// Instead, they are enqueued for the `Ui` to be processed next time `Ui::set_widgets` is
    /// called. The max number of pending `Input`s before they become ignored can be specified via
    /// the `pending_input_limit` method.
    ///
    /// By default this is `true`. Users should set this to `false` if they wish to manually filter
    /// and submit events (e.g. if something is occluding the `Ui` and the user wishes to filter
    /// occluded input).
    pub fn automatically_handle_input(mut self, b: bool) -> Self {
        self.automatically_handle_input = b;
        self
    }

    /// Specify the max number of pending `Input`s that can be enqueued for processing by the `Ui`
    /// before `Input`s start being ignored.
    ///
    /// By default this is `Ui::DEFAULT_PENDING_INPUT_LIMIT`.
    ///
    /// This has no affect if `automatically_handle_input` is set to `false`.
    pub fn pending_input_limit(mut self, limit: usize) -> Self {
        self.pending_input_limit = limit;
        self
    }

    /// Specify the path to the default font.
    ///
    /// By default this is `None` and the `notosans::REGULAR_TTF` font will be used. If the
    /// `notosans` feature is disabled, then the default font will be the first font detected
    /// within the `assets/fonts` directory.
    ///
    /// Fonts can also be specified manually after `Ui` creation using the `fonts_mut` method.
    pub fn default_font_path(mut self, path: PathBuf) -> Self {
        self.default_font_path = Some(path);
        self
    }

    /// Specify the dimensions of the texture used to cache glyphs on the GPU.
    ///
    /// By default this is equal to the framebuffer dimensions of the associated window at the time
    /// of building the `UI`.
    ///
    /// If you notice any glitching of UI text, this may be due to exceeding the bounds of the
    /// texture used to cache glyphs. Try using this to specify a larger glyph cache size to fix
    /// this.
    pub fn with_glyph_cache_dimensions(mut self, width: u32, height: u32) -> Self {
        self.glyph_cache_dimensions = Some((width, height));
        self
    }

    /// Optionally specify a render pass subpass in which this `Ui` should be drawn.
    ///
    /// If unspecified, the `Ui` will implicitly create its own single-pass render pass and
    /// necessary buffers.
    pub fn subpass(mut self, subpass: Subpass) -> Self {
        self.render_pass_subpass = Some(subpass);
        self
    }

    /// Build a `Ui` with the specified parameters.
    ///
    /// Returns `None` if the window at the given `Id` is closed or if the inner `Renderer` returns
    /// an error upon creation.
    pub fn build(self) -> Result<Ui, BuildError> {
        let Builder {
            app,
            window_id,
            dimensions,
            theme,
            pending_input_limit,
            automatically_handle_input,
            default_font_path,
            glyph_cache_dimensions,
            render_pass_subpass,
        } = self;

        // If the user didn't specify a window, use the "main" one.
        let window_id = window_id.unwrap_or(app.window_id());

        // The window on which the `Ui` will exist.
        let window = app.window(window_id).ok_or(BuildError::InvalidWindow)?;

        // The dimensions of the `Ui`.
        let dimensions = dimensions.unwrap_or_else(|| {
            let (win_w, win_h) = window.inner_size_points();
            [win_w as Scalar, win_h as Scalar]
        });

        // Build the conrod `Ui`.
        let theme = theme.unwrap_or_else(Theme::default);
        let ui = conrod_core::UiBuilder::new(dimensions).theme(theme).build();

        // The queue for receiving application events.
        let (input_rx, handle) = if automatically_handle_input {
            let (input_tx, input_rx) = mpsc::sync_channel(pending_input_limit);
            let input_tx = Some(input_tx);
            let input_rx = Some(input_rx);
            let handle = Handle { input_tx };
            (input_rx, handle)
        } else {
            let input_tx = None;
            let input_rx = None;
            let handle = Handle { input_tx };
            (input_rx, handle)
        };

        // Insert the handle into the app's UI arrangement.
        app.ui
            .windows
            .borrow_mut()
            .entry(window_id)
            .or_insert(Vec::new())
            .push(handle);

        // Determine the render_mode and retrieve the subpass for drawing the `Ui`.
        let (subpass, render_mode) = match render_pass_subpass {
            Some(subpass) => (subpass, Mutex::new(RenderMode::Subpass)),
            None => {
                let render_target = RenderTarget::new(&window)?;
                let subpass = Subpass::from(render_target.render_pass.clone(), 0)
                    .expect("unable to retrieve subpass for index `0`");
                let render_mode = Mutex::new(RenderMode::OwnedRenderTarget(render_target));
                (subpass, render_mode)
            }
        };

        // The device and queue with which to create the `Ui` renderer.
        let device = window.swapchain_device().clone();
        let queue = window.swapchain_queue().clone();

        // Initialise the renderer which draws conrod::render::Primitives to the frame.
        let renderer = match glyph_cache_dimensions {
            Some((w, h)) => conrod_vulkano::Renderer::with_glyph_cache_dimensions(
                device,
                subpass,
                queue.family(),
                [w as _, h as _],
            )?,
            None => conrod_vulkano::Renderer::new(
                device,
                subpass,
                queue.family(),
                [dimensions[0] as _, dimensions[1] as _],
                window.hidpi_factor() as _,
            )?,
        };
        let renderer = Mutex::new(renderer);

        // Initialise the image map.
        let image_map = image::Map::new();

        // Initialise the `Ui`.
        let mut ui = Ui {
            window_id,
            ui,
            input_rx,
            image_map,
            renderer,
            render_mode,
        };

        // If no font was specified use one from the notosans crate, otherwise load the given font.
        let default_font = default_font(default_font_path.as_ref().map(|path| path.as_path()))?;
        ui.fonts_mut().insert(default_font);

        Ok(ui)
    }
}

/// Create a minimal, single-pass render pass with which the `Ui` may be rendered.
///
/// This is used internally within the `Ui` build process so in most cases you should not need to
/// know about it. However, it is exposed in case for some reason you require manually creating it.
pub fn create_render_pass(
    device: Arc<vk::device::Device>,
    color_format: vk::Format,
    depth_format: vk::Format,
    msaa_samples: u32,
) -> Result<Arc<dyn vk::RenderPassAbstract + Send + Sync>, vk::RenderPassCreationError> {
    let render_pass = vk::single_pass_renderpass!(
        device,
        attachments: {
            color: {
                load: Load,
                store: Store,
                format: color_format,
                samples: msaa_samples,
            },
            depth: {
                load: Load,
                store: Store,
                format: depth_format,
                samples: msaa_samples,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {depth}
        }
    )?;
    Ok(Arc::new(render_pass))
}

impl RenderPassImages {
    // Create the buffers for a default render target.
    fn new(window: &Window, depth_format: vk::Format) -> Result<Self, vk::ImageCreationError> {
        let device = window.swapchain_device().clone();
        // TODO: Change this to use `window.inner_size_pixels/points` (which is correct?).
        let image_dims = window.swapchain_images()[0].dimensions();
        let msaa_samples = window.msaa_samples();
        let depth = vk::AttachmentImage::transient_multisampled(
            device.clone(),
            image_dims,
            msaa_samples,
            depth_format,
        )?;
        Ok(RenderPassImages { depth })
    }
}

impl RenderTarget {
    // Initialise a new render target.
    fn new(window: &Window) -> Result<Self, RenderTargetCreationError> {
        let device = window.swapchain_device().clone();
        let color_format = crate::frame::COLOR_FORMAT;
        let msaa_samples = window.msaa_samples();
        let depth_format = find_depth_format(device.clone())
            .ok_or(RenderTargetCreationError::NoSupportedDepthFormat)?;
        let render_pass = create_render_pass(device, color_format, depth_format, msaa_samples)?;
        let images = RenderPassImages::new(window, depth_format)?;
        let view_fbo = ViewFbo::default();
        Ok(RenderTarget {
            render_pass,
            images,
            view_fbo,
        })
    }
}

impl Deref for Ui {
    type Target = conrod_core::Ui;
    fn deref(&self) -> &Self::Target {
        &self.ui
    }
}

impl Ui {
    /// The default maximum number of `Input`s that a `Ui` will store in its pending `Input` queue
    /// before `Input`s start being ignored.
    pub const DEFAULT_PENDING_INPUT_LIMIT: usize = 1024;

    /// Generate a new, unique `widget::Id` into a Placeholder node within the widget graph. This
    /// should only be called once for each unique widget needed to avoid unnecessary bloat within
    /// the `Ui`'s internal widget graph.
    ///
    /// When using this method, be sure to store the returned `widget::Id` somewhere so that it can
    /// be re-used on next update.
    ///
    /// **Panics** if adding another node would exceed the maximum capacity for node indices.
    pub fn generate_widget_id(&mut self) -> widget::Id {
        self.widget_id_generator().next()
    }

    /// Produces the type that may be used to generate new unique `widget::Id`s.
    pub fn widget_id_generator(&mut self) -> widget::id::Generator {
        self.ui.widget_id_generator()
    }

    /// Handle a raw UI input event and update the **Ui** state accordingly.
    ///
    /// This method *drives* the **Ui** forward and interprets input into higher-level events (like
    /// clicks and drags) for widgets.
    ///
    /// Note: By default, this will be called automatically by the nannou `App`, so most of the
    /// time you should not need to call this (otherwise received inputs may double up). This
    /// method is particularly useful in the case that automatic input handling has been disabled,
    /// as this can be used to manually submit inputs.
    pub fn handle_input(&mut self, input: Input) {
        self.ui.handle_event(input)
    }

    /// Processes all pending input.
    ///
    /// This is automatically called at the beginning of the `set_widgets` method, so the user
    /// should never need to call this manually, however the method is exposed for flexibility
    /// just in case.
    ///
    /// This has no effect if automatic input handling is disabled.
    pub fn handle_pending_input(&mut self) {
        let Ui {
            ref mut ui,
            ref input_rx,
            ..
        } = *self;
        if let Some(ref rx) = *input_rx {
            for input in rx.try_iter() {
                ui.handle_event(input);
            }
        }
    }

    /// Returns a context upon which UI widgets can be instantiated.
    ///
    /// The **UiCell** simply acts as a wrapper around the **Ui** for the period over which widgets
    /// are instantiated. Once the **UiCell** is dropped, it does some cleanup and sorting that is
    /// required after widget instantiation.
    pub fn set_widgets(&mut self) -> UiCell {
        // Process any pending inputs first.
        self.handle_pending_input();
        self.ui.set_widgets()
    }

    /// Mutable access to the `Ui`'s font map.
    ///
    /// This allows for adding and removing fonts to the UI.
    pub fn fonts_mut(&mut self) -> &mut text::font::Map {
        &mut self.ui.fonts
    }

    /// Mutable access to the `Ui`'s `Theme`.
    ///
    /// This allows for making changes to the active theme.
    pub fn theme_mut(&mut self) -> &mut Theme {
        &mut self.ui.theme
    }

    /// The first of the `Primitives` yielded by `Ui::draw` will always be a `Rectangle` the size
    /// of the window in which the Ui is instantiated.
    ///
    /// This method sets the colour with which this `Rectangle` is drawn (the default being
    /// `color::TRANSPARENT`).
    pub fn clear_with(&mut self, color: Color) {
        self.ui.clear_with(color)
    }

    /// Draws the current state of the `Ui` to the given `Frame`.
    ///
    /// The `Ui` will automatically draw to its associated window within the given `Frame`.
    ///
    /// If you require more control over where the `Ui` is drawn within the `Frame`, the `draw`
    /// method offers more flexibility.
    ///
    /// This has no effect if the window originally associated with the `Ui` no longer exists.
    pub fn draw_to_frame(&self, app: &App, frame: &Frame) -> Result<(), DrawToFrameError> {
        let primitives = self.ui.draw();
        draw_primitives(self, app, frame, primitives)
    }

    /// Draws the current state of the `Ui` to the given `Frame` but only if the `Ui` has changed
    /// since last time either `draw_to_frame` or `draw_to_frame_if_changed` was called.
    ///
    /// The `Ui` will automatically draw to its associated window within the given `Frame`.
    ///
    /// If you require more control over where the `Ui` is drawn within the `Frame`, the `draw`
    /// method offers more flexibility.
    ///
    /// This has no effect if the window originally associated with the `Ui` no longer exists.
    ///
    /// Returns `true` if the call resulted in re-drawing the `Ui` due to changes.
    pub fn draw_to_frame_if_changed(
        &self,
        app: &App,
        frame: &Frame,
    ) -> Result<bool, DrawToFrameError> {
        match self.ui.draw_if_changed() {
            None => Ok(false),
            Some(primitives) => draw_primitives(self, app, frame, primitives).map(|()| true),
        }
    }
}

/// A function shared by the `draw_to_frame` and `draw_to_frame_if_changed` methods for renderering
/// the list of conrod primitives and presenting them to the frame.
pub fn draw_primitives(
    ui: &Ui,
    app: &App,
    frame: &Frame,
    primitives: conrod_core::render::Primitives,
) -> Result<(), DrawToFrameError> {
    let Ui {
        ref renderer,
        ref render_mode,
        ref image_map,
        window_id,
        ..
    } = *ui;

    let window = match app.window(window_id) {
        Some(window) => window,
        None => return Err(DrawToFrameError::InvalidWindow),
    };

    let mut renderer = renderer
        .lock()
        .map_err(|_| DrawToFrameError::RendererPoisoned)?;
    let mut render_mode = render_mode
        .lock()
        .map_err(|_| DrawToFrameError::RenderModePoisoned)?;

    let render_target = match *render_mode {
        RenderMode::Subpass => return Err(DrawToFrameError::InvalidRenderMode),
        RenderMode::OwnedRenderTarget(ref mut render_target) => render_target,
    };

    let RenderTarget {
        ref render_pass,
        ref mut images,
        ref mut view_fbo,
    } = *render_target;

    // Recreate buffers if the swapchain was recreated.
    let image_dims = frame.swapchain_image().dimensions();
    if image_dims != images.depth.dimensions() {
        let depth_format = vk::ImageAccess::format(&images.depth);
        *images = RenderPassImages::new(&window, depth_format)?;
    }

    // Ensure image framebuffer are up to date.
    view_fbo.update(&frame, render_pass.clone(), |builder, image| {
        builder.add(image.clone())?.add(images.depth.clone())
    })?;

    // Fill renderer with the primitives and cache glyphs.
    let (win_w, win_h) = window.inner_size_pixels();
    let dpi_factor = window.hidpi_factor() as f64;
    let viewport = [0.0, 0.0, win_w as f32, win_h as f32];
    if let Some(cmd) = renderer.fill(image_map, viewport, dpi_factor, primitives)? {
        let buffer = cmd
            .glyph_cpu_buffer_pool
            .chunk(cmd.glyph_cache_pixel_buffer.iter().cloned())?;
        frame
            .add_commands()
            .copy_buffer_to_image(buffer, cmd.glyph_cache_texture)?;
    }

    // Generate the draw commands and submit them.
    let queue = window.swapchain_queue().clone();
    let cmds = renderer.draw(queue, image_map, viewport)?;
    if !cmds.is_empty() {
        let color = vk::ClearValue::None;
        let depth = vk::ClearValue::None;
        let clear_values = vec![color, depth];
        frame
            .add_commands()
            .begin_render_pass(view_fbo.expect_inner(), clear_values.clone())
            .unwrap();
        for cmd in cmds {
            let conrod_vulkano::DrawCommand {
                graphics_pipeline,
                dynamic_state,
                vertex_buffer,
                descriptor_set,
            } = cmd;
            frame.add_commands().draw(
                graphics_pipeline,
                &dynamic_state,
                vec![vertex_buffer],
                descriptor_set,
                (),
            )?;
        }
        frame.add_commands().end_render_pass().unwrap();
    }

    Ok(())
}

mod conrod_winit_conv {
    conrod_winit::conversion_fns!();
}

/// Convert the given window event to a UI Input.
///
/// Returns `None` if there's no associated UI Input for the given event.
pub fn winit_window_event_to_input(event: winit::WindowEvent, window: &Window) -> Option<Input> {
    conrod_winit_conv::convert_window_event(event, window)
}

impl From<RenderTargetCreationError> for BuildError {
    fn from(err: RenderTargetCreationError) -> Self {
        BuildError::RenderTargetCreation(err)
    }
}

impl From<RendererCreationError> for BuildError {
    fn from(err: RendererCreationError) -> Self {
        BuildError::RendererCreation(err)
    }
}

impl From<text::font::Error> for BuildError {
    fn from(err: text::font::Error) -> Self {
        BuildError::FailedToLoadFont(err)
    }
}

impl From<vk::ImageCreationError> for RenderTargetCreationError {
    fn from(err: vk::ImageCreationError) -> Self {
        RenderTargetCreationError::ImageCreation(err)
    }
}

impl From<vk::RenderPassCreationError> for RenderTargetCreationError {
    fn from(err: vk::RenderPassCreationError) -> Self {
        RenderTargetCreationError::RenderPassCreation(err)
    }
}

impl From<vk::ImageCreationError> for DrawToFrameError {
    fn from(err: vk::ImageCreationError) -> Self {
        DrawToFrameError::ImageCreation(err)
    }
}

impl From<vk::FramebufferCreationError> for DrawToFrameError {
    fn from(err: vk::FramebufferCreationError) -> Self {
        DrawToFrameError::FramebufferCreation(err)
    }
}

impl From<CacheWriteErr> for DrawToFrameError {
    fn from(err: CacheWriteErr) -> Self {
        DrawToFrameError::RendererFill(err)
    }
}

impl From<vk::memory::DeviceMemoryAllocError> for DrawToFrameError {
    fn from(err: vk::memory::DeviceMemoryAllocError) -> Self {
        DrawToFrameError::GlyphPixelDataUpload(err)
    }
}

impl From<vk::command_buffer::CopyBufferImageError> for DrawToFrameError {
    fn from(err: vk::command_buffer::CopyBufferImageError) -> Self {
        DrawToFrameError::CopyBufferImageCommand(err)
    }
}

impl From<conrod_vulkano::DrawError> for DrawToFrameError {
    fn from(err: conrod_vulkano::DrawError) -> Self {
        DrawToFrameError::RendererDraw(err)
    }
}

impl From<vk::command_buffer::DrawError> for DrawToFrameError {
    fn from(err: vk::command_buffer::DrawError) -> Self {
        DrawToFrameError::DrawCommand(err)
    }
}

impl StdError for BuildError {
    fn description(&self) -> &str {
        match *self {
            BuildError::InvalidWindow => "no open window associated with the given `window_id`",
            BuildError::RendererCreation(ref err) => err.description(),
            BuildError::RenderTargetCreation(ref err) => err.description(),
            BuildError::FailedToLoadFont(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            BuildError::InvalidWindow => None,
            BuildError::RendererCreation(ref err) => Some(err),
            BuildError::RenderTargetCreation(ref err) => Some(err),
            BuildError::FailedToLoadFont(ref err) => Some(err),
        }
    }
}

impl StdError for RenderTargetCreationError {
    fn description(&self) -> &str {
        match *self {
            RenderTargetCreationError::RenderPassCreation(ref err) => err.description(),
            RenderTargetCreationError::ImageCreation(ref err) => err.description(),
            RenderTargetCreationError::NoSupportedDepthFormat => {
                "no supported vulkan depth format for UI"
            }
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            RenderTargetCreationError::RenderPassCreation(ref err) => Some(err),
            RenderTargetCreationError::ImageCreation(ref err) => Some(err),
            RenderTargetCreationError::NoSupportedDepthFormat => None,
        }
    }
}

impl StdError for DrawToFrameError {
    fn description(&self) -> &str {
        match *self {
            DrawToFrameError::InvalidWindow => {
                "no open window associated with the given `window_id`"
            }
            DrawToFrameError::RendererPoisoned => "`Mutex` containing `Renderer` was poisoned",
            DrawToFrameError::RenderModePoisoned => "`Mutex` containing `RenderMode` was poisoned",
            DrawToFrameError::InvalidRenderMode => {
                "`draw_to_frame` was called while `Ui` was in `Subpass` render mode"
            }
            DrawToFrameError::ImageCreation(ref err) => err.description(),
            DrawToFrameError::FramebufferCreation(ref err) => err.description(),
            DrawToFrameError::RendererFill(ref err) => err.description(),
            DrawToFrameError::GlyphPixelDataUpload(ref err) => err.description(),
            DrawToFrameError::CopyBufferImageCommand(ref err) => err.description(),
            DrawToFrameError::RendererDraw(ref err) => err.description(),
            DrawToFrameError::DrawCommand(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            DrawToFrameError::InvalidWindow => None,
            DrawToFrameError::RendererPoisoned => None,
            DrawToFrameError::RenderModePoisoned => None,
            DrawToFrameError::InvalidRenderMode => None,
            DrawToFrameError::ImageCreation(ref err) => Some(err),
            DrawToFrameError::FramebufferCreation(ref err) => Some(err),
            DrawToFrameError::RendererFill(ref err) => Some(err),
            DrawToFrameError::GlyphPixelDataUpload(ref err) => Some(err),
            DrawToFrameError::CopyBufferImageCommand(ref err) => Some(err),
            DrawToFrameError::RendererDraw(ref err) => Some(err),
            DrawToFrameError::DrawCommand(ref err) => Some(err),
        }
    }
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl fmt::Display for RenderTargetCreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl fmt::Display for DrawToFrameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// Find a compatible vulkan depth format for the UI.
pub fn find_depth_format(device: Arc<vk::Device>) -> Option<vk::Format> {
    let candidates = [
        vk::Format::D16Unorm,
        vk::Format::D32Sfloat,
        vk::Format::D16Unorm_S8Uint,
        vk::Format::D24Unorm_S8Uint,
        vk::Format::D32Sfloat_S8Uint,
    ];
    vk::find_supported_depth_image_format(device, &candidates)
}

// Retrieve the default font.
//
// Accepts an optional default font path if one was provided by the user.
fn default_font(default_font_path: Option<&Path>) -> Result<Font, text::font::Error> {
    // Convert the nannou text error to a conrod one.
    fn conv_err(err: font::Error) -> text::font::Error {
        match err {
            font::Error::Io(err) => text::font::Error::IO(err),
            font::Error::NoFont => text::font::Error::NoFont,
        }
    }

    let font = match default_font_path {
        None => {
            #[cfg(feature = "notosans")]
            {
                font::default_notosans()
            }
            #[cfg(not(feature = "notosans"))]
            {
                match crate::app::find_assets_path() {
                    Err(_err) => return Err(text::font::Error::NoFont)?,
                    Ok(assets) => font::default(&assets).map_err(conv_err)?,
                }
            }
        }
        Some(path) => {
            let font = font::from_file(path).map_err(conv_err)?;
            font
        }
    };
    Ok(font)
}
