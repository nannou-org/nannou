//! The User Interface API.
//!
//! Instantiate a [**Ui**](struct.Ui.html) via `app.new_ui()`.

pub use self::conrod_core::event::Input;
pub use self::conrod_core::{
    color, cursor, event, graph, image, input, position, scroll, text, theme, utils, widget,
    widget_ids,
};
pub use self::conrod_core::{
    Borderable, Bordering, Color, Colorable, Dimensions, FontSize, Labelable, Point, Positionable,
    Range, Rect, Scalar, Sizeable, Theme, UiCell, Widget,
};
pub use crate::conrod_core;
pub use crate::conrod_wgpu;
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
    pub use super::{color, image, position, text, widget, widget_ids};
}

use self::conrod_core::text::rt::gpu_cache::CacheWriteErr;
use crate::frame::Frame;
use crate::text::{font, Font};
use crate::window::{self, Window};
use crate::{wgpu, App};
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
    renderer: Mutex<conrod_wgpu::Renderer>,
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
}

/// Failed to build the `Ui`.
#[derive(Debug)]
pub enum BuildError {
    /// Either the given window `Id` is not associated with any open windows or the window was
    /// closed during the build process.
    InvalidWindow,
    FailedToLoadFont(text::font::Error),
}

/// An error that might occur while drawing to a `Frame`.
#[derive(Debug)]
pub enum DrawToFrameError {
    InvalidWindow,
    RendererPoisoned,
    RenderModePoisoned,
    InvalidRenderMode,
    RendererFill(CacheWriteErr),
}

/// A map from `image::Id`s to their associated `Texture2d`.
pub type ImageMap = conrod_core::image::Map<conrod_wgpu::Image>;

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
        } = self;

        // If the user didn't specify a window, use the "main" one.
        let window_id = window_id.unwrap_or(app.window_id());

        // The window on which the `Ui` will exist.
        let window = app.window(window_id).ok_or(BuildError::InvalidWindow)?;
        let msaa_samples = window.msaa_samples();

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

        // The device with which to create the `Ui` renderer.
        let device = window.swap_chain_device().clone();

        // Initialise the renderer which draws conrod::render::Primitives to the frame.
        let texture_format = Frame::TEXTURE_FORMAT;
        let renderer = match glyph_cache_dimensions {
            Some((w, h)) => {
                let dims = [w as _, h as _];
                conrod_wgpu::Renderer::with_glyph_cache_dimensions(
                    device,
                    msaa_samples,
                    texture_format,
                    dims,
                )
            }
            None => conrod_wgpu::Renderer::new(device, msaa_samples, texture_format),
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
        };

        // If no font was specified use one from the notosans crate, otherwise load the given font.
        let default_font = default_font(default_font_path.as_ref().map(|path| path.as_path()))?;
        ui.fonts_mut().insert(default_font);

        Ok(ui)
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
        let color_attachment_desc = frame.color_attachment_descriptor();
        let mut command_encoder = frame.command_encoder();
        let window = app
            .window(self.window_id)
            .ok_or(DrawToFrameError::InvalidWindow)?;
        encode_render_pass(
            self,
            &*window,
            primitives,
            color_attachment_desc,
            &mut *command_encoder,
        )
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
            Some(primitives) => {
                let window = app
                    .window(self.window_id)
                    .ok_or(DrawToFrameError::InvalidWindow)?;
                let color_attachment_desc = frame.color_attachment_descriptor();
                let mut command_encoder = frame.command_encoder();
                encode_render_pass(
                    self,
                    &*window,
                    primitives,
                    color_attachment_desc,
                    &mut *command_encoder,
                )?;
                Ok(true)
            }
        }
    }
}

impl wgpu::Texture {
    /// Convert the texture into an image compatible with the UI's image map.
    ///
    /// **Panic**s if the texture's `Arc<TextureHandle>` has been cloned and more than one unique
    /// reference to the inner data still exists.
    pub fn into_ui_image(self) -> conrod_wgpu::Image {
        let texture_format = self.format();
        let [width, height] = self.size();
        let texture = Arc::try_unwrap(self.into_inner())
            .expect("converting to UI image requires unique access to texture");
        conrod_wgpu::Image {
            texture,
            texture_format,
            width,
            height,
        }
    }
}

