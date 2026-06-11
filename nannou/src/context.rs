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
use bevy::camera::RenderTarget;
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::view::Msaa;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::{
    app::AppExit,
    diagnostic::{DiagnosticsStore, FrameCount, FrameTimeDiagnosticsPlugin},
    ecs::system::{ParallelCommands, SystemParam},
    prelude::*,
    window::{Monitor, PrimaryMonitor, PrimaryWindow, WindowRef},
    winit::{UpdateMode, WinitSettings},
};
use nannou_core::geom;
use nannou_draw::draw::Draw;
use nannou_draw::text::font::SharedTextCx;
use std::cell::{Cell, RefCell};
use std::ops::Deref;
use std::path::{Path, PathBuf};

use crate::app::find_project_path;
use crate::camera::{CameraComponents, SetCamera};
use crate::light::{LightComponents, SetLight};
use crate::prelude::render::NannouCamera;
#[cfg(feature = "egui")]
use bevy_egui::EguiContext;

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
    camera_msaa: Query<'w, 's, (&'static RenderTarget, &'static Msaa), With<NannouCamera>>,
    // `bevy_egui` attaches the `EguiContext` to the camera, so we resolve the `NannouCamera`
    // rendering to a given window to find its context.
    #[cfg(feature = "egui")]
    egui_cameras: Query<'w, 's, (Entity, &'static RenderTarget), With<NannouCamera>>,
    #[cfg(feature = "egui")]
    egui_contexts: Query<'w, 's, &'static EguiContext>,
    asset_server: Res<'w, AssetServer>,
    images: Res<'w, Assets<Image>>,
    text_cx: Res<'w, SharedTextCx>,
    // The render device and queue, available in the main world for wgpu interop. Optional so `App`
    // still resolves under a minimal plugin set without a renderer.
    render_device: Option<Res<'w, RenderDevice>>,
    render_queue: Option<Res<'w, RenderQueue>>,
    // The window whose `view` is currently being run, set by the classic driver systems so that
    // `draw()` targets the right window. `None` falls back to the focused window.
    current_view: Local<'s, Cell<Option<Entity>>>,
    // Windows created this run via `new_window` but not yet spawned (spawns are deferred through the
    // command queue). Lets the classic `model` read back a window it just created in the same call,
    // e.g. `app.new_window().build(); let r = app.window_rect();`. The `bool` records whether the
    // window was requested as primary, so `main_window` can resolve it before it spawns.
    pending_windows: Local<'s, RefCell<Vec<PendingWindow>>>,
}

