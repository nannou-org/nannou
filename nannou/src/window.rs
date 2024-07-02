//! The nannou Window API.
//!
//! Create a new window via `app.new_window()`. This produces a [**Builder**](./struct.Builder.html)
//! which can be used to build a [**Window**](./struct.Window.html).

use std::fmt;
use std::path::{Path, PathBuf};

use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::view::screenshot::ScreenshotManager;
use bevy::render::view::RenderLayers;
use bevy::window::{Cursor, CursorGrabMode, PrimaryWindow, WindowLevel, WindowMode, WindowRef};

use bevy_nannou::prelude::render::NannouCamera;
use bevy_nannou::prelude::MonitorSelection;
use nannou_core::geom;

use crate::geom::Point2;
use crate::glam::Vec2;
use crate::prelude::WindowResizeConstraints;
use crate::App;

/// A nannou window.
///
/// The **Window** acts as a wrapper around the `winit::window::Window` and the `wgpu::Surface`
/// types.
pub struct Window<'a, 'w> {
    entity: Entity,
    app: &'a App<'w>,
}

/// A context for building a window.
pub struct Builder<'a, 'w, M = ()> {
    app: &'a App<'w>,
    window: bevy::window::Window,
    camera: Option<Entity>,
    primary: bool,
    title_was_set: bool,
    user_functions: UserFunctions<M>,
    clear_color: Option<Color>,
    hdr: bool,
}

/// For storing all user functions within the window.
#[derive(Debug)]
pub(crate) struct UserFunctions<M> {
    pub(crate) view: Option<View<M>>,
    pub(crate) key_pressed: Option<KeyPressedFn<M>>,
    pub(crate) key_released: Option<KeyReleasedFn<M>>,
    pub(crate) received_character: Option<ReceivedCharacterFn<M>>,
    pub(crate) mouse_moved: Option<MouseMovedFn<M>>,
    pub(crate) mouse_pressed: Option<MousePressedFn<M>>,
    pub(crate) mouse_released: Option<MouseReleasedFn<M>>,
    pub(crate) mouse_entered: Option<MouseEnteredFn<M>>,
    pub(crate) mouse_exited: Option<MouseExitedFn<M>>,
    pub(crate) mouse_wheel: Option<MouseWheelFn<M>>,
    pub(crate) moved: Option<MovedFn<M>>,
    pub(crate) resized: Option<ResizedFn<M>>,
    pub(crate) touch: Option<TouchFn<M>>,
    // pub(crate) touchpad_pressure: Option<TouchpadPressureFn<M>>,
    pub(crate) hovered_file: Option<HoveredFileFn<M>>,
    pub(crate) hovered_file_cancelled: Option<HoveredFileCancelledFn<M>>,
    pub(crate) dropped_file: Option<DroppedFileFn<M>>,
    pub(crate) focused: Option<FocusedFn<M>>,
    pub(crate) unfocused: Option<UnfocusedFn<M>>,
    pub(crate) closed: Option<ClosedFn<M>>,
}

impl<'a, 'w> Window<'a, 'w> {
    pub fn new(app: &'a App<'w>, entity: Entity) -> Self {
        Window { app, entity }
    }

    fn window(&self) -> &bevy::window::Window {
        self.app
            .world()
            .get::<bevy::window::Window>(self.entity)
            .unwrap()
    }

    fn window_mut(&self) -> Mut<'a, bevy_nannou::prelude::Window> {
        self.app
            .world_mut()
            .get_mut::<bevy::window::Window>(self.entity)
            .unwrap()
    }
}

impl<M> Default for UserFunctions<M> {
    fn default() -> Self {
        UserFunctions {
            view: None,
            key_pressed: None,
            key_released: None,
            received_character: None,
            mouse_moved: None,
            mouse_pressed: None,
            mouse_released: None,
            mouse_entered: None,
            mouse_exited: None,
            mouse_wheel: None,
            moved: None,
            resized: None,
            touch: None,
            // touchpad_pressure: None,
            hovered_file: None,
            hovered_file_cancelled: None,
            dropped_file: None,
            focused: None,
            unfocused: None,
            closed: None,
        }
    }
}

