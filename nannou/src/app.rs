//! Items related to the `App` type and the application context in general.
//!
//! See here for items relating to the event loop, device access, creating and managing windows,
//! streams and more.
//!
//! - [**App**](./struct.App.html) - provides a context and API for windowing, devices, etc.
//! - [**Proxy**](./struct.Proxy.html) - a handle to an **App** that may be used from a non-main
//!   thread.
//! - [**LoopMode**](./enum.LoopMode.html) - describes the behaviour of the application event loop.
use bevy::asset::UnapprovedPathMode;
use bevy::{
    app::AppExit,
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
        mouse::{MouseButtonInput, MouseWheel},
    },
    prelude::*,
    reflect::{
        ApplyError, DynamicTypePath, GetTypeRegistration, ReflectMut, ReflectOwned, ReflectRef,
        TypeInfo,
    },
    render::extract_resource::ExtractResource,
    window::{ExitCondition, WindowClosed, WindowEvent, WindowFocused, WindowResized},
    winit::UpdateMode,
};
// TODO: re-enable once `bevy-inspector-egui` supports Bevy 0.19 (see the
// RC -> stable tracking issue), along with `Builder::model_ui` and the
// `inspector_ui` example.
//#[cfg(feature = "inspector")]
//use bevy_inspector_egui::quick::ResourceInspectorPlugin;

use crate::NannouPlugin;
use crate::context::App;
use crate::prelude::render::NannouCamera;
use crate::{
    frame::Frame,
    prelude::{
        bevy_ecs::system::SystemState,
        bevy_reflect::{DynamicTyped, ReflectCloneError},
        render::{NannouShaderModelPlugin, ShaderModel},
    },
    render::{
        RenderApp, RenderPlugin,
        compute::{Compute, ComputeModel, ComputePlugin, ComputeShaderHandle, ComputeState},
    },
    window,
    window::WindowUserFunctions,
};
use find_folder;
use std::{any::Any, hash::Hash, path::PathBuf, time::Duration};

/// The user function type for initialising their model.
pub type ModelFn<Model> = fn(&App<'_, '_>) -> Model;

/// The user function type for producing the compute model post-update.
pub type ComputeUpdateFn<Model, ComputeModel> =
    fn(
        &App<'_, '_>,
        &Model,
        <ComputeModel as Compute>::State,
        Entity,
    ) -> (<ComputeModel as Compute>::State, ComputeModel);

/// The user function type for updating their model in accordance with some event.
pub type EventFn<Model, Event> = fn(&App<'_, '_>, &mut Model, &Event);

/// The user function type for updating the user model within the application loop.
pub type UpdateFn<Model> = fn(&App<'_, '_>, &mut Model);

/// The user function type for drawing their model to the surface of a single window.
pub type ViewFn<Model> = fn(&App<'_, '_>, &Model, view: Entity);

/// A shorthand version of `ViewFn` for sketches where the user does not need a model.
pub type SketchViewFn = fn(&App<'_, '_>);

/// The user function type allowing them to consume the `model` when the application exits.
pub type ExitFn<Model> = fn(&App<'_, '_>, Model);

/// The user function type for rendering their model to the surface of a single window.
pub type RenderFn<Model> = fn(&RenderApp, &Model, Frame);

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
pub struct Builder<M = (), E = WindowEvent> {
    app: bevy::app::App,
    model: ModelFn<M>,
    config: Config,
    event: Option<EventFn<M, E>>,
    update: Option<UpdateFn<M>>,
    render: Option<RenderFn<M>>,
    default_view: Option<View<M>>,
    exit: Option<ExitFn<M>>,
}

/// A nannou `Sketch` builder.
pub struct SketchBuilder<E = WindowEvent> {
    builder: Builder<(), E>,
}

#[derive(Debug, Clone)]
enum DefaultWindowSize {
    /// Default window size in logical coordinates.
    Logical([u32; 2]),
    /// Fullscreen on whatever the primary monitor is at the time of window creation.
    Fullscreen,
}

/// The default `model` function used when none is specified by the user.
fn default_model(_: &App<'_, '_>) {}

#[derive(Resource, Deref, DerefMut)]
struct ModelFnRes<M>(ModelFn<M>);

#[derive(Resource, Deref, DerefMut)]
struct EventFnRes<M, E>(Option<EventFn<M, E>>);

