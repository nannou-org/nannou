//! A collection of commonly used items that are generally useful to have in scope.

pub use core::f32::consts::{PI, TAU};
pub use core::f64::consts::{PI as PI_F64, TAU as TAU_F64};

pub use crate::geom::{self, Cuboid, Point2, Point3, Rect, pt2, pt3};
#[allow(deprecated)]
pub use crate::geom::{Vector2, Vector3, Vector4};
pub use crate::glam::{
    Affine2, Affine3A, BVec2, BVec3, BVec4, DAffine2, DAffine3, DMat2, DMat3, DMat4, DQuat, DVec2,
    DVec3, DVec4, IVec2, IVec3, IVec4, Mat2, Mat3, Mat3A, Mat4, Quat, UVec2, UVec3, UVec4, Vec2,
    Vec3, Vec3A, Vec4, dmat2, dmat3, dmat4, dquat, dvec2, dvec3, dvec4, ivec2, ivec3, ivec4, mat2,
    mat3, mat3a, mat4, quat, vec2, vec3, vec3a, vec4,
};
pub use crate::math::num_traits::*;
pub use crate::math::{
    Mat4LookTo, Vec2Angle, Vec2Rotate, clamp, deg_to_rad, fmod, map_range, partial_max,
    partial_min, rad_to_deg, rad_to_turns, turns_to_rad,
};
// NOTE: These helper functions rely on a thread-local RNG and are currently only available via std.
#[cfg(feature = "std")]
pub use crate::rand::{random, random_ascii, random_f32, random_f64, random_range};