impl<M> Clone for UserFunctions<M> {
    fn clone(&self) -> Self {
        UserFunctions {
            view: self.view.clone(),
            key_pressed: self.key_pressed,
            key_released: self.key_released,
            received_character: self.received_character,
            mouse_moved: self.mouse_moved,
            mouse_pressed: self.mouse_pressed,
            mouse_released: self.mouse_released,
            mouse_entered: self.mouse_entered,
            mouse_exited: self.mouse_exited,
            mouse_wheel: self.mouse_wheel,
            moved: self.moved,
            resized: self.resized,
            touch: self.touch,
            // touchpad_pressure: self.touchpad_pressure,
            hovered_file: self.hovered_file,
            hovered_file_cancelled: self.hovered_file_cancelled,
            dropped_file: self.dropped_file,
            focused: self.focused,
            unfocused: self.unfocused,
            closed: self.closed,
        }
    }
}

#[derive(Component, Deref, DerefMut)]
pub(crate) struct WindowUserFunctions<M>(pub(crate) UserFunctions<M>);

/// The user function type for drawing their model to the surface of a single window.
pub type ViewFn<Model> = fn(&App, &Model);

/// The same as `ViewFn`, but provides no user model to draw from.
///
/// Useful for simple, stateless sketching.
pub type SketchFn = fn(&App);

/// The user's view function, whether with a model or without one.
pub(crate) enum View<M> {
    WithModel(ViewFn<M>),
    Sketch(SketchFn),
}

impl<M> Clone for View<M> {
    fn clone(&self) -> Self {
        match self {
            View::WithModel(f) => View::WithModel(*f),
            View::Sketch(f) => View::Sketch(*f),
        }
    }
}

/// A function for processing key press events.
pub type KeyPressedFn<Model> = fn(&App, &mut Model, KeyCode);

/// A function for processing key release events.
pub type KeyReleasedFn<Model> = fn(&App, &mut Model, KeyCode);

/// A function for processing received characters.
pub type ReceivedCharacterFn<Model> = fn(&App, &mut Model, char);

/// A function for processing mouse moved events.
pub type MouseMovedFn<Model> = fn(&App, &mut Model, Point2);

/// A function for processing mouse pressed events.
pub type MousePressedFn<Model> = fn(&App, &mut Model, MouseButton);

/// A function for processing mouse released events.
pub type MouseReleasedFn<Model> = fn(&App, &mut Model, MouseButton);

/// A function for processing mouse entered events.
pub type MouseEnteredFn<Model> = fn(&App, &mut Model);

/// A function for processing mouse exited events.
pub type MouseExitedFn<Model> = fn(&App, &mut Model);

/// A function for processing mouse wheel events.
pub type MouseWheelFn<Model> = fn(&App, &mut Model, MouseWheel);

/// A function for processing window moved events.
pub type MovedFn<Model> = fn(&App, &mut Model, IVec2);

/// A function for processing window resized events.
pub type ResizedFn<Model> = fn(&App, &mut Model, Vec2);

/// A function for processing touch events.
pub type TouchFn<Model> = fn(&App, &mut Model, TouchInput);

// https://github.com/bevyengine/bevy/issues/6174
// A function for processing touchpad pressure events.
// pub type TouchpadPressureFn<Model> = fn(&App, &mut Model, TouchpadPressure);

/// A function for processing hovered file events.
pub type HoveredFileFn<Model> = fn(&App, &mut Model, PathBuf);

/// A function for processing hovered file cancelled events.
pub type HoveredFileCancelledFn<Model> = fn(&App, &mut Model);

/// A function for processing dropped file events.
pub type DroppedFileFn<Model> = fn(&App, &mut Model, PathBuf);

/// A function for processing window focused events.
pub type FocusedFn<Model> = fn(&App, &mut Model);

/// A function for processing window unfocused events.
pub type UnfocusedFn<Model> = fn(&App, &mut Model);

/// A function for processing window closed events.
pub type ClosedFn<Model> = fn(&App, &mut Model);

