# Anatomy of a Nannou App

**Tutorial Info**

- Author: tpltnt, mitchmindtree
- Required Knowledge: [Getting Started](/getting_started.md)
- Reading Time: 10 minutes

---


**Nannou is a framework for creative coding in Rust.** A framework can be
thought of as a collection of building blocks to help accomplish a goal.

If romance stories were frameworks, then you might have the protagonist, their
love interest, some struggles, and a happy ending as the building blocks. All of
these need to be fleshed out by the author, but using clichés help to tell a
story without having to introduce everyone and everything in excruciating
detail. If the author wants to tell a horror story, then the clichés of a
romance story aren't very helpful.

In the same way you can use nannou to create programs for artisitc expression,
but you might find it hard to build an office suite. So let's take a look at the
building blocks for creative coding together.

Here's an example of a bare-bones nannou app that opens an empty window:

```rust,no_run
# extern crate nannou;
#
use nannou::prelude::*;

struct Model {}

fn main() {
    nannou::app(model)
        .event(event)
        .simple_window(view)
        .run();
}

fn model(_app: &App) -> Model {
    Model {}
}

fn event(_app: &App, _model: &mut Model, _event: Event) {
}

fn view(_app: &App, _model: &Model, frame: Frame) -> Frame {
    frame
}
```

We will start from the top!

## Import Common Items

```rust,no_run
# #![allow(unused_imports)] 
# extern crate nannou;
use nannou::prelude::*;
# fn main() {}
```

