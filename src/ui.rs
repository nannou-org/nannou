//! The User Interface API. Instantiate a [**Ui**](struct.Ui.html) via `app.new_ui()`.

pub extern crate conrod;

pub use self::conrod::event::Input;
pub use self::conrod::{
    backend, color, cursor, event, graph, image, input, position, scroll, text, theme, utils,
    widget,
};
pub use self::conrod::{
    Borderable, Bordering, Color, Colorable, Dimensions, FontSize, Labelable, Point, Positionable,
    Range, Rect, Scalar, Sizeable, Theme, UiCell, Widget,
};

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

use frame::Frame;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::mpsc;
use window::{self, Window};
use winit;
use App;

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
    ui: conrod::Ui,
    input_rx: Option<mpsc::Receiver<Input>>,
    // renderer: Mutex<conrod::backend::glium::Renderer>,
    pub image_map: conrod::image::Map<glium::texture::Texture2d>,
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

/// A map from `image::Id`s to their associatd `Texture2d`.
pub type Texture2dMap = conrod::image::Map<glium::texture::Texture2d>;

impl conrod::backend::winit::WinitWindow for Window {
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
    /// By default this is "fonts/NotoSans/NotoSans-Regular.ttf".
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
    pub fn build(self) -> Option<Ui> {
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

        let window_id = window_id.unwrap_or(app.window_id());

        let dimensions = match dimensions {
            None => match app.window(window_id) {
                None => return None,
                Some(window) => {
                    let (win_w, win_h) = window.inner_size_points();
                    [win_w as Scalar, win_h as Scalar]
                }
            },
            Some(dimensions) => dimensions,
        };
        let theme = theme.unwrap_or_else(Theme::default);
        let ui = conrod::UiBuilder::new(dimensions).theme(theme).build();

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

        // // Initialise the renderer which draws conrod::render::Primitives to the frame..
        // let renderer = match app.windows.borrow().get(&window_id) {
        //     None => return None,
        //     Some(window) => {
        //         let renderer = match glyph_cache_dimensions {
        //             Some((w, h)) => conrod::backend::glium::Renderer::with_glyph_cache_dimensions(
        //                 &window.display,
        //                 w,
        //                 h,
        //             ),
        //             None => conrod::backend::glium::Renderer::new(&window.display),
        //         };
        //         match renderer {
        //             Ok(renderer) => Mutex::new(renderer),
        //             Err(_) => return None,
        //         }
        //     }
        // };

        // Initialise the image map.
        let image_map = image::Map::new();

        // Initialise the `Ui`.
        let mut ui = Ui {
            window_id,
            ui,
            input_rx,
            // renderer,
            image_map,
        };

        // If the default font is in the assets/fonts directory, load it into the UI font map.
        let default_font_path = default_font_path.or_else(|| {
            app.assets_path()
                .ok()
                .map(|p| p.join(Ui::DEFAULT_FONT_PATH))
        });

        // If there is some font path to use for the default font, attempt to load it.
        if let Some(font_path) = default_font_path {
            ui.fonts_mut().insert_from_file(font_path).ok();
        }

        Some(ui)
    }
}

impl Deref for Ui {
    type Target = conrod::Ui;
    fn deref(&self) -> &Self::Target {
        &self.ui
    }
}

impl Ui {
    /// The default maximum number of `Input`s that a `Ui` will store in its pending `Input` queue
    /// before `Input`s start being ignored.
    pub const DEFAULT_PENDING_INPUT_LIMIT: usize = 1024;
    /// The path to the default font for the `Ui`.
    pub const DEFAULT_FONT_PATH: &'static str = "fonts/NotoSans/NotoSans-Regular.ttf";

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
    pub fn draw_to_frame(
        &self,
        app: &App,
        frame: &Frame,
    ) -> Result<(), ()> {
        unimplemented!()
        // let Ui {
        //     ref ui,
        //     ref renderer,
        //     ref image_map,
        //     window_id,
        //     ..
        // } = *self;
        // if let Some(window) = app.window(window_id) {
        //     if let Some(mut window_frame) = frame.window(window_id) {
        //         if let Ok(mut renderer) = renderer.lock() {
        //             let primitives = ui.draw();
        //             renderer.fill(&window.display, primitives, &image_map);
        //             renderer.draw(&window.display, &mut window_frame.frame.frame, image_map)?;
        //         }
        //     }
        // }
        // Ok(())
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
    ) -> Result<bool, ()> {
        unimplemented!()
        // let Ui {
        //     ref ui,
        //     ref renderer,
        //     ref image_map,
        //     window_id,
        //     ..
        // } = *self;
        // if let Some(window) = app.window(window_id) {
        //     if let Some(mut window_frame) = frame.window(window_id) {
        //         if let Ok(mut renderer) = renderer.lock() {
        //             if let Some(primitives) = ui.draw_if_changed() {
        //                 renderer.fill(&window.display, primitives, &image_map);
        //                 renderer.draw(&window.display, &mut window_frame.frame.frame, image_map)?;
        //                 return Ok(true);
        //             }
        //         }
        //     }
        // }
        // Ok(false)
    }
}

/// Convert the given window event to a UI Input.
///
/// Returns `None` if there's no associated UI Input for the given event.
pub fn winit_window_event_to_input(event: winit::WindowEvent, window: &Window) -> Option<Input> {
    // TODO: fix this once conrod winit and nannou winit versions match");
    None
    //conrod::backend::winit::convert_window_event(event, window)
}
