//! Items related to describing positioning along each axis as

use crate::geom::{self, Point2, Point3};
use crate::math::{BaseFloat, Zero};

/// Position properties for **Drawing** a **Primitive**.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Properties<S = geom::scalar::Default> {
    pub point: Point3<S>,
}

/// An API for setting the **position::Properties**.
pub trait SetPosition<S>: Sized {
    /// Provide a mutable reference to the **position::Properties** for updating.
    fn properties(&mut self) -> &mut Properties<S>;

    /// Build with the given **Absolute** **Position** along the *x* axis.
    fn x(mut self, x: S) -> Self {
        self.properties().point.x = x;
        self
    }

    /// Build with the given **Absolute** **Position** along the *y* axis.
    fn y(mut self, y: S) -> Self {
        self.properties().point.y = y;
        self
    }

    /// Build with the given **Absolute** **Position** along the *z* axis.
    fn z(mut self, z: S) -> Self {
        self.properties().point.z = z;
        self
    }

    /// Set the **Position** with some two-dimensional point.
    fn xy(self, p: Point2<S>) -> Self {
        self.x(p.x).y(p.y)
    }

    /// Set the **Position** with some three-dimensional point.
    fn xyz(self, p: Point3<S>) -> Self {
        self.x(p.x).y(p.y).z(p.z)
    }

    /// Set the **Position** with *x* *y* coordinates.
    fn x_y(self, x: S, y: S) -> Self {
        self.xy(Point2 { x, y })
    }

    /// Set the **Position** with *x* *y* *z* coordinates.
    fn x_y_z(self, x: S, y: S, z: S) -> Self {
        self.xyz(Point3 { x, y, z })
    }
}

impl<S> Properties<S> {
    pub fn transform(&self) -> cgmath::Matrix4<S>
    where
        S: BaseFloat,
    {
        cgmath::Matrix4::from_translation(self.point.into())
    }
}

impl<S> SetPosition<S> for Properties<S> {
    fn properties(&mut self) -> &mut Properties<S> {
        self
    }
}

impl<S> Default for Properties<S>
where
    S: Zero,
{
    fn default() -> Self {
        let point = Point3::zero();
        Self { point }
    }
}
