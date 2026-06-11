use bevy::prelude::*;

use crate::draw::{Draw, drawing};

/// Dimension properties for **Drawing** a **Primitive**.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Properties {
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub z: Option<f32>,
}

/// Primitives that support different dimensions.
pub trait SetDimensions: Sized {
    /// Provide a mutable reference to the **dimension::Properties** for updating.
    fn properties(&mut self) -> &mut Properties;

    /// Set the absolute width for the primitive.
    fn width(mut self, w: f32) -> Self {
        self.properties().x = Some(w);
        self
    }

    /// Set the absolute height for the primitive.
    fn height(mut self, h: f32) -> Self {
        self.properties().y = Some(h);
        self
    }

    /// Set the absolute depth for the primitive.
    fn depth(mut self, d: f32) -> Self {
        self.properties().z = Some(d);
        self
    }

    /// Short-hand for the **width** method.
    fn w(self, w: f32) -> Self {
        self.width(w)
    }

    /// Short-hand for the **height** method.
    fn h(self, h: f32) -> Self {
        self.height(h)
    }

    /// Short-hand for the **depth** method.
    fn d(self, d: f32) -> Self {
        self.depth(d)
    }

    /// Set the **x** and **y** dimensions for the primitive.
    fn wh(self, v: Vec2) -> Self {
        self.w(v.x).h(v.y)
    }

    /// Set the **x**, **y** and **z** dimensions for the primitive.
    fn whd(self, v: Vec3) -> Self {
        self.w(v.x).h(v.y).d(v.z)
    }

    /// Set the width and height for the primitive.
    fn w_h(self, x: f32, y: f32) -> Self {
        self.wh([x, y].into())
    }

    /// Set the width and height for the primitive.
    fn w_h_d(self, x: f32, y: f32, z: f32) -> Self {
        self.whd([x, y, z].into())
    }
}

impl SetDimensions for Properties {
    fn properties(&mut self) -> &mut Properties {
        self
    }
}

// Set the dimensions of the primitive being drawn at `index`.
//
// `None` leaves the corresponding dimension unchanged.
pub(crate) fn set_dimensions(
    draw: &Draw,
    index: usize,
    x: Option<f32>,
    y: Option<f32>,
    z: Option<f32>,
) {
    drawing::with_primitive(draw, index, |prim| match prim.dimensions_mut() {
        Some(props) => {
            if x.is_some() {
                props.x = x;
            }
            if y.is_some() {
                props.y = y;
            }
            if z.is_some() {
                props.z = z;
            }
        }
        None => bevy::log::warn_once!("drawing primitive does not support `dimensions`"),
    })
}
