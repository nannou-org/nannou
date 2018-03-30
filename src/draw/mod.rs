use geom;
use geom::graph::{edge, node};
use math::{BaseFloat, Vector3};
use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::mem;
use std::ops;

use self::properties::spatial::position::{self, Position};
use self::properties::{IntoDrawn, Primitive};
pub use self::background::Background;
pub use self::drawing::Drawing;
pub use self::mesh::Mesh;
pub use self::theme::Theme;

pub mod backend;
pub mod background;
mod drawing;
pub mod properties;
pub mod mesh;
pub mod theme;

/// A simple API for drawing 2D and 3D graphics.
///
/// **Draw** provides a simple way to compose together geometric primitives and text (TODO) with
/// custom colours and textures and draw them to the screen.
///
/// You can also ask **Draw** for the sequence of vertices or triangles (with or without
/// colours/textures) that make up the entire scene that you have created.
///
/// Internally **Draw** uses a **geom::Graph** for placing geometry and text in 3D space.
///
/// **Draw** has 2 groups of methods:
///
/// 1. **Creation**: These methods compose new geometry and text with colours and textures.
///
/// 2. **Rendering**: These methods provide ways of rendering the graphics either directly to the
///    frame for the current display or to a list of vertices or triangles for lower-level, more
///    flexible access.
#[derive(Clone, Debug)]
pub struct Draw<S = geom::DefaultScalar>
where
    S: BaseFloat,
{
    // The state of the **Draw** behind a RefCell. We do this in order to avoid requiring a `mut`
    // handle to a `draw`. The primary purpose of a **Draw** is to be an easy-as-possible,
    // high-level API for drawing stuff. In order to be friendlier to new users, we want to avoid
    // them having to think about mutability and focus on creativity. Rust-lang nuances can come
    // later.
    state: RefCell<State<S>>,
}

/// The inner state of the **Draw** type.
///
/// The **Draw** type stores its **State** behind a **RefCell** - a type used for moving mutability
/// checks from compile time to runtime. We do this in order to avoid requiring a `mut` handle to a
/// `draw`. The primary purpose of a **Draw** is to be an easy-as-possible, high-level API for
/// drawing stuff. In order to be friendlier to new users, we want to avoid requiring them to think
/// about mutability and instead focus on creativity. Rust-lang nuances can come later.
#[derive(Clone, Debug)]
pub struct State<S = geom::DefaultScalar>
where
    S: BaseFloat,
{
    /// Relative positioning, orientation and scaling of geometry.
    geom_graph: geom::Graph<S>,
    /// For performing a depth-first search over the geometry graph.
    geom_graph_dfs: RefCell<geom::graph::node::Dfs<S>>,
    /// The mesh containing vertices for all drawn shapes, etc.
    mesh: Mesh<S>,
    /// The map from node indices to their vertex and index ranges within the mesh.
    ranges: HashMap<node::Index, Ranges>,
    /// Primitives that are in the process of being drawn.
    drawing: HashMap<node::Index, properties::Primitive<S>>,
    /// The last node that was **Drawn**.
    last_node_drawn: Option<node::Index>,
    /// The theme containing default values.
    theme: Theme,
    /// If `Some`, the **Draw** should first clear the frame's gl context with the given color.
    background_color: Option<properties::Rgba>,
}

/// The vertex and index ranges into a mesh for a particular node.
#[derive(Clone, Debug)]
struct Ranges {
    vertices: ops::Range<usize>,
    indices: ops::Range<usize>,
}

const WOULD_CYCLE: &'static str =
    "drawing the given primitive with the given relative positioning would have caused a cycle \
    within the geometry graph";

/// An iterator yielding the transformed, indexed vertices for a node.
pub type NodeVertices<'a, S = geom::DefaultScalar> =
    node::TransformedVertices<::mesh::Vertices<Ref<'a, Mesh<S>>>, S>;

// /// An iterator yielding the transformed vertices for a node.
// pub struct NodeVertices<'a, S> {
// }

/// An iterator yielding the transformed raw vertices for a node.
pub type RawNodeVertices<'a, S = geom::DefaultScalar> =
    node::TransformedVertices<::mesh::RawVertices<Ref<'a, Mesh<S>>>, S>;

