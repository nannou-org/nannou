//! A collection of commonly used items that we recommend importing for ease of use.

pub use app::{self, App, LoopMode};
pub use audio;
pub use color::named::*;
pub use color::{Hsl, Hsla, Hsv, Hsva, Rgb, Rgba};
pub use event::WindowEvent::*;
pub use event::{AxisMotion, Event, Key, MouseButton, MouseScrollDelta, TouchEvent, TouchPhase,
                TouchpadPressure, Update, WindowEvent};
pub use frame::{Frame, RawFrame, ViewFbo, ViewFramebufferObject};
pub use geom::{
    self, pt2, pt3, vec2, vec3, vec4, Cuboid, Point2, Point3, Rect, Vector2, Vector3, Vector4,
};
pub use io::{load_from_json, load_from_toml, safe_file_save, save_to_json, save_to_toml};
pub use math::num_traits::*;
pub use math::prelude::*;
pub use math::{
    clamp, deg_to_rad, fmod, map_range, partial_max, partial_min, rad_to_deg, rad_to_turns,
    turns_to_rad,
};
pub use osc;
pub use rand::{random, random_f32, random_f64, random_range};
pub use time::DurationF64;
pub use ui;
pub use vk::{self, DeviceOwned as VulkanDeviceOwned, DynamicStateBuilder, GpuFuture};
pub use window::{self, Id as WindowId};

// The following constants have "regular" names for the `DefaultScalar` type and type suffixes for
// other types. If the `DefaultScalar` changes, these should probably change too.

pub use std::f32::consts::PI;
pub use std::f64::consts::PI as PI_F64;

/// Two times PI.
pub const TAU: f32 = PI * 2.0;
/// Two times PI.
pub const TAU_F64: f64 = PI_F64 * 2.0;
