use geom;
use geom::graph::node;
use math::Point3;

/// Orientation properties for **Drawing** a **Node**.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Properties<S> {
    /// The orientation described by an angle along each axis.
    Axes(Axes<S>),
    /// The orientation described by looking at some other point.
    LookAt(LookAt<S>),
}

/// The orientation along each axis.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Axes<S> {
    pub x: Option<Orientation<S>>,
    pub y: Option<Orientation<S>>,
    pub z: Option<Orientation<S>>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LookAt<S = geom::DefaultScalar> {
    Node(node::Index),
    Point(Point3<S>),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Orientation<S = geom::DefaultScalar> {
    Absolute(S),
    Relative(S, Option<node::Index>)
}

// Expects the `Axes` variant from the given properties.
fn expect_axes<S>(p: &mut Properties<S>) -> &mut Axes<S> {
    match *p {
        Properties::Axes(ref mut axes) => axes,
        Properties::LookAt(_) => panic!("expected `Axes`, found `LookAt`"),
    }
}

impl<S> Properties<S> {
    /// If the `Properties` was set to the `LookAt` variant, this method switches to the `Axes`
    /// variant.
    ///
    /// If the `Properties` is already `Axes`, nothing changes.
    pub fn switch_to_axes(&mut self) {
        if let Properties::LookAt(_) = *self {
            *self = Properties::Axes(Default::default());
        }
    }
}

/// An API for setting the **orientation::Properties**.
pub trait SetOrientation<S>: Sized {
    /// Provide a mutable reference to the **orientation::Properties** for updating.
    fn properties(&mut self) -> &mut Properties<S>;

    // Setters for each axis.

    /// Build with the given **Orientation** along the *x* axis.
    fn x_orientation(mut self, orientation: Orientation<S>) -> Self {
        self.properties().switch_to_axes();
        expect_axes(self.properties()).x = Some(orientation);
        self
    }

    /// Build with the given **Orientation** along the *x* axis.
    fn y_orientation(mut self, orientation: Orientation<S>) -> Self {
        self.properties().switch_to_axes();
        expect_axes(self.properties()).y = Some(orientation);
        self
    }

    /// Build with the given **Orientation** along the *x* axis.
    fn z_orientation(mut self, orientation: Orientation<S>) -> Self {
        self.properties().switch_to_axes();
        expect_axes(self.properties()).z = Some(orientation);
        self
    }
}

impl<S> Default for Properties<S> {
    fn default() -> Self {
        Properties::Axes(Default::default())
    }
}

impl<S> Default for Axes<S> {
    fn default() -> Self {
        Axes {
            x: None,
            y: None,
            z: None,
        }
    }
}
