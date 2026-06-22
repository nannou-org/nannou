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
//!
//! Nannou is built on the [Bevy](https://bevyengine.org) game engine. The [`nannou::app`](app())
//! and [`nannou::sketch`](sketch()) builders provide the familiar nannou entry points, while
//! [`NannouPlugin`] bundles nannou's functionality as a Bevy [`Plugin`] so it can also be added to
//! an existing Bevy `App`.
//!
//! When using nannou as a Bevy plugin, the [`context::App`] system param gives your own Bevy
//! systems access to nannou's conveniences (time, input, the focused window and the `Draw` API)
//! with no `unsafe` and no builder machinery. See the [`context`] module and the `system_param`
//! example for details.
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::{App as BevyApp, IntoScheduleConfigs, Plugin, PostUpdate};
use bevy::winit::WinitSettings;

pub use find_folder;
pub use lyon;

#[doc(inline)]
pub use nannou_core::{glam, math, rand};

pub use self::context::App;

pub mod app;
mod camera;
pub mod context;
mod frame;
pub mod geom;
pub mod image;
#[cfg(feature = "serde")]
pub mod io;
mod light;
pub mod noise;
pub mod prelude;
mod render;
pub mod sdf {
    pub use nannou_sdf::{
        MaterialId, NannouSdfPlugin, Sdf, SdfBounds, SdfCamera, SdfCapsuleBuilder, SdfConeBuilder,
        SdfConfigBuilder, SdfCuboidBuilder, SdfCylinderBuilder, SdfDebugView, SdfDistanceFormat,
        SdfDrawExt, SdfEllipsoidBuilder, SdfExpression, SdfHandle, SdfKey, SdfLighting,
        SdfPlaneBuilder, SdfQuality, SdfRenderBuilder, SdfRenderSettings, SdfSample,
        SdfSceneBuilder, SdfSphereBuilder, SdfStatus, SdfTerrainBuilder, SdfTerrainParams,
        SdfTorusBuilder, SdfUpdateBudget, SdfWarning, capsule, cone, cuboid, ellipsoid, group,
        interpolate, plane, rounded_cuboid, smootherstep, smoothstep, sphere, terrain, torus,
    };
}
pub mod time;
mod window;

pub use nannou_wgpu as wgpu;

pub struct NannouPlugin;

impl Plugin for NannouPlugin {
    fn build(&self, app: &mut BevyApp) {
        app.add_plugins(nannou_draw::NannouDrawPlugin);
        app.add_plugins(nannou_sdf::NannouSdfPlugin);
        // Keep each default window camera's orthographic z range sized to its window.
        app.add_systems(
            PostUpdate,
            window::update_default_camera_z_range.before(bevy::camera::CameraUpdateSystems),
        );
        // `FramePlugin` extracts per-window scale factors so a `Frame` can be constructed from a
        // (custom or classic) render-world system.
        app.add_plugins(crate::frame::FramePlugin);
        // Ensure the resources the `bevy::App` system param relies on are present regardless of
        // whether `NannouPlugin` is used standalone or via the `nannou::app`/`sketch` builders.
        // `FrameTimeDiagnosticsPlugin` backs `App::fps`; guard against a double-add in case the
        // user has already registered it.
        if !app.is_plugin_added::<FrameTimeDiagnosticsPlugin>() {
            app.add_plugins(FrameTimeDiagnosticsPlugin::default());
        }
        app.init_resource::<WinitSettings>();
        #[cfg(feature = "video")]
        {
            app.add_plugins(nannou_video::NannouVideoPlugin);
        }
    }
}

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
