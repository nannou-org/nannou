//! Items related to the `App` type and the application context in general.
//!
//! See here for items relating to the event loop, device access, creating and managing windows,
//! streams and more.
//!
//! - [**App**](./struct.App.html) - provides a context and API for windowing, devices, etc.
//! - [**Proxy**](./struct.Proxy.html) - a handle to an **App** that may be used from a non-main
//!   thread.
//! - [**LoopMode**](./enum.LoopMode.html) - describes the behaviour of the application event loop.
use std::any::Any;
use std::cell::RefCell;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use std::{self};

use bevy::app::AppExit;
use bevy::asset::io::file::FileAssetReader;
use bevy::asset::LoadState;
use bevy::core::FrameCount;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::ecs::system::SystemParam;
use bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::{MouseButtonInput, MouseWheel};
use bevy::input::ButtonState;
use bevy::pbr::ExtendedMaterial;
use bevy::prelude::*;
use bevy::reflect::{DynamicTypePath, GetTypeRegistration};
use bevy::window::{ExitCondition, PrimaryWindow, WindowClosed, WindowFocused, WindowResized};
use bevy::winit::{UpdateMode, WinitSettings};
#[cfg(feature = "egui")]
use bevy_egui::EguiContext;
// #[cfg(feature = "egui")]
// use bevy_inspector_egui::quick::ResourceInspectorPlugin;
// #[cfg(feature = "egui")]
// use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use find_folder;

use bevy_nannou::prelude::render::{DefaultNannouMaterial, ExtendedNannouMaterial};
use bevy_nannou::prelude::{draw, DrawHolder};
use bevy_nannou::NannouPlugin;

use crate::prelude::bevy_ecs::system::SystemState;
use crate::prelude::bevy_reflect::{ApplyError, ReflectMut, ReflectOwned, ReflectRef, TypeInfo};
use crate::prelude::render::{NannouMaterial, NannouMesh, NannouPersistentMesh};
use crate::prelude::NannouMaterialPlugin;
use crate::window::WindowUserFunctions;
use crate::{geom, window};

/// The user function type for initialising their model.
pub type ModelFn<Model> = fn(&App) -> Model;

/// The user function type for updating their model in accordance with some event.
pub type EventFn<Model, Event> = fn(&App, &mut Model, Event);

/// The user function type for updating the user model within the application loop.
pub type UpdateFn<Model> = fn(&App, &mut Model);

/// The user function type for drawing their model to the surface of a single window.
pub type ViewFn<Model> = fn(&App, &Model, view: Entity);

/// A shorthand version of `ViewFn` for sketches where the user does not need a model.
pub type SketchViewFn = fn(&App);

/// The user function type allowing them to consume the `model` when the application exits.
pub type ExitFn<Model> = fn(&App, Model);

/// The **App**'s view function.
enum View<Model = ()> {
    /// A view function allows for viewing the user's model.
    WithModel(ViewFn<Model>),
    /// A **Simple** view function does not require a user **Model**. Simpler to get started.
    Sketch(SketchViewFn),
}

impl<M> Clone for View<M> {
    fn clone(&self) -> Self {
        match self {
            View::WithModel(view) => View::WithModel(*view),
            View::Sketch(view) => View::Sketch(*view),
        }
    }
}

/// A nannou `App` builder.
pub struct Builder<M = ()> {
    app: bevy::app::App,
    model: ModelFn<M>,
    config: Config,
    update: Option<UpdateFn<M>>,
    default_view: Option<View<M>>,
    exit: Option<ExitFn<M>>,
}

/// A nannou `Sketch` builder.
pub struct SketchBuilder {
    builder: Builder<()>,
}

#[derive(Debug, Clone)]
enum DefaultWindowSize {
    /// Default window size in logical coordinates.
    Logical([u32; 2]),
    /// Fullscreen on whatever the primary monitor is at the time of window creation.
    Fullscreen,
}

/// The default `model` function used when none is specified by the user.
fn default_model(_: &App) -> () {
    ()
}

/// Each nannou application has a single **App** instance. This **App** represents the entire
/// context of the application.
///
/// The **App** provides access to most application, windowing and "IO" related APIs. In other
/// words, if you need access to windowing, the active wgpu devices, etc, the **App** will provide
/// access to this.
///
/// The **App** owns and manages:
///
/// - The **window and input event loop** used to drive the application forward.
/// - **All windows** for graphics and user input. Windows can be referenced via their IDs.
/// - The sharing of wgpu devices between windows.
/// - A default **Draw** instance for ease of use.
/// - A map of channels for submitting user input updates to active **Ui**s.
pub struct App<'w> {
    current_view: Option<Entity>,
    world: Rc<RefCell<UnsafeWorldCell<'w>>>,
    window_count: AtomicUsize,
}

