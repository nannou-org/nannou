extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::run(model, event, view);
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

// Remap a value from one range to another 
fn map(value: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32 ) -> f32 {
    ( (value - in_min) / ( in_max - in_min ) * ( out_max - out_min ) ) + out_min 
}

fn event(_app: &App, model: Model, event: Event) -> Model {
    match event {
        Event::WindowEvent { simple: Some(event), .. } => match event {

            KeyPressed(_key) => {
                println!("add 10 + 2 = {}", add(10,2));
                println!("subtract 100 - 30 = {}", subtract(100,30));
                println!("multiply 3.5 * 10.2 = {}", multiply(3.5,10.2));
                println!("random = {}", random());
                println!("remaped value = {}", map(random(),0.0,1.0,0.0,100.0));
            },

            MousePressed(_button) => {
                do_something();
            },

            _other => (),
        },

        Event::Update(_dt) => {
        },

        _ => (),
    }
    model
}

fn view(app: &App, _model: &Model, frame: Frame) -> Frame {
    // Our app only has one window, so retrieve this part of the `Frame`. Color it gray.
    frame.window(app.window.id()).unwrap().clear_color(0.1, 0.11, 0.12, 1.0);
    // Return the drawn frame.
    frame
}
