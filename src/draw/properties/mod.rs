//! Parameters which a **Drawing** instance may use to describe certain properties of a drawing.
//!
//! Each time a new method is chained onto a **Drawing** instance, it uses the given values to set
//! one or more properties for the drawing.
//!
//! Each **Drawing** instance is associated with a specific **Node** in the geometry graph and has
//! a unique **node::Index** to simplify this.

use crate::draw;
use crate::geom;
use crate::geom::graph::node;
use crate::math::BaseFloat;
use std::cell::RefCell;
use std::ops;

pub mod color;
pub mod primitive;
pub mod spatial;

use self::spatial::dimension;

pub use self::color::{IntoLinSrgba, SetColor};
pub use self::primitive::{Ellipse, Line, Primitive, Quad, Rect, Tri};
pub use self::spatial::dimension::SetDimensions;
pub use self::spatial::orientation::SetOrientation;
pub use self::spatial::position::SetPosition;

/// The scalar type used for the color channel values.
pub type ColorScalar = crate::color::DefaultScalar;

/// The RGBA type used by the `Common` params.
pub type Srgba = color::DefaultSrgba;

/// The RGBA type used by the `Common` params.
pub type LinSrgba = color::DefaultLinSrgba;

// Methods for updating **Draw**'s geometry graph and mesh upon completion of **Drawing**.

/// When a **Drawing** type is ready to be built it returns a **Drawn**.
///
/// **Drawn** is the information necessary to populate the parent **Draw** geometry graph and mesh.
pub type Drawn<S, V, I> = (spatial::Properties<S>, V, I);

/// A wrapper around the `draw::State` for the **IntoDrawn** trait implementations.
#[derive(Debug)]
pub struct Draw<'a, S = geom::scalar::Default>
where
    S: 'a + BaseFloat,
{
    state: RefCell<&'a mut draw::State<S>>,
}

/// Uses a set of ranges to index into the intermediary mesh and produce vertices.
#[derive(Debug)]
pub struct VerticesFromRanges {
    pub ranges: draw::IntermediaryVertexDataRanges,
    pub fill_color: Option<draw::mesh::vertex::Color>,
}

/// Uses a range to index into the intermediary mesh indices.
#[derive(Debug)]
pub struct IndicesFromRange {
    pub range: ops::Range<usize>,
}

