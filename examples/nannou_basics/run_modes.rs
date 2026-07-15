//! Switch run modes and update modes at runtime to feel how each behaves.
//!
//! Watch the orbiting dot and the `updates` counter - the dot's position follows the
//! counter, so it freezes whenever `update` stops running:
//!   - Continuous : counter climbs every frame; the dot orbits smoothly.
//!   - Rate (10)  : counter climbs ~10x/sec.
//!   - Wait       : counter climbs only when you give input (move the mouse / press keys).
//!   - loop_once  : counter stops at 1; the frame is held (~0 CPU).
//!   - loop_ntimes(60): counter stops at 60, then held.
//!
//! Keys:
//!   Space          toggle Continuous <-> loop_once
//!   1 Continuous   2 Rate(10)   3 Wait   4 loop_once   5 loop_ntimes(60)

use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Continuous,
    Rate,
    Wait,
    LoopOnce,
    Loop60,
}

impl Mode {
    fn label(self) -> &'static str {
        match self {
            Mode::Continuous => "Continuous",
            Mode::Rate => "Reactive rate (10 fps)",
            Mode::Wait => "Wait (redraw on input)",
            Mode::LoopOnce => "loop_once (draw once, held)",
            Mode::Loop60 => "loop_ntimes(60)",
        }
    }
}

struct Model {
    mode: Mode,
    count: u32,
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(640, 640)
        .view(view)
        .key_pressed(key_pressed)
        .build();
    Model {
        mode: Mode::Continuous,
        count: 0,
    }
}

fn update(_app: &App, model: &mut Model) {
    model.count += 1;
}

fn key_pressed(app: &App, model: &mut Model, key: KeyCode) {
    let mode = match key {
        KeyCode::Space => {
            if model.mode == Mode::LoopOnce {
                Mode::Continuous
            } else {
                Mode::LoopOnce
            }
        }
        KeyCode::Digit1 => Mode::Continuous,
        KeyCode::Digit2 => Mode::Rate,
        KeyCode::Digit3 => Mode::Wait,
        KeyCode::Digit4 => Mode::LoopOnce,
        KeyCode::Digit5 => Mode::Loop60,
        _ => return,
    };
    set_mode(app, model, mode);
}

// Apply a mode as a (run mode, update mode) pair. Loop modes only set the run mode - the
// framework drives them and then freezes; the others are `UntilExit` with an explicit
// update mode.
fn set_mode(app: &App, model: &mut Model, mode: Mode) {
    model.mode = mode;
    model.count = 0;
    match mode {
        Mode::Continuous => {
            app.set_run_mode(RunMode::UntilExit);
            app.set_update_mode(UpdateMode::Continuous);
        }
        Mode::Rate => {
            app.set_run_mode(RunMode::UntilExit);
            app.set_update_rate(10.0);
        }
        Mode::Wait => {
            app.set_run_mode(RunMode::UntilExit);
            app.set_update_mode(UpdateMode::wait());
        }
        Mode::LoopOnce => app.set_run_mode(RunMode::loop_once()),
        Mode::Loop60 => app.set_run_mode(RunMode::loop_ntimes(60)),
    }
}

fn view(app: &App, model: &Model) {
    let draw = app.draw();
    draw.background().srgb(0.09, 0.10, 0.13);
    let win = app.window_rect();

    // Orbiting dot - position advances with the update counter.
    let a = model.count as f32 * 0.08;
    let r = win.w().min(win.h()) * 0.3;
    draw.ellipse()
        .x_y(a.cos() * r, a.sin() * r)
        .radius(24.0)
        .color(TOMATO);

    // HUD: current mode + update count, and the key hints.
    draw.text(&format!("{}\nupdates: {}", model.mode.label(), model.count))
        .x_y(0.0, win.h() * 0.5 - 44.0)
        .wh(vec2(win.w() - 20.0, 80.0))
        .font_size(22)
        .color(WHITE);
    draw.text("space: Continuous <-> loop_once     1 Continuous   2 Rate   3 Wait   4 loop_once   5 loop60")
        .x_y(0.0, -win.h() * 0.5 + 24.0)
        .wh(vec2(win.w() - 20.0, 30.0))
        .font_size(13)
        .color(GRAY);
}