#[derive(Resource, Deref, DerefMut)]
struct ModelFnRes<M>(ModelFn<M>);

#[derive(Resource, Deref, DerefMut)]
struct UpdateFnRes<M>(Option<UpdateFn<M>>);

#[derive(Resource, Deref, DerefMut)]
struct ViewFnRes<M>(Option<View<M>>);

#[derive(Resource, Deref, DerefMut)]
struct ExitFnRes<M>(Option<ExitFn<M>>);

/// Miscellaneous app configuration parameters.
#[derive(Resource, Debug, Clone)]
struct Config {
    exit_on_escape: bool,
    fullscreen_on_shortcut: bool,
    default_window_size: Option<DefaultWindowSize>,
}

#[derive(Resource)]
struct CreateDefaultWindow;

/// Controls the behaviour of the application loop.
#[derive(Resource, Default)]
pub enum RunMode {
    /// Run until the user exits the application.
    #[default]
    UntilExit,
    /// Run for a fixed number of frames.
    Ticks(u64),
    /// Run for a fixed duration (best effort).
    Duration(Duration),
}

impl RunMode {
    /// Run the main update loop once.
    pub fn once() -> Self {
        RunMode::Ticks(1)
    }
}

impl<M> Builder<M>
where
    M: 'static,
{
    /// Begin building the `App`.
    ///
    /// The `model` argument is the function that the App will call to initialise your Model.
    ///
    /// The Model can be thought of as the state that you would like to track throughout the
    /// lifetime of your nannou program from start to exit.
    ///
    /// The given function is called before any event processing begins within the application.
    ///
    /// The Model that is returned by the function is the same model that will be passed to the
    /// given event and view functions.
    pub fn new(model: ModelFn<M>) -> Self {
        let mut app = bevy::app::App::new();
        app.add_plugins((
            DefaultPlugins.set(WindowPlugin {
                // Don't spawn a  window by default, we'll handle this ourselves
                primary_window: None,
                exit_condition: ExitCondition::DontExit,
                ..default()
            }),
            #[cfg(feature = "egui")]
            bevy_egui::EguiPlugin,
            NannouPlugin,
        ))
        .init_resource::<RunMode>();

        Builder {
            app,
            model,
            config: Config::default(),
            update: None,
            default_view: None,
            exit: None,
        }
    }
}