/// An iterator yielding the transformed triangles for a node.
pub type NodeTriangles<'a, S = geom::DefaultScalar> =
    geom::tri::IterFromVertices<NodeVertices<'a, S>>;

/// An iterator yielding all indexed mesh vertices transformed via the geometry graph.
#[derive(Debug)]
pub struct Vertices<'a, S = geom::DefaultScalar>
where
    S: 'a + BaseFloat,
{
    draw: &'a Draw<S>,
    node_vertices: Option<NodeVertices<'a, S>>,
}

/// An iterator yielding all indexed mesh triangles transformed via the geometry graph.
pub type Triangles<'a, S = geom::DefaultScalar> = geom::tri::IterFromVertices<Vertices<'a, S>>;

/// An iterator yielding all raw mesh vertices transformed via the geometry graph.
#[derive(Debug)]
pub struct RawVertices<'a, S = geom::DefaultScalar>
where
    S: 'a + BaseFloat,
{
    draw: &'a Draw<S>,
    node_vertices: Option<RawNodeVertices<'a, S>>,
}

// Given some `position` along the given axis return the resulting geom::Graph edge and the parent.
fn position_to_edge<S, F>(
    node_index: node::Index,
    position: &Position<S>,
    draw: &mut State<S>,
    axis: edge::Axis,
    point_axis: &F,
) -> (geom::graph::Edge<S>, node::Index)
where
    S: BaseFloat,
    F: Fn(&mesh::vertex::Point<S>) -> S,
{
    match *position {
        // *s* relative to *origin*.
        Position::Absolute(s) => {
            let edge = geom::graph::Edge::position(axis, s);
            let origin = draw.geom_graph.origin();
            (edge, origin)
        }

        Position::Relative(relative, maybe_parent) => {
            let parent = maybe_parent
                .or(draw.last_node_drawn)
                .unwrap_or(draw.geom_graph.origin());
            let edge = match relative {
                // Relative position.
                position::Relative::Scalar(s) => geom::graph::Edge::position(axis, s),

                // Align end with
                position::Relative::Align(align) => match align {
                    position::Align::Middle => {
                        let zero = S::zero();
                        geom::graph::Edge::position(axis, zero)
                    }
                    align => {
                        let one = S::one();
                        let (direction, margin) = match align {
                            position::Align::Start(mgn) => (-one, mgn.unwrap_or(S::zero())),
                            position::Align::End(mgn) => (one, mgn.unwrap_or(S::zero())),
                            _ => unreachable!(),
                        };
                        let node_dimension =
                            draw.untransformed_dimension_of(&node_index, point_axis).unwrap();
                        let parent_dimension = draw.dimension_of(&parent, point_axis)
                            .expect("no node for relative position");
                        let half = S::from(0.5).unwrap();
                        let node_half_dim = node_dimension * half;
                        let parent_half_dim = parent_dimension * half;
                        let weight = direction * (parent_half_dim - node_half_dim - margin);
                        geom::graph::Edge::position(axis, weight)
                    }
                },

                position::Relative::Direction(direction, amt) => {
                    let one = S::one();
                    let direction = match direction {
                        position::Direction::Backwards => -one,
                        position::Direction::Forwards => one,
                    };
                    let node_dimension = draw.untransformed_dimension_of(&node_index, point_axis).unwrap();
                    let parent_dimension = draw.dimension_of(&parent, point_axis)
                        .expect("no node for relative position");
                    let half = S::from(0.5).unwrap();
                    let node_half_dim = node_dimension * half;
                    let parent_half_dim = parent_dimension * half;
                    let weight = direction * (parent_half_dim + node_half_dim + amt);
                    geom::graph::Edge::position(axis, weight)
                }
            };
            (edge, parent)
        }
    }
}

fn point_x<S: Clone>(p: &mesh::vertex::Point<S>) -> S {
    p.x.clone()
}
fn point_y<S: Clone>(p: &mesh::vertex::Point<S>) -> S {
    p.y.clone()
}
fn point_z<S: Clone>(p: &mesh::vertex::Point<S>) -> S {
    p.z.clone()
}

