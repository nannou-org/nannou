**Tutorial Info**

- Author: [madskjeldgaard](https://madskjeldgaard.dk)
- Required Knowledge: [Anatomy of a nannou app](/tutorials/basics/anatomy-of-a-nannou-app.md), [Drawing 2D Shapes](/tutorials/basics/drawing-2d-shapes.md), [OSC introduction](/tutorials/osc/osc-introduction.md)
- Reading Time: 20 minutes
---
# Sending OSC

In this tutorial we will cover how to send OSC data from a nannou app to another application using the `nannou_osc` crate. 

We are going to write a simple program which has a circle moving about on the screen while the circle's position is sent via OSC to another application. We will continue working on the app from [Moving a circle about on the screen](/tutorials/basics/moving-a-circle-about.md).

## Setting up an OSC sender

At the top of your `main.rs`-file, import the `nannou_osc` crate and make it available in your program via the shorthand `osc`

```use nannou_osc as osc;```

The first thing we then need to do is set up our OSC-sender in the `Model`-struct you may have seen in other nannou-tutorials. 
Add a field to the struct called `sender` with a [Sender](https://docs.rs/nannou_osc/latest/nannou_osc/send/struct.Sender.html)-struct as the type input. 

```rust
struct Model {
    sender: osc::Sender<osc::Connected>,
}
```
Next, we need to setup our `Model` struct using the `model` function. Don't worry if it looks a bit daunting at first, we will go through it step by step.

```rust
fn model(_app: &App) -> Model {
    let port = 1234;
    let target_addr = format!("{}:{}", "127.0.0.1", port);

	let sender = osc::sender()
        .expect("Could not bind to default socket")
        .connect(target_addr)
        .expect("Could not connect to socket at address");

    Model { sender }
}
```

First, let's choose the network port that our data will be sent to.

```rust
let port = 1234;
```
The osc-sender expects a string in the format "address:port", for example `"127.0.0.1:1234"`.

The address can either be an internal address or the address of another computer on your network. In this tutorial we will be targetting our own computer's internal address which is represented by `"127.0.0.1"`.

```rust
let target_addr = format!("{}:{}", "127.0.0.1", port);
```

Lastly, we need to bind our OSC sender to the network socket. This isn't always successful, so we are attaching the `expect()`-method (read more about [expect here](https://doc.rust-lang.org/std/option/enum.Option.html#method.expect)) to post an error message if it is not successful. If it is successful, the `.connect(target_addr)`-method is used to connect the sender to the target address. Again, this may be unsuccesful so we use the `expect()`-method on the result of that operation as well.

```rust
let sender = osc::sender()
	.expect("Could not bind to default socket")
	.connect(target_addr)
	.expect("Could not connect to socket at address");
```

### Sending OSC messages

An OSC packet consists of at least two components: An OSC address and 0 or more arguments containing data. The OSC address is not to be confused with the network address we connected to before. Instead, an OSC address is a path sort of like a URL, for example `/circle/position`.

To create an OSC packet, we first need to make an address.
```rust
let osc_addr = "/circle/position".to_string();
```

Then create a vector of arguments. These need to be formatted using the types found in [osc::Type](https://docs.rs/nannou_osc/latest/nannou_osc/enum.Type.html) in the nannou_osc crate. Below we create an argument list of two floating point values: the `x` and `y` coordinates of our circle.

```rust
let args = vec![osc::Type::Float(x), osc::Type::Float(y)];
```

Now, bringing these two things together we get an OSC packet. The sender expect these to be delivered in a tuple.

```rust
let packet = (osc_addr, args);
```

[Reading the documentation](https://docs.rs/nannou_osc/latest/nannou_osc/send/struct.Sender.html#method.send-1) for the `send`-method, we can see that it returns a Result type which will either contain the number of bytes written (if it was successful) and, more importantly, some useful errors of type CommunicationError if it was not succesful. To discard the error part of this, we use the `ok()` method at the end. 

```rust
model.sender.send(packet).ok()
```

## The finished app

```rust
use nannou::prelude::*;
use nannou_osc as osc;

fn main() {
    nannou::app(model).update(update).simple_window(view).run();
}

struct Model {
    sender: osc::Sender<osc::Connected>,
}

fn model(_app: &App) -> Model {
    // The network port that data is being sent to
    let port = 1234;

    // The osc-sender expects a string in the format "address:port", for example "127.0.0.1:1234"
    // "127.0.0.1" is equivalent to your computers internal address.
    let target_addr = format!("{}:{}", "127.0.0.1", port);

    // This is the osc Sender which contains a couple of expectations in case something goes wrong.
    let sender = osc::sender()
        .expect("Could not bind to default socket")
        .connect(target_addr)
        .expect("Could not connect to socket at address");

    Model { sender }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {}

fn view(app: &App, model: &Model, frame: Frame) {
    // Use app time to progress through a sine wave
    let sine = app.time.sin();
    let slowersine = (app.time / 2.0).sin();

    // Get boundary of the window (to constrain the movements of our circle)
    let boundary = app.window_rect();

    // Map the sine wave functions to ranges between the boundaries of the window
    let x = map_range(sine, -1.0, 1.0, boundary.left(), boundary.right());
    let y = map_range(slowersine, -1.0, 1.0, boundary.bottom(), boundary.top());

    // Send x-y coordinates as OSC
    let osc_addr = "/circle/position".to_string();
    let args = vec![osc::Type::Float(x), osc::Type::Float(y)];
    let packet = (osc_addr, args);

    model.sender.send(packet).ok();

    // Prepare to draw.
    let draw = app.draw();

    // Clear the background to purple.
    draw.background().color(PLUM);

    // Draw a blue ellipse at the x/y coordinates 0.0, 0.0
    draw.ellipse().color(STEELBLUE).x_y(x, y);

    draw.to_frame(app, &frame).unwrap();
}
```
