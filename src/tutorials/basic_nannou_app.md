# Making Nothing from Something - Anatomy of a Nannou App
_written by tpltnt_

Nannou is a framework for creative coding in Rust. A framework like a general outline of building blocks to accomplish a goal. If romance stories were a framework, then you have the protagonist, her love interest, some struggles, and a (happy) end as the building blocks. All these need to be fleshed out by the author, but using clichés helps telling a story without having to introduce everyone and everything in incruciating detail. If the author wants to tell a horror story, then the clichés of a romance story aren't very helpful. In the same way you can use nannou to create programs for artisitc expression, but you will find it hard to build an office suite. So let's look at the building blocks for creative coding.

The minimal code for a Nannou app looks like this:
```
extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::run(model, event, view);
}

struct Model;

fn model(_app: &App) -> Model {
    Model
}

fn event(_app: &App, _model: Model, _event: Event) -> Model {
    model
}

fn view(_app: &App, _model: &Model, _frame: Frame) -> Frame {
    frame
}
```

Line 1 reads "extern crate nannou;". This means your program is going to use the functions and data structures provided by nannou. For now it is enough to think about it like enabling all the functionality of nannou into the scope of your code. Line 3 reads "use nannou::prelude::*;". This means all functions of nannou are imported. There is no need to type many lines pulling each module and function of nannou in. The lines 5 to 7 represent the main function of your Rust program. This function is going to be executed first when your application is run. Line 6 reads "nannou::run(model, event, view);" and basically says use the given model, event, and view. This shows the fundamental design archetype of nannou applications. They are build along the Model-View-Controller (MVC) model. This paradigm splits a program into a model (which holds the internal state and how it can be changed), a view (which defines what of the state is seen and how), and a controller (which can change the model). If you zoom out a bit you can think of the computer as a model, the screen as a view (the audio output is a view in that sense), and the keyboard (or mouse) as the controller. A user looks at the view and can change the state of the model using the controller. If a program does not require user input, the controller an use an algorithm to modify the model.
Line 9 defines an empty data structure to represent the model (read: internal state) of your application. A "struct" packs related data together. This struct has no fields and thus is empty. There is no state information to be tracked in this example.
Lines 11 to 13 set up the data structure of the model (in the context of our application). Line 12 yields an empty structure (since no members were defined in Line 12).
Lines 15 to 17 handle the incoming events (from the controller) to update the (internal) model. Here pattern matching is usually leveraged to filter the incoming events and process them properly. Don't worry about the details for now. Line 16 yields the (updated) model to be modified by the next event.
Finally lines 19 to 21 update the view for the app based on the model/internal state and current frame. The current model is not evaluated since it is an empty data structure with no information. This function yields the frame to be drawn. This frame is viewed by the user who in turn may a controller to modify the model as seen fit.

## References
* [wikipedia: Model-View-Controller model](https://en.wikipedia.org/wiki/Model%E2%80%93view%E2%80%93controller)
