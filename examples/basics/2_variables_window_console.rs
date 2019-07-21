use nannou::prelude::*;

fn main() {
    nannou::app(model).view(view).run();
}

struct Model;

fn model(app: &App) -> Model {
    // Below are some of the different primitive types available in Rust.
    let i = 50; // Integers store whole numbers.
    let f = 36.6; // Floats are used to store numbers with decimals or fractions.
    let b = true; // Boolean values can be either 'true' or 'false'.
    let c = '!'; // Characters represent a single UTF8 character.
    let message = "hello world"; // Strings are a sequence of characters.

    // Print the values stored in our varibales to the console
    println!("i = {}", i);
    println!("f = {}", f);
    println!("b = {}", b);
    println!("c = {}", c);
    println!("message = {}", message);

    // Construct and define the size of our window using `.with_dimensions(width, height)`.
    app.new_window().with_dimensions(640, 480).build().unwrap();

    Model
}

fn view(_app: &App, _model: &Model, frame: &Frame) {
    frame.clear(DIMGRAY);
}
