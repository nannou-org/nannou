//! The nannou Window API.
//!
//! Create a new window via `app.new_window()`. This produces a [**Builder**](./struct.Builder.html)
//! which can be used to build a [**Window**](./struct.Window.html).

use std::fmt;
use std::path::PathBuf;
use bevy::core_pipeline::bloom::{BloomCompositeMode, BloomPrefilterSettings, BloomSettings};
use bevy::core_pipeline::prepass::NormalPrepass;
use bevy::core_pipeline::tonemapping::Tonemapping;

use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::render::camera::{RenderTarget, ScalingMode};
use bevy::window::{PrimaryWindow, WindowLevel, WindowRef};

use bevy_nannou::prelude::MonitorSelection;

use crate::geom::Point2;
use crate::glam::Vec2;
use crate::prelude::WindowResizeConstraints;
use crate::App;

#[derive(Component)]
pub struct NannouCamera;

/// A context for building a window.
pub struct Builder<'a, 'w, M = ()> {
    app: &'a App<'w>,
    window: Window,
    primary: bool,
    title_was_set: bool,
    user_functions: UserFunctions<M>,
    clear_color: Option<Color>,
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

impl <M> Clone for UserFunctions<M> {
    fn clone(&self) -> Self {
        UserFunctions {
            view: self.view.clone(),
            key_pressed: self.key_pressed.clone(),
            key_released: self.key_released.clone(),
            received_character: self.received_character.clone(),
            mouse_moved: self.mouse_moved.clone(),
            mouse_pressed: self.mouse_pressed.clone(),
            mouse_released: self.mouse_released.clone(),
            mouse_entered: self.mouse_entered.clone(),
            mouse_exited: self.mouse_exited.clone(),
            mouse_wheel: self.mouse_wheel.clone(),
            moved: self.moved.clone(),
            resized: self.resized.clone(),
            touch: self.touch.clone(),
            // touchpad_pressure: self.touchpad_pressure,
            hovered_file: self.hovered_file.clone(),
            hovered_file_cancelled: self.hovered_file_cancelled.clone(),
            dropped_file: self.dropped_file.clone(),
            focused: self.focused.clone(),
            unfocused: self.unfocused.clone(),
            closed: self.closed.clone(),
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

impl <M> Clone for View<M> {
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
            window: Window::default(),
            primary: false,
            title_was_set: false,
            user_functions: UserFunctions::<M>::default(),
            clear_color: None,
        }
    }

    /// Build the window with some custom window parameters.
    pub fn window(mut self, window: Window) -> Self {
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
            if let Ok(_) = q.get_single(self.app.world_mut()) {
                panic!("Only one primary window can be created");
            }

            info!("Setting window {:?} as primary", entity);
            self.app.world_mut().entity_mut(entity).insert(PrimaryWindow);
        }

        self.app.world_mut().spawn((
            Camera3dBundle {
                camera: Camera {
                    // TODO: configure in builder
                    hdr: true,
                    target: RenderTarget::Window(WindowRef::Entity(entity)),
                    clear_color: self
                        .clear_color
                        .map(|c| ClearColorConfig::Custom(c))
                        .unwrap_or(ClearColorConfig::None),
                    ..Default::default()
                },
                transform: Transform::from_xyz(0.0, 0.0, 10.0)
                    .looking_at(Vec3::ZERO, Vec3::Y),
                projection: OrthographicProjection {
                    ..Default::default()
                }
                .into(),
                ..Default::default()
            },
            NannouCamera,
        ));

        self.app.world_mut().spawn((
            Camera3dBundle {
                camera: Camera {
                    // TODO: configure in builder
                    hdr: true,
                    target: RenderTarget::Window(WindowRef::Entity(entity)),
                    clear_color: ClearColorConfig::None,
                    order: 2,
                    ..Default::default()
                },
                tonemapping: Tonemapping::TonyMcMapface, // 2. Using a tonemapper that desaturates to white is recommended
                transform: Transform::from_xyz(0.0, 0.0, 10.0)
                    .looking_at(Vec3::ZERO, Vec3::Y),
                projection: OrthographicProjection {
                    ..Default::default()
                }
                    .into(),
                ..Default::default()
            },

            BloomSettings::OLD_SCHOOL,
            NannouCamera,
        ));

        entity
    }

    fn map_window<F>(self, map: F) -> Self
    where
        F: FnOnce(Window) -> Window,
    {
        let Builder {
            app,
            window,
            primary,
            title_was_set,
            user_functions,
            clear_color,
        } = self;
        let window = map(window);
        Builder {
            app,
            window,
            primary,
            title_was_set,
            user_functions,
            clear_color,
        }
    }

    /// Requests the window to be a specific size in points.
    ///
    /// This describes to the "inner" part of the window, not including desktop decorations like the
    /// title bar.
    pub fn size(self, width: f32, height: f32) -> Self {
        self.map_window(|mut w| {
            w.resolution.set(width, height);
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
