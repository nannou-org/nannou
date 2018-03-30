//! A collection of commonly used items that we recommend importing for ease of use.

pub use app::{App, LoopMode};
pub use color::named::*;
pub use event::SimpleWindowEvent::*;
pub use event::{Event, Key};
pub use frame::Frame;
pub use math::{Point2, Point3, Vector2, Vector3, vec1, vec2, vec3, vec4, pt1, pt2, pt3};
pub use math::prelude::*;
pub use window::Id as WindowId;

// The following constants have "regular" names for the `DefaultScalar` type and type suffixes for
// other types. If the `DefaultScalar` changes, these should probably change too.

pub use std::f32::consts::PI as PI;
pub use std::f64::consts::PI as PI_F64;

/// Two times PI.
pub const TAU: f32 = PI * 2.0;
/// Two times PI.
pub const TAU_F64: f64 = PI_F64 * 2.0;