impl<M> Builder<M>
where
    M: 'static,
{
    /// The default `view` function that the app will call to allow you to present your Model to
    /// the surface of a window on your display.
    ///
    /// This function will be used in the case that a window-specific view function has not been
    /// provided, e.g. via `window::Builder::view` or `window::Builder::sketch`.
    ///
    /// Note that when working with more than one window, you can use `frame.window_id()` to
    /// determine which window the current call is associated with.
    pub fn view(mut self, view: ViewFn<M>) -> Self {
        self.default_view = Some(View::WithModel(view));
        self
    }

    /// A function for updating the model within the application loop.
    ///
    /// See the `LoopMode` documentation for more information about the different kinds of
    /// application loop modes available in nannou and how they behave.
    ///
    /// Update events are also emitted as a variant of the `event` function. Note that if you
    /// specify both an `event` function and an `update` function, the `event` function will always
    /// be called with an update event prior to this `update` function.
    pub fn update(mut self, update: UpdateFn<M>) -> Self {
        self.update = Some(update);
        self
    }

    /// Tell the app that you would like it to create a single, simple, default window just before
    /// it calls your model function.
    ///
    /// The given `view` function will play the same role as if passed to the `view` builder
    /// method. Note that the `view` function passed to this method will overwrite any pre-existing
    /// view function specified by any preceding call to the `view`
    ///
    /// Note that calling this multiple times will not give you multiple windows, but instead will
    /// simply overwrite pre-existing calls to the method. If you would like to create multiple
    /// windows or would like more flexibility in your window creation process, please see the
    /// `App::new_window` method. The role of this `simple_window` method is to provide a
    /// quick-and-easy way to start with a simple window. This can be very useful for quick ideas,
    /// small single-window applications and examples.
    pub fn simple_window(mut self, view: ViewFn<M>) -> Self {
        self.default_view = Some(View::WithModel(view));
        self.app.insert_resource(CreateDefaultWindow);
        self
    }

    /// Specify an `exit` function to be called when the application exits.
    ///
    /// The exit function gives ownership of the model back to you for any cleanup that might be
    /// necessary.
    pub fn exit(mut self, exit: ExitFn<M>) -> Self {
        self.exit = Some(exit);
        self
    }

    /// Specify the behaviour of the application loop.
    pub fn set_run_mode(mut self, run_mode: RunMode) -> Self {
        self.app.insert_resource(run_mode);
        self
    }

    pub fn init_custom_material<T>(mut self) -> Self
    where
        T: Material + Default,
        T::Data: PartialEq + Eq + Hash + Clone,
    {
        self.app.add_plugins(NannouMaterialPlugin::<T>::default());
        self
    }

    /// Load a fragment shader asset from the given path for use with the nannou `Draw` API.
    #[cfg(feature = "nightly")]
    pub fn init_fragment_shader<const SHADER: &'static str>(mut self) -> Self {
        self.app
            .add_plugins(NannouMaterialPlugin::<ExtendedNannouMaterial<"", SHADER>>::default());
        self
    }

    /// Specify the default window size in points.
    ///
    /// If a window is created and its size is not specified, this size will be used.
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.config.default_window_size = Some(DefaultWindowSize::Logical([width, height]));
        self
    }

    /// Specify that windows should be created on the primary monitor by default.
    pub fn fullscreen(mut self) -> Self {
        self.config.default_window_size = Some(DefaultWindowSize::Fullscreen);
        self
    }

    /// Build and run an `App` with the specified parameters.
    ///
    /// This function will not return until the application has exited.
    ///
    /// If you wish to remain cross-platform friendly, we recommend that you call this on the main
    /// thread as some platforms require that their application event loop and windows are
    /// initialised on the main thread.
    pub fn run(mut self) {
        self.app
            // This ensures that color materials are rendered correctly.
            .insert_resource(AmbientLight {
                color: Color::WHITE,
                // This isn't randomly chosen
                // See:  https://discord.com/channels/691052431525675048/866787577687310356/1229248273735487560
                brightness: 998.096,
            })
            .insert_resource(self.config.clone())
            .insert_resource(ModelFnRes(self.model))
            .insert_resource(UpdateFnRes(self.update))
            .insert_resource(ViewFnRes(self.default_view))
            .insert_resource(ExitFnRes(self.exit))
            .add_systems(Startup, startup::<M>)
            .add_systems(First, first::<M>)
            .add_systems(
                Update,
                (
                    update::<M>,
                    key_events::<M>,
                    received_char_events::<M>,
                    cursor_moved_events::<M>,
                    mouse_button_events::<M>,
                    cursor_entered_events::<M>,
                    cursor_left_events::<M>,
                    mouse_wheel_events::<M>,
                    window_moved_events::<M>,
                    window_resized_events::<M>,
                    touch_events::<M>,
                    file_drop_events::<M>,
                    window_focus_events::<M>,
                    window_closed_events::<M>,
                ),
            )
            .add_systems(Last, last::<M>)
            .run();
    }
}

impl<M> Builder<M>
where
    M: Reflect + GetTypeRegistration + 'static,
{
    #[cfg(feature = "egui")]
    pub fn model_ui(mut self) -> Self {
        // .add_plugins(DefaultInspectorConfigPlugin)
        // .add_plugins(ResourceInspectorPlugin::<ModelHolder<M>>::default());
        self
    }
}

impl SketchBuilder {
    /// The size of the sketch window.
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.builder = self.builder.size(width, height);
        self
    }

    /// Build and run a `Sketch` with the specified parameters.
    ///
    /// This calls `App::run` internally. See that method for details!
    pub fn run(self) {
        self.builder.run()
    }
}

impl Builder<()> {
    /// Shorthand for building a simple app that has no model, handles no events and simply draws
    /// to a single window.
    ///
    /// This is useful for late night hack sessions where you just don't care about all that other
    /// stuff, you just want to play around with some ideas or make something pretty.
    pub fn sketch(view: SketchViewFn) -> SketchBuilder {
        let mut builder = Builder::new(default_model);
        builder.default_view = Some(View::Sketch(view));
        builder.app.insert_resource(CreateDefaultWindow);
        SketchBuilder { builder }
    }
}

impl Default for Config {
    fn default() -> Self {
        let exit_on_escape = App::DEFAULT_EXIT_ON_ESCAPE;
        let fullscreen_on_shortcut = App::DEFAULT_FULLSCREEN_ON_SHORTCUT;
        Config {
            exit_on_escape,
            fullscreen_on_shortcut,
            default_window_size: None,
        }
    }
}

impl<'w> App<'w> {
    pub const DEFAULT_EXIT_ON_ESCAPE: bool = true;
    pub const DEFAULT_FULLSCREEN_ON_SHORTCUT: bool = true;

    // Allocate a persistent entity
    pub fn entity(&self) -> Entity {
        let mut world = self.world_mut();
        world.spawn((NannouPersistentMesh,)).id()
    }

    pub fn assets(&self) -> Mut<AssetServer> {
        self.world_mut().resource_mut::<AssetServer>()
    }

    pub fn assets_path(&self) -> PathBuf {
        FileAssetReader::get_base_path().join("assets")
    }

    pub fn images(&self) -> &Assets<Image> {
        self.world().resource::<Assets<Image>>()
    }

