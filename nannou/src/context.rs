//! A Bevy-native nannou context.
//!
//! This module provides [`App`] - a Bevy [`SystemParam`] that bundles the curated set of
//! resources and queries nannou needs to expose its familiar conveniences (time, input, the
//! focused window, the [`Draw`] API, etc.). Unlike the classic [`crate::App`], this `App`
//! contains no `unsafe`: it is just a normal `SystemParam`, so the Bevy scheduler grants and
//! checks its world access like any other system parameter.
//!
//! Every method takes `&self`. Reads are served from the bundled resources and queries; mutations
//! (spawning windows/cameras/lights, quitting, changing the update mode, ...) are deferred through
//! [`ParallelCommands`], which provides shared (`&self`) command access. This is what lets the same
//! `App` be used both as a system parameter (`fn system(app: App)`) and behind a shared reference
//! (`fn view(app: &App, ..)`), which the classic `nannou::app`/`sketch` builders rely on.
//!
//! Use it directly from your own Bevy systems after adding [`crate::NannouPlugin`]:
//!
//! ```ignore
//! use bevy::prelude::*;
//! use nannou::prelude::*;
//!
//! fn main() {
//!     bevy::app::App::new()
//!         .add_plugins((DefaultPlugins, nannou::NannouPlugin))
//!         .add_systems(Startup, |app: nannou::context::App| { app.new_camera().build(); })
//!         .add_systems(Update, update)
//!         .run();
//! }
//!
//! fn update(app: nannou::context::App) {
//!     let draw = app.draw();
//!     draw.background().color(DIM_GRAY);
//!     draw.ellipse().xy(app.mouse()).color(BLUE).w_h(100.0, 100.0);
//! }
//! ```
//!
//! For anything outside this curated surface (arbitrary resources or assets), take the relevant
//! Bevy parameter (`Res<T>`, `ResMut<Assets<T>>`, etc.) directly in your system alongside `App`.

#[cfg(not(target_arch = "wasm32"))]
use bevy::asset::io::file::FileAssetReader;
use bevy::{
    app::AppExit,
    camera::{Hdr, RenderTarget, visibility::RenderLayers},
    diagnostic::{DiagnosticsStore, FrameCount, FrameTimeDiagnosticsPlugin},
    ecs::system::{ParallelCommands, SystemParam},
    prelude::*,
    window::{Monitor, PrimaryMonitor, PrimaryWindow, WindowRef},
    winit::{UpdateMode, WinitSettings},
};
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use nannou_core::geom;
use nannou_draw::draw::Draw;
use nannou_draw::text::font::SharedTextCx;
use std::cell::Cell;
use std::path::{Path, PathBuf};

use crate::app::find_project_path;
use crate::camera::{CameraComponents, SetCamera};
use crate::light::{LightComponents, SetLight};
use crate::prelude::render::NannouCamera;

