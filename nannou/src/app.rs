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
use std::path::Path;
use std::rc::Rc;
use std::{self};

use bevy::app::AppExit;
use bevy::core::FrameCount;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::{MouseButtonInput, MouseWheel};
use bevy::input::ButtonState;
use bevy::pbr::ExtendedMaterial;
use bevy::prelude::*;
use bevy::reflect::{DynamicTypePath, GetTypeRegistration};
use bevy::render::view::screenshot::ScreenshotManager;
use bevy::window::{PrimaryWindow, WindowClosed, WindowFocused, WindowResized};
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use find_folder;

use bevy_nannou::prelude::{draw, Draw};
use bevy_nannou::NannouPlugin;

use crate::prelude::bevy_reflect::{ReflectMut, ReflectOwned, ReflectRef, TypeInfo};
use crate::prelude::render::{NannouMaterial, NannouMesh, NannouPersistentMesh};
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
    Logical([f32; 2]),
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
struct ModelHolder<M>(M);

#[derive(Resource)]
struct CreateDefaultWindow;

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
                ..default()
            }),
            NannouPlugin,
        ));

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
    M: 'static + Send + Sync,
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

    pub fn init_fragment_shader<const SHADER: &'static str>(mut self) -> Self {
        self.app.add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, NannouMaterial<"", SHADER>>,
        >::default());
        self
    }

    /// Specify the default window size in points.
    ///
    /// If a window is created and its size is not specified, this size will be used.
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.config.default_window_size =
            Some(DefaultWindowSize::Logical([width as f32, height as f32]));
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
            .add_systems(Update, update::<M>)
            .add_systems(Last, last::<M>)
            .run();
    }
}