impl<'a, 'w, M> Builder<'a, 'w, M>
where
    M: 'static,
{
    /// Begin building a new window.
    pub fn new(app: &'a App<'w>) -> Self {
        Builder {
            app,
            window: bevy::window::Window::default(),
            camera: None,
            primary: false,
            title_was_set: false,
            user_functions: UserFunctions::<M>::default(),
            clear_color: None,
            hdr: false,
        }
    }

    /// Build the window with some custom window parameters.
    pub fn window(mut self, window: bevy::window::Window) -> Self {
        self.window = window;
        self
    }

    /// Provide a simple function for drawing to the window.
    ///
    /// This is similar to `view` but does not provide access to user data via a Model type. This
    /// is useful for sketches where you don't require tracking any state.
    pub fn sketch(mut self, sketch_fn: SketchFn) -> Self {
        self.user_functions.view = Some(View::Sketch(sketch_fn));
        self
    }

    /// The **view** function that the app will call to allow you to present your Model to the
    /// surface of the window on your display.
    pub fn view(mut self, view_fn: ViewFn<M>) -> Self {
        self.user_functions.view = Some(View::WithModel(view_fn));
        self
    }

    /// Set the initial color of the window background
    /// when its contents are invalidated, e.g. upon window resize.
    pub fn clear_color<C>(mut self, color: C) -> Self
    where
        C: Into<Color>,
    {
        let color = color.into();
        self.clear_color = Some(color);
        self
    }

    /// A function for processing key press events associated with this window.
    pub fn key_pressed(mut self, f: KeyPressedFn<M>) -> Self {
        self.user_functions.key_pressed = Some(f);
        self
    }

    /// A function for processing key release events associated with this window.
    pub fn key_released(mut self, f: KeyReleasedFn<M>) -> Self {
        self.user_functions.key_released = Some(f);
        self
    }

    pub fn received_character(mut self, f: ReceivedCharacterFn<M>) -> Self {
        self.user_functions.received_character = Some(f);
        self
    }

    /// A function for processing mouse moved events associated with this window.
    pub fn mouse_moved(mut self, f: MouseMovedFn<M>) -> Self {
        self.user_functions.mouse_moved = Some(f);
        self
    }

    /// A function for processing mouse pressed events associated with this window.
    pub fn mouse_pressed(mut self, f: MousePressedFn<M>) -> Self {
        self.user_functions.mouse_pressed = Some(f);
        self
    }

    /// A function for processing mouse released events associated with this window.
    pub fn mouse_released(mut self, f: MouseReleasedFn<M>) -> Self {
        self.user_functions.mouse_released = Some(f);
        self
    }

    /// A function for processing mouse wheel events associated with this window.
    pub fn mouse_wheel(mut self, f: MouseWheelFn<M>) -> Self {
        self.user_functions.mouse_wheel = Some(f);
        self
    }

    /// A function for processing mouse entered events associated with this window.
    pub fn mouse_entered(mut self, f: MouseEnteredFn<M>) -> Self {
        self.user_functions.mouse_entered = Some(f);
        self
    }

    /// A function for processing mouse exited events associated with this window.
    pub fn mouse_exited(mut self, f: MouseExitedFn<M>) -> Self {
        self.user_functions.mouse_exited = Some(f);
        self
    }

    /// A function for processing touch events associated with this window.
    pub fn touch(mut self, f: TouchFn<M>) -> Self {
        self.user_functions.touch = Some(f);
        self
    }
    //
    // /// A function for processing touchpad pressure events associated with this window.
    // pub fn touchpad_pressure<M>(mut self, f: TouchpadPressureFn<M>) -> Self
    // where
    //     M: 'static,
    // {
    //     self.user_functions.touchpad_pressure = Some(f);
    //     self
    // }

    /// A function for processing window moved events associated with this window.
    pub fn moved(mut self, f: MovedFn<M>) -> Self {
        self.user_functions.moved = Some(f);
        self
    }

    /// A function for processing window resized events associated with this window.
    pub fn resized(mut self, f: ResizedFn<M>) -> Self {
        self.user_functions.resized = Some(f);
        self
    }

    /// A function for processing hovered file events associated with this window.
    pub fn hovered_file(mut self, f: HoveredFileFn<M>) -> Self {
        self.user_functions.hovered_file = Some(f);
        self
    }

    /// A function for processing hovered file cancelled events associated with this window.
    pub fn hovered_file_cancelled(mut self, f: HoveredFileCancelledFn<M>) -> Self {
        self.user_functions.hovered_file_cancelled = Some(f);
        self
    }

    /// A function for processing dropped file events associated with this window.
    pub fn dropped_file(mut self, f: DroppedFileFn<M>) -> Self {
        self.user_functions.dropped_file = Some(f);
        self
    }

    /// A function for processing the focused event associated with this window.
    pub fn focused(mut self, f: FocusedFn<M>) -> Self {
        self.user_functions.focused = Some(f);
        self
    }

    /// A function for processing the unfocused event associated with this window.
    pub fn unfocused(mut self, f: UnfocusedFn<M>) -> Self {
        self.user_functions.unfocused = Some(f);
        self
    }

    /// A function for processing the window closed event associated with this window.
    pub fn closed(mut self, f: ClosedFn<M>) -> Self {
        self.user_functions.closed = Some(f);
        self
    }

    pub fn hdr(mut self, hdr: bool) -> Self {
        self.hdr = hdr;
        self
    }

    #[cfg(not(target_os = "unknown"))]
    /// Builds the window, inserts it into the `App`'s display map and returns the unique ID.
    pub fn build(self) -> Entity {
        let entity = self
            .app
            .world_mut()
            .spawn((self.window, WindowUserFunctions(self.user_functions)))
            .id();

        if self.primary {
            let mut q = self.app.world_mut().query::<&PrimaryWindow>();
            if q.get_single(self.app.world_mut()).is_ok() {
                panic!("Only one primary window can be created");
            }

            self.app
                .world_mut()
                .entity_mut(entity)
                .insert(PrimaryWindow);
        }

        if let Some(camera) = self.camera {
            // Update the camera's render target to be the window.
            let mut q = self
                .app
                .world_mut()
                .query::<(&mut Camera, Option<&mut RenderLayers>)>();
            if let Ok((mut camera, layers)) = q.get_mut(self.app.world_mut(), camera) {
                camera.target = RenderTarget::Window(WindowRef::Entity(entity));
                if let None = layers {
                    self.app
                        .world_mut()
                        .entity_mut(self.camera.unwrap())
                        .insert(RenderLayers::layer(self.app.window_count() * 2usize));
                }
            }
        } else {
            info!("No camera provided for window, creating a default camera");
            self.app.world_mut().spawn((
                Camera3dBundle {
                    camera: Camera {
                        hdr: self.hdr,
                        target: RenderTarget::Window(WindowRef::Entity(entity)),
                        clear_color: self
                            .clear_color
                            .map(ClearColorConfig::Custom)
                            .unwrap_or(ClearColorConfig::None),
                        ..default()
                    },
                    transform: Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
                    projection: OrthographicProjection::default().into(),
                    ..default()
                },
                RenderLayers::layer(self.app.window_count() * 2usize),
                NannouCamera,
            ));
        }

        entity
    }

    fn map_window<F>(self, map: F) -> Self
    where
        F: FnOnce(bevy::window::Window) -> bevy::window::Window,
    {
        Builder {
            window: map(self.window),
            ..self
        }
    }

    /// The camera to use for rendering to this window. Setting the camera will override the
    /// default camera that would be created for this window. The camera's render target
    /// will be set to the window when the window is built.
    pub fn camera(mut self, camera: Entity) -> Self {
        self.camera = Some(camera);
        self
    }

    /// Requests the window to be a specific size in points.
    ///
    /// This describes to the "inner" part of the window, not including desktop decorations like the
    /// title bar.
    pub fn size(self, width: u32, height: u32) -> Self {
        self.map_window(|mut w| {
            w.resolution.set(width as f32, height as f32);
            w
        })
    }

    /// Set the minimum size in points for the window.
    pub fn min_size(self, width: f32, height: f32) -> Self {
        self.map_window(|mut w| {
            w.resize_constraints = WindowResizeConstraints {
                min_width: width,
                min_height: height,
                ..w.resize_constraints
            };
            w
        })
    }

    /// Set the maximum size in points for the window.
    pub fn max_size(self, width: f32, height: f32) -> Self {
        self.map_window(|mut w| {
            w.resize_constraints = WindowResizeConstraints {
                max_width: width,
                max_height: height,
                ..w.resize_constraints
            };
            w
        })
    }

    /// Requests the window to be a specific size in points.
    ///
    /// This describes to the "inner" part of the window, not including desktop decorations like the
    /// title bar.
    pub fn size_pixels(self, width: u32, height: u32) -> Self {
        self.map_window(|mut w| {
            w.resolution.set_physical_resolution(width, height);
            w
        })
    }

    /// Whether or not the window should be resizable after creation.
    pub fn resizable(self, resizable: bool) -> Self {
        self.map_window(|mut w| {
            w.resizable = resizable;
            w
        })
    }

    /// Requests a specific title for the window.
    pub fn title<T>(mut self, title: T) -> Self
    where
        T: Into<String>,
    {
        self.title_was_set = true;
        self.map_window(|mut w| {
            w.title = title.into();
            w
        })
    }

    pub fn primary(mut self) -> Self {
        self.primary = true;
        self
    }

    /// Create the window fullscreened on the current monitor.
    pub fn fullscreen(self) -> Self {
        self.map_window(|mut w| {
            w.position = WindowPosition::Centered(MonitorSelection::Primary);
            w
        })
    }

    /// Requests maximized mode.
    pub fn maximized(self, maximized: bool) -> Self {
        self.map_window(|mut w| {
            w.set_maximized(maximized);
            w
        })
    }

    /// Sets whether the window will be initially hidden or visible.
    pub fn visible(self, visible: bool) -> Self {
        self.map_window(|mut w| {
            w.visible = visible;
            w
        })
    }

    /// Sets whether the background of the window should be transparent.
    pub fn transparent(self, transparent: bool) -> Self {
        self.map_window(|mut w| {
            w.transparent = transparent;
            w
        })
    }

    /// Sets whether the window should have a border, a title bar, etc.
    pub fn decorations(self, decorations: bool) -> Self {
        self.map_window(|mut w| {
            w.decorations = decorations;
            w
        })
    }

    /// Sets whether or not the window will always be on top of other windows.
    pub fn always_on_top(self, always_on_top: bool) -> Self {
        self.map_window(|mut w| {
            w.window_level = if always_on_top {
                WindowLevel::AlwaysOnTop
            } else {
                WindowLevel::Normal
            };
            w
        })
    }
}