/// Encode commands for drawing the given primitives.
pub fn encode_render_pass(
    ui: &Ui,
    window: &Window,
    primitives: conrod_core::render::Primitives,
    color_attachment_desc: wgpu::RenderPassColorAttachmentDescriptor,
    encoder: &mut wgpu::CommandEncoder,
) -> Result<(), DrawToFrameError> {
    // Feed the renderer primitives and update glyph cache texture if necessary.
    let mut renderer = ui
        .renderer
        .lock()
        .ok()
        .ok_or(DrawToFrameError::RendererPoisoned)?;
    let device = window.swap_chain_device();
    let scale_factor = window.scale_factor();
    let (win_w, win_h) = window.inner_size_pixels();
    let viewport = [0.0, 0.0, win_w as f32, win_h as f32];
    if let Some(cmd) = renderer
        .fill(&ui.image_map, viewport, scale_factor as f64, primitives)
        .unwrap()
    {
        cmd.load_buffer_and_encode(&device, encoder);
    }

    // Begin the render pass and add the draw commands.
    let render_pass_desc = wgpu::RenderPassDescriptor {
        color_attachments: &[color_attachment_desc],
        depth_stencil_attachment: None,
    };
    let render = renderer.render(&device, &ui.image_map);
    {
        let mut render_pass = encoder.begin_render_pass(&render_pass_desc);
        render_pass.set_vertex_buffer(0, render.vertex_buffer.slice(..));
        let instance_range = 0..1;
        for cmd in render.commands {
            match cmd {
                conrod_wgpu::RenderPassCommand::SetPipeline { pipeline } => {
                    render_pass.set_pipeline(pipeline);
                }
                conrod_wgpu::RenderPassCommand::SetBindGroup { bind_group } => {
                    render_pass.set_bind_group(0, bind_group, &[]);
                }
                conrod_wgpu::RenderPassCommand::SetScissor {
                    top_left,
                    dimensions,
                } => {
                    let [x, y] = top_left;
                    let [w, h] = dimensions;
                    render_pass.set_scissor_rect(x, y, w, h);
                }
                conrod_wgpu::RenderPassCommand::Draw { vertex_range } => {
                    render_pass.draw(vertex_range, instance_range.clone());
                }
            }
        }
    }

    Ok(())
}

mod conrod_winit_conv {
    conrod_winit::v023_conversion_fns!();
}

/// Convert the given window event to a UI Input.
///
/// Returns `None` if there's no associated UI Input for the given event.
pub fn winit_window_event_to_input(
    event: &winit::event::WindowEvent,
    window: &Window,
) -> Option<Input> {
    conrod_winit_conv::convert_window_event(event, &window.window)
}

impl Deref for Ui {
    type Target = conrod_core::Ui;
    fn deref(&self) -> &Self::Target {
        &self.ui
    }
}

impl From<text::font::Error> for BuildError {
    fn from(err: text::font::Error) -> Self {
        BuildError::FailedToLoadFont(err)
    }
}

impl From<CacheWriteErr> for DrawToFrameError {
    fn from(err: CacheWriteErr) -> Self {
        DrawToFrameError::RendererFill(err)
    }
}

impl StdError for BuildError {
    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            BuildError::InvalidWindow => None,
            BuildError::FailedToLoadFont(ref err) => Some(err),
        }
    }
}

impl StdError for DrawToFrameError {
    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            DrawToFrameError::InvalidWindow => None,
            DrawToFrameError::RendererPoisoned => None,
            DrawToFrameError::RenderModePoisoned => None,
            DrawToFrameError::InvalidRenderMode => None,
            DrawToFrameError::RendererFill(ref err) => Some(err),
        }
    }
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BuildError::InvalidWindow => {
                write!(f, "no open window associated with the given `window_id`")
            }
            BuildError::FailedToLoadFont(ref err) => fmt::Display::fmt(err, f),
        }
    }
}

impl fmt::Display for DrawToFrameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            DrawToFrameError::InvalidWindow => {
                "no open window associated with the given `window_id`"
            }
            DrawToFrameError::RendererPoisoned => "`Mutex` containing `Renderer` was poisoned",
            DrawToFrameError::RenderModePoisoned => "`Mutex` containing `RenderMode` was poisoned",
            DrawToFrameError::InvalidRenderMode => {
                "`draw_to_frame` was called while `Ui` was in `Subpass` render mode"
            }
            DrawToFrameError::RendererFill(ref err) => return fmt::Display::fmt(err, f),
        };
        write!(f, "{}", s)
    }
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
