//! Implementation of the **Point** types.
//!
//! Currently, **Point** is simply a type alias for **Vector**. While this makes it easier for new
//! uses to understand and switch between the two, it also conflates the the two mathematical
//! concepts which are quite distinct. It is possible that in the future, we will switch to using
//! distinct types. For now, we are attempting to monitor usage and feedback to determine whether
//! or not this change is worth it.

use geom::vector::{Vector2, Vector3, Vector4};

/// A 2-dimensional point type.
pub type Point2<S> = Vector2<S>;

/// A 3-dimensional point type.
pub type Point3<S> = Vector3<S>;

/// A 4-dimensional point type.
pub type Point4<S> = Vector4<S>;

/// Construct a 2-dimensional point.
pub fn pt2<S>(x: S, y: S) -> Point2<S> {
    Point2 { x, y }
}

/// Construct a 3-dimensional point.
pub fn pt3<S>(x: S, y: S, z: S) -> Point3<S> {
    Point3 { x, y, z }
}

/// Construct a 4-dimensional point.
pub fn pt4<S>(x: S, y: S, z: S, w: S) -> Point4<S> {
    Point4 { x, y, z, w }
}
