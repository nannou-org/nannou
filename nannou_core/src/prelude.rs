//! A collection of commonly used items that are generally useful to have in scope.

pub use crate::color::named::*;
pub use crate::color::{
    gray, hsl, hsla, hsv, hsva, lin_srgb, lin_srgba, rgb, rgb8, rgba, rgba8, srgb, srgb8, srgba,
    srgba8,
};
pub use crate::color::{
    Gray, Hsl, Hsla, Hsv, Hsva, LinSrgb, LinSrgba, Rgb, Rgb8, Rgba, Rgba8, Srgb, Srgba,
};
pub use crate::geom::{
    self, pt2, pt3, vec2, vec3, vec4, Cuboid, Point2, Point3, Rect, Vec2, Vec3, Vec4,
};
#[allow(deprecated)]
pub use crate::geom::{Vector2, Vector3, Vector4};
pub use crate::math::num_traits::*;
pub use crate::math::{
    clamp, deg_to_rad, fmod, map_range, partial_max, partial_min, rad_to_deg, rad_to_turns,
    turns_to_rad, Vec2Angle,
};

// NOTE: These helper functions rely on a thread-local RNG and are currently only available via std.
#[cfg(feature = "std")]
pub use crate::rand::{random, random_ascii, random_f32, random_f64, random_range};

pub use core::f32::consts::{PI, TAU};
pub use core::f64::consts::{PI as PI_F64, TAU as TAU_F64};