    pub fn images_mut(&self) -> Mut<'_, Assets<Image>> {
        self.world_mut().resource_mut::<Assets<Image>>()
    }

    #[cfg(feature = "egui")]
    pub fn egui_for_window(&self, window: Entity) -> Mut<EguiContext> {
        self.world_mut()
            .get_mut::<EguiContext>(window)
            .expect("No egui context")
    }

    #[cfg(feature = "egui")]
    pub fn egui(&self) -> Mut<EguiContext> {
        self.egui_for_window(self.window_id())
    }

    pub fn mouse(&self) -> Vec2 {
        let window = self.window_id();
        let window = self
            .world()
            .entity(window)
            .get::<Window>()
            .expect("Entity is not a window");
        let screen_position = window.cursor_position().unwrap_or(Vec2::ZERO);
        Vec2::new(
            screen_position.x - window.width() / 2.0,
            -(screen_position.y - window.height() / 2.0),
        )
    }

    pub fn mouse_buttons(&self) -> &ButtonInput<MouseButton> {
        let mut mouse_input = self.world_mut().resource::<ButtonInput<MouseButton>>();
        mouse_input
    }

    pub fn keys(&self) -> &ButtonInput<KeyCode> {
        let mut keyboard_input = self.world_mut().resource::<ButtonInput<KeyCode>>();
        keyboard_input
    }

    pub fn time(&self) -> Time {
        let time = self.world().get_resource::<Time>().unwrap();
        time.clone()
    }

    pub fn elapsed_seconds(&self) -> f32 {
        let time = self.world().get_resource::<Time>().unwrap();
        time.elapsed_seconds()
    }

    // Create a new `App`.
    fn new(world: &'w mut World) -> Self {
        let world = world.as_unsafe_world_cell();
        let window_count = unsafe { world.world_mut() }
            .query::<&Window>()
            .iter(unsafe { world.world_mut() })
            .count();
        let app = App {
            current_view: None,
            world: Rc::new(RefCell::new(world)),
            window_count: window_count.into(),
        };
        app
    }

    pub fn world(&self) -> &World {
        unsafe { self.world.borrow().world() }
    }

    pub fn world_mut(&self) -> &mut World {
        unsafe { self.world.borrow_mut().world_mut() }
    }

    /// Returns the list of all the monitors available on the system.
    pub fn available_monitors(&self) -> Vec<()> {
        // Bevy doesn't expose this right now but could be nice
        todo!()
    }

    /// Returns the primary monitor of the system.
    /// May return None if none can be detected. For example, this can happen when running on Linux
    /// with Wayland.
    pub fn primary_monitor(&self) -> Option<()> {
        // Bevy doesn't expose this right now but could be nice
        todo!()
    }

    /// Begin building a new window.
    pub fn new_window<'a, M>(&'a self) -> window::Builder<'a, 'w, M>
    where
        M: 'static,
    {
        self.window_count.fetch_add(1usize, Ordering::SeqCst);
        let builder = window::Builder::new(&self);
        let config = self.world().resource::<Config>();
        let builder = match config.default_window_size {
            Some(DefaultWindowSize::Fullscreen) => builder.fullscreen(),
            Some(DefaultWindowSize::Logical([w, h])) => builder.size(w, h),
            None => builder,
        };
        builder
    }

    /// The number of windows currently in the application.
    pub fn window_count(&self) -> usize {
        self.window_count.load(Ordering::SeqCst)
    }

    /// A reference to the window with the given `Id`.
    pub fn window<'a>(&'a self, id: Entity) -> window::Window<'a, 'w>
    where
        'a: 'w,
    {
        window::Window::new(&self, id)
    }

    /// Return the [Entity] of the currently focused window.
    ///
    /// **Panics** if there are no windows or if no window is in focus.
    pub fn window_id(&self) -> Entity {
        let mut window_q = self.world_mut().query::<(Entity, &Window)>();
        for (entity, window) in window_q.iter(self.world()) {
            if window.focused {
                return entity;
            }
        }

        let mut primary_window = self
            .world_mut()
            .query_filtered::<Entity, With<PrimaryWindow>>();
        primary_window
            .get_single(self.world())
            .expect("No windows are open in the App")
    }

    /// Return a [Vec] containing a unique [Entity] for each currently open window managed by
    /// the [App].
    pub fn window_ids(&self) -> Vec<Entity> {
        let mut window_q = self.world_mut().query::<(Entity, &Window)>();
        window_q
            .iter(self.world())
            .map(|(entity, _)| entity)
            .collect()
    }

    /// Return the **Rect** for the currently focused window.
    ///
    /// The **Rect** coords are described in "points" (pixels divided by the hidpi factor).
    ///
    /// **Panics** if there are no windows or if no window is in focus.
    pub fn window_rect(&self) -> geom::Rect<f32> {
        let window = self.window_id();
        let window = self
            .world()
            .entity(window)
            .get::<Window>()
            .expect("Entity is not a window");
        geom::Rect::from_w_h(window.width(), window.height())
    }

    /// A reference to the window currently in focus.
    ///
    /// **Panics** if their are no windows open in the **App**.
    pub fn main_window<'a>(&'a self) -> crate::window::Window<'a, 'w> {
        let mut window_q = self
            .world_mut()
            .query_filtered::<Entity, With<PrimaryWindow>>();
        let main_window = window_q
            .get_single(self.world())
            .expect("No windows are open in the App");
        window::Window::new(self, main_window)
    }

    /// Return whether or not the `App` is currently set to exit when the `Escape` key is pressed.
    pub fn exit_on_escape(&self) -> bool {
        let config = self.world().resource::<Config>();
        config.exit_on_escape
    }

    /// Specify whether or not the app should close when the `Escape` key is pressed.
    ///
    /// By default this is `true`.
    pub fn set_exit_on_escape(&mut self, b: bool) {
        let mut config = self.world_mut().resource_mut::<Config>();
        config.exit_on_escape = b;
    }

    /// Returns whether or not the `App` is currently allows the focused window to enter or exit
    /// fullscreen via typical platform-specific shortcuts.
    ///
    /// - Linux uses F11.
    /// - macOS uses apple key + f.
    /// - Windows uses windows key + f.
    pub fn fullscreen_on_shortcut(&mut self) -> bool {
        let mut config = self.world_mut().resource_mut::<Config>();
        config.fullscreen_on_shortcut
    }

    /// Set whether or not the `App` should allow the focused window to enter or exit fullscreen
    /// via typical platform-specific shortcuts.
    ///
    /// - Linux uses F11.
    /// - macOS uses apple key + f.
    /// - Windows uses windows key + f.
    pub fn set_fullscreen_on_shortcut(&mut self, b: bool) {
        let mut config = self.world_mut().resource_mut::<Config>();
        config.fullscreen_on_shortcut = b;
    }

    /// Produce the [App]'s [DrawHolder] API for drawing geometry and text with colors and textures.
    pub fn draw(&self) -> draw::Draw {
        let mut draw = self
            .world_mut()
            .entity(self.window_id())
            .get::<DrawHolder>();
        draw.unwrap().0.clone()
    }

    pub fn draw_for_window(&self, window: Entity) -> draw::Draw {
        let mut draw = self.world_mut().entity(window).get::<DrawHolder>();
        draw.unwrap().0.clone()
    }

    /// The number of times the focused window's **view** function has been called since the start
    /// of the program.
    pub fn elapsed_frames(&self) -> u64 {
        let frame_count = self.world().resource::<FrameCount>();
        frame_count.0 as u64
    }

    /// The number of frames that can currently be displayed a second
    pub fn fps(&self) -> f64 {
        let diagnostics = self.world().resource::<DiagnosticsStore>();
        diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .expect("FrameTime diagnostics not found")
            .smoothed()
            .expect("Could not get smoothed fps")
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

    /// The path to the current project directory.
    ///
    /// The current project directory is considered to be the directory containing the cargo
    /// manifest (aka the `Cargo.toml` file).
    ///
    /// **Note:** Be careful not to rely on this directory for apps or sketches that you wish to
    /// distribute! This directory is mostly useful for local sketches, experiments and testing.
    pub fn project_path(&self) -> Result<PathBuf, find_folder::Error> {
        find_project_path()
    }

    /// Quits the currently running application.
    pub fn quit(&mut self) {
        self.world_mut().send_event(AppExit::Success);
    }

    pub fn set_update_mode(&self, mode: UpdateMode) {
        let mut winit_settings = self.world_mut().resource_mut::<WinitSettings>();
        winit_settings.unfocused_mode = mode;
        winit_settings.focused_mode = mode;
    }

    pub fn set_unfocused_update_mode(&self, mode: UpdateMode) {
        let mut winit_settings = self.world_mut().resource_mut::<WinitSettings>();
        winit_settings.unfocused_mode = mode;
    }

    pub fn set_focused_update_mode(&self, mode: UpdateMode) {
        let mut winit_settings = self.world_mut().resource_mut::<WinitSettings>();
        winit_settings.focused_mode = mode;
    }
}

