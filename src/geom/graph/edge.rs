//! Items related to the edges of a geometry graph.
use daggy;
use math::{BaseFloat, Euler, Rad, Vector3};

/// Unique index for an **Edge** within a **Graph**.
pub type Index = daggy::EdgeIndex<usize>;

/// An iterator yielding multiple `Index`es.
pub type Indices = daggy::EdgeIndices<usize>;

/// Describes an edge within the geometry graph.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Edge<S> {
    /// The unique kind of edge.
    ///
    /// Represents the combination of `Axis` and `Relative` association described by the edge.
    pub kind: Kind,
    /// A weight whose value's meaning depends on the edge's `Relative` association.
    ///
    /// For `Position` kind edges this represents a relative scalar value.
    /// For `Orientation` kind edges this represents a relative angle in radians.
    /// For `Scale` kind edges this represents the relative scale.
    pub weight: S,
}

/// The unique `Edge` kind - a combo of a `Relative` association and an `Axis`.
///
/// Every incoming (parent) `Edge` for each `Node` in the graph *must* be a unique kind. E.g.
/// it does not make sense for a `Node` to be positioned relatively along the *x* axis to two
/// different parent `Node`s.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Kind {
    /// The axis along which this edge describes some relationship.
    pub axis: Axis,
    /// The relative association described by the edge.
    pub relative: Relative,
}

/// Describes one of the three axes in three-dimensional space.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Axis {
    X,
    Y,
    Z,
}

/// The various possible relative relationships that can be created between nodes.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Relative {
    /// A relative position as a scalar value.
    Position,
    /// A relative orientation in radians.
    Orientation,
    /// A relative scale.
    Scale,
}

impl Kind {
    /// Simple constructor for an edge `Kind`.
    pub fn new(axis: Axis, relative: Relative) -> Self {
        Kind { axis, relative }
    }

    /// Simple constructor for and Edge describing a relative association over the **X** axis.
    pub fn x(relative: Relative) -> Self {
        Kind::new(Axis::X, relative)
    }

    /// Simple constructor for and Edge describing a relative association over the **Y** axis.
    pub fn y(relative: Relative) -> Self {
        Kind::new(Axis::Y, relative)
    }

    /// Simple constructor for and Edge describing a relative association over the **Z** axis.
    pub fn z(relative: Relative) -> Self {
        Kind::new(Axis::Z, relative)
    }

    /// Simple constructor for an Edge describing a relative position along the given axis.
    pub fn position(axis: Axis) -> Self {
        let relative = Relative::Position;
        Kind { axis, relative }
    }

    /// Simple constructor for an Edge describing a relative orientation along the given axis.
    pub fn orientation(axis: Axis) -> Self {
        let relative = Relative::Orientation;
        Kind { axis, relative }
    }

    /// Simple constructor for an Edge describing a relative scale along the given axis.
    pub fn scale(axis: Axis) -> Self {
        let relative = Relative::Scale;
        Kind { axis, relative }
    }

    /// Simple constructor for and Edge describing a relative position over the **X** axis.
    pub fn x_position() -> Self {
        Kind::x(Relative::Position)
    }

    /// Simple constructor for and Edge describing a relative orientation over the **X** axis.
    pub fn x_orientation() -> Self {
        Kind::x(Relative::Orientation)
    }

    /// Simple constructor for and Edge describing a relative scale over the **X** axis.
    pub fn x_scale() -> Self {
        Kind::x(Relative::Scale)
    }

    /// Simple constructor for and Edge describing a relative position over the **Y** axis.
    pub fn y_position() -> Self {
        Kind::y(Relative::Position)
    }

    /// Simple constructor for and Edge describing a relative orientation over the **Y** axis.
    pub fn y_orientation() -> Self {
        Kind::y(Relative::Orientation)
    }

    /// Simple constructor for and Edge describing a relative scale over the **Y** axis.
    pub fn y_scale() -> Self {
        Kind::y(Relative::Scale)
    }

    /// Simple constructor for and Edge describing a relative position over the **Z** axis.
    pub fn z_position() -> Self {
        Kind::z(Relative::Position)
    }