/// A Bevy [`SystemParam`] providing nannou's application conveniences.
///
/// See the [module documentation](self) for an overview and example.
#[derive(SystemParam)]
pub struct App<'w, 's> {
    // Deferred command access usable through `&self` (unlike `Commands`, which needs `&mut`). This
    // is how every mutation - spawning, quitting, changing the update mode - is performed safely.
    par_commands: ParallelCommands<'w, 's>,
    time: Res<'w, Time>,
    frame_count: Res<'w, FrameCount>,
    // `DiagnosticsStore` is registered by `DefaultPlugins`, but the FPS diagnostic itself is only
    // present when `FrameTimeDiagnosticsPlugin` has been added (`NannouPlugin` adds it). Keep this
    // optional so `App` still resolves under a minimal plugin set.
    diagnostics: Option<Res<'w, DiagnosticsStore>>,
    keys: Res<'w, ButtonInput<KeyCode>>,
    mouse_buttons: Res<'w, ButtonInput<MouseButton>>,
    windows: Query<'w, 's, (Entity, &'static bevy::window::Window)>,
    primary_window: Query<'w, 's, Entity, With<PrimaryWindow>>,
    draws: Query<'w, 's, &'static Draw>,
    monitors: Query<'w, 's, (Entity, &'static Monitor)>,
    primary_monitor: Query<'w, 's, Entity, With<PrimaryMonitor>>,
    asset_server: Res<'w, AssetServer>,
    text_cx: Res<'w, SharedTextCx>,
    // The window whose `view` is currently being run, set by the classic driver systems so that
    // `draw()` targets the right window. `None` falls back to the focused window.
    current_view: Local<'s, Cell<Option<Entity>>>,
}

impl<'w, 's> App<'w, 's> {
    /// The elapsed seconds since startup.
    pub fn time(&self) -> f32 {
        self.time.elapsed_secs()
    }

    /// The elapsed seconds since the last frame.
    pub fn time_delta(&self) -> f32 {
        self.time.delta_secs()
    }

    /// The number of update frames that have elapsed since the start of the program.
    pub fn elapsed_frames(&self) -> u64 {
        (self.frame_count.0 as u64).saturating_sub(1)
    }

    /// The smoothed frames-per-second as reported by Bevy's frame-time diagnostics.
    ///
    /// Returns `0.0` if the FPS diagnostic is unavailable (e.g. if `FrameTimeDiagnosticsPlugin`
    /// was not registered).
    pub fn fps(&self) -> f64 {
        self.diagnostics
            .as_ref()
            .and_then(|d| d.get(&FrameTimeDiagnosticsPlugin::FPS))
            .and_then(|d| d.smoothed())
            .unwrap_or(0.0)
    }

    /// The current input state for the keyboard.
    pub fn keys(&self) -> ButtonInput<KeyCode> {
        self.keys.clone()
    }

    /// The current input state for the mouse buttons.
    pub fn mouse_buttons(&self) -> ButtonInput<MouseButton> {
        self.mouse_buttons.clone()
    }

    /// The current mouse position in points, relative to the centre of the focused window.
    pub fn mouse(&self) -> Vec2 {
        let (_, window) = self
            .windows
            .get(self.window_id())
            .expect("focused window entity is not a window");
        let screen_position = window.cursor_position().unwrap_or(Vec2::ZERO);
        Vec2::new(
            screen_position.x - window.width() / 2.0,
            -(screen_position.y - window.height() / 2.0),
        )
    }

    /// The [`Entity`] of the currently focused window, falling back to the primary window.
    ///
    /// **Panics** if there are no windows open.
    pub fn window_id(&self) -> Entity {
        for (entity, window) in self.windows.iter() {
            if window.focused {
                return entity;
            }
        }
        self.primary_window
            .single()
            .expect("no windows are open in the App")
    }

    /// The [`Entity`] for each currently open window.
    pub fn window_ids(&self) -> Vec<Entity> {
        self.windows.iter().map(|(entity, _)| entity).collect()
    }

    /// The number of windows currently open.
    pub fn window_count(&self) -> usize {
        self.windows.iter().count()
    }

    /// The [`geom::Rect`] of the currently focused window, in points.
    ///
    /// **Panics** if there are no windows open.
    pub fn window_rect(&self) -> geom::Rect<f32> {
        let (_, window) = self
            .windows
            .get(self.window_id())
            .expect("focused window entity is not a window");
        geom::Rect::from_w_h(window.width(), window.height())
    }

    /// The list of all monitors available on the system.
    pub fn available_monitors(&self) -> Vec<(Entity, Monitor)> {
        self.monitors
            .iter()
            .map(|(entity, monitor)| (entity, monitor.clone()))
            .collect()
    }

    /// The primary monitor of the system, if one can be detected.
    pub fn primary_monitor(&self) -> Option<Entity> {
        self.primary_monitor.single().ok()
    }

    /// The [`Draw`] API for the window whose `view` is currently running, or the focused window.
    ///
    /// **Panics** if there are no windows open.
    pub fn draw(&self) -> Draw {
        let window = self.current_view.get().unwrap_or_else(|| self.window_id());
        self.draw_for_window(window)
    }

    /// The [`Draw`] API for the given window.
    ///
    /// **Panics** if the entity has no [`Draw`] component (e.g. it is not a nannou window).
    pub fn draw_for_window(&self, window: Entity) -> Draw {
        self.draws
            .get(window)
            .expect("no `Draw` found for the given window")
            .clone()
    }

    /// Set the window whose `view` is currently being run, so [`draw`](Self::draw) targets it.
    ///
    /// Used by the classic driver systems; pass `None` to fall back to the focused window.
    // Wired up by the classic `app`/`sketch` driver systems in a follow-up step.
    #[allow(dead_code)]
    pub(crate) fn set_current_view(&self, view: Option<Entity>) {
        self.current_view.set(view);
    }

    /// A reference to the [`AssetServer`] for loading assets.
    pub fn asset_server(&self) -> &AssetServer {
        &self.asset_server
    }

    /// Build a text layout for measurement or glyph extraction.
    ///
    /// `App`-level equivalent of `draw.text_layout()`.
    pub fn text_layout<'b>(&self, s: &'b str) -> nannou_draw::text::Builder<'b> {
        nannou_draw::text::Builder::new(s, self.text_cx.clone())
    }

