//! Items related to the `App` type and the application context in general.
//!
//! See here for items relating to the event loop, device access, creating and managing windows,
//! streams and more.
//!
//! - [**App**](./struct.App.html) - provides a context and API for windowing, devices, etc.
//! - [**Proxy**](./struct.Proxy.html) - a handle to an **App** that may be used from a non-main
//!   thread.
//! - [**LoopMode**](./enum.LoopMode.html) - describes the behaviour of the application event loop.
use bevy::app::AppExit;
use bevy::core::FrameCount;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell;
use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use std::cell::RefCell;
use std::future::Future;
use std::ops::DerefMut;
use std::path::Path;
use std::rc::Rc;
use std::time::Duration;
use std::{self, future};

use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::render::view::screenshot::ScreenshotManager;
use bevy::window::{PrimaryWindow, WindowMode, WindowResolution};
use bevy_nannou::{NannouPlugin};
use find_folder;
use bevy_nannou::prelude::draw::Draw;

use crate::window::WindowUserFunctions;
use crate::{geom, window};
use crate::prelude::render::{NannouMaterial, NannouMesh, NannouPersistantMesh};

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
    create_default_window: bool,
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
            create_default_window: false,
        }
    }
}

impl<M> Builder<M>
where
    M: 'static,
{
    /// By default, we timeout if waiting for a frame capture job takes longer than 5 seconds. This
    /// is to avoid hanging forever in the case the frame writing process encounters an
    /// unrecoverable error.
    pub const DEFAULT_CAPTURE_FRAME_TIMEOUT: Duration = Duration::from_secs(10);

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
        self.create_default_window = true;
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
        let create_default_window = self.create_default_window;

        self.app
            // This ensures that color materials are rendered correctly.
            .insert_resource(AmbientLight {
                color: Color::WHITE,
                brightness: 1000.0,
            })
            .insert_resource(self.config.clone())
            .insert_resource(ModelFnRes(self.model))
            .insert_resource(UpdateFnRes(self.update))
            .insert_resource(ViewFnRes(self.default_view))
            .insert_resource(ExitFnRes(self.exit))
            .add_systems(Startup, move |world: &mut World| {
                let default_window_size = world.resource::<Config>().default_window_size.clone();
                let model_fn = world.resource::<ModelFnRes<M>>().0.clone();
                world.spawn(
                    DirectionalLightBundle {
                        transform: Transform::from_xyz(1.0, 1.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
                        ..Default::default()
                    }
                );

                let mut app = App::new(world);
                // Create our default window if necessary

                if create_default_window {
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
                world.insert_non_send_resource(model);
            })
            .add_systems(First, |
                mut commands: Commands,
                bg_color_q: Query<Entity, With<BackgroundColor>>,
                meshes_q: Query<Entity, (With<NannouMesh>, Without<NannouPersistantMesh>)>| {
                for entity in meshes_q.iter() {
                    commands.entity(entity).despawn_recursive();
                }
                for entity in bg_color_q.iter() {
                    commands.entity(entity).despawn_recursive();
                }
            })
            .add_systems(Update, update::<M>)
            .add_systems(Last, |world: &mut World| {
                let exit_events = world.resource::<Events<AppExit>>();
                let reader = exit_events.get_reader();
                let should_exit = !reader.is_empty(exit_events);
                if !should_exit {
                    return;
                }

                let exit_fn = world.resource::<ExitFnRes<M>>().0.clone();
                let model = world
                    .remove_non_send_resource::<M>()
                    .expect("Model not found");
                let app = App::new(world);
                if let Some(exit_fn) = exit_fn {
                    exit_fn(&app, model);
                }
            })
            .run()
    }
}

fn update<M: 'static>(world: &mut World) {
        // Get our update and view functions. These are just function pointers, so we can
        // clone them to avoid borrowing issues with app which contains a mutable reference
        // to the world.

        let update_fn = world.resource::<UpdateFnRes<M>>().0.clone();
        let view_fn = world.resource::<ViewFnRes<M>>().0.clone();

        let mut window_q =
            world.query::<(Entity, Option<&WindowUserFunctions<M>>)>();
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
            .remove_non_send_resource::<M>()
            .expect("Model not found");

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
            }
        }

        // Don't use `app` after this point.
        drop(app);

        // Re-insert the model for the next frame.
        world.insert_non_send_resource(model);
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
        builder.create_default_window = true;
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
        world.spawn((NannouPersistantMesh,)).id()
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
    pub fn draw(&self) -> Draw<'w> {
        Draw::new(self.world.clone(), self.current_view.unwrap())
    }

    pub fn draw_for_window(&self, window: Entity) -> Draw<'w> {
        Draw::new(self.world.clone(), self.current_view.unwrap())
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
        self.world_mut().send_event(AppExit);
    }
}
