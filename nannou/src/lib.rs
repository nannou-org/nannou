#![doc(
    html_logo_url = "https://raw.githubusercontent.com/nannou-org/nannou/master/assets/images/logo.png"
)]

//! An open-source creative-coding toolkit for Rust.
//!
//! [**Nannou**](http://nannou.cc) is a collection of code aimed at making it easy for artists to
//! express themselves with simple, fast, reliable, portable code. Whether working on a 12-month
//! laser installation or a 5 minute sketch, this framework aims to give artists easy access to the
//! tools they need.
//!
//! If you're new to nannou, we recommend checking out [the
//! examples](https://github.com/nannou-org/nannou/tree/master/examples) to get an idea of how
//! nannou applications are structured and how the API works.
#![cfg_attr(feature = "nightly", feature(adt_const_params))]
pub use find_folder;
pub use lyon;

pub use self::app::App;
#[doc(inline)]
pub use nannou_core::{glam, math, rand};

pub mod app;
pub mod event;
pub mod geom;
pub mod io;
pub mod noise;
pub mod prelude;
pub mod time;
mod window;
pub mod image;

/// Begin building the `App`.
///
/// The `model` argument is the function that the App will call to initialise your Model.
///
/// The Model can be thought of as the state that you would like to track throughout the
/// lifetime of your nannou program from start to exit.
///
/// The given function is called before any event processing begins within the application.
///
/// The Model that is returned by the function is the same model that will be passed to the
/// given event and view functions.
pub fn app<M: 'static + Send>(model: app::ModelFn<M>) -> app::Builder<M> {
    app::Builder::new(model)
}

/// Shorthand for building a simple app that has no model, handles no events and simply draws
/// to a single window.
///
/// This is useful for late night hack sessions where you just don't care about all that other
/// stuff, you just want to play around with some ideas or make something pretty.
pub fn sketch(view: app::SketchViewFn) -> app::SketchBuilder {
    app::Builder::sketch(view)
}
