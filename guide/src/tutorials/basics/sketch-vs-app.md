# Basics - `sketch` vs `app`

**Tutorial Info**

- Author: mitchmindtree
- Required Knowledge:
    - [Getting Started](/getting_started.md)
- Reading Time: 7 minutes

---

When creating a new nannou project we have two options for kicking off our
program:

1. `nannou::sketch` and
2. `nannou::app`.

Let's find out exactly what the differences are!

> **Note:** When referring to *app* throughout this tutorial, we are referring to
> a nannou project that is run via `nannou::app`. We are *not* referring to the
> `App` type that often appears as the first argument in nannou functions.
> Hopefully we can point to an `App` oriented tutorial some day soon!

## Sketches

**Sketches** are perfect for quick doodles and casual creations. They only
require a simple `view` function designed to make it easy to start drawing
quickly and easily.

Here is what the [sketch
template](https://github.com/nannou-org/nannou/blob/master/examples/templates/template_sketch.rs)
looks like:

```rust,no_run
use nannou::prelude::*;

fn main() {
    nannou::sketch(view).run()
}

fn view(app: &App) {
    let draw = app.draw();
    draw.background().color(PLUM);
    draw.ellipse().color(STEELBLUE);
    
}
```

While you cannot update or track any custom state, we still have access to
plenty of fun inputs including time, the state of the keyboard, mouse and more.

## Apps

**Apps** are better suited to more sophisticated artworks or even fully fledged
applications. They allow for greater flexibility and finer grained control than
sketches, but also require a little more setup.

Here is what the [app
template](https://github.com/nannou-org/nannou/blob/master/examples/templates/template_app.rs)
looks like:

```rust,no_run
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    _window: Entity,
}

fn model(app: &App) -> Model {
    let _window = app.new_window().view(view).build().unwrap();
    Model { _window }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {}

fn view(app: &App, _model: &Model) {
    let draw = app.draw();
    draw.background().color(PLUM);
    draw.ellipse().color(STEELBLUE);
    
}
```

More specifically, the primary difference is that an app allows for working with
custom ***state*** (i.e. the `Model`), whereas a sketch does not.

> ***Hot tip!***
>
> The line:
>
> ```rust,ignore
> nannou::sketch(view).run()
> ```
> is simply short-hand for
>
> ```rust,ignore
> nannou::app(model).simple_window(view).run()
> ```
> except without the need for `model` and with a slightly simpler `view` function.

## Switching from `sketch` to `app`

In the end it does not make a great deal of difference what you choose, you can
always switch from one to the other in the middle of a project!

If your sketch is getting particularly fancy and you would like to add some more
flexibility, you can turn it into an app by following these steps:

1. Change your `main` function from

   ```rust,no_run
   # #![allow(dead_code)]
   # use nannou::prelude::*;
   # fn main() {
   nannou::sketch(view).run()
   # }
   # fn view(_: &App, _: Frame) {}
   ```

   to

   ```rust,no_run
   # #![allow(dead_code)]
   # use nannou::prelude::*;
   # fn main() {
   nannou::app(model).simple_window(view).run()
   # }
   # struct Model {}
   # fn model(_: &App) -> Model { Model {} }
   # fn view(_: &App, _: &Model, _: Frame) {}
   ```

2. Add a `Model` for tracking state:

   ```rust,no_run
   # #![allow(dead_code)]
   # fn main() {}
   struct Model {}
   ```

3. Add a `model` function for creating the `Model`:

   ```rust,no_Run
   # #![allow(dead_code)]
   # use nannou::prelude::*;
   # fn main() {}
   # struct Model {}
   fn model(_app: &App) -> Model {
       Model {}
   }
   ```

4. Change the `view` function signature from:

   ```rust,no_run
   # #![allow(dead_code, unused_variables)]
   # use nannou::prelude::*;
   # fn main() {}
   fn view(app: &App) {
   # }
   ```

   to

   ```rust,no_run
   # #![allow(dead_code, unused_variables)]
   # use nannou::prelude::*;
   # fn main() {}
   # struct Model {}
   fn view(app: &App, _model: &Model) {
   # }
   ```

And that's it! You are now ready to take your sketch to the next level.

## Overview

|     | **Sketch** | **App** |
| --- | ---------- | ------- |
| Easier to start drawing quickly? | Yes | No |
| Allows for a `Model`? | No | Yes |
| Allows for  audio/LASER/MIDI/etc? | No | Yes |
| The `main` function looks like: | `nannou::sketch(view)` | `nannou::app(model)` |
| Templates | [template_sketch.rs](https://github.com/nannou-org/nannou/blob/master/examples/templates/template_sketch.rs) | [template_app.rs](https://github.com/nannou-org/nannou/blob/master/examples/templates/template_app.rs) |
| Can make awesome stuff? | Yes | Yes |

To learn more about nannou **sketches** visit the [Draw a sketch](/tutorials/basics/draw-a-sketch.md) tutorial.

To learn more about nannou **apps** visit the [Anatomy of a nannou app](/tutorials/basics/anatomy-of-a-nannou-app.md) tutorial.
