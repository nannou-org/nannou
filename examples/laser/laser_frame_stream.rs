//! A simple example demonstrating how to use draw various patterns via a laser frame streams.

use nannou::prelude::*;
use nannou_laser as laser;

fn main() {
    nannou::app(model).run();
}

struct Model {
    _laser_api: laser::Api,
    laser_stream: laser::FrameStream<Laser>,
}

struct Laser {
    test_pattern: TestPattern,
}

// A collection of laser test patterns. We'll toggle between these with the numeric keys.
pub enum TestPattern {
    // A rectangle that outlines the laser's entire field of projection.
    Rectangle,
    // A triangle in the centre of the projection field.
    Triangle,
    // A crosshair in the centre of the projection field that reaches the edges.
    Crosshair,
    // Three vertical lines. One to the far left, one in the centre and one on the right.
    ThreeVerticalLines,
    // A circle whose diameter reaches the edges of the projection field.
    Circle,
    // A spiral that starts from the centre and revolves out towards the edge of the field.
    Spiral,
}

fn model(app: &App) -> Model {
    // Create a window to receive keyboard events.
    app.new_window()
        .key_pressed(key_pressed)
        .view(view)
        .build();

    // Initialise the state that we want to live on the laser thread and spawn the stream.
    let laser_model = Laser {
        test_pattern: TestPattern::Rectangle,
    };
    let _laser_api = laser::Api::new();
    let laser_stream = _laser_api
        .new_frame_stream(laser_model, laser)
        .build()
        .unwrap();

    Model {
        _laser_api,
        laser_stream,
    }
}

fn laser(laser: &mut Laser, frame: &mut laser::Frame) {
    // Simple constructors for white or blank points.
    let lit_p = |position| laser::Point::new(position, [1.0; 3]);

    // Draw the frame with the selected pattern.
    match laser.test_pattern {
        TestPattern::Rectangle => {
            let tl = [-1.0, 1.0];
            let tr = [1.0, 1.0];
            let br = [1.0, -1.0];
            let bl = [-1.0, -1.0];
            let positions = [tl, tr, br, bl, tl];
            let points = positions.iter().cloned().map(lit_p);
            frame.add_lines(points);
        }

        TestPattern::Triangle => {
            let a = [-0.75, -0.75];
            let b = [0.0, 0.75];
            let c = [0.75, -0.75];
            let positions = [a, b, c, a];
            let points = positions.iter().cloned().map(lit_p);
            frame.add_lines(points);
        }

        TestPattern::Crosshair => {
            let xa = [-1.0, 0.0];
            let xb = [1.0, 0.0];
            let ya = [0.0, -1.0];
            let yb = [0.0, 1.0];
            let x = [lit_p(xa), lit_p(xb)];
            let y = [lit_p(ya), lit_p(yb)];
            frame.add_lines(&x);
            frame.add_lines(&y);
        }

        TestPattern::ThreeVerticalLines => {
            let la = [-1.0, -0.5];
            let lb = [-1.0, 0.5];
            let ma = [0.0, 0.5];
            let mb = [0.0, -0.5];
            let ra = [1.0, -0.5];
            let rb = [1.0, 0.5];
            let l = [lit_p(la), lit_p(lb)];
            let m = [lit_p(ma), lit_p(mb)];
            let r = [lit_p(ra), lit_p(rb)];
            frame.add_lines(&l);
            frame.add_lines(&m);
            frame.add_lines(&r);
        }

        TestPattern::Circle => {
            let n_points = frame.points_per_frame() as usize / 4;
            let rect = Rect::from_w_h(1.0, 1.0);
            let ellipse: Vec<_> = geom::ellipse::Circumference::new(rect, n_points as f32)
                .map(|[x, y]| lit_p([x, y]))
                .collect();
            frame.add_lines(&ellipse);
        }

        TestPattern::Spiral => {
            let n_points = frame.points_per_frame() as usize / 2;
            let radius = 1.0;
            let rings = 5.0;
            let points = (0..n_points)
                .map(|i| {
                    let fract = i as f32 / n_points as f32;
                    let mag = fract * radius;
                    let phase = rings * fract * 2.0 * std::f32::consts::PI;
                    let y = mag * -phase.sin();
                    let x = mag * phase.cos();
                    [x, y]
                })
                .map(lit_p)
                .collect::<Vec<_>>();
            frame.add_lines(points);
        }
    }
}

fn key_pressed(_app: &App, model: &mut Model, key: KeyCode) {
    // Send a new pattern to the laser on keys 1, 2, 3 and 4.
    let new_pattern = match key {
        KeyCode::Digit1 => TestPattern::Rectangle,
        KeyCode::Digit2 => TestPattern::Triangle,
        KeyCode::Digit3 => TestPattern::Crosshair,
        KeyCode::Digit4 => TestPattern::ThreeVerticalLines,
        KeyCode::Digit5 => TestPattern::Circle,
        KeyCode::Digit6 => TestPattern::Spiral,
        _ => return,
    };
    model
        .laser_stream
        .send(|laser| {
            laser.test_pattern = new_pattern;
        })
        .unwrap();
}

fn view(_app: &App, _model: &Model) {
    draw.background().color(DIMGRAY);
}
