//! Use the `nannou::app::Builder::vulkan_debug_callback` method to specify a callback.
//!
//! Use `Default::default()` to simply use the default callback which will enable all message types
//! including error, warning, performance_warning, information and debug. See the
//! `nannou::vk::DebugCallbackBuilder` docs for more information on how to customise this.
//!
//! If you require specifying custom validation layers, please see
//! `nannou::app::Builder::vulkan_instance` and the `nannou::vk::InstanceBuilder` which will
//! allow you to specify your own custom set of validation layers. To determine what layers are
//! available on a system, see the `nannou::vk::instance::layers_list` function.

use nannou::prelude::*;

fn main() {
    nannou::app(model)
        .vk_debug_callback(Default::default()) // The vulkan debug callback.
        .simple_window(view)
        .run();
}

struct Model;

fn model(_app: &App) -> Model {
    Model
}

fn view(_app: &App, _model: &Model, frame: &Frame) {
    frame.clear(DARK_BLUE);
}