// Convert the given `drawing` into its **Drawn** state and insert it into the mesh and geometry
// graph.
fn into_drawn<T, S>(
    draw: &mut State<S>,
    node_index: node::Index,
    drawing: T,
) -> Result<(), geom::graph::WouldCycle<S>>
where
    T: IntoDrawn<S>,
    S: BaseFloat,
{
    // Convert the target into its **Drawn** state.
    let (spatial, vertices, indices) = drawing.into_drawn(properties::Draw::new(draw));

    // Update the mesh with the non-transformed vertices.
    let vertices_start_index = draw.mesh.raw_vertex_count();
    let indices_start_index = draw.mesh.indices().len();
    let indices = indices.into_iter().map(|i| vertices_start_index + i);
    draw.mesh.extend(vertices, indices);

    // Update the **Draw**'s range map.
    let vertices_end_index = draw.mesh.raw_vertex_count();
    let indices_end_index = draw.mesh.indices().len();
    let vertices = vertices_start_index..vertices_end_index;
    let indices = indices_start_index..indices_end_index;
    let ranges = Ranges { vertices, indices };
    draw.ranges.insert(node_index, ranges);

    // Update the edges within the geometry graph.
    let p = &spatial.position;
    let x = p.x
        .map(|pos| (pos, edge::Axis::X, point_x as fn(&mesh::vertex::Point<S>) -> S));
    let y = p.y.map(|pos| (pos, edge::Axis::Y, point_y as _));
    let z = p.z.map(|pos| (pos, edge::Axis::Z, point_z as _));
    let positions = x.into_iter().chain(y).chain(z);
    for (position, axis, point_axis) in positions {
        let (edge, parent) =
            position_to_edge(node_index, &position, draw, axis, &point_axis);
        draw.geom_graph.set_edge(parent, node_index, edge)?;
    }

    // Set this node as the last drawn node.
    draw.last_node_drawn = Some(node_index);

    Ok(())
}

// Convert the given `primitive` into its **Drawn** state and insert it into the mesh and geometry
// graph.
fn draw_primitive<S>(
    draw: &mut State<S>,
    node_index: node::Index,
    primitive: Primitive<S>,
) -> Result<(), geom::graph::WouldCycle<S>>
where
    S: BaseFloat,
{
    match primitive {
        Primitive::Ellipse(prim) => {
            into_drawn(draw, node_index, prim)
        },
        Primitive::Quad(prim) => {
            into_drawn(draw, node_index, prim)
        }
        Primitive::Rect(prim) => {
            into_drawn(draw, node_index, prim)
        }
        Primitive::Tri(prim) => {
            into_drawn(draw, node_index, prim)
        }
    }
}

// Produce the min and max over the axis yielded via `point_axis` for the given `points`.
fn min_max_dimension<I, F, S>(points: I, point_axis: &F) -> Option<(S, S)>
where
    I: IntoIterator<Item = mesh::vertex::Point<S>>,
    F: Fn(&mesh::vertex::Point<S>) -> S,
    S: BaseFloat,
{
    let mut points = points.into_iter();
    points.next().map(|first| {
        let s = point_axis(&first);
        let init = (s, s);
        points.fold(init, |(min, max), p| {
            let s = point_axis(&p);
            (s.min(min), s.max(max))
        })
    })
}

