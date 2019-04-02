//! A simple example demonstrating how to use the position of the mouse to control a single-point
//! beam via a raw laser stream.

extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::app(model).run();
}

struct Model {
    laser_api: lasy::Lasy,
    laser_stream: lasy::FrameStream<Laser>,
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
}

fn model(app: &App) -> Model {
    // Create a window to receive keyboard events.
    app.new_window()
        .key_pressed(key_pressed)
        .view(view)
        .build()
        .unwrap();

    // Initialise the state that we want to live on the laser thread and spawn the stream.
    let laser_model = Laser { test_pattern: TestPattern::Rectangle };
    let laser_api = lasy::Lasy::new();
    let laser_stream = laser_api
        .new_frame_stream(laser_model, laser)
        .build()
        .unwrap();

    Model { laser_api, laser_stream }
}

fn laser(laser: &mut Laser, frame: &mut lasy::Frame) {
    // Simple constructors for white or blank points.
    let white_p = |position| lasy::Point { position, color: [1.0; 3] };

    // Draw the frame with the selected pattern.
    match laser.test_pattern {
        TestPattern::Rectangle => {
            let tl = [-1.0, 1.0];
            let tr = [1.0, 1.0];
            let br = [1.0, -1.0];
            let bl = [-1.0, -1.0];
            let positions = [tl, tr, br, bl, tl];
            let points = positions.iter().cloned().map(white_p);
            frame.add_points(points);
        }

        TestPattern::Triangle => {
            let a = [-0.75, -0.75];
            let b = [0.0, 0.75];
            let c = [0.75, -0.75];
            let positions = [a, b, c, a];
            let points = positions.iter().cloned().map(white_p);
            frame.add_points(points);
        }

        TestPattern::Crosshair => {
            let xa = [-1.0, 0.0];
            let xb = [1.0, 0.0];
            let ya = [0.0, -1.0];
            let yb = [0.0, 1.0];
            let x = [white_p(xa), white_p(xb)];
            let y = [white_p(ya), white_p(yb)];
            frame.add_points(&x);
            frame.add_points(&y);
        }

        TestPattern::ThreeVerticalLines => {
            let la = [-1.0, -0.5];
            let lb = [-1.0, 0.5];
            let ma = [0.0, 0.5];
            let mb = [0.0, -0.5];
            let ra = [1.0, -0.5];
            let rb = [1.0, 0.5];
            let l = [white_p(la), white_p(lb)];
            let m = [white_p(ma), white_p(mb)];
            let r = [white_p(ra), white_p(rb)];
            frame.add_points(&l);
            frame.add_points(&m);
            frame.add_points(&r);
        }
    }
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    // Send a new pattern to the laser on keys 1, 2, 3 and 4.
    let new_pattern = match key {
        Key::Key1 => TestPattern::Rectangle,
        Key::Key2 => TestPattern::Triangle,
        Key::Key3 => TestPattern::Crosshair,
        Key::Key4 => TestPattern::ThreeVerticalLines,
        _ => return,
    };
    model.laser_stream.send(|laser| {
        laser.test_pattern = new_pattern;
    }).unwrap();
}

fn view(_app: &App, _model: &Model, frame: Frame) -> Frame {
    frame.clear(DARK_CHARCOAL);
    frame
}