impl<M> fmt::Debug for View<M> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let variant = match *self {
            View::WithModel(ref v) => format!("WithModel({:?})", v),
            View::Sketch(_) => "Sketch".to_string(),
        };

        write!(f, "View::{}", variant)
    }
}

impl<'a, 'w> Window<'a, 'w> {
    /// A unique identifier associated with this window.
    pub fn id(&self) -> Entity {
        self.entity
    }

    /// Returns the scale factor that can be used to map logical pixels to physical pixels and vice
    /// versa.
    ///
    /// Throughout nannou, you will see "logical pixels" referred to as "points", and "physical
    /// pixels" referred to as "pixels".
    ///
    /// This is typically `1.0` for a normal display, `2.0` for a retina display and higher on more
    /// modern displays.
    ///
    /// You can read more about what this scale factor means within winit's [dpi module
    /// documentation](https://docs.rs/winit/latest/winit/dpi/index.html).
    ///
    /// ## Platform-specific
    ///
    /// - **X11:** This respects Xft.dpi, and can be overridden using the `WINIT_X11_SCALE_FACTOR`
    ///   environment variable.
    /// - **Android:** Always returns 1.0.
    /// - **iOS:** Can only be called on the main thread. Returns the underlying `UiView`'s
    ///   `contentScaleFactor`.
    pub fn scale_factor(&self) -> f32 {
        self.window_mut().scale_factor()
    }