#[derive(Resource, Deref, DerefMut)]
struct UpdateFnRes<M>(Option<UpdateFn<M>>);

#[derive(Resource, Deref, DerefMut)]
struct ComputeUpdateFnRes<M, CM: Compute>(ComputeUpdateFn<M, CM>);

#[derive(Resource, Deref, DerefMut)]
pub(crate) struct RenderFnRes<M>(Option<RenderFn<M>>);

impl<M> ExtractResource for RenderFnRes<M>
where
    M: Clone + Send + Sync + 'static,
{
    type Source = Self;

    fn extract_resource(source: &Self::Source) -> Self {
        RenderFnRes(source.0.clone())
    }
}

#[derive(Resource, Deref, DerefMut)]
struct ViewFnRes<M>(Option<View<M>>);

#[derive(Resource, Deref, DerefMut)]
struct ExitFnRes<M>(Option<ExitFn<M>>);

/// Miscellaneous app configuration parameters.
#[derive(Resource, Debug, Clone)]
struct Config {
    default_window_size: Option<DefaultWindowSize>,
}

#[derive(Resource, Deref, DerefMut)]
pub struct ModelHolder<M>(pub M);

impl<M> ExtractResource for ModelHolder<M>
where
    M: Clone + Send + Sync + 'static,
{
    type Source = Self;

    fn extract_resource(source: &Self::Source) -> Self {
        ModelHolder(source.0.clone())
    }
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
            DefaultPlugins
                .set(AssetPlugin {
                    unapproved_path_mode: UnapprovedPathMode::Allow,
                    ..default()
                })
                .set(WindowPlugin {
                #[cfg(not(target_arch = "wasm32"))]
                // Don't spawn a  window by default, we'll handle this ourselves
                primary_window: None,
                #[cfg(target_arch = "wasm32")]
                // We create a default window on wasm to make sure that the render initialization
                // has a canvas to attach to when configuring the surface.
                primary_window: Some(Window {
                    title: "Nannou".to_string(),
                    resolution: (1024.0, 768.0).into(),
                    ..default()
                }),
                exit_condition: ExitCondition::OnAllClosed,
                ..default()
            }),
            #[cfg(feature = "egui")]
            // Single-pass mode lets nannou users build egui UI imperatively from
            // their `update`/`view` functions (the multi-pass default expects UI
            // to be built within the dedicated `EguiPrimaryContextPass` schedule).
            bevy_egui::EguiPlugin {
                enable_multipass_for_primary_context: false,
                ..bevy_egui::EguiPlugin::default()
            },
            NannouPlugin,
        ))
        .init_resource::<RunMode>();

        Builder {
            app,
            model,
            config: Config::default(),
            event: None,
            update: None,
            render: None,
            default_view: None,
            exit: None,
        }
    }
}

