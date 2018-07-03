//! An open-source creative-coding toolkit for Rust.
//!
//! [**Nannou**](http://nannou.cc) is a collection of code aimed at making it easy for artists to
//! express themselves with simple, fast, reliable, portable code. Whether working on a 12-month
//! laser installation or a 5 minute sketch, this framework aims to give artists easy access to the
//! tools they need.
//!
//! If you're new to nannou, we recommend checking out [the
//! examples](https://github.com/nannou-org/nannou/tree/master/examples) to get an idea of how
//! nannou applications are structured and how the API works.

pub extern crate daggy;
pub extern crate find_folder;
#[macro_use]
pub extern crate glium;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate toml;

use event::LoopEvent;
use glium::glutin;
use std::cell::RefCell;
use std::sync::atomic;
use std::time::Instant;

pub use self::event::Event;
pub use self::frame::Frame;
pub use self::ui::Ui;
pub use app::{App, LoopMode};
pub use draw::Draw;
pub use glium::glutin::{
    ContextBuilder, CursorState, ElementState, MonitorId, MouseCursor, VirtualKeyCode,
    WindowBuilder, WindowEvent,
};

pub mod app;
pub mod audio;
pub mod color;
pub mod draw;
pub mod ease;
pub mod event;
mod frame;
pub mod geom;
pub mod image;
pub mod io;
pub mod math;
pub mod mesh;
pub mod noise;
pub mod osc;
pub mod prelude;
pub mod rand;
pub mod state;
pub mod ui;
pub mod window;

pub type ModelFn<Model> = fn(&App) -> Model;
pub type EventFn<Model, Event> = fn(&App, Model, Event) -> Model;
pub type ViewFn<Model> = fn(&App, &Model, Frame) -> Frame;
pub type SimpleViewFn = fn(&App, Frame) -> Frame;
pub type ExitFn<Model> = fn(&App, Model);

/// The **App**'s view function.
pub enum View<Model = ()> {
    /// A **Full** view function allows for viewing the user's model.
    Full(ViewFn<Model>),
    /// A **Simple** view function does not require a user **Model**. Simpler to get started.
    Simple(SimpleViewFn),
}

/// Begin building a nannou App.
///
/// Every nannou App must have `model`, `event` and `draw` functions.
///
/// An `exit` function can be optionally specified using the `exit` builder method.
pub fn app<M, E>(model: ModelFn<M>, event: EventFn<M, E>, view: ViewFn<M>) -> Builder<M, E>
where
    E: LoopEvent,
{
    let view = view.into();
    app_inner(model, event, view)
}

fn app_inner<M, E>(model: ModelFn<M>, event: EventFn<M, E>, view: View<M>) -> Builder<M, E>
where
    E: LoopEvent,
{
    let exit = None;
    let create_default_window = false;
    Builder {
        model,
        event,
        view,
        exit,
        create_default_window,
    }
}

/// Build a simple nannou `App` with a default window and a view function.
///
/// This is the same as calling `app` and providing `model` and `event` functions that do nothing.
pub fn view(view: SimpleViewFn) {
    fn default_model(_app: &App) -> () {}
    fn default_event(_app: &App, _model: (), _event: Event) -> () {}
    let view: View<()> = view.into();
    app_inner(default_model, default_event, view).run_window()
}

/// A nannou application builder.
pub struct Builder<M, E> {
    model: ModelFn<M>,
    event: EventFn<M, E>,
    view: View<M>,
    exit: Option<ExitFn<M>>,
    create_default_window: bool,
}

impl<M, E> Builder<M, E>
where
    E: LoopEvent,
{
    /// Specify an `exit` function to be called when the application exits.
    ///
    /// The exit function gives ownership of the model back to the user.
    pub fn exit(mut self, exit: ExitFn<M>) -> Self {
        self.exit = Some(exit);
        self
    }

    /// Creates and runs the nannou `App` with a default window.
    pub fn run_window(mut self) {
        self.create_default_window = true;
        self.run()
    }

    /// Creates and runs the nannou `App`.
    pub fn run(self) {
        let Builder {
            model,
            event,
            view,
            exit,
            create_default_window,
        } = self;

        // Start the glutin window event loop.
        let events_loop = glutin::EventsLoop::new();

        // Initialise the app.
        let app = App::new(events_loop);

        // Create the default window if necessary
        if create_default_window {
            let window_id = app
                .new_window()
                .build()
                .expect("could not build default app window");
            *app.focused_window.borrow_mut() = Some(window_id);
        }

        // Call the user's model function.
        let model = model(&app);

        // If there is not yet some default window in "focus" check to see if one has been created.
        if app.focused_window.borrow().is_none() {
            if let Some(id) = app.windows.borrow().keys().next() {
                *app.focused_window.borrow_mut() = Some(id.clone());
            }
        }

        run_loop(app, model, event, view, exit)
    }
}