fn get_app_and_state<'w, 's, S: SystemParam + 'static>(
    world: &'w mut World,
    state: &'s mut SystemState<S>,
) -> (App<'w>, <S as SystemParam>::Item<'w, 's>) {
    state.update_archetypes(world);
    let mut app = App::new(world);
    let param = unsafe { state.get_unchecked_manual(*app.world.borrow_mut()) };
    (app, param)
}

fn startup<M>(world: &mut World)
where
    M: 'static,
{
    let default_window_size = world.resource::<Config>().default_window_size.clone();
    let model_fn = world.resource::<ModelFnRes<M>>().0.clone();

    let mut app = App::new(world);

    // Create our default window if necessary
    if let Some(_) = app.world().get_resource::<CreateDefaultWindow>() {
        let mut window: window::Builder<'_, '_, M> = app.new_window();
        match default_window_size {
            None => {}
            Some(default_window) => {
                match default_window {
                    DefaultWindowSize::Logical([w, h]) => {
                        window = window.size(w, h);
                    }
                    DefaultWindowSize::Fullscreen => {
                        window = window.fullscreen();
                    }
                };
            }
        };
        let _ = window.primary().build();
    }

    // Initialise the model.
    let model = model_fn(&mut app);
    world.insert_non_send_resource(model);
}