    /// Simple constructor for and Edge describing a relative orientation over the **Z** axis.
    pub fn z_orientation() -> Self {
        Kind::z(Relative::Orientation)
    }

    /// Simple constructor for and Edge describing a relative scale over the **Z** axis.
    pub fn z_scale() -> Self {
        Kind::z(Relative::Scale)
    }
}

impl<S> Edge<S> {
    /// Simple constructor for an `Edge`.
    pub fn new(kind: Kind, weight: S) -> Self {
        Edge { kind, weight }
    }

    /// Simple constructor for an `Edge` describing a relative association over the **X** axis.
    pub fn x(relative: Relative, weight: S) -> Self {
        Edge::new(Kind::x(relative), weight)
    }

    /// Simple constructor for an `Edge` describing a relative association over the **Y** axis.
    pub fn y(relative: Relative, weight: S) -> Self {
        Edge::new(Kind::y(relative), weight)
    }

    /// Simple constructor for an `Edge` describing a relative association over the **Z** axis.
    pub fn z(relative: Relative, weight: S) -> Self {
        Edge::new(Kind::z(relative), weight)
    }

    /// Simple constructor for an `Edge` describing a relative position over the given axis.
    pub fn position(axis: Axis, weight: S) -> Self {
        Edge::new(Kind::position(axis), weight)
    }

    /// Simple constructor for an `Edge` describing a relative orientation over the given axis.
    pub fn orientation(axis: Axis, weight: S) -> Self {
        Edge::new(Kind::orientation(axis), weight)
    }

    /// Simple constructor for an `Edge` describing a relative scale over the given axis.
    pub fn scale(axis: Axis, weight: S) -> Self {
        Edge::new(Kind::scale(axis), weight)
    }

    /// Simple constructor for an `Edge` describing a relative position over the **X** axis.
    pub fn x_position(weight: S) -> Self {
        Edge::new(Kind::x_position(), weight)
    }

    /// Simple constructor for an `Edge` describing a relative orientation over the **X** axis.
    pub fn x_orientation(weight: S) -> Self {
        Edge::new(Kind::x_orientation(), weight)
    }

    /// Simple constructor for an `Edge` describing a relative scale over the **X** axis.
    pub fn x_scale(weight: S) -> Self {
        Edge::new(Kind::x_scale(), weight)
    }

    /// Simple constructor for an `Edge` describing a relative position over the **Y** axis.
    pub fn y_position(weight: S) -> Self {
        Edge::new(Kind::y_position(), weight)
    }

    /// Simple constructor for an `Edge` describing a relative orientation over the **Y** axis.
    pub fn y_orientation(weight: S) -> Self {
        Edge::new(Kind::y_orientation(), weight)
    }

    /// Simple constructor for an `Edge` describing a relative scale over the **Y** axis.
    pub fn y_scale(weight: S) -> Self {
        Edge::new(Kind::y_scale(), weight)
    }

    /// Simple constructor for an `Edge` describing a relative position over the **Z** axis.
    pub fn z_position(weight: S) -> Self {
        Edge::new(Kind::z_position(), weight)
    }

    /// Simple constructor for an `Edge` describing a relative orientation over the **Z** axis.
    pub fn z_orientation(weight: S) -> Self {
        Edge::new(Kind::z_orientation(), weight)
    }

    /// Simple constructor for an `Edge` describing a relative scale over the **Z** axis.
    pub fn z_scale(weight: S) -> Self {
        Edge::new(Kind::z_scale(), weight)
    }
}

/// The three edges describing the given position displacement.
pub fn displace<S>(v: Vector3<S>) -> [Edge<S>; 3] {
    [Edge::x_position(v.x), Edge::y_position(v.y), Edge::z_position(v.z)]
}

/// The three edges describing the given orientation rotation.
pub fn rotate<S: BaseFloat>(e: Euler<Rad<S>>) -> [Edge<S>; 3] {
    [Edge::x_orientation(e.x.0), Edge::y_orientation(e.x.0), Edge::z_orientation(e.z.0)]
}

/// An edge for scaling each axis using the given single scalar scale value.
pub fn scale<S: Copy>(scale: S) -> [Edge<S>; 3] {
    [Edge::x_scale(scale), Edge::y_scale(scale), Edge::z_scale(scale)]
}
