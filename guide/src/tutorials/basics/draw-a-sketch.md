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

fn view(app: &App) {
    // get canvas to draw on
    let draw = app.draw();

    // set background to blue
    draw.background().color(BLUE);

    // put everything on the frame
    
}
```

You can exit the sketch by pressing `ESC`.

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
# fn view(_app: &App) {}
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
fn view(app: &App) {
    let draw = app.draw();

    draw.background().color(BLUE);

    
}
```

This function follows the same scheme. First some setup is done. The line

```rust,no_run
# #![allow(unused_imports, unused_variables)]
# use nannou::prelude::*;
# fn main() {
#    nannou::sketch(view).run();
# }
# fn view(app: &App) {
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
# fn view(app: &App) {
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
# fn view(app: &App) {
# let draw = app.draw();

# }
```

If you find the desire to respond to other kinds of events, interact with other
kinds of input/output, track some state, or need more flexibility in general,
you might be interested in creating a [nannou app](./anatomy-of-a-nannou-app.md)
instead! You can also learn more about the difference between sketches and apps
[here](./sketch-vs-app.md).