fn first<M>(
    mut commands: Commands,
    bg_color_q: Query<Entity, With<BackgroundColor>>,
    meshes_q: Query<Entity, (With<NannouMesh>, Without<NannouPersistentMesh>)>,
) where
    M: 'static,
{
    for entity in meshes_q.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in bg_color_q.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn update<M>(
    world: &mut World,
    mut state: &mut SystemState<(
        Res<UpdateFnRes<M>>,
        Res<ViewFnRes<M>>,
        NonSendMut<M>,
        Res<RunMode>,
        Res<Time>,
        Local<u64>,
        Query<(Entity, &WindowUserFunctions<M>)>,
    )>,
) where
    M: 'static,
{
    let (mut app, (update_fn, view_fn, mut model, run_mode, time, mut ticks, windows)) =
        get_app_and_state(world, state);

    match *run_mode {
        RunMode::UntilExit => {
            // Do nothing, we'll quit when the user closes the window.
        }
        RunMode::Ticks(run_ticks) => {
            if *ticks >= run_ticks {
                app.quit();
                return;
            }
        }
        RunMode::Duration(duration) => {
            if time.elapsed() >= duration {
                app.quit();
                return;
            }
        }
    };

    // Run the model update function.
    if let Some(update_fn) = update_fn.0 {
        update_fn(&app, &mut model);
    }

    // Run the view function for each window's draw.
    for (entity, user_fns) in windows.iter() {
        // Makes sure we return the correct draw component
        app.current_view = Some(entity);

        // Run user fns
        if let Some(view) = &user_fns.view {
            match view {
                window::View::WithModel(view_fn) => {
                    view_fn(&app, &model);
                }
                window::View::Sketch(view_fn) => {
                    view_fn(&app);
                }
            }
        } else {
            if let Some(view) = view_fn.0.as_ref() {
                match view {
                    View::WithModel(view_fn) => {
                        view_fn(&app, &mut model, entity);
                    }
                    View::Sketch(view_fn) => {
                        view_fn(&app);
                    }
                }
            }
        }
    }

    // Increment the frame count.
    *ticks += 1;
}

fn key_events<M>(
    world: &mut World,
    state: &mut SystemState<(
        EventReader<KeyboardInput>,
        Query<&WindowUserFunctions<M>>,
        NonSendMut<M>,
    )>,
) where
    M: 'static,
{
    let (mut app, (mut key_events, user_fns, mut model)) = get_app_and_state(world, state);

    for evt in key_events.read() {
        if let Ok(user_fns) = user_fns.get(evt.window) {
            match evt.state {
                ButtonState::Pressed => {
                    if let Some(f) = user_fns.key_pressed {
                        app.current_view = Some(evt.window);
                        f(&app, &mut model, evt.key_code);
                    }
                }
                ButtonState::Released => {
                    if let Some(f) = user_fns.key_released {
                        app.current_view = Some(evt.window);
                        f(&app, &mut model, evt.key_code);
                    }
                }
            }
        }
    }
}

fn received_char_events<M>(
    world: &mut World,
    state: &mut SystemState<(
        EventReader<ReceivedCharacter>,
        Query<&WindowUserFunctions<M>>,
        NonSendMut<M>,
    )>,
) where
    M: 'static,
{
    let (mut app, (mut received_char_events, user_fns, mut model)) =
        get_app_and_state(world, state);

    for evt in received_char_events.read() {
        if let Ok(user_fns) = user_fns.get(evt.window) {
            if let Some(f) = user_fns.received_character {
                app.current_view = Some(evt.window);
                let char = evt.char.chars().into_iter().next().unwrap();
                f(&app, &mut model, char);
            }
        }
    }
}

fn cursor_moved_events<M>(
    world: &mut World,
    state: &mut SystemState<(
        EventReader<CursorMoved>,
        Query<&WindowUserFunctions<M>>,
        NonSendMut<M>,
    )>,
) where
    M: 'static,
{
    let (mut app, (mut cursor_moved_events, user_fns, mut model)) = get_app_and_state(world, state);

    for evt in cursor_moved_events.read() {
        if let Ok(user_fns) = user_fns.get(evt.window) {
            if let Some(f) = user_fns.mouse_moved {
                app.current_view = Some(evt.window);
                f(&app, &mut model, evt.position);
            }
        }
    }
}

fn mouse_button_events<M>(
    world: &mut World,
    state: &mut SystemState<(
        EventReader<MouseButtonInput>,
        Query<&WindowUserFunctions<M>>,
        NonSendMut<M>,
    )>,
) where
    M: 'static,
{
    let (mut app, (mut mouse_button_events, user_fns, mut model)) = get_app_and_state(world, state);

    for evt in mouse_button_events.read() {
        if let Ok(user_fns) = user_fns.get(evt.window) {
            match evt.state {
                ButtonState::Pressed => {
                    if let Some(f) = user_fns.mouse_pressed {
                        app.current_view = Some(evt.window);
                        f(&app, &mut model, evt.button);
                    }
                }
                ButtonState::Released => {
                    if let Some(f) = user_fns.mouse_released {
                        app.current_view = Some(evt.window);
                        f(&app, &mut model, evt.button);
                    }
                }
            }
        }
    }
}

fn cursor_entered_events<M>(
    world: &mut World,
    state: &mut SystemState<(
        EventReader<CursorEntered>,
        Query<&WindowUserFunctions<M>>,
        NonSendMut<M>,
    )>,
) where
    M: 'static,
{
    let (mut app, (mut cursor_entered_events, user_fns, mut model)) =
        get_app_and_state(world, state);

    for evt in cursor_entered_events.read() {
        if let Ok(user_fns) = user_fns.get(evt.window) {
            if let Some(f) = user_fns.mouse_entered {
                app.current_view = Some(evt.window);
                f(&app, &mut model);
            }
        }
    }
}

fn cursor_left_events<M>(
    world: &mut World,
    state: &mut SystemState<(
        EventReader<CursorLeft>,
        Query<&WindowUserFunctions<M>>,
        NonSendMut<M>,
    )>,
) where
    M: 'static,
{
    let (mut app, (mut cursor_left_events, user_fns, mut model)) = get_app_and_state(world, state);

    for evt in cursor_left_events.read() {
        if let Ok(user_fns) = user_fns.get(evt.window) {
            if let Some(f) = user_fns.mouse_exited {
                app.current_view = Some(evt.window);
                f(&app, &mut model);
            }
        }
    }
}

fn mouse_wheel_events<M>(
    world: &mut World,
    state: &mut SystemState<(
        EventReader<MouseWheel>,
        Query<&WindowUserFunctions<M>>,
        NonSendMut<M>,
    )>,
) where
    M: 'static,
{
    let (mut app, (mut mouse_wheel_events, user_fns, mut model)) = get_app_and_state(world, state);

    for evt in mouse_wheel_events.read() {
        if let Ok(user_fns) = user_fns.get(evt.window) {
            if let Some(f) = user_fns.mouse_wheel {
                app.current_view = Some(evt.window);
                f(&app, &mut model, evt.clone());
            }
        }
    }
}

fn window_moved_events<M>(
    world: &mut World,
    state: &mut SystemState<(
        EventReader<WindowMoved>,
        Query<&WindowUserFunctions<M>>,
        NonSendMut<M>,
    )>,
) where
    M: 'static,
{
    let (mut app, (mut window_moved_events, user_fns, mut model)) = get_app_and_state(world, state);

    for evt in window_moved_events.read() {
        if let Ok(user_fns) = user_fns.get(evt.window) {
            if let Some(f) = user_fns.moved {
                app.current_view = Some(evt.window);
                f(&app, &mut model, evt.position);
            }
        }
    }
}

fn window_resized_events<M>(
    world: &mut World,
    state: &mut SystemState<(
        EventReader<WindowResized>,
        Query<&WindowUserFunctions<M>>,
        NonSendMut<M>,
    )>,
) where
    M: 'static,
{
    let (mut app, (mut window_resized_events, user_fns, mut model)) =
        get_app_and_state(world, state);

    for evt in window_resized_events.read() {
        if let Ok(user_fns) = user_fns.get(evt.window) {
            if let Some(f) = user_fns.resized {
                app.current_view = Some(evt.window);
                f(&app, &mut model, Vec2::new(evt.width, evt.height));
            }
        }
    }
}

fn touch_events<M>(
    world: &mut World,
    state: &mut SystemState<(
        EventReader<TouchInput>,
        Query<&WindowUserFunctions<M>>,
        NonSendMut<M>,
    )>,
) where
    M: 'static,
{
    let (mut app, (mut touch_events, user_fns, mut model)) = get_app_and_state(world, state);

    for evt in touch_events.read() {
        if let Ok(user_fns) = user_fns.get(evt.window) {
            if let Some(f) = user_fns.touch {
                app.current_view = Some(evt.window);
                f(&app, &mut model, evt.clone());
            }
        }
    }
}

fn file_drop_events<M>(
    world: &mut World,
    state: &mut SystemState<(
        EventReader<FileDragAndDrop>,
        Query<&WindowUserFunctions<M>>,
        NonSendMut<M>,
    )>,
) where
    M: 'static,
{
    let (mut app, (mut file_drop_events, user_fns, mut model)) = get_app_and_state(world, state);

    for evt in file_drop_events.read() {
        match evt {
            FileDragAndDrop::DroppedFile { window, path_buf } => {
                if let Ok(user_fns) = user_fns.get(*window) {
                    if let Some(f) = user_fns.dropped_file {
                        app.current_view = Some(*window);
                        f(&app, &mut model, path_buf.clone());
                    }
                }
            }
            FileDragAndDrop::HoveredFile { window, path_buf } => {
                if let Ok(user_fns) = user_fns.get(*window) {
                    if let Some(f) = user_fns.hovered_file {
                        app.current_view = Some(*window);
                        f(&app, &mut model, path_buf.clone());
                    }
                }
            }
            FileDragAndDrop::HoveredFileCanceled { window } => {
                if let Ok(user_fns) = user_fns.get(*window) {
                    if let Some(f) = user_fns.hovered_file_cancelled {
                        app.current_view = Some(*window);
                        f(&app, &mut model);
                    }
                }
            }
        }
    }
}

fn window_focus_events<M>(
    world: &mut World,
    state: &mut SystemState<(
        EventReader<WindowFocused>,
        Query<&WindowUserFunctions<M>>,
        NonSendMut<M>,
    )>,
) where
    M: 'static,
{
    let (mut app, (mut window_focus_events, user_fns, mut model)) = get_app_and_state(world, state);

    for evt in window_focus_events.read() {
        if let Ok(user_fns) = user_fns.get(evt.window) {
            if evt.focused {
                if let Some(f) = user_fns.focused {
                    app.current_view = Some(evt.window);
                    f(&app, &mut model);
                }
            } else {
                if let Some(f) = user_fns.unfocused {
                    app.current_view = Some(evt.window);
                    f(&app, &mut model);
                }
            }
        }
    }
}

fn window_closed_events<M>(
    world: &mut World,
    state: &mut SystemState<(
        EventReader<WindowClosed>,
        Query<&WindowUserFunctions<M>>,
        NonSendMut<M>,
    )>,
) where
    M: 'static,
{
    let (mut app, (mut window_closed_events, user_fns, mut model)) =
        get_app_and_state(world, state);

    for evt in window_closed_events.read() {
        if let Ok(user_fns) = user_fns.get(evt.window) {
            if let Some(f) = user_fns.closed {
                app.current_view = Some(evt.window);
                f(&app, &mut model);
            }
        }
    }
}

fn last<M>(world: &mut World, state: &mut SystemState<(EventReader<AppExit>, Res<ExitFnRes<M>>)>)
where
    M: 'static,
{
    let (mut app, (mut exit_events, exit_fn)) = get_app_and_state(world, state);

    let should_exit = !exit_events.is_empty();
    if !should_exit {
        return;
    }

    let model = app
        .world_mut()
        .remove_non_send_resource::<M>()
        .expect("ModelHolder resource not found");

    if let Some(exit_fn) = exit_fn.0 {
        exit_fn(&app, model);
    }
}

pub trait UpdateModeExt {
    /// Wait indefinitely for the next update.
    fn wait() -> UpdateMode;
    /// Freeze the application, sending no further updates.
    fn freeze() -> UpdateMode;
}

impl UpdateModeExt for UpdateMode {
    fn wait() -> UpdateMode {
        UpdateMode::Reactive {
            wait: Duration::MAX,
            react_to_device_events: true,
            react_to_user_events: true,
            react_to_window_events: true,
        }
    }

    fn freeze() -> UpdateMode {
        UpdateMode::Reactive {
            wait: Duration::MAX,
            react_to_device_events: false,
            react_to_user_events: false,
            react_to_window_events: false,
        }
    }
}

/// Attempt to find the assets directory path relative to the executable location.
pub fn find_project_path() -> Result<PathBuf, find_folder::Error> {
    let exe_path = std::env::current_exe()?;
    let mut path = exe_path.parent().expect("exe has no parent directory");
    while let Some(parent) = path.parent() {
        path = parent;
        if path.join("Cargo").with_extension("toml").exists() {
            return Ok(path.to_path_buf());
        }
    }
    Err(find_folder::Error::NotFound)
}
