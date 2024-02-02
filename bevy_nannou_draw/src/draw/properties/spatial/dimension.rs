use bevy::prelude::*;
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
