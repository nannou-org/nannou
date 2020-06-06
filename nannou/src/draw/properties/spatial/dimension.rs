use crate::geom::{self, Vector2, Vector3};

/// Dimension properties for **Drawing** a **Primitive**.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Properties<S = geom::scalar::Default> {
    pub x: Option<S>,
    pub y: Option<S>,
    pub z: Option<S>,
}

/// Primitives that support different dimensions.
pub trait SetDimensions<S>: Sized {
    /// Provide a mutable reference to the **dimension::Properties** for updating.
    fn properties(&mut self) -> &mut Properties<S>;

    /// Set the absolute width for the primitive.
    fn width(mut self, w: S) -> Self {
        self.properties().x = Some(w);
        self
    }

    /// Set the absolute height for the primitive.
    fn height(mut self, h: S) -> Self {
        self.properties().y = Some(h);
        self
    }

    /// Set the absolute depth for the primitive.
    fn depth(mut self, d: S) -> Self {
        self.properties().z = Some(d);
        self
    }

    /// Short-hand for the **width** method.
    fn w(self, w: S) -> Self {
        self.width(w)
    }

    /// Short-hand for the **height** method.
    fn h(self, h: S) -> Self {
        self.height(h)
    }

    /// Short-hand for the **depth** method.
    fn d(self, d: S) -> Self {
        self.depth(d)
    }

    /// Set the **x** and **y** dimensions for the primitive.
    fn wh(self, v: Vector2<S>) -> Self {
        self.w(v.x).h(v.y)
    }

    /// Set the **x**, **y** and **z** dimensions for the primitive.
    fn whd(self, v: Vector3<S>) -> Self {
        self.w(v.x).h(v.y).d(v.z)
    }

    /// Set the width and height for the primitive.
    fn w_h(self, x: S, y: S) -> Self {
        self.wh(Vector2 { x, y })
    }

    /// Set the width and height for the primitive.
    fn w_h_d(self, x: S, y: S, z: S) -> Self {
        self.whd(Vector3 { x, y, z })
    }
}

impl<S> SetDimensions<S> for Properties<S> {
    fn properties(&mut self) -> &mut Properties<S> {
        self
    }
}

impl<S> Default for Properties<S> {
    fn default() -> Self {
        Self {
            x: None,
            y: None,
            z: None,
        }
    }
}
