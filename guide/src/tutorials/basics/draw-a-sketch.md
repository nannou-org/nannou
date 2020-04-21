# Draw a Sketch

**Tutorial Info**

- Author: tpltnt
- Required Knowledge:
    - [Getting Started](/getting_started.md)
- Reading Time: 5 minutes

---


**Nannou is a framework for creative coding in Rust.** A framework can be
thought of as a collection of building blocks to help accomplish a goal.
A sketch is the smallest/fastest way to get results with nannou.
Here is one example which just yields a blue window:

```rust,no_run
use nannou::prelude::*;

fn main() {
    nannou::sketch(view).run();
}

fn view(app: &App, frame: Frame) {
    // get canvas to draw on
    let draw = app.draw();

    // set background to blue
    draw.background().color(BLUE);

    // put everything on the frame
    draw.to_frame(app, &frame).unwrap();
}
```

You can exit the sketch by pressing `ESC`.

## Sidenote: Sketches vs. Apps

Nannou can be used to create many things with very different levels of
complexity, similar to pen and paper. Sketches are more like squiggles on
napkins while apps can be really elaborate artworks or fully fledged
applications. Sketches offer a constrained space to work with, but a lot is
taken care of behind the scenes. Apps allow for more fine grained control, but
also require a little more setup on your part. The main difference is that an
app provides a model for working with state, while a sketch provides a simpler
API to get drawing quickly. A good overview of how apps work can be found in the
chapter [Anatomy of a nannou app](/tutorials/basics/anatomy-of-a-nannou-app.md).

## Explaining the Code

A sketch consists of at least two functions: `main()` and `view()`.
First we import some building blocks:

```rust,no_run
# #![allow(unused_imports)]
use nannou::prelude::*;
# fn main() {}
```

After this import the actual sketching code starts. The `main()` functions is where all your logic starts. The code

```rust,no_run
# use nannou::prelude::*;
#
# fn main() {
    nannou::sketch(view).run();
# }
# fn view(_app: &App, _frame: Frame) {}
```

calls a function to draw on the single window (`view()` in this case). This
function has the signature `fn(_: &App, _: Frame);`. Don't worry if you
don't know what a function signature is. Just copy the `main()` function
and you will be fine.

Within the view() function, what we draw to the Frame will be presented in our window.

```rust,no_run
# #![allow(unused_imports)]
# use nannou::prelude::*;
# fn main() {
#    nannou::sketch(view).run();
# }
fn view(app: &App, frame: Frame) {
    let draw = app.draw();

    draw.background().color(BLUE);

    draw.to_frame(app, &frame).unwrap();
}
```

This function follows the same scheme. First some setup is done. The line

```rust,no_run
# #![allow(unused_imports, unused_variables)]
# use nannou::prelude::*;
# fn main() {
#    nannou::sketch(view).run();
# }
# fn view(app: &App, _frame: Frame) {
let draw = app.draw();
# }
```

lets us assign a canvas-like datatype to the variable `draw`.
We can now paint on the this canvas by setting the background to blue.

```rust,no_run
# #![allow(unused_imports)]
# use nannou::prelude::*;
# fn main() {
#    nannou::sketch(view).run();
# }
# fn view(app: &App, _frame: Frame) {
# let draw = app.draw();
draw.background().color(BLUE);
# }
```

Now we have a canvas with only a blue background. We take this canvas and
create a computer graphics frame from it to display in the main window.

```rust,no_run
# #![allow(unused_imports)]
# use nannou::prelude::*;
# fn main() {
#    nannou::sketch(view).run();
# }
# fn view(app: &App, frame: Frame) {
# let draw = app.draw();
draw.to_frame(app, &frame).unwrap();
# }
```

Note that the `view()` function is called repeatedly, but not at a constant
rate. This might be problematic if you use randomness to draw things.