impl<S> State<S>
where
    S: BaseFloat,
{
    // Resets all state within the `Draw` instance.
    fn reset(&mut self) {
        self.geom_graph.clear();
        self.geom_graph_dfs.borrow_mut().reset(&self.geom_graph);
        self.drawing.clear();
        self.ranges.clear();
        self.mesh.clear();
        self.background_color = None;
        self.last_node_drawn = None;
    }

    // // Produce the transformed mesh vertices for the node at the given index.
    // //
    // // Returns **None** if there is no node for the given index.
    // fn node_vertices(&mut self, n: node::Index) -> Option<NodeVertices<S>> {

    // }

    // Drain any remaining `drawing`s, convert them to their **Drawn** state and insert them into
    // the inner mesh and geometry graph.
    fn finish_remaining_drawings(&mut self) -> Result<(), geom::graph::WouldCycle<S>> {
        let mut drawing = mem::replace(&mut self.drawing, Default::default());
        for (node_index, primitive) in drawing.drain() {
            draw_primitive(self, node_index, primitive)?;
        }
        mem::replace(&mut self.drawing, drawing);
        Ok(())
    }

    // Finish the drawing at the given node index if it is not yet complete.
    fn finish_drawing(&mut self, n: &node::Index) -> Result<(), geom::graph::WouldCycle<S>> {
        if let Some(primitive) = self.drawing.remove(n) {
            draw_primitive(self, *n, primitive)?;
        }
        Ok(())
    }

    // The length of the untransformed node at the given index along the axis returned by the
    // given `point_axis` function.
    //
    // **Note:** If this node's **Drawing** is not yet complete, this method will cause it to
    // finish and submit the **Drawn** state to the inner geometry graph and mesh.
    fn untransformed_dimension_of<F>(&mut self, n: &node::Index, point_axis: &F) -> Option<S>
    where
        F: Fn(&mesh::vertex::Point<S>) -> S,
    {
        self.finish_drawing(n).expect(WOULD_CYCLE);
        self.ranges.get(n).and_then(|ranges| {
            let points = self.mesh.points()[ranges.vertices.clone()].iter().cloned();
            min_max_dimension(points, point_axis).map(|(min, max)| max - min)
        })
    }

    // The length of the untransformed node at the given index along the *x* axis.
    fn untransformed_x_dimension_of(&mut self, n: &node::Index) -> Option<S> {
        self.untransformed_dimension_of(n, &point_x)
    }

    // The length of the untransformed node at the given index along the *y* axis.
    fn untransformed_y_dimension_of(&mut self, n: &node::Index) -> Option<S> {
        self.untransformed_dimension_of(n, &point_y)
    }

    // The length of the untransformed node at the given index along the *y* axis.
    fn untransformed_z_dimension_of(&mut self, n: &node::Index) -> Option<S> {
        self.untransformed_dimension_of(n, &point_z)
    }

    // The length of the transformed node at the given index along the axis returned by the given
    // `point_axis` function.
    //
    // **Note:** If this node's **Drawing** is not yet complete, this method will cause it to
    // finish and submit the **Drawn** state to the inner geometry graph and mesh.
    fn dimension_of<F>(&mut self, n: &node::Index, point_axis: &F) -> Option<S>
    where
        F: Fn(&mesh::vertex::Point<S>) -> S,
    {
        self.finish_drawing(n).expect(WOULD_CYCLE);
        self.ranges.get(n).and_then(|ranges| {
            let points = self.mesh.points()[ranges.vertices.clone()].iter().cloned();
            let points = self.geom_graph
                .node_vertices(*n, points)
                .expect("no node at index");
            min_max_dimension(points, point_axis).map(|(min, max)| max - min)
        })
    }

    // The length of the transformed node at the given index along the *x* axis.
    fn x_dimension_of(&mut self, n: &node::Index) -> Option<S> {
        self.dimension_of(n, &point_x)
    }

    // The length of the transformed node at the given index along the *y* axis.
    fn y_dimension_of(&mut self, n: &node::Index) -> Option<S> {
        self.dimension_of(n, &point_y)
    }

    // The length of the transformed node at the given index along the *z* axis.
    fn z_dimension_of(&mut self, n: &node::Index) -> Option<S> {
        self.dimension_of(n, &point_z)
    }
}