This line imports all of the commonly used items from nannou into scope. These
include items such as `App`, `Frame`, and many more that we will learn about
over time. To see the full list of items re-exported by the prelude, see
[here](https://docs.rs/nannou/latest/nannou/prelude/index.html).

> Note: Unlike some other languages, Rust does not automatically include
> everything from the libraries added to the project. This approach results in
> very clean namespaces and avoids conflicts between different items from
> different crates. That said, it also means we need to manually import every
> item we *do* want to use into scope. By providing a prelude nannou makes it a
> little easier to access all of the commonly used items.

## **Model** - Our app state

```rust,no_run
# #![allow(dead_code)] 
struct Model {}
# fn main() {}
```

The **Model** is where we define the state of our application. We can think of
the model as the representation of our program at any point in time. Throughout
the life of our program, we can update the model as certain events occur such as
mouse presses, key presses or timed updates. We can then present the model using
some kind of output, e.g. by drawing to the screen or outputting to a laser. We
will look at these input and output events in more detail in another tutorial!
Our example is as simple as possible, and we have no state to track. Thus our
model can stay empty.

> Note: A `struct` describes a set of data. Our struct has no fields and thus is
> empty. There is no state information to be tracked in this example.

## **main** - Where Rust programs begin and end

```rust,no_run
# extern crate nannou;
# use nannou::prelude::*;
# struct Model {}
fn main() {
    nannou::app(model)
        .event(event)
        .simple_window(view)
        .run();
}
# fn model(_app: &App) -> Model {
#     Model {}
# }
# fn event(_app: &App, _model: &mut Model, _event: Event) {
# }
# fn view(_app: &App, _model: &Model, frame: Frame) -> Frame {
#     frame
# }
```

All Rust programs begin executing at the start of the `main` function and end
when the `main` function ends. In most nannou programs, the main function is
quite small. In short, we build a description of our app and then run it!

```rust,no_run
# extern crate nannou;
# use nannou::prelude::*;
# struct Model {}
# fn main() {
    nannou::app(model)       // Start building the app and specify our `model`
        .event(event)        // Specify that we want to handle app events with `event`
        .simple_window(view) // Request a simple window to which we'll draw with `view`
        .run();              // Run it!
# }
# fn model(_app: &App) -> Model {
#     Model {}
# }
# fn event(_app: &App, _model: &mut Model, _event: Event) {
# }
# fn view(_app: &App, _model: &Model, frame: Frame) -> Frame {
#     frame
# }
```

We will describe what these **model**, **event** and **view** functions do
below!

> Note: In this app building process we get a hint at the fundamental design
> archetype of nannou apps. The approach is roughly based on the
> [Model-View-Controller (MVC)
> pattern](https://en.wikipedia.org/wiki/Model%E2%80%93view%E2%80%93controller),
> though equally inspired by [Functional Reactive Programming
> (FRP)](https://en.wikipedia.org/wiki/Functional_reactive_programming).
> 
> In general, these paradigms split a program into:
> 
> - a **model** describing the internal state
> - a **view** describing how to present the model and
> - a **controller** describing how to update the model on certain events.
> 
> If you zoom out a bit you can think of the computer as a model, the screen as
> a view (the audio output could also be thought of as a view), and the keyboard
> (or mouse) as the controller. A user looks at the view and can change the
> state of the model using the controller. If a program does not require user
> input, the controller might use an algorithm based on time or some other
> application event to modify the model.

## **model** - initialise our Model

```rust,no_run
# #![allow(dead_code)] 
# extern crate nannou;
# use nannou::prelude::*;
# struct Model {}
fn model(_app: &App) -> Model {
    Model {}
}
# fn main() {}
```

The `model` function is run once at the beginning of the nannou app and produces
a fresh, new instance of the **Model** that we declared previously, AKA the app
state. This can be thought of as the "setup" stage of our application. Here, we
might do things like create some windows, create a GUI, load some images or
anything else that we want to happen once at the beginning of our program. We
will learn how to do all of these things in future tutorials, but for now we
will just return an instance of our empty **Model**.

> Note: To assist with the creation of windows, GUIs, audio streams and other
> kinds of I/O, access to the **App** is provided as an *input* to the function.
> The **App** type can be thought of as a helper type that wraps up the finicky
> details of the application (such as establishing event loops, spawning I/O
> streams, etc) and provides an easy to use, high-level API on top. Providing
> access to the **App** via a function's first argument is a common practise
> throughout nannou's API.
>
> ```rust,no_run
> # #![allow(dead_code)] 
> # extern crate nannou;
> # use nannou::prelude::*;
> # struct Model {}
> //                ----- Access to the `App` passed as an input to the function.
> //               /
> //              v
> fn model(_app: &App) -> Model {
>     Model {}
> }
> # fn main() {}
> ```
>
> You can learn more about what the **App** is responsible for and capable of
> [here](https://docs.rs/nannou/0.9.0/nannou/app/struct.App.html).

## **event** - updating the Model on app events

```rust,no_run
# #![allow(dead_code)] 
# extern crate nannou;
# use nannou::prelude::*;
# struct Model {}
fn event(_app: &App, _model: &mut Model, _event: Event) {
}
# fn main() {}
```

The **event** function is some code that will run every time some kind of app
event occurs. There are many different kinds of app events including mouse and
keyboard presses, window resizes, timed updates and many more. Each of these are
events during which we may wish to update our **Model** in some way. For
example, we may wish to turn a camera when a mouse is moved, begin drawing a
shape when a button is pressed, or step forward an animation on timed updates.

All of these events are described within the **Event** type. One way to
distinguish between which event is currently occurring is to ["pattern
match"](https://doc.rust-lang.org/book/ch06-02-match.html) on the event and
handle only those events that we care about, ignoring all the others. A simpler
approach is to not register an **event** function while building the app at all,
and instead only register more specific functions for those events that we care
about.

For example, if instead of handling *all* events we only want to handle timed
updates (an event that by default occurs 60 times per second) we could change
our app building code to this:

```rust,no_run
# #![allow(dead_code)] 
# extern crate nannou;
# use nannou::prelude::*;
# struct Model {}
fn main() {
    nannou::app(model)
        .update(update) // rather than `.event(event)`, now we only subscribe to updates
        .simple_window(view)
        .run();
}
# fn model(_app: &App) -> Model {
#     Model {}
# }
# fn update(_app: &App, _model: &mut Model, _update: Update) {
# }
# fn view(_app: &App, _model: &Model, frame: Frame) -> Frame {
#     frame
# }
```

And remove our `event` function in favour of an `update` function:

```rust,no_run
# #![allow(dead_code)] 
# extern crate nannou;
# use nannou::prelude::*;
# struct Model {}
fn update(_app: &App, _model: &mut Model, _update: Update) {
}
# fn main() {}
```

Now, our new **update** function will only run each time a timed update
occurs.

> Note: Nannou provides a whole suite of different events that may be registered
> while building an app or window in this way. See the [all_functions.rs
> example](https://github.com/nannou-org/nannou/blob/master/examples/all_functions.rs)
> for a demonstration of most of the different kinds of events that are
> available.

## **view** - presenting the Model to a window

```rust,no_run
# #![allow(dead_code)] 
# extern crate nannou;
# use nannou::prelude::*;
# struct Model {}
fn view(_app: &App, _model: &Model, frame: Frame) -> Frame {
    frame
}
# fn main() {}
```

Finally, the **view** allows us to present the state of the model to a window by
drawing to its **Frame** and returning the frame at the end. Here we can change
the background colour, use the **Draw** API to draw a scene, draw a GUI to the
window or even use the low-level Vulkan API to draw to the frame using our own
graphics pipeline. All of this will be covered by future tutorials.

## Concluding Remarks

Hopefully this has given you a rough idea of how nannou apps work! Do not stress
if some of the syntax looks confusing or some of the specifics still seem
unclear - we will aim to cover these and more in future tutorials :)