    /// The width and height in pixels of the client area of the window.
    ///
    /// The client area is the content of the window, excluding the title bar and borders.
    pub fn size_pixels(&self) -> UVec2 {
        self.window_mut().physical_size()
    }

    /// The width and height in points of the client area of the window.
    ///
    /// The client area is the content of the window, excluding the title bar and borders.
    ///
    /// This is the same as dividing the result  of `size_pixels()` by `scale_factor()`.
    pub fn size_points(&self) -> Vec2 {
        self.window_mut().size()
    }

    /// Modifies the inner size of the window.
    ///
    /// See the `size` methods for more informations about the values.
    pub fn set_size_pixels(&self, width: u32, height: u32) {
        self.window_mut()
            .resolution
            .set_physical_resolution(width, height)
    }

    /// Modifies the inner size of the window using point values.
    ///
    /// See the `size` methods for more informations about the values.
    pub fn set_size_points(&self, width: f32, height: f32) {
        self.window_mut().resolution.set(width, height)
    }

    /// Sets a minimum size for the window.
    pub fn set_min_size_points(&self, size: Option<Vec2>) {
        if let Some(size) = size {
            self.window_mut().resize_constraints.min_width = size.x;
            self.window_mut().resize_constraints.min_height = size.y;
        } else {
            self.window_mut().resize_constraints.min_width = f32::INFINITY;
            self.window_mut().resize_constraints.min_height = f32::INFINITY;
        }
    }