impl<S> Draw<S>
where
    S: BaseFloat,
{
    /// Create a new **Draw** instance.
    ///
    /// This is the same as calling **Draw::default**.
    pub fn new() -> Self {
        Self::default()
    }

    /// Resets all state within the `Draw` instance.
    pub fn reset(&self) {
        self.state.borrow_mut().reset();
    }

    // Primitive geometry.

    /// Specify a color with which the background should be cleared.
    pub fn background(&self) -> Background<S> {
        background::new(self)
    }

    /// Add the given type to be drawn.
    pub fn a<T>(&self, primitive: T) -> Drawing<T, S>
    where
        T: IntoDrawn<S> + Into<Primitive<S>>,
        Primitive<S>: Into<Option<T>>,
    {
        let index = self.state.borrow_mut().geom_graph.add_node(geom::graph::Node::Point);
        let primitive: Primitive<S> = primitive.into();
        self.state.borrow_mut().drawing.insert(index, primitive);
        drawing::new(self, index)
    }

    /// Begin drawing an **Ellipse**.
    pub fn ellipse(&self) -> Drawing<properties::Ellipse<S>, S> {
        self.a(Default::default())
    }

    /// Begin drawing a **Quad**.
    pub fn quad(&self) -> Drawing<properties::Quad<S>, S> {
        self.a(Default::default())
    }

    /// Begin drawing a **Rect**.
    pub fn rect(&self) -> Drawing<properties::Rect<S>, S> {
        self.a(Default::default())
    }

    /// Begin drawing a **Triangle**.
    pub fn tri(&self) -> Drawing<properties::Tri<S>, S> {
        self.a(Default::default())
    }

    /// Produce the transformed mesh vertices for the node at the given index.
    ///
    /// Returns **None** if there is no node for the given index.
    pub fn node_vertices(&self, n: node::Index) -> Option<NodeVertices<S>> {
        self.state.borrow_mut().finish_drawing(&n).expect(WOULD_CYCLE);
        let index_range = match self.state.borrow().ranges.get(&n) {
            None => return None,
            Some(ranges) => ranges.indices.clone(),
        };
        let vertices = ::mesh::vertices(self.mesh()).index_range(index_range);
        self.state.borrow().geom_graph.node_vertices(n, vertices)
    }

    /// Produce the transformed triangles for the node at the given index.
    ///
    /// **Note:** If the node's **Drawing** was still in progress, it will first be finished and
    /// inserted into the mesh and geometry graph before producing the triangles iterator.
    pub fn node_triangles(&self, n: node::Index) -> Option<NodeTriangles<S>> {
        self.node_vertices(n)
            .map(geom::tri::iter_from_vertices)
    }

    /// Produce an iterator yielding all vertices from the inner mesh transformed via the inner
    /// geometry graph.
    ///
    /// This method ignores the mesh indices buffer and instead produces the vertices "raw".
    ///
    /// **Note:** If there are any **Drawing**s in progress, these will first be drained and
    /// completed before any vertices are yielded.
    pub fn raw_vertices(&self) -> RawVertices<S> {
        self.finish_remaining_drawings().expect(WOULD_CYCLE);
        let state = self.state.borrow();
        state.geom_graph_dfs.borrow_mut().reset(&state.geom_graph);
        let draw = self;
        let node_vertices = None;
        RawVertices { draw, node_vertices }
    }

    /// Produce an iterator yielding all indexed vertices from the inner mesh transformed via the
    /// inner geometry graph.
    ///
    /// Vertices are yielded in depth-first-order of the geometry graph nodes from which they are
    /// produced.
    ///
    /// **Note:** If there are any **Drawing**s in progress, these will first be drained and
    /// completed before any vertices are yielded.
    pub fn vertices(&self) -> Vertices<S> {
        self.finish_remaining_drawings().expect(WOULD_CYCLE);
        let state = self.state.borrow();
        state.geom_graph_dfs.borrow_mut().reset(&state.geom_graph);
        let draw = self;
        let node_vertices = None;
        Vertices { draw, node_vertices }
    }

    /// Produce an iterator yielding all triangles from the inner mesh transformed via the inner
    /// geometry graph.
    ///
    /// Triangles are yielded in depth-first-order of the geometry graph nodes from which they are
    /// produced.
    ///
    /// **Note:** If there are any **Drawing**s in progress, these will first be drained and
    /// completed before any vertices are yielded.
    pub fn triangles(&self) -> Triangles<S> {
        geom::tri::iter_from_vertices(self.vertices())
    }

    /// Borrow the **Draw**'s inner **Mesh**.
    pub fn mesh(&self) -> Ref<Mesh<S>> {
        Ref::map(self.state.borrow(), |s| &s.mesh)
    }

    // Dimensions methods.

    /// The length of the untransformed node at the given index along the axis returned by the
    /// given `point_axis` function.
    pub fn untransformed_dimension_of<F>(&self, n: &node::Index, point_axis: &F) -> Option<S>
    where
        F: Fn(&mesh::vertex::Point<S>) -> S,
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

    /// Determine the raw, untransformed dimensions of the node at the given index.
    ///
    /// Returns `None` if their is no node within the **geom::Graph** for the given index or if
    /// the node has not yet been **Drawn**.
    pub fn untransformed_dimensions_of(&self, n: &node::Index) -> Option<Vector3<S>> {
        if self.state.borrow().geom_graph.node(*n).is_none()
        || !self.state.borrow().ranges.contains_key(n) {
            return None;
        }
        let dimensions = Vector3 {
            x: self.untransformed_x_dimension_of(n).unwrap_or_else(S::zero),
            y: self.untransformed_y_dimension_of(n).unwrap_or_else(S::zero),
            z: self.untransformed_z_dimension_of(n).unwrap_or_else(S::zero),
        };
        Some(dimensions)
    }

    /// The length of the transformed node at the given index along the axis returned by the given
    /// `point_axis` function.
    pub fn dimension_of<F>(&self, n: &node::Index, point_axis: &F) -> Option<S>
    where
        F: Fn(&mesh::vertex::Point<S>) -> S,
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

    /// Drain any remaining `drawing`s, convert them to their **Drawn** state and insert them into
    /// the inner mesh and geometry graph.
    pub fn finish_remaining_drawings(&self) -> Result<(), geom::graph::WouldCycle<S>> {
        self.state.borrow_mut().finish_remaining_drawings()
    }
}

