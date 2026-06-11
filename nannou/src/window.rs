//! The nannou Window API.
//!
//! Create a new window via `app.new_window()`. This produces a [**Builder**](Builder) which can be
//! used to build a window and obtain its [`Entity`].

use std::{fmt, path::PathBuf};

use crate::context::App;
use crate::prelude::{MonitorSelection, render::NannouCamera};
use crate::{geom::Point2, glam::Vec2, prelude::WindowResizeConstraints};
use bevy::{
    camera::Hdr,
    camera::RenderTarget,
    camera::visibility::RenderLayers,
    input::mouse::MouseWheel,
    prelude::*,
    render::extract_component::ExtractComponent,
    window::{PrimaryWindow, WindowLevel, WindowRef},
};

/// A context for building a window.
pub struct Builder<'a, 'w, 's, M = ()> {
    app: &'a App<'w, 's>,
    window: bevy::window::Window,
    // TODO: make cameras and lights an array
    camera: Option<Entity>,
    light: Option<Entity>,
    primary: bool,
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

#[derive(Component, Deref, DerefMut, ExtractComponent)]
pub(crate) struct WindowUserFunctions<M: 'static>(pub(crate) UserFunctions<M>);

/// The user function type for drawing their model to the surface of a single window.
pub type ViewFn<Model> = fn(&App<'_, '_>, &Model);

/// The same as `ViewFn`, but provides no user model to draw from.
///
/// Useful for simple, stateless sketching.
pub type SketchFn = fn(&App<'_, '_>);

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
pub type KeyPressedFn<Model> = fn(&App<'_, '_>, &mut Model, KeyCode);

/// A function for processing key release events.
pub type KeyReleasedFn<Model> = fn(&App<'_, '_>, &mut Model, KeyCode);

/// A function for processing received characters.
pub type ReceivedCharacterFn<Model> = fn(&App<'_, '_>, &mut Model, char);

/// A function for processing mouse moved events.
pub type MouseMovedFn<Model> = fn(&App<'_, '_>, &mut Model, Point2);

/// A function for processing mouse pressed events.
pub type MousePressedFn<Model> = fn(&App<'_, '_>, &mut Model, MouseButton);

/// A function for processing mouse released events.
pub type MouseReleasedFn<Model> = fn(&App<'_, '_>, &mut Model, MouseButton);

/// A function for processing mouse entered events.
pub type MouseEnteredFn<Model> = fn(&App<'_, '_>, &mut Model);

/// A function for processing mouse exited events.
pub type MouseExitedFn<Model> = fn(&App<'_, '_>, &mut Model);

/// A function for processing mouse wheel events.
pub type MouseWheelFn<Model> = fn(&App<'_, '_>, &mut Model, MouseWheel);

/// A function for processing window moved events.
pub type MovedFn<Model> = fn(&App<'_, '_>, &mut Model, IVec2);

/// A function for processing window resized events.
pub type ResizedFn<Model> = fn(&App<'_, '_>, &mut Model, Vec2);

/// A function for processing touch events.
pub type TouchFn<Model> = fn(&App<'_, '_>, &mut Model, TouchInput);

// https://github.com/bevyengine/bevy/issues/6174
// A function for processing touchpad pressure events.
// pub type TouchpadPressureFn<Model> = fn(&App, &mut Model, TouchpadPressure);

/// A function for processing hovered file events.
pub type HoveredFileFn<Model> = fn(&App<'_, '_>, &mut Model, PathBuf);

/// A function for processing hovered file cancelled events.
pub type HoveredFileCancelledFn<Model> = fn(&App<'_, '_>, &mut Model);

/// A function for processing dropped file events.
pub type DroppedFileFn<Model> = fn(&App<'_, '_>, &mut Model, PathBuf);

/// A function for processing window focused events.
pub type FocusedFn<Model> = fn(&App<'_, '_>, &mut Model);

/// A function for processing window unfocused events.
pub type UnfocusedFn<Model> = fn(&App<'_, '_>, &mut Model);

/// A function for processing window closed events.
pub type ClosedFn<Model> = fn(&App<'_, '_>, &mut Model);