impl<'a, S> Draw<'a, S>
where
    S: BaseFloat,
{
    /// Create a new **Draw**.
    pub fn new(state: &'a mut draw::State<S>) -> Self {
        Draw {
            state: RefCell::new(state),
        }
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
        self.state
            .borrow_mut()
            .untransformed_dimension_of(n, point_axis)
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
        F: FnOnce(&draw::Theme) -> T,
    {
        let state = self.state.borrow();
        get(&state.theme)
    }
}

/// Similar to the `Iterator` trait, but provides access to the **IntermediaryMesh** on each call
/// to the **next** method.
pub trait Vertices<S>: Sized {
    /// Return the next **Vertex** within the sequence.
    fn next(&mut self, data: &draw::IntermediaryMesh<S>) -> Option<draw::mesh::Vertex<S>>;
    /// Converts `self` and the given `data` into an iterator yielding vertices.
    fn into_iter(self, data: &draw::IntermediaryMesh<S>) -> IterVertices<Self, S> {
        IterVertices {
            vertices: self,
            data,
        }
    }
}

/// Similar to the `Iterator` trait, but provides access to the **IntermediaryMesh** on each call
/// to the **next** method.
pub trait Indices: Sized {
    /// Return the next index within the sequence.
    fn next(&mut self, intermediary_indices: &[usize]) -> Option<usize>;
    /// Converts `self` and the given `intermediary_indices` into an iterator yielding indices.
    fn into_iter(self, intermediary_indices: &[usize]) -> IterIndices<Self> {
        IterIndices {
            indices: self,
            intermediary_indices,
        }
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
    type Vertices: Vertices<S>;
    /// The iterator type yielding all vertex indices, describing edges of the drawing.
    type Indices: Indices;
    /// Consume `self` and return its **Drawn** form.
    fn into_drawn(self, _: Draw<S>) -> Drawn<S, Self::Vertices, Self::Indices>;
}

/// An iterator adaptor around a type implementing the **Vertices** trait and the
/// **IntermediaryMesh** necessary for producing vertices.
pub struct IterVertices<'a, V, S: 'a> {
    vertices: V,
    data: &'a draw::IntermediaryMesh<S>,
}

/// An iterator adaptor around a type implementing the **Vertices** trait and the
/// **IntermediaryMesh** necessary for producing vertices.
pub struct IterIndices<'a, I> {
    indices: I,
    intermediary_indices: &'a [usize],
}

// Implement a method to simplify retrieving the dimensions of a type from `dimension::Properties`.

impl<S> dimension::Properties<S>
where
    S: BaseFloat,
{
    /// Return the **Dimension**s as scalar values.
    pub fn to_scalars(&self, draw: &Draw<S>) -> (Option<S>, Option<S>, Option<S>) {
        const EXPECT_DIMENSION: &'static str = "no raw dimension for node";
        let x = self.x.as_ref().map(|x| {
            x.to_scalar(|n| {
                draw.untransformed_x_dimension_of(n)
                    .expect(EXPECT_DIMENSION)
            })
        });
        let y = self.y.as_ref().map(|y| {
            y.to_scalar(|n| {
                draw.untransformed_y_dimension_of(n)
                    .expect(EXPECT_DIMENSION)
            })
        });
        let z = self.z.as_ref().map(|z| {
            z.to_scalar(|n| {
                draw.untransformed_z_dimension_of(n)
                    .expect(EXPECT_DIMENSION)
            })
        });
        (x, y, z)
    }
}

impl<S, I> Vertices<S> for I
where
    I: Iterator<Item = draw::mesh::Vertex<S>>,
{
    fn next(&mut self, _data: &draw::IntermediaryMesh<S>) -> Option<draw::mesh::Vertex<S>> {
        self.next()
    }
}

impl<I> Indices for I
where
    I: Iterator<Item = usize>,
{
    fn next(&mut self, _intermediary_indices: &[usize]) -> Option<usize> {
        self.next()
    }
}

impl<'a, V, S> Iterator for IterVertices<'a, V, S>
where
    V: Vertices<S>,
{
    type Item = draw::mesh::Vertex<S>;
    fn next(&mut self) -> Option<Self::Item> {
        let IterVertices {
            ref mut vertices,
            data,
        } = *self;
        Vertices::next(vertices, data)
    }
}

impl<'a, I> Iterator for IterIndices<'a, I>
where
    I: Indices,
{
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        let IterIndices {
            ref mut indices,
            intermediary_indices,
        } = *self;
        Indices::next(indices, intermediary_indices)
    }
}

impl<S> Vertices<S> for VerticesFromRanges
where
    S: BaseFloat,
{
    fn next(&mut self, mesh: &draw::IntermediaryMesh<S>) -> Option<draw::mesh::Vertex<S>> {
        let VerticesFromRanges {
            ref mut ranges,
            fill_color,
        } = *self;

        let point = Iterator::next(&mut ranges.points);
        let color = Iterator::next(&mut ranges.colors);
        let tex_coords = Iterator::next(&mut ranges.tex_coords);

        let point = match point {
            None => return None,
            Some(point_ix) => *mesh
                .vertex_data
                .points
                .get(point_ix)
                .expect("no point for point index in IntermediaryMesh"),
        };

        let color = color
            .map(|color_ix| {
                *mesh
                    .vertex_data
                    .colors
                    .get(color_ix)
                    .expect("no color for color index in IntermediaryMesh")
            })
            .or(fill_color)
            .expect("no color for vertex");

        let tex_coords = tex_coords
            .map(|tex_coords_ix| {
                *mesh
                    .vertex_data
                    .tex_coords
                    .get(tex_coords_ix)
                    .expect("no tex_coords for tex_coords index in IntermediaryMesh")
            })
            .unwrap_or_else(draw::mesh::vertex::default_tex_coords);

        Some(draw::mesh::vertex::new(point, color, tex_coords))
    }
}

impl Indices for IndicesFromRange {
    fn next(&mut self, intermediary_indices: &[usize]) -> Option<usize> {
        Iterator::next(&mut self.range).map(|ix| {
            *intermediary_indices
                .get(ix)
                .expect("index into `intermediary_indices` is out of range")
        })
    }
}