    /// Sets a maximum size for the window.
    pub fn set_max_size_points(&self, size: Option<Vec2>) {
        if let Some(size) = size {
            self.window_mut().resize_constraints.max_width = size.x;
            self.window_mut().resize_constraints.max_height = size.y;
        } else {
            self.window_mut().resize_constraints.max_width = f32::INFINITY;
            self.window_mut().resize_constraints.max_height = f32::INFINITY;
        }
    }

    /// Modifies the title of the window.
    ///
    /// This is a no-op if the window has already been closed.
    pub fn set_title(&self, title: String) {
        self.window_mut().title = title;
    }

    /// Set the visibility of the window.
    ///
    /// ## Platform-specific
    ///
    /// - Android: Has no effect.
    /// - iOS: Can only be called on the main thread.
    /// - Web: Has no effect.
    pub fn set_visible(&self, visible: bool) {
        self.window_mut().visible = visible;
    }

    /// Sets whether the window is resizable or not.
    ///
    /// Note that making the window unresizable doesn't exempt you from handling **Resized**, as
    /// that event can still be triggered by DPI scaling, entering fullscreen mode, etc.
    pub fn set_resizable(&self, resizable: bool) {
        self.window_mut().resizable = resizable
    }

    /// Sets the window to minimized or back.
    pub fn set_minimized(&self, minimized: bool) {
        self.window_mut().set_minimized(minimized)
    }

    /// Sets the window to maximized or back.
    pub fn set_maximized(&self, maximized: bool) {
        self.window_mut().set_maximized(maximized)
    }

    /// Set the window to fullscreen on the primary monitor.
    ///
    /// `true` enables fullscreen, `false` disables fullscreen.
    ///
    /// See the `set_fullscreen_with` method for more options and details about behaviour related
    /// to fullscreen.
    pub fn set_fullscreen(&self, fullscreen: bool) {
        if fullscreen {
            self.window_mut().mode = WindowMode::BorderlessFullscreen;
        } else {
            self.window_mut().mode = WindowMode::Windowed;
        }
    }

    /// Sets the window mode
    pub fn set_mode(&self, mode: WindowMode) {
        self.window_mut().mode = mode;
    }

    /// Gets the window's current fullscreen state.
    ///
    /// ## Platform-specific
    ///
    /// - **iOS:** Can only be called on the main thread.
    pub fn mode(&self) -> WindowMode {
        self.window_mut().mode
    }

    /// Turn window decorations on or off.
    ///
    /// ## Platform-specific
    ///
    /// - **iOS:** Can only be called on the main thread. Controls whether the status bar is hidden
    ///   via `setPrefersStatusBarHidden`.
    /// - **Web:** Has no effect.
    pub fn set_decorations(&self, decorations: bool) {
        self.window_mut().decorations = decorations;
    }

