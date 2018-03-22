extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::run(model, event, view);
}

struct Model;

fn model(_app: &App) -> Model {
    // For loops in Rust is an iterator which gives back a series of elemaents 
    // one at a time. The current value of the iterator is assigned in this case to 'i'
    for i in 0..10 {
        println!("for iterator = {}", i);
    }

    // While loops continue until a certain condition is met. Note that we need
    // to make x 'mutable' so we can change its value inside of the loop.
    let mut x = 0;
    while x < 10 {
        println!("while = {}", x);
        x += 1;
    }

    // If you want to loop forever, Rust provides a dedicated keyword to handle this 
    // Note your code will be stuck in this loop until you decide to break.
    // In this case we loop while y is less than 30 and then we use the 'break'
    // keyword to exit the loop.  
    let mut y = 0;
    loop {
        y += 1;
        println!("loooooping");

        if y > 30 { 
            println!("breaking out of loop");
            break; 
        }
    }

    Model
}

fn event(_app: &App, model: Model, _event: Event) -> Model {
    model
}

fn view(app: &App, _model: &Model, frame: Frame) -> Frame {
    // Our app only has one window, so retrieve this part of the `Frame`. Color it gray.
    frame.window(app.window.id()).unwrap().clear_color(0.1, 0.11, 0.12, 1.0);
    // Return the drawn frame.
    frame
}
