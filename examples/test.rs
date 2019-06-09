#[macro_use]
extern crate conrod;
extern crate core;
extern crate find_folder;
extern crate rand;
extern crate timeline;

use conrod::backend::glium::glium::{self, Surface};
use core::{time, BarIterator, Note};
use std::iter::once;
use timeline::track;

/// Demonstration app data.
struct App {
    playhead_secs: f64,
    bars: Vec<core::Bar>,
    notes: Vec<Note>,
    tempo_envelope: track::automation::numeric::Envelope<f32>,
    octave_envelope: track::automation::numeric::Envelope<i32>,
    toggle_envelope: track::automation::toggle::Envelope,
    bang_envelope: track::automation::bang::Envelope,
}

const ONE_SECOND_MS: time::calc::Ms = 1_000.0;
const BPM: time::calc::Bpm = 140.0;

// Create all of our unique `WidgetId`s with the `widget_ids!` macro.
widget_ids! {
    struct Ids {
        window,
        ruler,
        timeline,
    }
}

fn main() {
    const WIDTH: u32 = 800;
    const HEIGHT: u32 = 600;

    // Build the window.
    let mut events_loop = glium::glutin::EventsLoop::new();
    let window = glium::glutin::WindowBuilder::new()
        .with_dimensions(WIDTH, HEIGHT)
        .with_title("Timeline Demo");
    let context = glium::glutin::ContextBuilder::new().with_vsync(true);
    let display = glium::Display::new(window, context, &events_loop).unwrap();

    // construct our `Ui`.
    let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let assets = find_folder::Search::KidsThenParents(3, 5)
        .for_folder("assets")
        .unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();

    let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();

    // The image map describing each of our widget->image mappings (in our case, none).
    let image_map = conrod::image::Map::<glium::texture::Texture2d>::new();

    // A short-hand constructor for a Bar.
    fn new_bar(top: u16, bottom: u16) -> core::Bar {
        core::Bar {
            time_sig: time::TimeSig {
                top: top,
                bottom: bottom,
            },
            maybe_swing: None,
        }
    }

    // Construct our initial App data.
    let mut app =
        {
            use timeline::track::automation::{
                BangValue as Bang, Envelope, Point, ToggleValue as Toggle,
            };

            let bars = vec![
                new_bar(4, 4),
                new_bar(4, 4),
                new_bar(6, 8),
                new_bar(6, 8),
                new_bar(4, 4),
                new_bar(4, 4),
                new_bar(7, 8),
                new_bar(7, 8),
            ];

            let notes = bars
                .iter()
                .cloned()
                .periods()
                .enumerate()
                .map(|(i, period)| Note {
                    period: period,
                    pitch: core::pitch::Step((24 + (i * 5) % 12) as f32).to_letter_octave(),
                    velocity: 1.0,
                })
                .collect();

            let tempo_envelope = {
                let start = Point {
                    ticks: time::Ticks(0),
                    value: 20.0,
                };
                let points = bars
                    .iter()
                    .cloned()
                    .periods()
                    .enumerate()
                    .map(|(i, period)| Point {
                        ticks: period.end(),
                        value: 20.0 + (i + 1) as f32 * 60.0 % 220.0,
                    });
                Envelope::from_points(once(start).chain(points), 20.0, 240.0)
            };

            let octave_envelope =
                {
                    let start = Point {
                        ticks: time::Ticks(0),
                        value: 0,
                    };
                    let points = bars.iter().cloned().with_starts().enumerate().flat_map(
                        |(i, (bar, start))| {
                            bar.division_periods(time::Division::Beat).enumerate().map(
                                move |(j, period)| Point {
                                    ticks: start + period.end(),
                                    value: 1 + ((i as i32 + j as i32) * 3) % 12,
                                },
                            )
                        },
                    );
                    Envelope::from_points(once(start).chain(points), 0, 12)
                };

            let toggle_envelope = {
                let start = Point {
                    ticks: time::Ticks(0),
                    value: Toggle(rand::random()),
                };
                let points = bars.iter().cloned().periods().map(|period| Point {
                    ticks: period.end(),
                    value: Toggle(rand::random()),
                });
                Envelope::from_points(once(start).chain(points), Toggle(false), Toggle(true))
            };

            let bang_envelope = {
                let points = bars.iter().cloned().periods().map(|period| Point {
                    ticks: period.start(),
                    value: Bang,
                });
                Envelope::from_points(points, Bang, Bang)
            };

            App {
                playhead_secs: 0.0,
                bars: bars,
                notes: notes,
                tempo_envelope: tempo_envelope,
                octave_envelope: octave_envelope,
                toggle_envelope: toggle_envelope,
                bang_envelope: bang_envelope,
            }
        };

    let ids = Ids::new(ui.widget_id_generator());

    // Draws the given `primitives` to the given `Display`.
    fn draw(
        display: &glium::Display,
        renderer: &mut conrod::backend::glium::Renderer,
        image_map: &conrod::image::Map<glium::Texture2d>,
        primitives: conrod::render::Primitives,
    ) {
        renderer.fill(display, primitives, &image_map);
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        renderer.draw(display, &mut target, &image_map).unwrap();
        target.finish().unwrap();
    }

    let mut closed = false;
    while !closed {
        // Poll for events.
        events_loop.poll_events(|event| {
            // Use the `glutin` backend feature to convert the glutin event to a conrod one.
            if let Some(event) = conrod::backend::winit::convert_event(event.clone(), &display) {
                ui.handle_event(event);
            }

            match event {
                glium::glutin::Event::WindowEvent { event, .. } => match event {
                    // Update the GUI and redraw on Resized because macOS will block.
                    glium::glutin::WindowEvent::Resized(..) => {
                        // Instantiate a GUI demonstrating the timeline.
                        {
                            let mut ui = ui.set_widgets();
                            set_widgets(&mut ui, &ids, &mut app);
                        }
                        let primitives = ui.draw();
                        draw(&display, &mut renderer, &image_map, primitives);
                    }

                    // Break from the loop upon `Escape`.
                    glium::glutin::WindowEvent::Closed
                    | glium::glutin::WindowEvent::KeyboardInput {
                        input:
                            glium::glutin::KeyboardInput {
                                virtual_keycode: Some(glium::glutin::VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => closed = true,
                    _ => (),
                },
                _ => (),
            }
        });

        // Instantiate a GUI demonstrating the timeline.
        {
            let mut ui = ui.set_widgets();
            set_widgets(&mut ui, &ids, &mut app);
        }

        // Draw the `Ui`.
        if let Some(primitives) = ui.draw_if_changed() {
            draw(&display, &mut renderer, &image_map, primitives);
        }

        // Update the playhead. This is just a rough estimate based on the sleep duration.
        let total_duration_ticks = app.bars.iter().cloned().total_duration();
        let total_duration_ms = total_duration_ticks.ms(BPM, core::PPQN);
        let total_duration_secs = total_duration_ms / ONE_SECOND_MS;
        app.playhead_secs = (app.playhead_secs + 1.0 / 60.0) % total_duration_secs;

        // Avoid hogging the CPU.
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

// Update / draw the Ui.
fn set_widgets(ui: &mut conrod::UiCell, ids: &Ids, app: &mut App) {
    use conrod::{widget, Borderable, Colorable, Positionable, Sizeable, Widget};
    use timeline::{track, Timeline};

    // Main window canvas.
    widget::Canvas::new()
        .border(0.0)
        .color(conrod::color::DARK_CHARCOAL.alpha(0.5))
        .set(ids.window, ui);

    let App {
        ref mut playhead_secs,
        ref bars,
        ref notes,
        ref mut tempo_envelope,
        ref mut octave_envelope,
        ref mut toggle_envelope,
        ref mut bang_envelope,
    } = *app;

    let ticks = time::Ms(*playhead_secs * ONE_SECOND_MS).to_ticks(BPM, core::PPQN);
    let color = conrod::color::LIGHT_BLUE;

    ////////////////////
    ///// TIMELINE /////
    ////////////////////
    //
    // Set the `Timeline` widget.
    //
    // This returns a context on which we can begin setting our tracks, playhead and scrollbar.
    //
    // The context is used in three stages:
    //
    // 1. `PinnedTracks` for setting tracks that should be pinned to the top of the timeline.
    // 2. `Tracks` for setting regular tracks.
    // 3. `Final` for setting the `Playhead` and `Scrollbar` widgets after all tracks are set.

    let context = Timeline::new(bars.iter().cloned())
        .playhead(ticks)
        .color(color)
        .wh_of(ids.window)
        .middle_of(ids.window)
        .border(1.0)
        .border_color(conrod::color::CHARCOAL)
        .set(ids.timeline, ui);

    /////////////////////////
    ///// PINNED TRACKS /////
    /////////////////////////
    //
    // Pin the ruler track to the top of the timeline.
    //
    // All pinned tracks must be `set` prior to non-pinned tracks.
    {
        let ruler = track::Ruler::new(context.ruler, &context.bars).color(color);
        let track = context.set_next_pinned_track(ruler, ui);
        for triggered in track.event {
            *playhead_secs = triggered.ticks.ms(BPM, core::PPQN) / ONE_SECOND_MS;
        }
    }

    //////////////////
    ///// TRACKS /////
    //////////////////

    // Now that we've finished setting the pinned tracks, move on to the `Tracks` context.
    let context = context.start_tracks(ui);

    {
        // Piano roll.
        let piano_roll = track::PianoRoll::new(&context.bars, &notes[..]).color(color);
        let track = context.set_next_track(piano_roll, ui);
        for event in track.event {
            use timeline::track::piano_roll::Event;
            match event {
                Event::NoteOn(_note_idx) => (),
                Event::NoteOff(_note_idx) => (),
                Event::NotePlayed(_note_idx) => (),
            }
        }

        // A macro for common logic between tempo and octave "numeric" envelopes.
        macro_rules! numeric_automation {
            ($envelope:expr) => {
                let track = {
                    let automation =
                        track::automation::Numeric::new(&context.bars, $envelope).color(color);
                    context.set_next_track(automation, ui)
                };
                for event in track.event {
                    use timeline::track::automation::numeric::Event;
                    match event {
                        Event::Interpolate(_number) => (),
                        Event::Mutate(mutate) => mutate.apply($envelope),
                    }
                }
            };
        }

        // Tempo automation.
        numeric_automation!(tempo_envelope);
        // Octave automation.
        numeric_automation!(octave_envelope);

        // Toggle automation.
        let track = {
            let automation =
                track::automation::Toggle::new(&context.bars, toggle_envelope).color(color);
            context.set_next_track(automation, ui)
        };
        for event in track.event {
            use timeline::track::automation::toggle::Event;
            match event {
                Event::Interpolate(_toggle) => (),
                Event::SwitchTo(_toggle) => (),
                Event::Mutate(mutate) => mutate.apply(toggle_envelope),
            }
        }

        // Bang automation.
        let track = {
            let automation =
                track::automation::Bang::new(&context.bars, bang_envelope).color(color);
            context.set_next_track(automation, ui)
        };
        for event in track.event {
            use timeline::track::automation::bang::Event;
            match event {
                Event::Bang => println!("BANG!"),
                Event::Mutate(mutate) => mutate.apply(bang_envelope),
            }
        }
    }

    ////////////////////////////////
    ///// PLAYHEAD & SCROLLBAR /////
    ////////////////////////////////

    // Now that all tracks have been set, finish up and set the `Playhead` and `Scrollbar`.
    let context = context.end_tracks();

    // Set the playhead after all tracks have been set.
    for event in context.set_playhead(ui) {
        use timeline::playhead::Event;
        match event {
            Event::Pressed => println!("Playhead pressed!"),
            Event::DraggedTo(ticks) => *playhead_secs = ticks.ms(BPM, core::PPQN) / ONE_SECOND_MS,
            Event::Released => println!("Playhead released!"),
        }
    }

    // Set the scrollbar if it is visible.
    context.set_scrollbar(ui);
}
