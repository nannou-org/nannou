pub extern crate find_folder;
pub extern crate glium;
pub extern crate rand;

use event::LoopEvent;
use glium::glutin;
use std::cell::RefCell;
use std::time::Instant;

pub use app::{App, LoopMode};
pub use glium::glutin::{ContextBuilder, CursorState, ElementState, MonitorId, MouseCursor,
                        WindowBuilder, WindowEvent, VirtualKeyCode};
pub use self::event::Event;
pub use self::frame::Frame;
pub use self::ui::Ui;

pub mod app;
pub mod audio;
pub mod color;
pub mod ease;
pub mod event;
mod frame;
pub mod window;
pub mod image;
pub mod math;
pub mod osc;
pub mod prelude;
pub mod ui;

pub type ModelFn<Model> = fn(&App) -> Model;
pub type UpdateFn<Model, Event> = fn(&App, Model, Event) -> Model;
pub type DrawFn<Model> = fn(&App, &Model, Frame) -> Frame;
pub type ExitFn<Model> = fn(&App, Model);

/// Begin building a nannou App.
///
/// Every nannou App must have `model`, `update` and `draw` functions.
///
/// An `exit` function can be optionally specified using the `exit` builder method.
pub fn app<M, E>(model: ModelFn<M>, update: UpdateFn<M, E>, draw: DrawFn<M>) -> Builder<M, E>
    where E: LoopEvent,
{
    let exit = None;
    Builder { model, update, draw, exit }
}

/// A nannou application builder.
pub struct Builder<M, E> {
    model: ModelFn<M>,
    update: UpdateFn<M, E>,
    draw: DrawFn<M>,
    exit: Option<ExitFn<M>>,
}

impl<M, E> Builder<M, E>
    where E: LoopEvent,
{
    /// Specify an `exit` function to be called when the application exits.
    ///
    /// The exit function gives ownership of the model back to the user.
    pub fn exit(mut self, exit: ExitFn<M>) -> Self {
        self.exit = Some(exit);
        self
    }

    /// Creates and runs the nannou `App`.
    pub fn run(self) {
        let Builder { model, update, draw, exit } = self;
        let events_loop = glutin::EventsLoop::new();
        let app = App::new(events_loop);
        let model = model(&app);
        run_loop(app, model, update, draw, exit)
    }
}

/// A simple function for creating and running a nannou `App`!
///
/// Calling this is just like calling `nannou::app(model, update, draw).run()`.
pub fn run<M, E>(model: ModelFn<M>, update: UpdateFn<M, E>, draw: DrawFn<M>)
    where E: LoopEvent,
{
    app(model, update, draw).run()
}

fn run_loop<M, E>(
    mut app: App,
    mut model: M,
    update_fn: UpdateFn<M, E>,
    draw_fn: DrawFn<M>,
    exit_fn: Option<ExitFn<M>>,
)
where
    E: LoopEvent,
{
    // A function to re-use when drawing for each of the loop modes.
    fn draw<M>(app: &App, model: &M, draw_fn: DrawFn<M>) -> Result<(), glium::SwapBuffersError> {
        // Draw the state of the model to the screen.
        let gl_frames = app.windows
            .borrow()
            .iter()
            .map(|(&id, window)| {
                let gl_frame = RefCell::new(frame::GlFrame::new(window.display.draw()));
                (id, gl_frame)
            })
            .collect();
        let frame = draw_fn(&app, &model, frame::new(gl_frames));
        frame::finish(frame)
    }

    // A function to simplify the creation of an `Update` event.
    //
    // Also updates the given `last_update` instant to `Instant::now()`.
    fn update_event(loop_start: Instant, last_update: &mut Instant) -> event::Update {
        // Emit an update event.
        let now = Instant::now();
        let since_last = now.duration_since(*last_update);
        let since_start = now.duration_since(loop_start);
        let update = event::Update { since_last, since_start };
        *last_update = now;
        update
    };

    // Event handling boilerplate shared between the `Rate` and `Wait` loop modes.
    //
    // 1. Checks for exit on escape.
    // 2. Removes closed windows from app.
    // 3. Emits event via `update_fn`.
    // 4. Returns whether or not we should break from the loop.
    fn process_and_emit_glutin_event<M, E>(
        app: &App,
        mut model: M,
        update_fn: UpdateFn<M, E>,
        glutin_event: glutin::Event,
    ) -> (M, bool)
    where
        E: LoopEvent,
    {
        // Inspect the event to see if it would require closing the App.
        let mut exit_on_escape = false;
        if let glutin::Event::WindowEvent { window_id, ref event } = glutin_event {

            // If we should exit the app on escape, check for the escape key.
            if app.exit_on_escape.get() {
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
            model = update_fn(&app, model, event);
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
                if loop_mode_has_changed {
                }

                // First handle any pending window events.
                app.events_loop.poll_events(|event| glutin_events.push(event));
                for glutin_event in glutin_events.drain(..) {
                    let (new_model, exit) = process_and_emit_glutin_event(&app, model, update_fn, glutin_event);
                    model = new_model;
                    if exit {
                        break 'main;
                    }
                }

                // Emit an update event.
                let event = E::from(update_event(loop_start, &mut last_update));
                model = update_fn(&app, model, event);

                // Draw the state of the model to the screen.
                draw(&app, &model, draw_fn).unwrap();

                // Sleep if there's still some time left within the interval.
                let now = Instant::now();
                let since_last_loop_end = now.duration_since(last_loop_end);
                if since_last_loop_end < update_interval {
                    std::thread::sleep(update_interval - since_last_loop_end);
                }
                last_loop_end = Instant::now();
            },

            Some(LoopMode::Wait { updates_following_event, update_interval }) => {
                // If the loop mode has changed, initialise the necessary `Rate` state.
                if loop_mode_has_changed {
                    updates_remaining = updates_following_event;
                }

                // First collect any pending window events.
                app.events_loop.poll_events(|event| glutin_events.push(event));

                // If there are no events and the `Ui` does not need updating,
                // wait for the next event.
                if glutin_events.is_empty() && updates_remaining == 0 {
                    app.events_loop.run_forever(|event| {
                        glutin_events.push(event);
                        glium::glutin::ControlFlow::Break
                    });
                }

                // If there are some glutin events to process, reset the updates-remaining count.
                if !glutin_events.is_empty() {
                    updates_remaining = updates_following_event;
                }

                for glutin_event in glutin_events.drain(..) {
                    let (new_model, exit) = process_and_emit_glutin_event(&app, model, update_fn, glutin_event);
                    model = new_model;
                    if exit {
                        break 'main;
                    }
                }

                // Emit an update event.
                let event = E::from(update_event(loop_start, &mut last_update));
                model = update_fn(&app, model, event);
                updates_remaining -= 1;

                // Draw the state of the model to the screen.
                draw(&app, &model, draw_fn).unwrap();

                // Sleep if there's still some time left within the interval.
                let now = Instant::now();
                let since_last_loop_end = now.duration_since(last_loop_end);
                if since_last_loop_end < update_interval {
                    std::thread::sleep(update_interval - since_last_loop_end);
                }
                last_loop_end = Instant::now();
            },

            // Loop mode is always `Some` after the beginning of the `'main` loop.
            None => unreachable!(),
        }
    }

    // Emit an application exit event.
    if let Some(exit_fn) = exit_fn {
        exit_fn(&app, model);
    }
}