impl<M> Builder<M>
where
    M: Reflect + GetTypeRegistration + 'static,
{
    pub fn model_ui(mut self) -> Self {
        self.app.register_type::<ModelHolder<M>>();
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

impl<M> GetTypeRegistration for ModelHolder<M>
where
    M: GetTypeRegistration,
{
    fn get_type_registration() -> bevy::reflect::TypeRegistration {
        M::get_type_registration()
    }
}

impl<M> DynamicTypePath for ModelHolder<M>
where
    M: DynamicTypePath,
{
    fn reflect_type_path(&self) -> &str {
        self.0.reflect_type_path()
    }

    fn reflect_short_type_path(&self) -> &str {
        self.0.reflect_short_type_path()
    }

    fn reflect_type_ident(&self) -> Option<&str> {
        self.0.reflect_type_ident()
    }

    fn reflect_crate_name(&self) -> Option<&str> {
        self.0.reflect_crate_name()
    }

    fn reflect_module_path(&self) -> Option<&str> {
        self.0.reflect_module_path()
    }
}

impl<M> Reflect for ModelHolder<M>
where
    M: Reflect + DynamicTypePath + Any + GetTypeRegistration + 'static,
{
    fn get_represented_type_info(&self) -> Option<&'static TypeInfo> {
        self.0.get_represented_type_info()
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        Box::new(self.0).into_any()
    }

    fn as_any(&self) -> &dyn Any {
        self.0.as_any()
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self.0.as_any_mut()
    }

    fn into_reflect(self: Box<Self>) -> Box<dyn Reflect> {
        Box::new(self.0).into_reflect()
    }

    fn as_reflect(&self) -> &dyn Reflect {
        self.0.as_reflect()
    }

    fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
        self.0.as_reflect_mut()
    }

    fn apply(&mut self, value: &dyn Reflect) {
        self.0.apply(value)
    }

    fn set(&mut self, value: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>> {
        self.0.set(value)
    }

    fn reflect_ref(&self) -> ReflectRef {
        self.0.reflect_ref()
    }

    fn reflect_mut(&mut self) -> ReflectMut {
        self.0.reflect_mut()
    }

    fn reflect_owned(self: Box<Self>) -> ReflectOwned {
        Box::new(self.0).reflect_owned()
    }

    fn clone_value(&self) -> Box<dyn Reflect> {
        self.0.clone_value()
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

    pub fn mouse(&self) -> Vec2 {
        let window = self.window_id();
        let window = self
            .world()
            .entity(window)
            .get::<Window>()
            .expect("Entity is not a window");
        let screen_position = window.cursor_position().unwrap_or(Vec2::ZERO);
        screen_position - geom::pt2(window.width() / 2.0, window.height() / 2.0)
    }

    pub fn time(&self) -> Time {
        let time = self.world().get_resource::<Time>().unwrap();
        time.clone()
    }

    // Create a new `App`.
    fn new(world: &'w mut World) -> Self {
        let app = App {
            current_view: None,
            world: Rc::new(RefCell::new(world.as_unsafe_world_cell())),
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
    pub fn window_count(&mut self) -> usize {
        let mut window_q = self.world_mut().query::<&Window>();
        window_q.iter(self.world()).count()
    }

    /// A reference to the window with the given `Id`.
    pub fn window(&self, id: Entity) -> Option<&Window> {
        self.world().entity(id).get::<Window>()
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
    pub fn main_window(&self) -> Entity {
        let mut window_q = self
            .world_mut()
            .query_filtered::<Entity, With<PrimaryWindow>>();
        let main_window = window_q
            .get_single(self.world())
            .expect("No windows are open in the App");
        main_window
    }

    fn save_screenshot<P: AsRef<Path>>(&mut self, window: Entity, path: P) {
        let mut screenshot_manager = self
            .world_mut()
            .get_resource_mut::<ScreenshotManager>()
            .expect("ScreenshotManager resource not found");
        screenshot_manager
            .save_screenshot_to_disk(window, path)
            .expect("Failed to save screenshot");
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

    /// Produce the [App]'s [Draw] API for drawing geometry and text with colors and textures.
    pub fn draw(&self) -> draw::Draw {
        let mut draw = self.world_mut().entity(self.window_id()).get::<Draw>();

        if draw.is_none() {
            self.world_mut()
                .entity_mut(self.window_id())
                .insert(Draw(draw::Draw::new(self.window_id())));

            draw = self.world_mut().entity(self.window_id()).get::<Draw>();
        }

        draw.unwrap().0.clone()
    }

    pub fn draw_for_window(&self, window: Entity) -> draw::Draw {
        let mut draw = self.world_mut().entity(window).get::<Draw>();

        if draw.is_none() {
            self.world_mut()
                .entity_mut(window)
                .insert(Draw(draw::Draw::new(window)));

            draw = self.world_mut().entity(window).get::<Draw>();
        }

        draw.unwrap().0.clone()
    }

    /// The number of times the focused window's **view** function has been called since the start
    /// of the program.
    pub fn elapsed_frames(&self) -> u32 {
        let frame_count = self.world().resource::<FrameCount>();
        frame_count.0
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

    /// Quits the currently running application.
    pub fn quit(&mut self) {
        self.world_mut().send_event(AppExit::Success);
    }
}

fn startup<M>(world: &mut World)
where
    M: 'static + Send + Sync,
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
    // Insert the model into the world. We use a non-send resource here to allow
    // maximum flexibility for the user to provide their own model type that doesn't
    // implement `Send`. Bevy will ensure that the model is only accessed on the main
    // thread.
    world.insert_resource(ModelHolder(model));
}

fn first<M>(
    mut commands: Commands,
    bg_color_q: Query<Entity, With<BackgroundColor>>,
    meshes_q: Query<Entity, (With<NannouMesh>, Without<NannouPersistentMesh>)>,
) where
    M: 'static + Send + Sync,
{
    for entity in meshes_q.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in bg_color_q.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn update<M>(world: &mut World)
where
    M: 'static + Send + Sync,
{
    // We can clone these since they are just function pointers
    let update_fn = world.resource::<UpdateFnRes<M>>().0.clone();
    let view_fn = world.resource::<ViewFnRes<M>>().0.clone();

    let mut window_q = world.query::<(Entity, Option<&WindowUserFunctions<M>>)>();
    let windows = window_q
        .iter(world)
        .map(|(entity, user_fns)| {
            (
                entity,
                user_fns.map(|fns| {
                    let fns = fns.0.clone();
                    fns
                }),
            )
        })
        .collect::<Vec<_>>();

    // Extract the model from the world, this avoids borrowing issues.
    let mut model = world
        .remove_resource::<ModelHolder<M>>()
        .expect("Model not found")
        .0;

    let mut key_events = world.resource_mut::<Events<KeyboardInput>>();
    let mut key_events_reader = key_events.get_reader();
    let key_events = key_events_reader
        .read(&key_events)
        .into_iter()
        .map(|event| event.clone())
        .collect::<Vec<KeyboardInput>>();

    let received_char_events = world.resource::<Events<ReceivedCharacter>>();
    let mut received_char_events_reader = received_char_events.get_reader();
    let received_char_events = received_char_events_reader
        .read(&received_char_events)
        .into_iter()
        .map(|event| event.clone())
        .collect::<Vec<ReceivedCharacter>>();

    let cursor_moved_events = world.resource::<Events<CursorMoved>>();
    let mut cursor_moved_events_reader = cursor_moved_events.get_reader();
    let cursor_moved_events = cursor_moved_events_reader
        .read(&cursor_moved_events)
        .into_iter()
        .map(|event| event.clone())
        .collect::<Vec<CursorMoved>>();

    let mouse_button_events = world.resource::<Events<MouseButtonInput>>();
    let mut mouse_button_events_reader = mouse_button_events.get_reader();
    let mouse_button_events = mouse_button_events_reader
        .read(&mouse_button_events)
        .into_iter()
        .map(|event| event.clone())
        .collect::<Vec<MouseButtonInput>>();

    let cursor_entered_events = world.resource::<Events<CursorEntered>>();
    let mut cursor_entered_events_reader = cursor_entered_events.get_reader();
    let cursor_entered_events = cursor_entered_events_reader
        .read(&cursor_entered_events)
        .into_iter()
        .map(|event| event.clone())
        .collect::<Vec<CursorEntered>>();

    let cursor_left_events = world.resource::<Events<CursorLeft>>();
    let mut cursor_left_events_reader = cursor_left_events.get_reader();
    let cursor_left_events = cursor_left_events_reader
        .read(&cursor_left_events)
        .into_iter()
        .map(|event| event.clone())
        .collect::<Vec<CursorLeft>>();

    let mouse_wheel_events = world.resource::<Events<MouseWheel>>();
    let mut mouse_wheel_events_reader = mouse_wheel_events.get_reader();
    let mouse_wheel_events = mouse_wheel_events_reader
        .read(&mouse_wheel_events)
        .into_iter()
        .map(|event| event.clone())
        .collect::<Vec<MouseWheel>>();

    let window_moved_events = world.resource::<Events<WindowMoved>>();
    let mut window_moved_events_reader = window_moved_events.get_reader();
    let window_moved_events = window_moved_events_reader
        .read(&window_moved_events)
        .into_iter()
        .map(|event| event.clone())
        .collect::<Vec<WindowMoved>>();

    let window_resized_events = world.resource::<Events<WindowResized>>();
    let mut window_resized_events_reader = window_resized_events.get_reader();
    let window_resized_events = window_resized_events_reader
        .read(&window_resized_events)
        .into_iter()
        .map(|event| event.clone())
        .collect::<Vec<WindowResized>>();

    let touch_events = world.resource::<Events<TouchInput>>();
    let mut touch_events_reader = touch_events.get_reader();
    let touch_events = touch_events_reader
        .read(&touch_events)
        .into_iter()
        .map(|event| event.clone())
        .collect::<Vec<TouchInput>>();

    let file_drop_events = world.resource::<Events<FileDragAndDrop>>();
    let mut file_drop_events_reader = file_drop_events.get_reader();
    let file_drop_events = file_drop_events_reader
        .read(&file_drop_events)
        .into_iter()
        .map(|event| event.clone())
        .collect::<Vec<FileDragAndDrop>>();

    let window_focus_events = world.resource::<Events<WindowFocused>>();
    let mut window_focus_events_reader = window_focus_events.get_reader();
    let window_focus_events = window_focus_events_reader
        .read(&window_focus_events)
        .into_iter()
        .map(|event| event.clone())
        .collect::<Vec<WindowFocused>>();

    let window_closed_events = world.resource::<Events<WindowClosed>>();
    let mut window_closed_events_reader = window_closed_events.get_reader();
    let window_closed_events = window_closed_events_reader
        .read(&window_closed_events)
        .into_iter()
        .map(|event| event.clone())
        .collect::<Vec<WindowClosed>>();

    // Create a new app instance for each frame that wraps the world.
    let mut app = App::new(world);

    // Run the model update function.
    if let Some(update_fn) = update_fn {
        update_fn(&app, &mut model);
    }

    // Run the view function for each window's draw.
    for (entity, user_fns) in windows {
        // Makes sure we return the correct draw component
        app.current_view = Some(entity);

        // Run user fns
        if let Some(user_fns) = user_fns {
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
                if let Some(view) = view_fn.as_ref() {
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

            for evt in &key_events {
                if evt.window != entity {
                    continue;
                }

                match evt.state {
                    ButtonState::Pressed => {
                        if let Some(f) = user_fns.key_pressed {
                            f(&app, &mut model, evt.key_code);
                        }
                    }
                    ButtonState::Released => {
                        if let Some(f) = user_fns.key_released {
                            f(&app, &mut model, evt.key_code);
                        }
                    }
                }
            }

            for evt in &received_char_events {
                if evt.window != entity {
                    continue;
                }

                if let Some(f) = user_fns.received_character {
                    let char = evt.char.chars().into_iter().next().unwrap();
                    f(&app, &mut model, char);
                }
            }

            for evt in &cursor_moved_events {
                if evt.window != entity {
                    continue;
                }

                if let Some(f) = user_fns.mouse_moved {
                    f(&app, &mut model, evt.position);
                }
            }

            for evt in &mouse_button_events {
                if evt.window != entity {
                    continue;
                }

                match evt.state {
                    ButtonState::Pressed => {
                        if let Some(f) = user_fns.mouse_pressed {
                            f(&app, &mut model, evt.button);
                        }
                    }
                    ButtonState::Released => {
                        if let Some(f) = user_fns.mouse_released {
                            f(&app, &mut model, evt.button);
                        }
                    }
                }
            }

            for evt in &cursor_entered_events {
                if evt.window != entity {
                    continue;
                }

                if let Some(f) = user_fns.mouse_entered {
                    f(&app, &mut model);
                }
            }

            for evt in &cursor_left_events {
                if evt.window != entity {
                    continue;
                }

                if let Some(f) = user_fns.mouse_exited {
                    f(&app, &mut model);
                }
            }

            for evt in &mouse_wheel_events {
                if evt.window != entity {
                    continue;
                }

                if let Some(f) = user_fns.mouse_wheel {
                    f(&app, &mut model, evt.clone());
                }
            }

            for evt in &window_moved_events {
                if evt.window != entity {
                    continue;
                }

                if let Some(f) = user_fns.moved {
                    f(&app, &mut model, evt.position);
                }
            }

            for evt in &window_resized_events {
                if evt.window != entity {
                    continue;
                }

                if let Some(f) = user_fns.resized {
                    f(&app, &mut model, Vec2::new(evt.width, evt.height));
                }
            }

            for evt in &touch_events {
                if evt.window != entity {
                    continue;
                }

                if let Some(f) = user_fns.touch {
                    f(&app, &mut model, evt.clone());
                }
            }

            for evt in &file_drop_events {
                match evt {
                    FileDragAndDrop::DroppedFile { window, path_buf } => {
                        if *window != entity {
                            continue;
                        }

                        if let Some(f) = user_fns.dropped_file {
                            f(&app, &mut model, path_buf.clone());
                        }
                    }
                    FileDragAndDrop::HoveredFile { window, path_buf } => {
                        if *window != entity {
                            continue;
                        }

                        if let Some(f) = user_fns.hovered_file {
                            f(&app, &mut model, path_buf.clone());
                        }
                    }
                    FileDragAndDrop::HoveredFileCanceled { window } => {
                        if *window != entity {
                            continue;
                        }

                        if let Some(f) = user_fns.hovered_file_cancelled {
                            f(&app, &mut model);
                        }
                    }
                }
            }

            for evt in &window_focus_events {
                if evt.window != entity {
                    continue;
                }

                if evt.focused {
                    if let Some(f) = user_fns.focused {
                        f(&app, &mut model);
                    }
                } else {
                    if let Some(f) = user_fns.unfocused {
                        f(&app, &mut model);
                    }
                }
            }

            for evt in &window_closed_events {
                if evt.window != entity {
                    continue;
                }

                if let Some(f) = user_fns.closed {
                    f(&app, &mut model);
                }
            }
        }
    }

    // Don't use `app` after this point.
    drop(app);

    // Re-insert the model for the next frame.
    world.insert_resource(ModelHolder(model));
}

fn last<M>(world: &mut World)
where
    M: 'static + Send + Sync,
{
    let exit_events = world.resource::<Events<AppExit>>();
    let reader = exit_events.get_reader();

    let should_exit = !reader.is_empty(exit_events);
    if !should_exit {
        return;
    }

    let exit_fn = world.resource::<ExitFnRes<M>>().0.clone();
    let model = world
        .remove_resource::<ModelHolder<M>>()
        .expect("Model not found")
        .0;
    let app = App::new(world);
    if let Some(exit_fn) = exit_fn {
        exit_fn(&app, model);
    }
}
