extern crate nannou;
use nannou::math::map_range;

use nannou::prelude::*;

fn main() {
    nannou::run(model, event, view);
}

struct Model {
    time: f64,
}

fn model(_app: &App) -> Model {
    // Initialise our models variables
    let mut time = 0.0;

    // Construct and return the model with our initialised values
    Model { time }
}

fn event(_app: &App, mut model: Model, event: Event) -> Model {
    if let Event::Update(duration) = event {
        model.time = (duration.since_start.as_secs() as f64
            + duration.since_start.subsec_nanos() as f64 * 1e-9) % 10.0
    }
    model
}

fn view(app: &App, model: &Model, frame: Frame) -> Frame {
    // Prepare to draw.
    let draw = app.draw();

    draw.background().rgba(0.0,0.0,0.0,0.0);

    for x in 0..20 {
        for y in 0..20 {
                let x_phase = map_range(x,0,20,0.0,3.14) as f64;
                let y_phase = map_range(y,0,20,0.0,3.14) as f64;
                let radius = x_phase.sin() * (model.time * x_phase.sin()).cos() * ((model.time * 0.2).sin() * 40.0);
                let speed1 = ((model.time.sin() * 0.3).cos()).fract().sin();
                let speed2 = ((model.time.cos() * 0.2).sin()).fract() * (radius * 0.1);
                draw.ellipse()
                    .rgba(
                        1.0 - x_phase.cos().abs() as f32 * speed1 as f32,
                        y_phase.cos() as f32 * speed2 as f32,
                        1.0 - x_phase.sin() as f32 + model.time.cos() as f32,
                        0.5,
                    )
                    .x_y_z(
                        x_phase.sin() * y_phase.cos() * ((model.time * speed1).fract() * 40.0) * radius,
                        x_phase.cos() * y_phase.sin() * ((model.time * speed2).fract() * 40.0) * radius,
                        radius.sin(),
                    )
                    .w_h(x_phase.atan() * radius, y_phase.fract() * radius);
            }
    }

    // Write to the window frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
