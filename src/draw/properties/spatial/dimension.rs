use geom::{self, Vector2, Vector3};
use geom::graph::node;
use math::BaseFloat;

/// Dimension properties for **Drawing** a **Node**.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Properties<S = geom::scalar::Default> {
    /// Dimension over the *x* axis.
    pub x: Option<Dimension<S>>,
    /// Dimension over the *y* axis.
    pub y: Option<Dimension<S>>,
    /// Dimension over the *z* axis.
    pub z: Option<Dimension<S>>,
}

/// The length of a **Node** over either the *x* or *y* axes.
///
/// This type is used to represent the different ways in which a dimension may be sized.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Dimension<S = geom::scalar::Default> {
    /// Some specific length has been given.
    Absolute(S),
    /// The dimension is described as relative to the node at the given index.
    Relative(node::Index, Relative<S>),
}

/// Describes a dimension that is relative to some other node.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Relative<S = geom::scalar::Default> {
    /// Match the exact dimension of the other node.
    Matching,
    /// Match the dimension but pad it with the given Scalar.
    Padded(S),
    /// Multiply the dimension of the other relative node's dimension.
    Scaled(S),
}

/// Nodes that support different dimensions.
pub trait SetDimensions<S>: Sized {
    /// Provide a mutable reference to the **dimension::Properties** for updating.
    fn properties(&mut self) -> &mut Properties<S>;

    // Setters for each axis.

    /// Set the length along the x axis.
    fn x_dimension(mut self, x: Dimension<S>) -> Self {
        self.properties().x = Some(x);
        self
    }

    /// Set the length along the y axis.
    fn y_dimension(mut self, y: Dimension<S>) -> Self {
        self.properties().y = Some(y);
        self
    }

    /// Set the length along the z axis.
    fn z_dimension(mut self, z: Dimension<S>) -> Self {
        self.properties().z = Some(z);
        self
    }

    // Absolute dimensions.

    /// Set the absolute width for the node.
    fn width(self, w: S) -> Self {
        self.x_dimension(Dimension::Absolute(w))
    }

    /// Set the absolute height for the node.
    fn height(self, h: S) -> Self {
        self.y_dimension(Dimension::Absolute(h))
    }

