//! Parameters which a **Drawing** instance may use to describe certain properties of a drawing.
//!
//! Each time a new method is chained onto a **Drawing** instance, it uses the given values to set
//! one or more properties for the drawing.
//!
//! Each **Drawing** instance is associated with a specific **Node** in the geometry graph and has
//! a unique **node::Index** to simplify this.

use draw;
use geom;
use geom::graph::node;
use math::BaseFloat;
use std::cell::RefCell;

pub mod color;
pub mod primitive;
pub mod spatial;

use self::spatial::dimension;

pub use self::color::{IntoRgba, SetColor};
pub use self::primitive::{Ellipse, Line, Quad, Primitive, Rect, Tri};
pub use self::spatial::dimension::SetDimensions;
pub use self::spatial::position::SetPosition;

/// The scalar type used for the color channel values.
pub type ColorScalar = color::DefaultScalar;

/// The RGBA type used by the `Common` params.
pub type Rgba = color::DefaultRgba;

// Methods for updating **Draw**'s geometry graph and mesh upon completion of **Drawing**.

/// When a **Drawing** type is ready to be built it returns a **Drawn**.
///
/// **Drawn** is the information necessary to populate the parent **Draw** geometry graph and mesh.
pub type Drawn<S, V, I> = (spatial::Properties<S>, V, I);

/// A wrapper around the `draw::State` for the **IntoDrawn** trait implementations.
#[derive(Debug)]
pub struct Draw<'a, S = geom::DefaultScalar>
where
    S: 'a + BaseFloat,
{
    state: RefCell<&'a mut draw::State<S>>,
}

impl<'a, S> Draw<'a, S>
where
    S: BaseFloat,
{
    /// Create a new **Draw**.
    pub fn new(state: &'a mut draw::State<S>) -> Self {
        Draw { state: RefCell::new(state) }
    }

    /// The length of the untransformed node at the given index along the axis returned by the
    /// given `point_axis` function.
    ///
    /// **Note:** If this node's **Drawing** is not yet complete, this method will cause it to
    /// finish and submit the **Drawn** state to the inner geometry graph and mesh.
    pub fn untransformed_dimension_of<F>(&self, n: &node::Index, point_axis: &F) -> Option<S>
    where
        F: Fn(&draw::mesh::vertex::Point<S>) -> S,
    {
        self.state.borrow_mut().untransformed_dimension_of(n, point_axis)
    }

    /// The length of the untransformed node at the given index along the *x* axis.
    pub fn untransformed_x_dimension_of(&self, n: &node::Index) -> Option<S> {
        self.state.borrow_mut().untransformed_x_dimension_of(n)
    }

    /// The length of the untransformed node at the given index along the *y* axis.
    pub fn untransformed_y_dimension_of(&self, n: &node::Index) -> Option<S> {
        self.state.borrow_mut().untransformed_y_dimension_of(n)
    }

    /// The length of the untransformed node at the given index along the *y* axis.
    pub fn untransformed_z_dimension_of(&self, n: &node::Index) -> Option<S> {
        self.state.borrow_mut().untransformed_z_dimension_of(n)
    }

    /// The length of the transformed node at the given index along the axis returned by the given
    /// `point_axis` function.
    ///
    /// **Note:** If this node's **Drawing** is not yet complete, this method will cause it to
    /// finish and submit the **Drawn** state to the inner geometry graph and mesh.
    pub fn dimension_of<F>(&mut self, n: &node::Index, point_axis: &F) -> Option<S>
    where
        F: Fn(&draw::mesh::vertex::Point<S>) -> S,
    {
        self.state.borrow_mut().dimension_of(n, point_axis)
    }

    /// The length of the transformed node at the given index along the *x* axis.
    pub fn x_dimension_of(&self, n: &node::Index) -> Option<S> {
        self.state.borrow_mut().x_dimension_of(n)
    }

    /// The length of the transformed node at the given index along the *y* axis.
    pub fn y_dimension_of(&self, n: &node::Index) -> Option<S> {
        self.state.borrow_mut().y_dimension_of(n)
    }

    /// The length of the transformed node at the given index along the *z* axis.
    pub fn z_dimension_of(&self, n: &node::Index) -> Option<S> {
        self.state.borrow_mut().z_dimension_of(n)
    }

    /// Retrieve the given element from the inner **Theme**.
    pub fn theme<F, T>(&self, get: F) -> T
    where
        F: FnOnce(&draw::Theme) -> T
    {
        let state = self.state.borrow();
        get(&state.theme)
    }
}

/// Types that can be **Drawn** into a parent **Draw** geometry graph and mesh.
pub trait IntoDrawn<S>
where
    S: BaseFloat,
{
    /// The iterator type yielding all unique vertices in the drawing.
    ///
    /// The position of each yielded vertex should be relative to `0, 0, 0` as all displacement,
    /// scaling and rotation transformations will be performed via the geometry graph.
    type Vertices: IntoIterator<Item = draw::mesh::Vertex<S>>;
    /// The iterator type yielding all vertex indices, describing edges of the drawing.
    type Indices: IntoIterator<Item = usize>;
    /// Consume `self` and return its **Drawn** form.
    fn into_drawn(self, Draw<S>) -> Drawn<S, Self::Vertices, Self::Indices>;
}

// Implement a method to simplify retrieving the dimensions of a type from `dimension::Properties`.

impl<S> dimension::Properties<S>
where
    S: BaseFloat,
{
    /// Return the **Dimension**s as scalar values.
    pub fn to_scalars(&self, draw: &Draw<S>) -> (Option<S>, Option<S>, Option<S>) {
        const EXPECT_DIMENSION: &'static str = "no raw dimension for node";
        let x = self.x
            .as_ref()
            .map(|x| x.to_scalar(|n| draw.untransformed_x_dimension_of(n).expect(EXPECT_DIMENSION)));
        let y = self.y
            .as_ref()
            .map(|y| y.to_scalar(|n| draw.untransformed_y_dimension_of(n).expect(EXPECT_DIMENSION)));
        let z = self.z
            .as_ref()
            .map(|z| z.to_scalar(|n| draw.untransformed_z_dimension_of(n).expect(EXPECT_DIMENSION)));
        (x, y, z)
    }
}