    /// Run a closure with deferred [`Commands`] access, returning its result.
    ///
    /// A convenience around [`ParallelCommands::command_scope`] for spawning entities or queueing
    /// world mutations from a shared `&App`. For example, spawn a camera bound to a window:
    ///
    /// ```ignore
    /// app.command_scope(|mut commands| {
    ///     commands.spawn(render::NannouCamera::for_window(window));
    /// });
    /// ```
    pub fn command_scope<R>(&self, f: impl FnOnce(Commands) -> R) -> R {
        self.par_commands.command_scope(f)
    }

    /// Quit the currently running application.
    pub fn quit(&self) {
        self.par_commands.command_scope(|mut commands| {
            commands.queue(|world: &mut World| {
                world.write_message(AppExit::Success);
            });
        });
    }

    /// Set the update mode used while the window is both focused and unfocused.
    ///
    /// See [`UpdateModeExt`](crate::app::UpdateModeExt) for convenient `wait`/`freeze` modes.
    pub fn set_update_mode(&self, mode: UpdateMode) {
        self.set_winit_settings(move |settings| {
            settings.focused_mode = mode;
            settings.unfocused_mode = mode;
        });
    }

    /// Set the update mode used while the window is unfocused.
    pub fn set_unfocused_update_mode(&self, mode: UpdateMode) {
        self.set_winit_settings(move |settings| settings.unfocused_mode = mode);
    }

    /// Set the update mode used while the window is focused.
    pub fn set_focused_update_mode(&self, mode: UpdateMode) {
        self.set_winit_settings(move |settings| settings.focused_mode = mode);
    }

    /// Queue a mutation of the [`WinitSettings`] resource.
    fn set_winit_settings(&self, f: impl FnOnce(&mut WinitSettings) + Send + 'static) {
        self.par_commands.command_scope(move |mut commands| {
            commands.queue(move |world: &mut World| {
                f(&mut world.resource_mut::<WinitSettings>());
            });
        });
    }

    /// The name of the nannou executable that is currently running.
    pub fn exe_name(&self) -> std::io::Result<String> {
        let string = std::env::current_exe()?
            .file_stem()
            .expect("exe path contained no file stem")
            .to_string_lossy()
            .to_string();
        Ok(string)
    }

    /// The path to the current project directory (the directory containing `Cargo.toml`).
    pub fn project_path(&self) -> Result<PathBuf, find_folder::Error> {
        find_project_path()
    }