    /// Set the absolute depth for the node.
    fn depth(self, d: S) -> Self {
        self.z_dimension(Dimension::Absolute(d))
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

    /// Set the **x** and **y** dimensions for the node.
    fn wh(self, v: Vector2<S>) -> Self {
        self.w(v.x).h(v.y)
    }

    /// Set the **x**, **y** and **z** dimensions for the node.
    fn whd(self, v: Vector3<S>) -> Self {
        self.w(v.x).h(v.y).d(v.z)
    }

    /// Set the width and height for the node.
    fn w_h(self, x: S, y: S) -> Self {
        self.wh(Vector2 { x, y })
    }

    /// Set the width and height for the node.
    fn w_h_d(self, x: S, y: S, z: S) -> Self {
        self.whd(Vector3 { x, y, z })
    }

    // Relative dimensions.

    /// Some relative dimension along the **x** axis.
    fn x_dimension_relative(self, other: node::Index, x: Relative<S>) -> Self {
        self.x_dimension(Dimension::Relative(other, x))
    }

    /// Some relative dimension along the **y** axis.
    fn y_dimension_relative(self, other: node::Index, y: Relative<S>) -> Self {
        self.y_dimension(Dimension::Relative(other, y))
    }

    /// Some relative dimension along the **z** axis.
    fn z_dimension_relative(self, other: node::Index, z: Relative<S>) -> Self {
        self.z_dimension(Dimension::Relative(other, z))
    }

    /// Set the x-axis dimension as the width of the node at the given index.
    fn w_of(self, other: node::Index) -> Self {
        self.x_dimension_relative(other, Relative::Matching)
    }

    /// Set the y-axis dimension as the height of the node at the given index.
    fn h_of(self, other: node::Index) -> Self {
        self.y_dimension_relative(other, Relative::Matching)
    }

    /// Set the z-axis dimension as the depth of the node at the given index.
    fn d_of(self, other: node::Index) -> Self {
        self.z_dimension_relative(other, Relative::Matching)
    }

    /// Set the dimensions as the dimensions of the node at the given index.
    fn wh_of(self, other: node::Index) -> Self {
        self.w_of(other).h_of(other)
    }

    /// Set the dimensions as the dimensions of the node at the given index.
    fn whd_of(self, other: node::Index) -> Self {
        self.w_of(other).h_of(other).d_of(other)
    }

    /// Set the width as the width of the node at the given index padded at both ends by the
    /// given Scalar.
    fn padded_w_of(self, other: node::Index, pad: S) -> Self {
        self.x_dimension_relative(other, Relative::Padded(pad))
    }

    /// Set the height as the height of the node at the given index padded at both ends by the
    /// given Scalar.
    fn padded_h_of(self, other: node::Index, pad: S) -> Self {
        self.y_dimension_relative(other, Relative::Padded(pad))
    }

    /// Set the depth as the depth of the node at the given index padded at both ends by the
    /// given Scalar.
    fn padded_d_of(self, other: node::Index, pad: S) -> Self {
        self.z_dimension_relative(other, Relative::Padded(pad))
    }

    /// Set the dimensions as the dimensions of the node at the given index with each dimension
    /// padded by the given scalar.
    fn padded_wh_of(self, other: node::Index, pad: S) -> Self
    where
        S: Clone,
    {
        self.padded_w_of(other, pad.clone()).padded_h_of(other, pad)
    }

    /// Set the dimensions as the dimensions of the node at the given index with each dimension
    /// padded by the given scalar.
    fn padded_whd_of(self, other: node::Index, pad: S) -> Self
    where
        S: Clone,
    {
        self.padded_w_of(other, pad.clone())
            .padded_h_of(other, pad.clone())
            .padded_d_of(other, pad)
    }

    /// Set the width as the width of the node at the given index multiplied by the given **scale**
    /// Scalar value.
    fn scaled_w_of(self, other: node::Index, scale: S) -> Self {
        self.x_dimension_relative(other, Relative::Scaled(scale))
    }

    /// Set the height as the height of the node at the given index multiplied by the given **scale**
    /// Scalar value.
    fn scaled_h_of(self, other: node::Index, scale: S) -> Self {
        self.y_dimension_relative(other, Relative::Scaled(scale))
    }

    /// Set the depth as the depth of the node at the given index multiplied by the given **scale**
    /// Scalar value.
    fn scaled_d_of(self, other: node::Index, scale: S) -> Self {
        self.z_dimension_relative(other, Relative::Scaled(scale))
    }

    /// Set the dimensions as the dimensions of the node at the given index multiplied by the given
    /// **scale** Scalar value.
    fn scaled_wh_of(self, other: node::Index, scale: S) -> Self
    where
        S: Clone,
    {
        self.scaled_w_of(other, scale.clone())
            .scaled_h_of(other, scale)
    }

    /// Set the dimensions as the dimensions of the node at the given index multiplied by the given
    /// **scale** Scalar value.
    fn scaled_whd_of(self, other: node::Index, scale: S) -> Self
    where
        S: Clone,
    {
        self.scaled_w_of(other, scale.clone())
            .scaled_h_of(other, scale.clone())
            .scaled_d_of(other, scale)
    }
}

impl<S> SetDimensions<S> for Properties<S> {
    fn properties(&mut self) -> &mut Properties<S> {
        self
    }
}

impl<S> Default for Properties<S> {
    fn default() -> Self {
        let x = None;
        let y = None;
        let z = None;
        Properties { x, y, z }
    }
}

impl<S> Dimension<S>
where
    S: BaseFloat,
{
    /// Return the **Dimension** as a scalar value.
    ///
    /// Relative dimensions are produced by accessing the dimension of some relative node via the
    /// given `dimension_of` function.
    pub fn to_scalar<F>(&self, dimension_of: F) -> S
    where
        F: FnOnce(&node::Index) -> S,
    {
        match *self {
            Dimension::Absolute(s) => s,
            Dimension::Relative(ref n, relative) => match relative {
                Relative::Matching => dimension_of(n),
                Relative::Padded(pad) => dimension_of(n) - pad * (S::one() + S::one()),
                Relative::Scaled(scale) => dimension_of(n) * scale,
            },
        }
    }
}
