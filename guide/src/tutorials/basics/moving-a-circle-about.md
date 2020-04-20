**Tutorial Info**

- Author: [madskjeldgaard](https://madskjeldgaard.dk)
- Required Knowledge: [Anatomy of a nannou app](/tutorials/basics/anatomy-of-a-nannou-app.md), [Drawing 2D Shapes](/tutorials/basics/drawing-2d-shapes.md)
- Reading Time: 10 minutes
---

![osc-circle](/tutorials/basics/images/moving-circle.gif)

# Moving a circle about on the screen
In this tutorial we will cover the basics of moving a shape around in the window of a nannou app.

Let's start by making a simple program which draws a circle in the center of the screen.

We will be using the barebones app from [Anatomy of a nannou app](./tutorials/basics/anatomy-of-a-nannou-app.md) as a starting point for this.

Update the view function of your nannou-app to look like this: 

```rust,no_run
# extern crate nannou;
#fn main() {
#    nannou::app(model)
#        .event(event)
#        .simple_window(view)
#        .run();
#}
#fn model(_app: &App) -> Model {
#    Model {}
#}
#fn event(_app: &App, _model: &mut Model, _event: Event) {
#}
fn view(app: &App, _model: &Model, frame: Frame) {
	// Prepare to draw.
    let draw = app.draw();

    // Clear the background to purple.
    draw.background().color(PLUM);

	// Draw a blue ellipse with a radius of 10 at the (x,y) coordinates of (0.0, 0.0)
    draw.ellipse().color(STEELBLUE).x_y(0.0,0.0);

    draw.to_frame(app, &frame).unwrap();
}
```
## Adding movement

Let's now add some movement to our circle to give it a bit of life. 

To do this, we will make use of the ever wonderful [sinewave](https://en.wikipedia.org/wiki/Sine_wave). 

These can be generated in nannou by taking the progressed time of the application and feeding it to a sine function.
```rust,no_run
# extern crate nannou;
#fn main() {
#    nannou::app(model)
#        .event(event)
#        .simple_window(view)
#        .run();
#}
#fn model(_app: &App) -> Model {
#    Model {}
#}
#fn event(_app: &App, _model: &mut Model, _event: Event) {
#}
#fn view(app: &App, _model: &Model, frame: Frame) {
#
let sine = app.time.sin();
#}
```
Let's make another one but at half the speed by dividing the time value by two
```rust,no_run
# extern crate nannou;
#fn main() {
#    nannou::app(model)
#        .event(event)
#        .simple_window(view)
#        .run();
#}
#fn model(_app: &App) -> Model {
#    Model {}
#}
#fn event(_app: &App, _model: &mut Model, _event: Event) {
#}
#fn view(app: &App, _model: &Model, frame: Frame) {
#
let slowersine = (app.time / 2.0).sin();
#}
```

Now that we have two functions generating nice, smooth wave movements, let's use them to control our little circle.

If we put these values directly in the ellipse's `.x_y()`-method we would not see much movement. That's because the sine waves generate values between -1.0 and 1.0 and the coordinates expect a pixel position.

But how wide is our window ? To get a precise idea of this, we can use a handy method called [window_rect](https://docs.rs/nannou/latest/nannou/app/struct.App.html#method.window_rect) which is available in the `app` variable.
```rust,no_run
# extern crate nannou;
#fn main() { nannou::app(model).event(event).simple_window(view).run(); }
#fn model(_app: &App) -> Model {Model {}}
#fn event(_app: &App, _model: &mut Model, _event: Event) {}
#fn view(app: &App, _model: &Model, frame: Frame) {
let boundary = app.window_rect();
#}
```

This will give us the boundary of the window as a handy `Rect`. This is a struct that responds to [tons of useful methods](https://docs.rs/nannou/latest/nannou/geom/rect/struct.Rect.html) that we can use to define the minimum and maximum values of our x and y coordinates respectively to constrain the movements of our circle.

The minimum x value is thus available as:
```rust,no_run
# extern crate nannou;
#fn main() { nannou::app(model).event(event).simple_window(view).run(); }
#fn model(_app: &App) -> Model {Model {}}
#fn event(_app: &App, _model: &mut Model, _event: Event) {}
#fn view(app: &App, _model: &Model, frame: Frame) {
#let boundary = app.window_rect();
boundary.left(); 
#}
```
And the maximum x value is 
```rust,no_run
# extern crate nannou;
#fn main() { nannou::app(model).event(event).simple_window(view).run(); }
#fn model(_app: &App) -> Model {Model {}}
#fn event(_app: &App, _model: &mut Model, _event: Event) {}
#fn view(app: &App, _model: &Model, frame: Frame) {
#let boundary = app.window_rect();
boundary.left(); 
#}
```
The minimum y value is 
```rust,no_run
# extern crate nannou;
#fn main() { nannou::app(model).event(event).simple_window(view).run(); }
#fn model(_app: &App) -> Model {Model {}}
#fn event(_app: &App, _model: &mut Model, _event: Event) {}
#fn view(app: &App, _model: &Model, frame: Frame) {
#let boundary = app.window_rect();
boundary.bottom(); 
#}
```
And the maximum y value is 
```rust,no_run
# extern crate nannou;
#fn main() { nannou::app(model).event(event).simple_window(view).run(); }
#fn model(_app: &App) -> Model {Model {}}
#fn event(_app: &App, _model: &mut Model, _event: Event) {}
#fn view(app: &App, _model: &Model, frame: Frame) {
#let boundary = app.window_rect();
boundary.top(); 
#}
```
## Mapping values to a range
Using these values, we can map our `sine` and `slowersine` values to ranges of values that are within the boundary of our window. To do this, we will use the [map_range](https://docs.rs/nannou/latest/nannou/math/fn.map_range.html) function available in nannou.

The `map_range` function takes 5 arguments: `val`, `in_min`, `in_max`, `out_min`, `out_max`. The `val` here is our sinewaves which has a minimum value of -1.0 and a maximum value of 1.0. For the x-coordinate, we then map it to a range of values between the leftmost point and the rightmost point.
```rust,no_run
# extern crate nannou;
#fn main() {
#    nannou::app(model)
#        .event(event)
#        .simple_window(view)
#        .run();
#}
#fn model(_app: &App) -> Model {
#    Model {}
#}
#fn event(_app: &App, _model: &mut Model, _event: Event) {
#}
#fn view(app: &App, _model: &Model, frame: Frame) {
#    let draw = app.draw();
#    let sine = app.time.sin();
#    let slowersine = (app.time / 2.0).sin();
#    let boundary = app.window_rect();
let x = map_range(sine, -1.0, 1.0, boundary.left(), boundary.right());
#    let y = map_range(slowersine, -1.0, 1.0, boundary.bottom(), boundary.top());
#    draw.background().color(PLUM);
#    draw.ellipse().color(STEELBLUE).x_y(x, y);
#    draw.to_frame(app, &frame).unwrap();
#}
```
And then the same for the y value but using the `slowersine` variable.
```rust,no_run
# extern crate nannou;
#fn main() {
#    nannou::app(model)
#        .event(event)
#        .simple_window(view)
#        .run();
#}
#fn model(_app: &App) -> Model {
#    Model {}
#}
#fn event(_app: &App, _model: &mut Model, _event: Event) {
#}
#fn view(app: &App, _model: &Model, frame: Frame) {
#    let draw = app.draw();
#    let sine = app.time.sin();
#    let slowersine = (app.time / 2.0).sin();
#    let boundary = app.window_rect();
#let x = map_range(sine, -1.0, 1.0, boundary.left(), boundary.right());
let y = map_range(slowersine, -1.0, 1.0, boundary.bottom(), boundary.top());
#    draw.background().color(PLUM);
#    draw.ellipse().color(STEELBLUE).x_y(x, y);
#    draw.to_frame(app, &frame).unwrap();
#}
```
The only thing left to do now is to put this into the arguments of our circle-drawing function.
```rust,no_run
# extern crate nannou;
#fn main() {
#    nannou::app(model)
#        .event(event)
#        .simple_window(view)
#        .run();
#}
#fn model(_app: &App) -> Model {
#    Model {}
#}
#fn event(_app: &App, _model: &mut Model, _event: Event) {
#}
#fn view(app: &App, _model: &Model, frame: Frame) {
#    let draw = app.draw();
#    let sine = app.time.sin();
#    let slowersine = (app.time / 2.0).sin();
#    let boundary = app.window_rect();
#let x = map_range(sine, -1.0, 1.0, boundary.left(), boundary.right());
#let y = map_range(slowersine, -1.0, 1.0, boundary.bottom(), boundary.top());
#    draw.background().color(PLUM);
draw.ellipse().color(STEELBLUE).x_y(x, y);
#    draw.to_frame(app, &frame).unwrap();
#}
```
Your updated `view`-function should now look something like this:

```rust,no_run
# extern crate nannou;
#fn main() {
#    nannou::app(model)
#        .event(event)
#        .simple_window(view)
#        .run();
#}
#fn model(_app: &App) -> Model {
#    Model {}
#}
#fn event(_app: &App, _model: &mut Model, _event: Event) {
#}
fn view(app: &App, _model: &Model, frame: Frame) {
    // Prepare to draw.
    let draw = app.draw();

    // Generate sine wave data based on the time of the app
    let sine = app.time.sin();
    let slowersine = (app.time / 2.0).sin();

    // Get boundary of the window (to constrain the movements of our circle)
    let boundary = app.window_rect();

    // Map the sine wave functions to ranges between the boundaries of the window
    let x = map_range(sine, -1.0, 1.0, boundary.left(), boundary.right());
    let y = map_range(slowersine, -1.0, 1.0, boundary.bottom(), boundary.top());

    // Clear the background to purple.
    draw.background().color(PLUM);

    // Draw a blue ellipse at the x/y coordinates 0.0, 0.0
    draw.ellipse().color(STEELBLUE).x_y(x, y);

    draw.to_frame(app, &frame).unwrap();
}
```