impl<M, E> Builder<M, E>
where
    M: 'static + Send + Sync,
    E: Message,
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

    pub fn render(mut self, render: RenderFn<M>) -> Self
    where
        M: Send + Sync + Clone + 'static,
    {
        self.render = Some(render);
        self.app.add_plugins(RenderPlugin::<M>::default());
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

    pub fn shader_model<SM>(mut self) -> Self
    where
        SM: ShaderModel,
        SM::Data: PartialEq + Eq + Hash + Clone,
    {
        self.app
            .add_plugins((NannouShaderModelPlugin::<SM>::default(),));
        self
    }

    pub fn compute<CM: Compute>(mut self, compute_fn: ComputeUpdateFn<M, CM>) -> Self {
        let render_app = self.app.sub_app_mut(bevy::render::RenderApp);
        render_app.insert_resource(ComputeShaderHandle(CM::shader()));
        self.app
            .add_systems(
                First,
                |mut commands: Commands, views_q: Query<Entity, Added<NannouCamera>>| {
                    for view in views_q.iter() {
                        info!("Adding compute state to view {:?}", view);
                        commands.entity(view).insert(ComputeState {
                            current: CM::State::default(),
                            next: None,
                            next_ready: false,
                        });
                    }
                },
            )
            .insert_resource(ComputeUpdateFnRes(compute_fn))
            .add_systems(Update, compute::<M, CM>.after(update::<M>))
            .add_systems(Last, |_query: Query<&ComputeModel<CM>>| {})
            .add_plugins(ComputePlugin::<CM>::default());
        self
    }

    #[cfg(any(feature = "config_json", feature = "config_toml"))]
    pub fn init_config<T>(mut self) -> Self
    where
        for<'de> T: serde::Deserialize<'de> + Asset,
    {
        self.app.add_plugins((
            #[cfg(feature = "config_json")]
            bevy_common_assets::json::JsonAssetPlugin::<T>::new(&[".json"]),
            #[cfg(feature = "config_toml")]
            bevy_common_assets::toml::TomlAssetPlugin::<T>::new(&[".toml"]),
        ));
        self
    }

    pub fn add_plugin<P>(mut self, plugin: P) -> Self
    where
        P: Plugin,
    {
        self.app.add_plugins(plugin);
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
            .insert_resource(self.config.clone())
            .insert_resource(ModelFnRes(self.model))
            .insert_resource(EventFnRes(self.event))
            .insert_resource(UpdateFnRes(self.update))
            .insert_resource(RenderFnRes(self.render))
            .insert_resource(ViewFnRes(self.default_view))
            .insert_resource(ExitFnRes(self.exit))
            .add_systems(Startup, startup::<M>)
            .add_systems(
                Update,
                (
                    update::<M>,
                    events::<M, E>,
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

// TODO: re-enable once `bevy-inspector-egui` supports Bevy 0.19 (see the
// RC -> stable tracking issue).
//impl<M> Builder<M>
//where
//    M: Reflect + GetTypeRegistration + 'static,
//{
//    #[cfg(feature = "inspector")]
//    pub fn model_ui(mut self) -> Self {
//        self.app
//            .add_plugins(ResourceInspectorPlugin::<ModelHolder<M>>::default());
//        self
//    }
//}

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
        Config {
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

impl<M> PartialReflect for ModelHolder<M>
where
    M: PartialReflect + DynamicTypePath + Any + GetTypeRegistration + 'static,
{
    fn get_represented_type_info(&self) -> Option<&'static TypeInfo> {
        self.0.get_represented_type_info()
    }

    fn into_partial_reflect(self: Box<Self>) -> Box<dyn PartialReflect> {
        Box::new(ModelHolder(self.0))
    }

    fn as_partial_reflect(&self) -> &dyn PartialReflect {
        self.0.as_partial_reflect()
    }

    fn as_partial_reflect_mut(&mut self) -> &mut dyn PartialReflect {
        self.0.as_partial_reflect_mut()
    }

    fn try_into_reflect(self: Box<Self>) -> Result<Box<dyn Reflect>, Box<dyn PartialReflect>> {
        Box::new(self.0).try_into_reflect()
    }

    fn try_as_reflect(&self) -> Option<&dyn Reflect> {
        self.0.try_as_reflect()
    }

    fn try_as_reflect_mut(&mut self) -> Option<&mut dyn Reflect> {
        self.0.try_as_reflect_mut()
    }

    fn try_apply(&mut self, value: &dyn PartialReflect) -> Result<(), ApplyError> {
        self.0.try_apply(value)
    }

    fn reflect_ref(&self) -> ReflectRef<'_> {
        self.0.reflect_ref()
    }

    fn reflect_mut(&mut self) -> ReflectMut<'_> {
        self.0.reflect_mut()
    }

    fn reflect_owned(self: Box<Self>) -> ReflectOwned {
        Box::new(self.0).reflect_owned()
    }

    fn reflect_clone(&self) -> std::result::Result<Box<dyn Reflect>, ReflectCloneError> {
        self.0.reflect_clone()
    }
}

impl<M> DynamicTyped for ModelHolder<M>
where
    M: 'static + Any + DynamicTypePath + GetTypeRegistration + Reflect,
{
    fn reflect_type_info(&self) -> &'static TypeInfo {
        self.0.reflect_type_info()
    }
}

impl<M> Reflect for ModelHolder<M>
where
    M: Reflect + DynamicTypePath + Any + GetTypeRegistration + 'static,
{
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

    fn set(&mut self, value: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>> {
        self.0.set(value)
    }
}

fn startup<M>(
    app: App,
    config: Res<Config>,
    model_fn: Res<ModelFnRes<M>>,
    create_default_window: Option<Res<CreateDefaultWindow>>,
) where
    M: 'static + Send + Sync,
{
    // Create our default window if necessary.
    if create_default_window.is_some() {
        let mut window = app.new_window::<M>();
        match &config.default_window_size {
            None => {}
            Some(DefaultWindowSize::Logical([w, h])) => {
                window = window.size(*w, *h);
            }
            Some(DefaultWindowSize::Fullscreen) => {
                window = window.fullscreen();
            }
        }
        let _ = window.primary().build();
    }

    // Initialise the model and insert it as a resource.
    let model_fn = model_fn.0;
    let model = model_fn(&app);
    app.command_scope(move |mut commands| {
        commands.insert_resource(ModelHolder(model));
    });
}

#[allow(clippy::type_complexity)]
fn update<M>(
    app: App,
    update_fn: Res<UpdateFnRes<M>>,
    view_fn: Res<ViewFnRes<M>>,
    mut model: ResMut<ModelHolder<M>>,
    run_mode: Res<RunMode>,
    time: Res<Time>,
    mut ticks: Local<u64>,
    windows: Query<(Entity, &WindowUserFunctions<M>)>,
) where
    M: 'static + Send + Sync,
{
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

    // Run the model update function. Reset the current view first so `app.draw()` called from
    // `update` targets the focused window rather than the last window of the previous frame's
    // view loop (`current_view` is a `Local` that persists across frames).
    app.set_current_view(None);
    if let Some(update_fn) = update_fn.0 {
        update_fn(&app, &mut model);
    }

    // Run the view function for each window's draw.
    for (entity, user_fns) in windows.iter() {
        // Makes sure `app.draw()` returns the draw for the correct window.
        app.set_current_view(Some(entity));

        if let Some(view) = &user_fns.view {
            match view {
                window::View::WithModel(view_fn) => {
                    view_fn(&app, &model);
                }
                window::View::Sketch(view_fn) => {
                    view_fn(&app);
                }
            }
        } else if let Some(view) = view_fn.0.as_ref() {
            match view {
                View::WithModel(view_fn) => {
                    view_fn(&app, &model, entity);
                }
                View::Sketch(view_fn) => {
                    view_fn(&app);
                }
            }
        }
    }

    *ticks += 1;
}

#[allow(clippy::type_complexity)]
fn compute<M, CM>(
    app: App,
    model: Res<ModelHolder<M>>,
    compute: Res<ComputeUpdateFnRes<M, CM>>,
    views_q: Query<(Entity, &ComputeState<CM::State>)>,
) where
    M: 'static + Send + Sync,
    CM: Compute,
{
    let compute = compute.0;
    for (view, state) in views_q.iter() {
        // Advance to the state queued last frame once the render world has signalled that its
        // pipeline is ready. Doing this here (rather than in the render world) keeps `current`
        // and the `ComputeModel` bind group built below in sync for the same state.
        let current = match (state.next_ready, &state.next) {
            (true, Some(next)) => next.clone(),
            _ => state.current.clone(),
        };
        let (new_state, compute_model) = compute(&app, &model, current.clone(), view);
        let next = (new_state != current).then_some(new_state);
        app.command_scope(move |mut commands| {
            commands.entity(view).insert((
                ComputeState {
                    current,
                    next,
                    next_ready: false,
                },
                ComputeModel(compute_model),
            ));
        });
    }
}

fn events<M, E>(
    app: App,
    mut events: MessageReader<E>,
    event_fn: Res<EventFnRes<M, E>>,
    mut model: ResMut<ModelHolder<M>>,
) where
    M: Send + Sync + 'static,
    E: Message,
{
    for evt in events.read() {
        if let Some(f) = event_fn.0.as_ref() {
            f(&app, &mut model, evt);
        }
    }
}

#[allow(clippy::type_complexity)]
fn key_events<M>(
    app: App,
    mut key_events: MessageReader<KeyboardInput>,
    user_fns: Query<&WindowUserFunctions<M>>,
    mut model: ResMut<ModelHolder<M>>,
) where
    M: 'static + Send + Sync,
{
    for evt in key_events.read() {
        if let Ok(user_fns) = user_fns.get(evt.window) {
            match evt.state {
                ButtonState::Pressed => {
                    if let Some(f) = user_fns.key_pressed {
                        app.set_current_view(Some(evt.window));
                        f(&app, &mut model, evt.key_code);
                    }
                }
                ButtonState::Released => {
                    if let Some(f) = user_fns.key_released {
                        app.set_current_view(Some(evt.window));
                        f(&app, &mut model, evt.key_code);
                    }
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn received_char_events<M>(
    app: App,
    mut received_char_events: MessageReader<KeyboardInput>,
    user_fns: Query<&WindowUserFunctions<M>>,
    mut model: ResMut<ModelHolder<M>>,
) where
    M: 'static + Send + Sync,
{
    for evt in received_char_events.read() {
        if let Ok(user_fns) = user_fns.get(evt.window) {
            if let Some(f) = user_fns.received_character {
                app.set_current_view(Some(evt.window));
                if let Key::Character(char) = &evt.logical_key {
                    for char in char.chars() {
                        f(&app, &mut model, char);
                    }
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn cursor_moved_events<M>(
    app: App,
    mut cursor_moved_events: MessageReader<CursorMoved>,
    user_fns: Query<&WindowUserFunctions<M>>,
    mut model: ResMut<ModelHolder<M>>,
) where
    M: 'static + Send + Sync,
{
    for evt in cursor_moved_events.read() {
        if let Ok(user_fns) = user_fns.get(evt.window) {
            if let Some(f) = user_fns.mouse_moved {
                app.set_current_view(Some(evt.window));
                f(&app, &mut model, evt.position);
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn mouse_button_events<M>(
    app: App,
    mut mouse_button_events: MessageReader<MouseButtonInput>,
    user_fns: Query<&WindowUserFunctions<M>>,
    mut model: ResMut<ModelHolder<M>>,
) where
    M: 'static + Send + Sync,
{
    for evt in mouse_button_events.read() {
        if let Ok(user_fns) = user_fns.get(evt.window) {
            match evt.state {
                ButtonState::Pressed => {
                    if let Some(f) = user_fns.mouse_pressed {
                        app.set_current_view(Some(evt.window));
                        f(&app, &mut model, evt.button);
                    }
                }
                ButtonState::Released => {
                    if let Some(f) = user_fns.mouse_released {
                        app.set_current_view(Some(evt.window));
                        f(&app, &mut model, evt.button);
                    }
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn cursor_entered_events<M>(
    app: App,
    mut cursor_entered_events: MessageReader<CursorEntered>,
    user_fns: Query<&WindowUserFunctions<M>>,
    mut model: ResMut<ModelHolder<M>>,
) where
    M: 'static + Send + Sync,
{
    for evt in cursor_entered_events.read() {
        if let Ok(user_fns) = user_fns.get(evt.window) {
            if let Some(f) = user_fns.mouse_entered {
                app.set_current_view(Some(evt.window));
                f(&app, &mut model);
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn cursor_left_events<M>(
    app: App,
    mut cursor_left_events: MessageReader<CursorLeft>,
    user_fns: Query<&WindowUserFunctions<M>>,
    mut model: ResMut<ModelHolder<M>>,
) where
    M: 'static + Send + Sync,
{
    for evt in cursor_left_events.read() {
        if let Ok(user_fns) = user_fns.get(evt.window) {
            if let Some(f) = user_fns.mouse_exited {
                app.set_current_view(Some(evt.window));
                f(&app, &mut model);
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn mouse_wheel_events<M>(
    app: App,
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    user_fns: Query<&WindowUserFunctions<M>>,
    mut model: ResMut<ModelHolder<M>>,
) where
    M: 'static + Send + Sync,
{
    for evt in mouse_wheel_events.read() {
        if let Ok(user_fns) = user_fns.get(evt.window) {
            if let Some(f) = user_fns.mouse_wheel {
                app.set_current_view(Some(evt.window));
                f(&app, &mut model, *evt);
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn window_moved_events<M>(
    app: App,
    mut window_moved_events: MessageReader<WindowMoved>,
    user_fns: Query<&WindowUserFunctions<M>>,
    mut model: ResMut<ModelHolder<M>>,
) where
    M: 'static + Send + Sync,
{
    for evt in window_moved_events.read() {
        if let Ok(user_fns) = user_fns.get(evt.window) {
            if let Some(f) = user_fns.moved {
                app.set_current_view(Some(evt.window));
                f(&app, &mut model, evt.position);
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn window_resized_events<M>(
    app: App,
    mut window_resized_events: MessageReader<WindowResized>,
    user_fns: Query<&WindowUserFunctions<M>>,
    mut model: ResMut<ModelHolder<M>>,
) where
    M: 'static + Send + Sync,
{
    for evt in window_resized_events.read() {
        if let Ok(user_fns) = user_fns.get(evt.window) {
            if let Some(f) = user_fns.resized {
                app.set_current_view(Some(evt.window));
                f(&app, &mut model, Vec2::new(evt.width, evt.height));
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn touch_events<M>(
    app: App,
    mut touch_events: MessageReader<TouchInput>,
    user_fns: Query<&WindowUserFunctions<M>>,
    mut model: ResMut<ModelHolder<M>>,
) where
    M: 'static + Send + Sync,
{
    for evt in touch_events.read() {
        if let Ok(user_fns) = user_fns.get(evt.window) {
            if let Some(f) = user_fns.touch {
                app.set_current_view(Some(evt.window));
                f(&app, &mut model, *evt);
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn file_drop_events<M>(
    app: App,
    mut file_drop_events: MessageReader<FileDragAndDrop>,
    user_fns: Query<&WindowUserFunctions<M>>,
    mut model: ResMut<ModelHolder<M>>,
) where
    M: 'static + Send + Sync,
{
    for evt in file_drop_events.read() {
        match evt {
            FileDragAndDrop::DroppedFile { window, path_buf } => {
                if let Ok(user_fns) = user_fns.get(*window) {
                    if let Some(f) = user_fns.dropped_file {
                        app.set_current_view(Some(*window));
                        f(&app, &mut model, path_buf.clone());
                    }
                }
            }
            FileDragAndDrop::HoveredFile { window, path_buf } => {
                if let Ok(user_fns) = user_fns.get(*window) {
                    if let Some(f) = user_fns.hovered_file {
                        app.set_current_view(Some(*window));
                        f(&app, &mut model, path_buf.clone());
                    }
                }
            }
            FileDragAndDrop::HoveredFileCanceled { window } => {
                if let Ok(user_fns) = user_fns.get(*window) {
                    if let Some(f) = user_fns.hovered_file_cancelled {
                        app.set_current_view(Some(*window));
                        f(&app, &mut model);
                    }
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn window_focus_events<M>(
    app: App,
    mut window_focus_events: MessageReader<WindowFocused>,
    user_fns: Query<&WindowUserFunctions<M>>,
    mut model: ResMut<ModelHolder<M>>,
) where
    M: 'static + Send + Sync,
{
    for evt in window_focus_events.read() {
        if let Ok(user_fns) = user_fns.get(evt.window) {
            if evt.focused {
                if let Some(f) = user_fns.focused {
                    app.set_current_view(Some(evt.window));
                    f(&app, &mut model);
                }
            } else if let Some(f) = user_fns.unfocused {
                app.set_current_view(Some(evt.window));
                f(&app, &mut model);
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn window_closed_events<M>(
    app: App,
    mut window_closed_events: MessageReader<WindowClosed>,
    user_fns: Query<&WindowUserFunctions<M>>,
    mut model: ResMut<ModelHolder<M>>,
) where
    M: 'static + Send + Sync,
{
    for evt in window_closed_events.read() {
        if let Ok(user_fns) = user_fns.get(evt.window) {
            if let Some(f) = user_fns.closed {
                app.set_current_view(Some(evt.window));
                f(&app, &mut model);
            }
        }
    }
}

fn last<M>(world: &mut World, exit_state: &mut SystemState<MessageReader<AppExit>>)
where
    M: 'static + Send + Sync,
{
    let should_exit = {
        let exit_events = exit_state
            .get(world)
            .expect("failed to fetch system params");
        !exit_events.is_empty()
    };
    if !should_exit {
        return;
    }

    let exit_fn = world.resource::<ExitFnRes<M>>().0;
    let Some(model) = world.remove_resource::<ModelHolder<M>>() else {
        return;
    };

    if let Some(exit_fn) = exit_fn {
        let mut app_state = SystemState::<App>::new(world);
        {
            let app = app_state
                .get(world)
                .expect("failed to fetch system params");
            exit_fn(&app, model.0);
        }
        // Flush any commands the exit fn queued via the `App` (e.g. a final screenshot).
        app_state.apply(world);
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
