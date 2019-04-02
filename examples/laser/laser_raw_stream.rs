//! A simple example demonstrating how to use the position of the mouse to control a single-point
//! beam via a raw laser stream.

extern crate lasy;
extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::app(model).run();
}

struct Model {
    laser_api: lasy::Lasy,
    laser_stream: lasy::RawStream<Laser>,
}

struct Laser {
    point_idx: usize,
    position: Point2,
}

fn model(app: &App) -> Model {
    // Create a window to receive mouse events.
    app.new_window()
        .mouse_moved(mouse_moved)
        .view(view)
        .build()
        .unwrap();

    // Initialise the state that we want to live on the laser thread and spawn the stream.
    let laser_model = Laser { point_idx: 0, position: pt2(0.0, 0.0) };
    let laser_api = lasy::Lasy::new();
    let laser_stream = laser_api
        .new_raw_stream(laser_model, laser)
        .build()
        .unwrap();

    Model { laser_api, laser_stream }
}

fn laser(laser: &mut Laser, buffer: &mut lasy::Buffer) {
    // Write white points to the laser stream at the current position.
    for point in buffer.iter_mut() {
        point.color = [1.0, 1.0, 1.0];
        point.position = laser.position.into();
        // Many lasers have a feature called "scan fail safety" (SFS) where the beam will
        // automatically cut out if the scanner is not moving for safety.
        // To avoid cutting out, we'll offset the point slightly to make a diamond shape.
        let offset = 0.125;
        match laser.point_idx % 4 {
            0 => point.position[0] += offset * 0.5,
            1 => point.position[1] += offset * 0.5,
            2 => point.position[0] -= offset * 0.5,
            _ => point.position[1] -= offset * 0.5,
        }
        laser.point_idx = laser.point_idx.wrapping_add(1);
    }
}

fn mouse_moved(app: &App, model: &mut Model, pos: Point2) {
    // Lets use the mouse position to control the laser position.
    let win_rect = app.window_rect();
    let laser_rect = geom::Rect::from_w_h(2.0, 2.0);
    let x = win_rect.x.map_value(pos.x, &laser_rect.x);
    let y = win_rect.y.map_value(pos.y, &laser_rect.y);
    model.laser_stream.send(move |laser| {
        laser.position = pt2(x, y);
    }).unwrap();
}

fn view(app: &App, _model: &Model, frame: Frame) -> Frame {
    // Visualise the point in the window.
    let draw = app.draw();
    draw.background().color(DARK_CHARCOAL);
    draw.ellipse().w_h(5.0, 5.0).xy(app.mouse.position());
    draw.to_frame(app, &frame).unwrap();
    frame
}
