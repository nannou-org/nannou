# Draw a Nannou Sketch

**Tutorial Info**

- Author: tpltnt
- Required Knowledge: [Getting Started](/getting_started.md)
- Reading Time: 10 minutes

---


**Nannou is a framework for creative coding in Rust.** A framework can be
thought of as a collection of building blocks to help accomplish a goal.

A sketch is the smallest/fastest way to get results with nannou.
Here is one example which just yields a blue window:

```rust,no_run
# extern crate nannou;
#
// minimal example of a nannou sketch
use nannou::prelude::*;

fn main() {
    nannou::sketch(view);
}

fn view(app: &App, frame: &Frame) {
    // get canvas to draw on
    let draw = app.draw();

    // set background to blue
    draw.background().color(BLUE);

    // put everything on the frame
    draw.to_frame(app, &frame).unwrap();
}
```

You can exit the sketch by pressing `ESC`.

## Sketches vs. Apps

Nannou can be used to create many things with very different levels
of complexity, similar to pen and paper. Sketches are more like
squiggles on napkins while apps can be really elaborate drawings.
Sketches offer a contrained space to work with, but a lot is taken
care of behind the scenes. Apps allow for more fine grained control,
but also require more (explicit) work on your part. A good overview
of how apps work can be found in the chapter [Anatomy of a nannou app](/tutorials/basics/anatomy-of-a-nannou-app.md).
