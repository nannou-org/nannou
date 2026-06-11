//! Items related to describing positioning along each axis as
use bevy::prelude::*;

use crate::draw::{Draw, drawing};

/// Position properties for **Drawing** a **Primitive**.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Properties {
    pub point: Vec3,
}

/// An API for setting the **position::Properties**.
pub trait SetPosition: Sized {
    /// Provide a mutable reference to the **position::Properties** for updating.
    fn properties(&mut self) -> &mut Properties;

    /// Build with the given **Absolute** **Position** along the *x* axis.
    fn x(mut self, x: f32) -> Self {
        self.properties().point.x = x;
        self
    }

    /// Build with the given **Absolute** **Position** along the *y* axis.
    fn y(mut self, y: f32) -> Self {
        self.properties().point.y = y;
        self
    }

    /// Build with the given **Absolute** **Position** along the *z* axis.
    fn z(mut self, z: f32) -> Self {
        self.properties().point.z = z;
        self
    }

    /// Set the **Position** with some two-dimensional point.
    fn xy(self, p: Vec2) -> Self {
        self.x(p.x).y(p.y)
    }

    /// Set the **Position** with some three-dimensional point.
    fn xyz(self, p: Vec3) -> Self {
        self.x(p.x).y(p.y).z(p.z)
    }

    /// Set the **Position** with *x* *y* coordinates.
    fn x_y(self, x: f32, y: f32) -> Self {
        self.xy([x, y].into())
    }

    /// Set the **Position** with *x* *y* *z* coordinates.
    fn x_y_z(self, x: f32, y: f32, z: f32) -> Self {
        self.xyz([x, y, z].into())
    }
}

impl Properties {
    pub fn transform(&self) -> Mat4 {
        Mat4::from_translation(self.point.into())
    }
}

impl SetPosition for Properties {
    fn properties(&mut self) -> &mut Properties {
        self
    }
}

impl Default for Properties {
    fn default() -> Self {
        let point = Vec3::ZERO;
        Self { point }
    }
}

// Set the position of the primitive being drawn at `index`.
//
// `None` leaves the corresponding axis unchanged.
pub(crate) fn set_position(
    draw: &Draw,
    index: usize,
    x: Option<f32>,
    y: Option<f32>,
    z: Option<f32>,
) {
    drawing::with_primitive(draw, index, |prim| match prim.position_mut() {
        Some(props) => {
            if let Some(x) = x {
                props.point.x = x;
            }
            if let Some(y) = y {
                props.point.y = y;
            }
            if let Some(z) = z {
                props.point.z = z;
            }
        }
        None => bevy::log::warn_once!("drawing primitive does not support `position`"),
    })
}