    /// The path to the assets directory.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn assets_path(&self) -> PathBuf {
        FileAssetReader::get_base_path().join("assets")
    }

    /// Begin building a new window.
    ///
    /// The returned [`WindowBuilder`] spawns the window along with a [`NannouCamera`] targeting it
    /// (so its [`Draw`] is rendered), and returns the window [`Entity`] from
    /// [`build`](WindowBuilder::build).
    ///
    /// The window is assigned a [`RenderLayers`] based on the current window count, so the first
    /// window renders on layer `0`. When spawning multiple windows within a single system run,
    /// assign distinct layers explicitly via [`WindowBuilder::layer`] (deferred spawns are not yet
    /// reflected in the window count).
    pub fn new_window(&self) -> WindowBuilder<'_, 'w, 's> {
        let layer = RenderLayers::layer(self.window_count());
        WindowBuilder {
            commands: &self.par_commands,
            window: bevy::window::Window::default(),
            primary: false,
            clear_color: None,
            hdr: false,
            layer,
        }
    }

    /// Begin building a new [`NannouCamera`].
    ///
    /// Configure it with the [`SetCamera`] methods, then call [`build`](CameraBuilder::build) to
    /// spawn it and obtain its [`Entity`].
    pub fn new_camera(&self) -> CameraBuilder<'_, 'w, 's> {
        CameraBuilder {
            commands: &self.par_commands,
            components: CameraComponents {
                transform: Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
                projection: OrthographicProjection::default_3d().into(),
                ..default()
            },
        }
    }

    /// Begin building a new directional light.
    ///
    /// Configure it with the [`SetLight`] methods, then call [`build`](LightBuilder::build) to
    /// spawn it and obtain its [`Entity`].
    pub fn new_light(&self) -> LightBuilder<'_, 'w, 's> {
        LightBuilder {
            commands: &self.par_commands,
            components: LightComponents {
                transform: Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            },
        }
    }

    /// A handle for reading and updating the window with the given [`Entity`].
    pub fn window(&self, entity: Entity) -> Window<'_, 'w, 's> {
        Window { app: self, entity }
    }

    /// A handle for reading and updating the primary window.
    ///
    /// **Panics** if there is no primary window.
    pub fn main_window(&self) -> Window<'_, 'w, 's> {
        let entity = self
            .primary_window
            .single()
            .expect("no primary window is open in the App");
        Window { app: self, entity }
    }
}

/// A handle to a window [`Entity`], for reading and updating its state. Created via [`App::window`].
///
/// Reads are served from the window component directly; updates are deferred through the `App`'s
/// command queue.
pub struct Window<'a, 'w, 's> {
    app: &'a App<'w, 's>,
    entity: Entity,
}

impl Window<'_, '_, '_> {
    /// The window's [`Entity`].
    pub fn id(&self) -> Entity {
        self.entity
    }

    fn component(&self) -> &bevy::window::Window {
        let (_, window) = self
            .app
            .windows
            .get(self.entity)
            .expect("entity is not a window");
        window
    }

    /// The scale factor mapping logical points to physical pixels for this window.
    pub fn scale_factor(&self) -> f32 {
        self.component().scale_factor()
    }

    /// The width and height of the window's client area in physical pixels.
    pub fn size_pixels(&self) -> UVec2 {
        self.component().physical_size()
    }

    /// The width and height of the window's client area in points.
    pub fn size_points(&self) -> Vec2 {
        self.component().size()
    }

    /// The [`geom::Rect`] of the window in points, centred on the origin.
    pub fn rect(&self) -> geom::Rect<f32> {
        let window = self.component();
        geom::Rect::from_w_h(window.width(), window.height())
    }

    /// Set the window's title.
    pub fn set_title(&self, title: impl Into<String>) {
        let entity = self.entity;
        let title = title.into();
        self.app.command_scope(move |mut commands| {
            commands.queue(move |world: &mut World| {
                if let Some(mut window) = world.get_mut::<bevy::window::Window>(entity) {
                    window.title = title;
                }
            });
        });
    }

    /// Set the window's inner size in points.
    pub fn set_size_points(&self, width: f32, height: f32) {
        let entity = self.entity;
        self.app.command_scope(move |mut commands| {
            commands.queue(move |world: &mut World| {
                if let Some(mut window) = world.get_mut::<bevy::window::Window>(entity) {
                    window.resolution.set(width, height);
                }
            });
        });
    }

    /// Save a screenshot of the window to the given path once it has been rendered.
    pub fn save_screenshot<P: AsRef<Path> + Send + 'static>(&self, path: P) {
        let entity = self.entity;
        self.app.command_scope(move |mut commands| {
            commands
                .spawn(Screenshot::window(entity))
                .observe(save_to_disk(path));
        });
    }
}

/// A context for building and spawning a new window via [`App::new_window`].
pub struct WindowBuilder<'a, 'w, 's> {
    commands: &'a ParallelCommands<'w, 's>,
    window: bevy::window::Window,
    primary: bool,
    clear_color: Option<Color>,
    hdr: bool,
    layer: RenderLayers,
}

