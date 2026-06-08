//! A Bevy-native nannou context.
//!
//! This module provides [`App`] - a Bevy [`SystemParam`] that bundles the curated set of
//! resources and queries nannou needs to expose its familiar conveniences (time, input, the
//! focused window, the [`Draw`] API, etc.). Unlike the classic [`crate::App`], this `App`
//! contains no `unsafe`: it is just a normal `SystemParam`, so the Bevy scheduler grants and
//! checks its world access like any other system parameter.
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
//!         .add_systems(Startup, |mut commands: Commands| {
//!             commands.spawn(render::NannouCamera);
//!         })
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
    ecs::system::SystemParam,
    prelude::*,
    window::{Monitor, PrimaryMonitor, PrimaryWindow, WindowRef},
    winit::{UpdateMode, WinitSettings},
};
use nannou_core::geom;
use nannou_draw::draw::Draw;
use nannou_draw::text::font::SharedTextCx;
use std::path::PathBuf;

use crate::app::find_project_path;
use crate::camera::{CameraComponents, SetCamera};
use crate::light::{LightComponents, SetLight};
use crate::prelude::render::NannouCamera;

/// A Bevy [`SystemParam`] providing nannou's application conveniences.
///
/// See the [module documentation](self) for an overview and example.
#[derive(SystemParam)]
pub struct App<'w, 's> {
    commands: Commands<'w, 's>,
    time: Res<'w, Time>,
    frame_count: Res<'w, FrameCount>,
    // `DiagnosticsStore` is registered by `DefaultPlugins`, but the FPS diagnostic itself is only
    // present when `FrameTimeDiagnosticsPlugin` has been added (`NannouPlugin` adds it). Keep this
    // optional so `App` still resolves under a minimal plugin set.
    diagnostics: Option<Res<'w, DiagnosticsStore>>,
    keys: Res<'w, ButtonInput<KeyCode>>,
    mouse_buttons: Res<'w, ButtonInput<MouseButton>>,
    windows: Query<'w, 's, (Entity, &'static Window)>,
    primary_window: Query<'w, 's, Entity, With<PrimaryWindow>>,
    draws: Query<'w, 's, &'static Draw>,
    monitors: Query<'w, 's, (Entity, &'static Monitor)>,
    primary_monitor: Query<'w, 's, Entity, With<PrimaryMonitor>>,
    asset_server: Res<'w, AssetServer>,
    text_cx: Res<'w, SharedTextCx>,
    winit_settings: ResMut<'w, WinitSettings>,
    exit: MessageWriter<'w, AppExit>,
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

    /// The [`Draw`] API for the currently focused window.
    ///
    /// **Panics** if there are no windows open.
    pub fn draw(&self) -> Draw {
        self.draw_for_window(self.window_id())
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

    /// Access the inner [`Commands`] to spawn windows, cameras, lights or any other entities.
    ///
    /// For example, spawn an additional nannou camera bound to a window:
    ///
    /// ```ignore
    /// app.commands().spawn(render::NannouCamera::for_window(window));
    /// ```
    pub fn commands(&mut self) -> &mut Commands<'w, 's> {
        &mut self.commands
    }

    /// Quit the currently running application.
    pub fn quit(&mut self) {
        self.exit.write(AppExit::Success);
    }

    /// Set the update mode used while the window is both focused and unfocused.
    ///
    /// See [`UpdateModeExt`](crate::app::UpdateModeExt) for convenient `wait`/`freeze` modes.
    pub fn set_update_mode(&mut self, mode: UpdateMode) {
        self.winit_settings.unfocused_mode = mode;
        self.winit_settings.focused_mode = mode;
    }

    /// Set the update mode used while the window is unfocused.
    pub fn set_unfocused_update_mode(&mut self, mode: UpdateMode) {
        self.winit_settings.unfocused_mode = mode;
    }

    /// Set the update mode used while the window is focused.
    pub fn set_focused_update_mode(&mut self, mode: UpdateMode) {
        self.winit_settings.focused_mode = mode;
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
    pub fn new_window(&mut self) -> WindowBuilder<'_, 'w, 's> {
        let layer = RenderLayers::layer(self.window_count());
        WindowBuilder {
            commands: &mut self.commands,
            window: Window::default(),
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
    pub fn new_camera(&mut self) -> CameraBuilder<'_, 'w, 's> {
        CameraBuilder {
            commands: &mut self.commands,
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
    pub fn new_light(&mut self) -> LightBuilder<'_, 'w, 's> {
        LightBuilder {
            commands: &mut self.commands,
            components: LightComponents {
                transform: Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            },
        }
    }
}

/// A context for building and spawning a new window via [`App::new_window`].
pub struct WindowBuilder<'a, 'w, 's> {
    commands: &'a mut Commands<'w, 's>,
    window: Window,
    primary: bool,
    clear_color: Option<Color>,
    hdr: bool,
    layer: RenderLayers,
}

impl WindowBuilder<'_, '_, '_> {
    /// Use the given [`Window`] component, replacing any prior window configuration.
    pub fn window(mut self, window: Window) -> Self {
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
        let mut window = self.commands.spawn((self.window, self.layer.clone()));
        if self.primary {
            window.insert(PrimaryWindow);
        }
        let window_entity = window.id();

        let mut camera = self.commands.spawn((
            Camera {
                clear_color: self
                    .clear_color
                    .map(ClearColorConfig::Custom)
                    .unwrap_or(ClearColorConfig::None),
                ..default()
            },
            RenderTarget::Window(WindowRef::Entity(window_entity)),
            Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            Projection::Orthographic(OrthographicProjection::default_3d()),
            self.layer,
            NannouCamera,
        ));
        if self.hdr {
            camera.insert(Hdr);
        }

        window_entity
    }
}

/// A context for building and spawning a new [`NannouCamera`] via [`App::new_camera`].
pub struct CameraBuilder<'a, 'w, 's> {
    commands: &'a mut Commands<'w, 's>,
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
        let mut entity = self.commands.spawn((
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
    }
}

/// A context for building and spawning a new directional light via [`App::new_light`].
pub struct LightBuilder<'a, 'w, 's> {
    commands: &'a mut Commands<'w, 's>,
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
        self.commands
            .spawn((transform, directional_light, render_layers))
            .id()
    }
}