impl<S> Default for State<S>
where
    S: BaseFloat,
{
    fn default() -> Self {
        let geom_graph = Default::default();
        let geom_graph_dfs = RefCell::new(geom::graph::node::Dfs::new(&geom_graph));
        let drawing = Default::default();
        let mesh = Default::default();
        let ranges = Default::default();
        let theme = Default::default();
        let last_node_drawn = Default::default();
        let background_color = Default::default();
        State {
            geom_graph,
            geom_graph_dfs,
            mesh,
            drawing,
            ranges,
            theme,
            last_node_drawn,
            background_color,
        }
    }
}

impl<S> Default for Draw<S>
where
    S: BaseFloat,
{
    fn default() -> Self {
        let state = RefCell::new(Default::default());
        Draw { state }
    }
}

impl<'a, S> Iterator for Vertices<'a, S>
where
    S: BaseFloat,
{
    type Item = mesh::Vertex<S>;
    fn next(&mut self) -> Option<Self::Item> {
        let Vertices { ref draw, ref mut node_vertices } = *self;
        loop {
            if let Some(v) = node_vertices.as_mut().and_then(|n| n.next()) {
                return Some(v);
            }
            let next_transform = {
                let state = draw.state.borrow();
                let mut dfs = state.geom_graph_dfs.borrow_mut();
                dfs.next_transform(&state.geom_graph)
            };
            match next_transform {
                None => return None,
                Some((n, transform)) => {
                    let index_range = match draw.state.borrow().ranges.get(&n) {
                        None => continue,
                        Some(ranges) => ranges.indices.clone(),
                    };
                    let vertices = ::mesh::vertices(draw.mesh()).index_range(index_range);
                    let transformed_vertices = transform.vertices(vertices);
                    *node_vertices = Some(transformed_vertices);
                },
            }
        }
    }
}

impl<'a, S> Iterator for RawVertices<'a, S>
where
    S: BaseFloat,
{
    type Item = mesh::Vertex<S>;
    fn next(&mut self) -> Option<Self::Item> {
        let RawVertices { ref draw, ref mut node_vertices } = *self;
        loop {
            if let Some(v) = node_vertices.as_mut().and_then(|n| n.next()) {
                return Some(v);
            }
            let next_transform = {
                let state = draw.state.borrow();
                let mut dfs = state.geom_graph_dfs.borrow_mut();
                dfs.next_transform(&state.geom_graph)
            };
            match next_transform {
                None => return None,
                Some((n, transform)) => {
                    let vertex_range = match draw.state.borrow().ranges.get(&n) {
                        None => continue,
                        Some(ranges) => ranges.vertices.clone(),
                    };
                    let vertices = ::mesh::raw_vertices(draw.mesh()).range(vertex_range);
                    let transformed_vertices = transform.vertices(vertices);
                    *node_vertices = Some(transformed_vertices);
                },
            }
        }
    }
}