impl<'a, 'w, 's, M> Builder<'a, 'w, 's, M>
where
    M: 'static,
{
    /// Begin building a new window.
    pub fn new(app: &'a App<'w, 's>) -> Self {
        Builder {
            app,
            window: bevy::window::Window::default(),
            camera: None,
            light: None,
            primary: false,
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

    /// Builds the window and its camera, returning the window's [`Entity`].
    ///
    /// The window and camera are spawned via the `App`'s deferred command queue, so they become
    /// available on the following frame. The returned [`Entity`] is reserved immediately and is
    /// safe to store and use straight away.
    pub fn build(self) -> Entity {
        let Builder {
            app,
            window,
            camera,
            light,
            primary,
            user_functions,
            clear_color,
            hdr,
        } = self;

        if cfg!(target_arch = "wasm32") && !primary {
            // TODO: figure out a way to dynamically attach to a canvas with new windows.
            panic!("Non-primary windows are not supported on wasm");
        }

        let layer = RenderLayers::layer(app.window_count());
        let user_functions = WindowUserFunctions(user_functions);
        // Remember the window so it can be read back before the deferred spawn is applied.
        let window_for_cache = window.clone();
        let half_z_range = default_camera_half_z_range(&window_for_cache);

        // On wasm we reuse the existing primary window (created up-front so the renderer has a
        // canvas) rather than spawning a new one.
        #[cfg(target_arch = "wasm32")]
        let existing_primary = Some(app.main_window().id());
        #[cfg(not(target_arch = "wasm32"))]
        let existing_primary: Option<Entity> = None;

        let window_entity = app.command_scope(move |mut commands| {
            let window_entity = match existing_primary {
                Some(entity) => {
                    commands
                        .entity(entity)
                        .insert((window, user_functions, layer.clone()));
                    entity
                }
                None => {
                    let mut window = commands.spawn((window, user_functions, layer.clone()));
                    if primary {
                        window.insert(PrimaryWindow);
                    }
                    window.id()
                }
            };

            match camera {
                // Point an existing camera at the new window.
                Some(camera) => {
                    commands.entity(camera).insert((
                        RenderTarget::Window(WindowRef::Entity(window_entity)),
                        layer.clone(),
                    ));
                }
                // Otherwise spawn a default camera that renders to the window.
                None => {
                    let mut camera = commands.spawn((
                        Camera {
                            clear_color: clear_color
                                .map(ClearColorConfig::Custom)
                                .unwrap_or(ClearColorConfig::None),
                            ..default()
                        },
                        RenderTarget::Window(WindowRef::Entity(window_entity)),
                        Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
                        Projection::Orthographic(OrthographicProjection {
                            near: -half_z_range,
                            far: half_z_range,
                            ..OrthographicProjection::default_3d()
                        }),
                        layer.clone(),
                        NannouCamera,
                        WindowDefaultCamera,
                    ));
                    if hdr {
                        camera.insert(Hdr);
                    }
                }
            }

            if let Some(light) = light {
                commands.entity(light).insert(layer.clone());
            }

            window_entity
        });
        app.record_pending_window(window_entity, primary, window_for_cache);
        window_entity
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

    pub fn light(mut self, light: Entity) -> Self {
        self.light = Some(light);
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

    /// Requests the window to be a specific size in pixels.
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
    pub fn title<T>(self, title: T) -> Self
    where
        T: Into<String>,
    {
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

    /// Move the window to the center of the given monitor.
    pub fn monitor(self, monitor: MonitorSelection) -> Self {
        self.map_window(|mut w| {
            w.position = WindowPosition::Centered(monitor);
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

/// Marks the default camera spawned for a window when none was provided via the window
/// builder. Nannou manages this camera's orthographic projection to match its window.
#[derive(Component)]
pub(crate) struct WindowDefaultCamera;

// The half-extent of the z range visible to a window's default camera, matching the
// pre-bevy renderer: content within +-max(width, height) of the xy plane is visible.
fn default_camera_half_z_range(window: &bevy::window::Window) -> f32 {
    window.width().max(window.height())
}

/// Keep each default window camera's orthographic z range sized to its window, so that
/// 3D draw content up to the size of the window renders without near/far clipping while
/// depth precision stays proportionate to the scene.
pub(crate) fn update_default_camera_z_range(
    mut cameras: Query<(&mut Projection, &RenderTarget), With<WindowDefaultCamera>>,
    windows: Query<&bevy::window::Window>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) {
    for (mut projection, target) in cameras.iter_mut() {
        let window = match target {
            RenderTarget::Window(WindowRef::Entity(entity)) => *entity,
            RenderTarget::Window(WindowRef::Primary) => match primary_window.single() {
                Ok(entity) => entity,
                Err(_) => continue,
            },
            _ => continue,
        };
        let Ok(window) = windows.get(window) else {
            continue;
        };
        let half_z_range = default_camera_half_z_range(window);
        let Projection::Orthographic(ortho) = projection.as_ref() else {
            continue;
        };
        // Only write through the `Mut` when the range actually changes, to avoid
        // triggering change detection every frame.
        if ortho.near != -half_z_range || ortho.far != half_z_range {
            if let Projection::Orthographic(ortho) = &mut *projection {
                ortho.near = -half_z_range;
                ortho.far = half_z_range;
            }
        }
    }
}