    /// Change whether or not the window will always be on top of other windows.
    pub fn set_always_on_top(&self, always_on_top: bool) {
        self.window_mut().window_level = if always_on_top {
            WindowLevel::AlwaysOnTop
        } else {
            WindowLevel::Normal
        }
    }

    /// Sets the window icon. On Windows and X11, this is typically the small icon in the top-left
    /// corner of the titlebar.
    ///
    /// ## Platform-specific
    ///
    /// This only has effect on Windows and X11.
    ///
    /// On Windows, this sets ICON_SMALL. The base size for a window icon is 16x16, but it's
    /// recommended to account for screen scaling and pick a multiple of that, i.e. 32x32.
    ///
    /// X11 has no universal guidelines for icon sizes, so you're at the whims of the WM. That
    /// said, it's usually in the same ballpark as on Windows.
    pub fn set_window_icon(&self, _window_icon: Option<()>) {
        todo!("https://github.com/bevyengine/bevy/issues/1031")
    }

    /// Sets the location of IME candidate box in client area coordinates relative to the top left.
    ///
    /// ## Platform-specific
    ///
    /// - **iOS:** Has no effect.
    /// - **Web:** Has no effect.
    pub fn set_ime_position_points(&self, x: f32, y: f32) {
        self.window_mut().ime_position = Vec2::new(x, y);
    }

    /// Modifies the mouse cursor of the window.
    ///
    /// ## Platform-specific
    ///
    /// - **iOS:** Has no effect.
    /// - **Android:** Has no effect.
    pub fn set_cursor_icon(&self, cursor: Cursor) {
        self.window_mut().cursor = cursor;
    }

    /// Changes the position of the cursor in logical window coordinates.
    ///
    /// ## Platform-specific
    ///
    /// - **iOS:** Always returns an `Err`.
    /// - **Web:** Has no effect.
    pub fn set_cursor_position_points(&self, x: f32, y: f32) {
        self.window_mut().set_cursor_position(Some(Vec2::new(x, y)));
    }

    /// Grabs the cursor, preventing it from leaving the window.
    ///
    /// ## Platform-specific
    ///
    /// - **macOS:** Locks the cursor in a fixed location.
    /// - **Wayland:** Locks the cursor in a fixed location.
    /// - **Android:** Has no effect.
    /// - **iOS:** Always returns an Err.
    /// - **Web:** Has no effect.
    pub fn set_cursor_grab(&self, grab: bool) {
        self.window_mut().cursor.grab_mode = if grab {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        };
    }

    /// Set the cursor's visibility.
    ///
    /// If `false`, hides the cursor. If `true`, shows the cursor.
    ///
    /// ## Platform-specific
    ///
    /// On **Windows**, **X11** and **Wayland**, the cursor is only hidden within the confines of
    /// the window.
    ///
    /// On **macOS**, the cursor is hidden as long as the window has input focus, even if the
    /// cursor is outside of the window.
    ///
    /// This has no effect on **Android** or **iOS**.
    pub fn set_cursor_visible(&self, visible: bool) {
        self.window_mut().cursor.visible = visible;
    }

    /// Attempts to determine whether or not the window is currently fullscreen.
    pub fn is_fullscreen(&self) -> bool {
        self.window_mut().mode == WindowMode::Fullscreen
    }

    /// The rectangle representing the position and dimensions of the window.
    ///
    /// The window's position will always be `[0.0, 0.0]`, as positions are generally described
    /// relative to the centre of the window itself.
    ///
    /// The dimensions will be equal to the result of `size_points`. This represents the area
    /// of the that we can draw to in a DPI-agnostic manner, typically useful for drawing and UI
    /// positioning.
    pub fn rect(&self) -> geom::Rect {
        let size = self.size_points();
        geom::Rect::from_x_y_w_h(0.0, 0.0, size.x, size.y)
    }

    /// Saves a screenshot of the window to the given path.
    pub fn save_screenshot<P: AsRef<Path>>(&mut self, path: P) {
        let mut screenshot_manager = self
            .app
            .world_mut()
            .get_resource_mut::<ScreenshotManager>()
            .expect("ScreenshotManager resource not found");
        screenshot_manager
            .save_screenshot_to_disk(self.entity, path)
            .expect("Failed to save screenshot");
    }
}
