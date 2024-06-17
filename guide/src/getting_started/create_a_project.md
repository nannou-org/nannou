# Create A Project

Whether we are creating an artwork, an app, a quick sketch or an installation,
we want to begin by creating a new project. A new nannou project lets us build a
nannou application the way that *we* want to use it.

Eventually, the aim for Nannou is to provide a project generator tool which will
allow us to do the following and much more in just a few clicks. For now, we can
create a new project with just a few small steps:

1. Create the Rust project with the name of our project:

   ```bash
   cargo new my-project
   ```

2. Change directory to the generated project.

   ```bash
   cd my-project
   ```

3. Edit the `Cargo.toml` file and add the latest version of nannou to the bottom
   like so:

   ```toml
   [package]
   name = "my_project"
   version = "0.1.0"
   authors = ["mitchmindtree <mitchell.nordine@gmail.com>"]
   edition = "2018"

   [dependencies]
   nannou = "0.17"
   ```

   Note that there is a chance the nannou version above might be out of date.
   You can check the latest version by typing `cargo search nannou` in your
   terminal. Be sure to change the author to your name too!

4. Replace the code in `src/main.rs` with the following to setup our nannou
   application.

   ```rust,no_run
   # extern crate nannou;
   use nannou::prelude::*;

   fn main() {
       nannou::app(model)
           .update(update)
           .simple_window(view)
           .run();
   }

   struct Model {}

   fn model(_app: &App) -> Model {
       Model {}
   }

   fn update(_app: &App, _model: &mut Model) {
   }

   fn view(app: &App, _model: &Model, _window: Entity) {
       let draw = app.draw();
       draw.background().color(PURPLE);
   }
   ```

   If you are new to Rust or simply do not understand the code above just yet,
   do not fear! In the first tutorial of the next chapter we will break down
   this code step-by-step.

5. Trigger the initial build and check that everything is working nicely by
   running our app!

   ```bash
   cargo run --release
   ```

   The first build might take a while, as we must build nannou and all of its
   dependencies from scratch. The following times that we run our app should be
   much faster!

   Once the project has finished building, it will begin running and we should
   be able to see a purple window.


**That's it!** If everything went as planned, you are now ready to start
building your own nannou project. Of course, we probably want our application to
be more than just a purple window.

To find out how to add more features to our project like graphics, audio,
multiple windows, laser output, projection mapping and much more, let's take a
look at the next chapter.
