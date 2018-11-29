extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::app(model).event(event).simple_window(view).run();
}

struct Model;

fn model(_app: &App) -> Model {
    Model
}

// Void functions that do not return anything
fn do_something() -> () {
    println!("DO IT!!!!");
}

// Add 2 integer values together and return the result
fn add(x: i32, y: i32) -> i32 {
    x + y
}

// Subtract 2 integer values and return the result
fn subtract(x: i32, y: i32) -> i32 {
    x - y
}

// Multiply 2 float values and return the result
fn multiply(x: f32, y: f32) -> f32 {
    x * y
}

// Return a random floating point value between 0.0 and 1.0
fn random() -> f32 {
    nannou::rand::random()
}

fn event(_app: &App, model: Model, event: Event) -> Model {
    match event {
        Event::WindowEvent {
            simple: Some(event),
            ..
        } => match event {
            KeyPressed(_key) => {
                println!("add 10 + 2 = {}", add(10, 2));
                println!("subtract 100 - 30 = {}", subtract(100, 30));
                println!("multiply 3.5 * 10.2 = {}", multiply(3.5, 10.2));
                println!("random = {}", random());
                println!("remaped value = {}", random_range(0.0f32, 100.0));
            }

            MousePressed(_button) => {
                do_something();
            }

            _other => (),
        },

        Event::Update(_dt) => {}

        _ => (),
    }
    model
}

fn view(_app: &App, _model: &Model, frame: Frame) -> Frame {
    // Clear the window with dark charcoal.
    frame.clear(DARK_CHARCOAL);
    // Return the drawn frame.
    frame
}
