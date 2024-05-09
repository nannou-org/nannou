use nannou::prelude::*;

fn main() {
    nannou::app(model).simple_window(view).run();
}

struct Model;

fn model(_app: &App) -> Model {
    // For loops let us loop over many items, one at a time.
    // The part between `for` and `in` is the name that we give the item we are currently handling.
    // The part between `in` and `{` is the sequence we will loop over, known as an `Iterator`.
    // The part between `{` and `}` is where we put everything we want to happen for every `i`.
    println!("`for` loop:");
    for i in 0..10 {
        println!("{}", i);
    }

    // While loops continue until a certain condition is met.
    // Note that we need to make x 'mutable' so we can change its value inside of the loop.
    println!("`while` loop:");
    let mut x = 0;
    while x < 10 {
        println!("{}", x);
        x += 1;
    }

    // If you want to loop forever, Rust provides a dedicated keyword called `loop`.
    // Note your code will be stuck in this loop until you decide to `break` from the loop.
    // In this case we loop while y is less than 30 and then we use 'break' to exit.
    println!("`loop`:");
    let mut y = 0;
    loop {
        println!("{}", y);
        y += 1;
        if y > 30 {
            println!("`break`");
            break;
        }
    }

    Model
}

fn view(_app: &App, _model: &Model, frame: Frame) {
    draw.background().color(DIMGRAY);
}
