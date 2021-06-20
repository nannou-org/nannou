//! Implementation of the **Point** types.
//!
//! Currently, **Point** is simply a type alias for **Vector**. While this makes it easier for new
//! uses to understand and switch between the two, it also conflates the the two mathematical
//! concepts which are quite distinct. It is possible that in the future, we will switch to using
//! distinct types. For now, we are attempting to monitor usage and feedback to determine whether
//! or not this change is worth it.

/// A 2-dimensional point type.
pub type Point2 = glam::Vec2;

/// A 3-dimensional point type.
pub type Point3 = glam::Vec3;

/// A 4-dimensional point type.
pub type Point4 = glam::Vec4;

/// Construct a 2-dimensional point.
pub fn pt2(x: f32, y: f32) -> Point2 {
    (x, y).into()
}

/// Construct a 3-dimensional point.
pub fn pt3(x: f32, y: f32, z: f32) -> Point3 {
    (x, y, z).into()
}

/// Construct a 4-dimensional point.
pub fn pt4(x: f32, y: f32, z: f32, w: f32) -> Point4 {
    (x, y, z, w).into()
}