/// A window created this call but not yet spawned: `(entity, primary, component)`.
type PendingWindow = (Entity, bool, bevy::window::Window);

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

    /// Run `f` with the [`Window`](bevy::window::Window) component for `entity`, from the world or
    /// (for a window created this call but not yet spawned) the pending-window cache.
    pub(crate) fn with_window<R>(
        &self,
        entity: Entity,
        f: impl FnOnce(&bevy::window::Window) -> R,
    ) -> Option<R> {
        if let Ok((_, window)) = self.windows.get(entity) {
            return Some(f(window));
        }
        let pending = self.pending_windows.borrow();
        pending
            .iter()
            .rev()
            .find(|(e, _, _)| *e == entity)
            .map(|(_, _, w)| f(w))
    }

    /// Record a window created this call but not yet spawned, so it can be read back immediately.
    pub(crate) fn record_pending_window(
        &self,
        entity: Entity,
        primary: bool,
        window: bevy::window::Window,
    ) {
        self.pending_windows
            .borrow_mut()
            .push((entity, primary, window));
    }

    /// The current mouse position in points, relative to the centre of the focused window.
    pub fn mouse(&self) -> Vec2 {
        self.with_window(self.window_id(), |window| {
            let screen_position = window.cursor_position().unwrap_or(Vec2::ZERO);
            Vec2::new(
                screen_position.x - window.width() / 2.0,
                -(screen_position.y - window.height() / 2.0),
            )
        })
        .expect("focused window entity is not a window")
    }

    /// The [`Entity`] of the "current" window: the focused window, else the primary window, else a
    /// window created this call (but not yet spawned), else any open window.
    ///
    /// **Panics** if there are no windows open.
    pub fn window_id(&self) -> Entity {
        // Prefer the focused window.
        for (entity, window) in self.windows.iter() {
            if window.focused {
                return entity;
            }
        }
        // Then the primary window.
        if let Ok(entity) = self.primary_window.single() {
            return entity;
        }
        // Then a window created this call but not yet spawned (e.g. just built in `model`).
        if let Some((entity, _, _)) = self.pending_windows.borrow().last() {
            return *entity;
        }
        // Finally, any open window (e.g. a freshly-spawned window not yet focused).
        self.windows
            .iter()
            .next()
            .map(|(entity, _)| entity)
            .expect("no windows are open in the App")
    }

    /// The [`Entity`] for each currently open window.
    pub fn window_ids(&self) -> Vec<Entity> {
        self.windows.iter().map(|(entity, _)| entity).collect()
    }

    /// The number of windows currently open (including any created this call but not yet spawned).
    pub fn window_count(&self) -> usize {
        let query_count = self.windows.iter().count();
        let pending = self.pending_windows.borrow();
        let pending_count = pending
            .iter()
            .filter(|(e, _, _)| self.windows.get(*e).is_err())
            .count();
        query_count + pending_count
    }

    /// The [`geom::Rect`] of the currently focused window, in points.
    ///
    /// **Panics** if there are no windows open.
    pub fn window_rect(&self) -> geom::Rect<f32> {
        self.with_window(self.window_id(), |window| {
            geom::Rect::from_w_h(window.width(), window.height())
        })
        .expect("focused window entity is not a window")
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

    /// The [`egui`](bevy_egui::egui) context for the given window.
    ///
    /// The returned context is a cheap, internally-shared handle - build your UI on it directly
    /// (e.g. `egui::Window::new(..).show(&ctx, ..)`).
    ///
    /// **Panics** if no camera renders to the given window.
    #[cfg(feature = "egui")]
    pub fn egui_for_window(&self, window: Entity) -> bevy_egui::egui::Context {
        use bevy::window::WindowRef;
        let camera = self
            .egui_cameras
            .iter()
            .find_map(|(camera, target)| match target {
                RenderTarget::Window(WindowRef::Entity(entity)) if *entity == window => {
                    Some(camera)
                }
                _ => None,
            })
            .expect("no camera found for window");
        self.egui_contexts
            .get(camera)
            .expect("no egui context for the window's camera")
            .get()
            .clone()
    }

    /// The [`egui`](bevy_egui::egui) context for the focused window.
    ///
    /// See [`egui_for_window`](Self::egui_for_window).
    #[cfg(feature = "egui")]
    pub fn egui(&self) -> bevy_egui::egui::Context {
        self.egui_for_window(self.window_id())
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
    pub(crate) fn set_current_view(&self, view: Option<Entity>) {
        self.current_view.set(view);
    }

    /// A reference to the [`AssetServer`] for loading assets.
    pub fn asset_server(&self) -> &AssetServer {
        &self.asset_server
    }

    /// A reference to the [`Image`] assets, for reading texture data.
    ///
    /// To add a new asset, use [`asset_server().add(..)`](AssetServer::add); for arbitrary asset
    /// types or mutable access, take the relevant `Assets<T>` parameter in your own Bevy system.
    pub fn image_assets(&self) -> &Assets<Image> {
        &self.images
    }

    /// Queue a mutation of the [`Image`] behind `handle`, applied via the deferred command queue.
    ///
    /// A convenience for editing a texture from a classic `update`/`view` - e.g. overwriting its
    /// pixel `data` or swapping its `sampler` - without dropping to a custom Bevy system. Bevy
    /// re-uploads the image to its GPU texture on the next extract. The mutation lands after the
    /// current `update`/`view` returns, and `f` is skipped if the handle no longer resolves.
    ///
    /// ```ignore
    /// // Overwrite a texture's pixels each frame.
    /// app.modify_image(&model.texture, move |image| image.data = Some(pixels));
    /// ```
    pub fn modify_image(
        &self,
        handle: &Handle<Image>,
        f: impl FnOnce(&mut Image) + Send + 'static,
    ) {
        let handle = handle.clone();
        self.command_scope(move |mut commands| {
            commands.queue(move |world: &mut World| {
                if let Some(mut image) = world.resource_mut::<Assets<Image>>().get_mut(&handle) {
                    f(&mut image);
                }
            });
        });
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
    ///
    /// Do not call another mutating `App` method (`quit`, `new_window().build()`,
    /// `window(..).set_title(..)`, ...) from inside the closure - they re-enter `command_scope`,
    /// which panics on the already-borrowed thread-local command queue. Use `commands` directly.
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
    /// The returned [`window::Builder`](crate::window::Builder) spawns the window along with a
    /// [`NannouCamera`] targeting it (so its [`Draw`] is rendered), and returns the window
    /// [`Entity`] from [`build`](crate::window::Builder::build).
    pub fn new_window<M: 'static>(&self) -> crate::window::Builder<'_, 'w, 's, M> {
        // Drop any pending windows that have since been spawned, so the cache stays bounded.
        self.pending_windows
            .borrow_mut()
            .retain(|(e, _, _)| self.windows.get(*e).is_err());
        crate::window::Builder::new(self)
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

    /// A handle for reading and updating the primary window, falling back to a primary window
    /// created this call (but not yet spawned).
    ///
    /// **Panics** if there is no primary window.
    pub fn main_window(&self) -> Window<'_, 'w, 's> {
        let entity = self
            .primary_window
            .single()
            .ok()
            .or_else(|| {
                // Fall back to a primary window created this call but not yet spawned.
                self.pending_windows
                    .borrow()
                    .iter()
                    .rev()
                    .find(|(_, primary, _)| *primary)
                    .map(|(e, _, _)| *e)
            })
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

    /// The scale factor mapping logical points to physical pixels for this window.
    pub fn scale_factor(&self) -> f32 {
        self.app
            .with_window(self.entity, |w| w.scale_factor())
            .expect("entity is not a window")
    }

    /// The width and height of the window's client area in physical pixels.
    pub fn size_pixels(&self) -> UVec2 {
        self.app
            .with_window(self.entity, |w| w.physical_size())
            .expect("entity is not a window")
    }

    /// The width and height of the window's client area in points.
    pub fn size_points(&self) -> Vec2 {
        self.app
            .with_window(self.entity, |w| w.size())
            .expect("entity is not a window")
    }

    /// The [`geom::Rect`] of the window in points, centred on the origin.
    pub fn rect(&self) -> geom::Rect<f32> {
        self.app
            .with_window(self.entity, |w| geom::Rect::from_w_h(w.width(), w.height()))
            .expect("entity is not a window")
    }

    /// Queue a mutation of this window's [`Window`](bevy::window::Window) component (deferred,
    /// applied through the `App`'s command queue).
    fn update_window(&self, f: impl FnOnce(&mut bevy::window::Window) + Send + 'static) {
        let entity = self.entity;
        self.app.command_scope(move |mut commands| {
            commands.queue(move |world: &mut World| {
                if let Some(mut window) = world.get_mut::<bevy::window::Window>(entity) {
                    f(&mut window);
                }
            });
        });
    }

    /// Set the window's title.
    pub fn set_title(&self, title: impl Into<String>) {
        let title = title.into();
        self.update_window(move |w| w.title = title);
    }

    /// Set the window's inner size in points.
    pub fn set_size_points(&self, width: f32, height: f32) {
        self.update_window(move |w| w.resolution.set(width, height));
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

    /// The wgpu [`Device`](crate::wgpu::Device) used by the renderer.
    pub fn device(&self) -> &crate::wgpu::Device {
        self.app
            .render_device
            .as_ref()
            .expect("the RenderDevice resource is not available")
            .wgpu_device()
    }

    /// The wgpu [`Queue`](crate::wgpu::Queue) used by the renderer.
    pub fn queue(&self) -> &crate::wgpu::Queue {
        self.app
            .render_queue
            .as_ref()
            .expect("the RenderQueue resource is not available")
            .deref()
            .deref()
            .deref()
    }

    /// The number of MSAA samples used by the camera rendering to this window.
    pub fn msaa_samples(&self) -> u32 {
        for (render_target, msaa) in self.app.camera_msaa.iter() {
            if let RenderTarget::Window(WindowRef::Entity(entity)) = render_target {
                if *entity == self.entity {
                    return msaa.samples();
                }
            }
        }
        // The camera may not be spawned yet (e.g. when queried from `model`, before the deferred
        // window/camera spawn is applied). Fall back to the default `NannouCamera` MSAA.
        Msaa::default().samples()
    }
}

impl nannou_wgpu::WithDeviceQueuePair for &Window<'_, '_, '_> {
    fn with_device_queue_pair<F, O>(self, f: F) -> O
    where
        F: FnOnce(&crate::wgpu::Device, &crate::wgpu::Queue) -> O,
    {
        f(self.device(), self.queue())
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