impl WindowBuilder<'_, '_, '_> {
    /// Use the given [`Window`](bevy::window::Window) component, replacing any prior configuration.
    pub fn window(mut self, window: bevy::window::Window) -> Self {
        self.window = window;
        self
    }

    /// Request the window be a specific size in points.
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.window.resolution.set(width as f32, height as f32);
        self
    }

    /// Request a specific title for the window.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.window.title = title.into();
        self
    }

    /// Mark this window as the primary window.
    pub fn primary(mut self) -> Self {
        self.primary = true;
        self
    }

    /// Set the initial clear color for the window's camera.
    pub fn clear_color(mut self, color: impl Into<Color>) -> Self {
        self.clear_color = Some(color.into());
        self
    }

    /// Render the window's camera in HDR.
    pub fn hdr(mut self, hdr: bool) -> Self {
        self.hdr = hdr;
        self
    }

    /// Set the [`RenderLayers`] used by the window and its camera.
    pub fn layer(mut self, layer: RenderLayers) -> Self {
        self.layer = layer;
        self
    }

    /// Spawn the window and its camera, returning the window [`Entity`].
    pub fn build(self) -> Entity {
        let WindowBuilder {
            commands,
            window,
            primary,
            clear_color,
            hdr,
            layer,
        } = self;
        commands.command_scope(move |mut commands| {
            let window_entity = {
                let mut window = commands.spawn((window, layer.clone()));
                if primary {
                    window.insert(PrimaryWindow);
                }
                window.id()
            };

            let mut camera = commands.spawn((
                Camera {
                    clear_color: clear_color
                        .map(ClearColorConfig::Custom)
                        .unwrap_or(ClearColorConfig::None),
                    ..default()
                },
                RenderTarget::Window(WindowRef::Entity(window_entity)),
                Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
                Projection::Orthographic(OrthographicProjection::default_3d()),
                layer,
                NannouCamera,
            ));
            if hdr {
                camera.insert(Hdr);
            }

            window_entity
        })
    }
}

/// A context for building and spawning a new [`NannouCamera`] via [`App::new_camera`].
pub struct CameraBuilder<'a, 'w, 's> {
    commands: &'a ParallelCommands<'w, 's>,
    components: CameraComponents,
}

impl SetCamera for CameraBuilder<'_, '_, '_> {
    fn map_camera<F>(mut self, f: F) -> Self
    where
        F: FnOnce(CameraComponents) -> CameraComponents,
    {
        self.components = f(self.components);
        self
    }
}

impl CameraBuilder<'_, '_, '_> {
    /// Spawn the camera, returning its [`Entity`].
    pub fn build(self) -> Entity {
        let CameraComponents {
            transform,
            camera,
            hdr,
            projection,
            tonemapping,
            bloom_settings,
            render_layers,
            render_target,
        } = self.components;
        self.commands.command_scope(move |mut commands| {
            let mut entity = commands.spawn((
                transform,
                camera,
                projection,
                tonemapping,
                render_layers,
                NannouCamera,
            ));
            if let Some(bloom_settings) = bloom_settings {
                entity.insert(bloom_settings);
            }
            if let Some(hdr) = hdr {
                entity.insert(hdr);
            }
            if let Some(render_target) = render_target {
                entity.insert(render_target);
            }
            entity.id()
        })
    }
}

/// A context for building and spawning a new directional light via [`App::new_light`].
pub struct LightBuilder<'a, 'w, 's> {
    commands: &'a ParallelCommands<'w, 's>,
    components: LightComponents,
}

impl SetLight for LightBuilder<'_, '_, '_> {
    fn map_light<F>(mut self, f: F) -> Self
    where
        F: FnOnce(LightComponents) -> LightComponents,
    {
        self.components = f(self.components);
        self
    }
}

impl LightBuilder<'_, '_, '_> {
    /// Spawn the directional light, returning its [`Entity`].
    pub fn build(self) -> Entity {
        let LightComponents {
            transform,
            directional_light,
            render_layers,
        } = self.components;
        self.commands.command_scope(move |mut commands| {
            commands
                .spawn((transform, directional_light, render_layers))
                .id()
        })
    }
}
