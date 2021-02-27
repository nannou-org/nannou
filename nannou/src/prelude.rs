//! A collection of commonly used items that we recommend importing for ease of use.

pub use crate::app::{self, App, LoopMode};
pub use crate::color::named::*;
pub use crate::color::{
    gray, hsl, hsla, hsv, hsva, lin_srgb, lin_srgba, rgb, rgb8, rgba, rgba8, srgb, srgb8, srgba,
    srgba8,
};
pub use crate::color::{
    Gray, Hsl, Hsla, Hsv, Hsva, LinSrgb, LinSrgba, Rgb, Rgb8, Rgba, Rgba8, Srgb, Srgba,
};
pub use crate::draw::Draw;
pub use crate::event::WindowEvent::*;
pub use crate::event::{
    AxisMotion, Event, Key, MouseButton, MouseScrollDelta, TouchEvent, TouchPhase,
    TouchpadPressure, Update, WindowEvent,
};
pub use crate::frame::{Frame, RawFrame};
pub use crate::geom::{
    self, pt2, pt3, vec2, vec3, vec4, Cuboid, Point2, Point3, Rect, Vector2, Vector3, Vector4,
};
pub use crate::io::{load_from_json, load_from_toml, safe_file_save, save_to_json, save_to_toml};
pub use crate::math::num_traits::*;
pub use crate::math::prelude::*;
pub use crate::math::{
    clamp, deg_to_rad, fmod, map_range, partial_max, partial_min, rad_to_deg, rad_to_turns,
    turns_to_rad,
};
pub use crate::rand::{random, random_ascii, random_f32, random_f64, random_range};
pub use crate::text::{self, text};
pub use crate::time::DurationF64;
pub use crate::ui;
pub use crate::wgpu;
pub use crate::wgpu::blend::{
    ADD as BLEND_ADD, DARKEST as BLEND_DARKEST, LIGHTEST as BLEND_LIGHTEST, NORMAL as BLEND_NORMAL,
    REVERSE_SUBTRACT as BLEND_REVERSE_SUBTRACT, SUBTRACT as BLEND_SUBTRACT,
};
pub use crate::wgpu::util::{BufferInitDescriptor, DeviceExt};
pub use crate::window::{self, Id as WindowId};
pub use crate::window::{Fullscreen, Window};

// The following constants have "regular" names for the `DefaultScalar` type and type suffixes for
// other types. If the `DefaultScalar` changes, these should probably change too.

pub use std::f32::consts::PI;
pub use std::f64::consts::PI as PI_F64;

/// Two times PI.
pub const TAU: f32 = PI * 2.0;
/// Two times PI.
pub const TAU_F64: f64 = PI_F64 * 2.0;