/// A simple function for creating and running a nannou `App` with a default window!
///
/// Calling this is just like calling `nannou::app(model, event, view).run_window()`.
pub fn run<M, E>(model: ModelFn<M>, event: EventFn<M, E>, view: ViewFn<M>)
where
    E: LoopEvent,
{
    app(model, event, view).run_window()
}

fn run_loop<M, E>(
    mut app: App,
    mut model: M,
    event_fn: EventFn<M, E>,
    view: View<M>,
    exit_fn: Option<ExitFn<M>>,
) where
    E: LoopEvent,
{
    // A function to re-use when drawing for each of the loop modes.
    fn draw<M>(app: &App, model: &M, view: &View<M>) -> Result<(), glium::SwapBuffersError> {
        // Draw the state of the model to the screen.
        let gl_frames = app
            .windows
            .borrow()
            .iter()
            .map(|(&id, window)| {
                let gl_frame = RefCell::new(frame::GlFrame::new(window.display.draw()));
                (id, gl_frame)
            })
            .collect();
        // TODO: This currently passes the *focused* window but should pass the *main* one.
        let undrawn_frame = frame::new(gl_frames, *app.focused_window.borrow());
        let frame = match *view {
            View::Full(view_fn) => view_fn(&app, &model, undrawn_frame),
            View::Simple(view_fn) => view_fn(&app, undrawn_frame),
        };
        frame::finish(frame)
    }

    // Whether or not the given event should toggle fullscreen.
    fn should_toggle_fullscreen(glutin_event: &glutin::WindowEvent) -> bool {
        let input = match *glutin_event {
            glutin::WindowEvent::KeyboardInput { ref input, .. } => match input.state {
                event::ElementState::Pressed => input,
                _ => return false,
            },
            _ => return false,
        };

        let key = match input.virtual_keycode {
            None => return false,
            Some(k) => k,
        };
        let mods = &input.modifiers;

        // On linux, check for the F11 key (with no modifiers down).
        //
        // TODO: Somehow add special case for KDE?
        if cfg!(target_os = "linux") {
            if !mods.logo && !mods.shift && !mods.alt && !mods.ctrl {
                if let VirtualKeyCode::F11 = key {
                    return true;
                }
            }

        // On macos and windows check for the logo key plus `f` with no other modifiers.
        } else if cfg!(target_os = "macos") || cfg!(target_os = "windows") {
            if mods.logo && !mods.shift && !mods.alt && !mods.ctrl {
                if let VirtualKeyCode::F = key {
                    return true;
                }
            }
        }

        false
    }

    // A function to simplify the creation of an `Update` event.
    //
    // Also updates the given `last_update` instant to `Instant::now()`.
    fn update_event(loop_start: Instant, last_update: &mut Instant) -> event::Update {
        // Emit an update event.
        let now = Instant::now();
        let since_last = now.duration_since(*last_update).into();
        let since_start = now.duration_since(loop_start).into();
        let update = event::Update {
            since_last,
            since_start,
        };
        *last_update = now;
        update
    };

    // Event handling boilerplate shared between the `Rate` and `Wait` loop modes.
    //
    // 1. Checks for exit on escape.
    // 2. Removes closed windows from app.
    // 3. Emits event via `event_fn`.
    // 4. Returns whether or not we should break from the loop.
    fn process_and_emit_glutin_event<M, E>(
        app: &mut App,
        mut model: M,
        event_fn: EventFn<M, E>,
        glutin_event: glutin::Event,
    ) -> (M, bool)
    where
        E: LoopEvent,
    {
        // Inspect the event to see if it would require closing the App.
        let mut exit_on_escape = false;
        if let glutin::Event::WindowEvent {
            window_id,
            ref event,
        } = glutin_event
        {
            // If we should exit the app on escape, check for the escape key.
            if app.exit_on_escape() {
                if let glutin::WindowEvent::KeyboardInput { input, .. } = *event {
                    if let Some(VirtualKeyCode::Escape) = input.virtual_keycode {
                        exit_on_escape = true;
                    }
                }
            }

            // If a window was closed, remove it from the display map.
            if let glutin::WindowEvent::Closed = *event {
                app.windows.borrow_mut().remove(&window_id);
            } else {
                // Get the size of the screen for translating coords and dimensions.
                let (win_w_px, win_h_px, hidpi_factor) = match app.window(window_id) {
                    Some(win) => {
                        // If we should toggle fullscreen for this window, do so.
                        if app.fullscreen_on_shortcut() {
                            if should_toggle_fullscreen(event) {
                                if win.is_fullscreen() {
                                    win.set_fullscreen(None);
                                } else {
                                    let monitor = win.current_monitor();
                                    win.set_fullscreen(Some(monitor));
                                }
                            }
                        }

                        let (w_px, h_px) = win.inner_size_pixels();
                        let hidpi_factor = win.hidpi_factor();
                        (w_px, h_px, hidpi_factor)
                    }
                    None => (0, 0, 1.0),
                };

                // Translate the coordinates from top-left-origin-with-y-down to centre-origin-with-y-up.
                //
                // winit produces input events in pixels, so these positions need to be divided by the
                // width and height of the window in order to be DPI agnostic.
                let tx = |x: geom::scalar::Default| {
                    (x - win_w_px as geom::scalar::Default / 2.0) / hidpi_factor
                };
                let ty = |y: geom::scalar::Default| {
                    -((y - win_h_px as geom::scalar::Default / 2.0) / hidpi_factor)
                };

                // If the window ID has changed, ensure the dimensions are up to date.
                if *app.focused_window.borrow() != Some(window_id) {
                    if app.window(window_id).is_some() {
                        *app.focused_window.borrow_mut() = Some(window_id);
                    }
                }

                // Check for events that would update either mouse, keyboard or window state.
                match *event {
                    glutin::WindowEvent::CursorMoved {
                        position: (x, y), ..
                    } => {
                        let x = tx(x as _);
                        let y = ty(y as _);
                        app.mouse.x = x;
                        app.mouse.y = y;
                        app.mouse.window = Some(window_id);
                    }

                    glutin::WindowEvent::MouseInput { state, button, .. } => {
                        match state {
                            event::ElementState::Pressed => {
                                let p = app.mouse.position();
                                app.mouse.buttons.press(button, p);
                            }
                            event::ElementState::Released => {
                                app.mouse.buttons.release(button);
                            }
                        }
                        app.mouse.window = Some(window_id);
                    }

                    glutin::WindowEvent::KeyboardInput { input, .. } => {
                        app.keys.mods = input.modifiers;
                        if let Some(key) = input.virtual_keycode {
                            match input.state {
                                event::ElementState::Pressed => {
                                    app.keys.down.keys.insert(key);
                                }
                                event::ElementState::Released => {
                                    app.keys.down.keys.remove(&key);
                                }
                            }
                        }
                    }

                    _ => (),
                }

                // See if the event could be interpreted as a `ui::Input`. If so, submit it to the
                // `Ui`s associated with this window.
                if let Some(window) = app.windows.borrow().get(&window_id) {
                    if let Some(input) = ui::glutin_window_event_to_input(event.clone(), window) {
                        if let Some(handles) = app.ui.windows.borrow().get(&window_id) {
                            for handle in handles {
                                if let Some(ref tx) = handle.input_tx {
                                    tx.try_send(input.clone()).ok();
                                }
                            }
                        }
                    }
                }
            }
        }

        // If the glutin::Event could be interpreted as some event `E`, use it to update the model.
        if let Some(event) = E::from_glutin_event(glutin_event, app) {
            model = event_fn(&app, model, event);
        }

        // If exit on escape was triggered, we're done.
        let exit = if exit_on_escape || app.windows.borrow().is_empty() {
            true
        } else {
            false
        };

        (model, exit)
    }

    let loop_start = Instant::now();

    // A vec for collecting events.
    let mut glutin_events = Vec::new();

    // Begin looping.
    let mut last_update = loop_start;
    let mut last_loop_end = loop_start;
    let mut updates_remaining = LoopMode::DEFAULT_UPDATES_FOLLOWING_EVENT;
    let mut loop_mode = None;

    // Begin looping.
    'main: loop {
        // See if the loop mode has changed.
        let new_loop_mode = app.loop_mode();
        let loop_mode_has_changed = loop_mode != Some(new_loop_mode);
        loop_mode = Some(new_loop_mode);

        // The kind of iteration to perform will depend on the loop mode.
        match loop_mode {
            Some(LoopMode::Rate { update_interval }) => {
                // If the loop mode has changed since the last iteration, initialise the necessary
                // `Rate` state.
                if loop_mode_has_changed {}

                // First handle any pending window events.
                app.events_loop
                    .poll_events(|event| glutin_events.push(event));
                for glutin_event in glutin_events.drain(..) {
                    let (new_model, exit) =
                        process_and_emit_glutin_event(&mut app, model, event_fn, glutin_event);
                    model = new_model;
                    if exit {
                        break 'main;
                    }
                }

                // Update the app's durations.
                let now = Instant::now();
                let since_last = now.duration_since(last_update).into();
                let since_start = now.duration_since(loop_start).into();
                app.duration.since_start = since_start;
                app.duration.since_prev_update = since_last;
                app.time = app.duration.since_start.secs() as _;

                // Emit an update event.
                let event = E::from(update_event(loop_start, &mut last_update));
                model = event_fn(&app, model, event);

                // Draw the state of the model to the screen.
                draw(&app, &model, &view).unwrap();
                app.elapsed_frames += 1;

                // Sleep if there's still some time left within the interval.
                let now = Instant::now();
                let since_last_loop_end = now.duration_since(last_loop_end);
                if since_last_loop_end < update_interval {
                    std::thread::sleep(update_interval - since_last_loop_end);
                }
                last_loop_end = Instant::now();
            }

            Some(LoopMode::Wait {
                updates_following_event,
                update_interval,
            }) => {
                // If the loop mode has changed, initialise the necessary `Rate` state.
                if loop_mode_has_changed {
                    updates_remaining = updates_following_event;
                }

                // First collect any pending window events.
                app.events_loop
                    .poll_events(|event| glutin_events.push(event));

                // If there are no events and the `Ui` does not need updating,
                // wait for the next event.
                if glutin_events.is_empty() && updates_remaining == 0 {
                    let events_loop_is_asleep = app.events_loop_is_asleep.clone();
                    events_loop_is_asleep.store(true, atomic::Ordering::Relaxed);
                    app.events_loop.run_forever(|event| {
                        events_loop_is_asleep.store(false, atomic::Ordering::Relaxed);
                        glutin_events.push(event);
                        glium::glutin::ControlFlow::Break
                    });
                }

                // If there are some glutin events to process, reset the updates-remaining count.
                if !glutin_events.is_empty() {
                    updates_remaining = updates_following_event;
                }

                for glutin_event in glutin_events.drain(..) {
                    let (new_model, exit) =
                        process_and_emit_glutin_event(&mut app, model, event_fn, glutin_event);
                    model = new_model;
                    if exit {
                        break 'main;
                    }
                }

                // Update the app's durations.
                let now = Instant::now();
                let since_last = now.duration_since(last_update).into();
                let since_start = now.duration_since(loop_start).into();
                app.duration.since_start = since_start;
                app.duration.since_prev_update = since_last;
                app.time = app.duration.since_start.secs() as _;

                // Emit an update event.
                let event = E::from(update_event(loop_start, &mut last_update));
                model = event_fn(&app, model, event);
                updates_remaining -= 1;

                // Draw the state of the model to the screen.
                draw(&app, &model, &view).unwrap();
                app.elapsed_frames += 1;

                // Sleep if there's still some time left within the interval.
                let now = Instant::now();
                let since_last_loop_end = now.duration_since(last_loop_end);
                if since_last_loop_end < update_interval {
                    std::thread::sleep(update_interval - since_last_loop_end);
                }
                last_loop_end = Instant::now();
            }

            // Loop mode is always `Some` after the beginning of the `'main` loop.
            None => unreachable!(),
        }
    }

    // Emit an application exit event.
    if let Some(exit_fn) = exit_fn {
        exit_fn(&app, model);
    }
}

impl<M> From<ViewFn<M>> for View<M> {
    fn from(v: ViewFn<M>) -> Self {
        View::Full(v)
    }
}

impl From<SimpleViewFn> for View<()> {
    fn from(v: SimpleViewFn) -> Self {
        View::Simple(v)
    }
}
