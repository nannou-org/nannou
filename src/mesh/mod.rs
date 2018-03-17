use geom;
use math::{BaseFloat, BaseNum, EuclideanSpace, Point2};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

pub mod channel;
pub mod vertex;

pub use self::channel::{Channel, ChannelMut};

// Type aliases.

/// The fallback scalar type used for texture coordinates if none is specified.
pub type TexCoordScalarDefault = f64;

// Traits describing meshes with access to certain channels.

/// Mesh types that can be indexed to produce a vertex.
pub trait GetVertex {
    /// The vertex type representing all channels of data within the mesh at a single index.
    type Vertex;
    /// Create a vertex containing all channel properties for the given index.
    fn get_vertex(&self, index: usize) -> Option<Self::Vertex>;
}

/// All meshes must contain at least one vertex channel.
pub trait Points {
    /// The scalar value used for the vertex coordinates.
    type Scalar: BaseNum;
    /// The vertex type used to represent the location of a vertex.
    type Point: geom::Vertex<Scalar = Self::Scalar>;
    /// The channel type containing points.
    type Points: Channel<Element = Self::Point>;
    /// Borrow the vertex channel from the mesh.
    fn points(&self) -> &Self::Points;
}

/// Meshes that contain a channel of indices that describe the edges between points.
pub trait Indices {
    /// The channel type containing indices.
    type Indices: Channel<Element = usize>;
    /// Borrow the index channel from the mesh.
    fn indices(&self) -> &Self::Indices;
}

/// Meshes that contain a channel of colors.
pub trait Colors {
    /// The color type stored within the channel.
    type Color;
    /// The channel type containing colors.
    type Colors: Channel<Element = Self::Color>;
    /// Borrow the color channel from the mesh.
    fn colors(&self) -> &Self::Colors;
}

/// Meshes that contain a channel of texture coordinates.
pub trait TexCoords {
    /// The scalar value used for the texture coordinates.
    type TexCoordScalar: BaseFloat;
    /// The channel type containing texture coordinates.
    type TexCoords: Channel<Element = Point2<Self::TexCoordScalar>>;
    /// Borrow the texture coordinate channel from the mesh.
    fn tex_coords(&self) -> &Self::TexCoords;
}

/// Meshes that contain a channel of vertex normals.
pub trait Normals: Points
where
    Self::Point: EuclideanSpace,
{
    /// The channel type containing vertex normals.
    type Normals: Channel<Element = <Self::Point as EuclideanSpace>::Diff>;
    /// Borrow the normal channel from the mesh.
    fn normals(&self) -> &Self::Normals;
}

// Mesh types.

/// The base mesh type with only a single vertex channel.
///
/// Extra channels can be added to the mesh via the `WithIndices`, `WithColors`, `WithTexCoords`
/// and `WithNormals` adaptor types.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct MeshPoints<P> {
    pub points: P,
}

/// A mesh type with an added channel containing indices describing the edges between vertices.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct WithIndices<M, I> {
    pub mesh: M,
    pub indices: I,
}

/// A `Mesh` type with an added channel containing colors.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct WithColors<M, C> {
    pub mesh: M,
    pub colors: C,
}

/// A `Mesh` type with an added channel containing texture coordinates.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct WithTexCoords<M, T, S = TexCoordScalarDefault> {
    pub mesh: M,
    pub tex_coords: T,
    pub _tex_coord_scalar: PhantomData<S>, // Required due to lack of HKT.
}

/// A `Mesh` type with an added channel containing vertex normals.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct WithNormals<M, N> {
    pub mesh: M,
    pub normals: N,
}

// **GetVertex** implementations.

impl<'a, M> GetVertex for &'a M
where
    M: GetVertex,
{
    type Vertex = M::Vertex;
    fn get_vertex(&self, index: usize) -> Option<Self::Vertex> {
        (**self).get_vertex(index)
    }
}

impl<'a, M> GetVertex for &'a mut M
where
    M: GetVertex,
{
    type Vertex = M::Vertex;
    fn get_vertex(&self, index: usize) -> Option<Self::Vertex> {
        (**self).get_vertex(index)
    }
}

impl<P> GetVertex for MeshPoints<P>
where
    P: Channel,
    P::Element: geom::Vertex,
{
    type Vertex = P::Element;
    fn get_vertex(&self, index: usize) -> Option<Self::Vertex> {
        self.points.channel().get(index).map(|&p| p)
    }
}

impl<M, I> GetVertex for WithIndices<M, I>
where
    M: GetVertex,
{
    type Vertex = M::Vertex;
    fn get_vertex(&self, index: usize) -> Option<Self::Vertex> {
        self.mesh.get_vertex(index)
    }
}

impl<M, C> GetVertex for WithColors<M, C>
where
    M: GetVertex,
    C: Channel,
    C::Element: Clone,
{
    type Vertex = vertex::WithColor<M::Vertex, C::Element>;
    fn get_vertex(&self, index: usize) -> Option<Self::Vertex> {
        self.mesh.get_vertex(index)
            .and_then(|vertex| {
                self.colors.channel()
                    .get(index)
                    .map(|color| {
                        let color = color.clone();
                        vertex::WithColor { vertex, color }
                    })
            })
    }
}

impl<M, T, S> GetVertex for WithTexCoords<M, T, S>
where
    M: GetVertex,
    T: Channel,
    T::Element: Clone,
{
    type Vertex = vertex::WithTexCoords<M::Vertex, T::Element>;
    fn get_vertex(&self, index: usize) -> Option<Self::Vertex> {
        self.mesh.get_vertex(index)
            .and_then(|vertex| {
                self.tex_coords.channel()
                    .get(index)
                    .map(|tex_coords| {
                        let tex_coords = tex_coords.clone();
                        vertex::WithTexCoords { vertex, tex_coords }
                    })
            })
    }
}

impl<M, N> GetVertex for WithNormals<M, N>
where
    M: GetVertex,
    N: Channel,
    N::Element: Clone,
{
    type Vertex = vertex::WithNormal<M::Vertex, N::Element>;
    fn get_vertex(&self, index: usize) -> Option<Self::Vertex> {
        self.mesh.get_vertex(index)
            .and_then(|vertex| {
                self.normals.channel()
                    .get(index)
                    .map(|normal| {
                        let normal = normal.clone();
                        vertex::WithNormal { vertex, normal }
                    })
            })
    }
}

// **Points** implementations.

impl<P> Points for MeshPoints<P>
where
    P: Channel,
    P::Element: geom::Vertex,
{
    type Scalar = <P::Element as geom::Vertex>::Scalar;
    type Point = P::Element;
    type Points = P;
    fn points(&self) -> &Self::Points {
        &self.points
    }
}

impl<'a, M> Points for &'a M
where
    M: Points,
{
    type Scalar = M::Scalar;
    type Point = M::Point;
    type Points = M::Points;
    fn points(&self) -> &Self::Points {
        (**self).points()
    }
}

impl<'a, M> Points for &'a mut M
where
    M: Points,
{
    type Scalar = M::Scalar;
    type Point = M::Point;
    type Points = M::Points;
    fn points(&self) -> &Self::Points {
        (**self).points()
    }
}

impl<M, I> Points for WithIndices<M, I>
where
    M: Points,
{
    type Scalar = M::Scalar;
    type Point = M::Point;
    type Points = M::Points;
    fn points(&self) -> &Self::Points {
        self.mesh.points()
    }
}

impl<M, C> Points for WithColors<M, C>
where
    M: Points,
{
    type Scalar = M::Scalar;
    type Point = M::Point;
    type Points = M::Points;
    fn points(&self) -> &Self::Points {
        self.mesh.points()
    }
}

impl<M, T, S> Points for WithTexCoords<M, T, S>
where
    M: Points,
{
    type Scalar = M::Scalar;
    type Point = M::Point;
    type Points = M::Points;
    fn points(&self) -> &Self::Points {
        self.mesh.points()
    }
}

impl<M, N> Points for WithNormals<M, N>
where
    M: Points,
{
    type Scalar = M::Scalar;
    type Point = M::Point;
    type Points = M::Points;
    fn points(&self) -> &Self::Points {
        self.mesh.points()
    }
}

// **Indices** implementations.

impl<M, I> Indices for WithIndices<M, I>
where
    I: Channel<Element = usize>,
{
    type Indices = I;
    fn indices(&self) -> &Self::Indices {
        &self.indices
    }
}

impl<'a, M> Indices for &'a M
where
    M: Indices,
{
    type Indices = M::Indices;
    fn indices(&self) -> &Self::Indices {
        (**self).indices()
    }
}

impl<'a, M> Indices for &'a mut M
where
    M: Indices,
{
    type Indices = M::Indices;
    fn indices(&self) -> &Self::Indices {
        (**self).indices()
    }
}

impl<M, C> Indices for WithColors<M, C>
where
    M: Indices,
{
    type Indices = M::Indices;
    fn indices(&self) -> &Self::Indices {
        self.mesh.indices()
    }
}

impl<M, T, S> Indices for WithTexCoords<M, T, S>
where
    M: Indices,
{
    type Indices = M::Indices;
    fn indices(&self) -> &Self::Indices {
        self.mesh.indices()
    }
}

impl<M, N> Indices for WithNormals<M, N>
where
    M: Indices,
{
    type Indices = M::Indices;
    fn indices(&self) -> &Self::Indices {
        self.mesh.indices()
    }
}

// **Colors** implementations.

impl<M, C> Colors for WithColors<M, C>
where
    C: Channel,
{
    type Color = C::Element;
    type Colors = C;
    fn colors(&self) -> &Self::Colors {
        &self.colors
    }
}

impl<'a, M> Colors for &'a M
where
    M: Colors,
{
    type Color = M::Color;
    type Colors = M::Colors;
    fn colors(&self) -> &Self::Colors {
        (**self).colors()
    }
}

impl<'a, M> Colors for &'a mut M
where
    M: Colors,
{
    type Color = M::Color;
    type Colors = M::Colors;
    fn colors(&self) -> &Self::Colors {
        (**self).colors()
    }
}

impl<M, I> Colors for WithIndices<M, I>
where
    M: Colors,
{
    type Color = M::Color;
    type Colors = M::Colors;
    fn colors(&self) -> &Self::Colors {
        self.mesh.colors()
    }
}

impl<M, T, S> Colors for WithTexCoords<M, T, S>
where
    M: Colors,
{
    type Color = M::Color;
    type Colors = M::Colors;
    fn colors(&self) -> &Self::Colors {
        self.mesh.colors()
    }
}

impl<M, N> Colors for WithNormals<M, N>
where
    M: Colors,
{
    type Color = M::Color;
    type Colors = M::Colors;
    fn colors(&self) -> &Self::Colors {
        self.mesh.colors()
    }
}

// **TexCoords** implementations.

impl<M, T, S> TexCoords for WithTexCoords<M, T, S>
where
    T: Channel<Element = Point2<S>>,
    S: BaseFloat,
{
    type TexCoordScalar = S;
    type TexCoords = T;
    fn tex_coords(&self) -> &Self::TexCoords {
        &self.tex_coords
    }
}

impl<'a, M> TexCoords for &'a M
where
    M: TexCoords,
{
    type TexCoordScalar = M::TexCoordScalar;
    type TexCoords = M::TexCoords;
    fn tex_coords(&self) -> &Self::TexCoords {
        (**self).tex_coords()
    }
}

impl<'a, M> TexCoords for &'a mut M
where
    M: TexCoords,
{
    type TexCoordScalar = M::TexCoordScalar;
    type TexCoords = M::TexCoords;
    fn tex_coords(&self) -> &Self::TexCoords {
        (**self).tex_coords()
    }
}

impl<M, I> TexCoords for WithIndices<M, I>
where
    M: TexCoords,
{
    type TexCoordScalar = M::TexCoordScalar;
    type TexCoords = M::TexCoords;
    fn tex_coords(&self) -> &Self::TexCoords {
        self.mesh.tex_coords()
    }
}

impl<M, C> TexCoords for WithColors<M, C>
where
    M: TexCoords,
{
    type TexCoordScalar = M::TexCoordScalar;
    type TexCoords = M::TexCoords;
    fn tex_coords(&self) -> &Self::TexCoords {
        self.mesh.tex_coords()
    }
}

impl<M, N> TexCoords for WithNormals<M, N>
where
    M: TexCoords,
{
    type TexCoordScalar = M::TexCoordScalar;
    type TexCoords = M::TexCoords;
    fn tex_coords(&self) -> &Self::TexCoords {
        self.mesh.tex_coords()
    }
}

// **Normals** implementations.

impl<M, N> Normals for WithNormals<M, N>
where
    M: Points,
    M::Point: EuclideanSpace,
    N: Channel<Element = <M::Point as EuclideanSpace>::Diff>,
{
    type Normals = N;
    fn normals(&self) -> &Self::Normals {
        &self.normals
    }
}

impl<'a, M> Normals for &'a M
where
    M: Normals,
    M::Point: EuclideanSpace,
{
    type Normals = M::Normals;
    fn normals(&self) -> &Self::Normals {
        (**self).normals()
    }
}

impl<'a, M> Normals for &'a mut M
where
    M: Normals,
    M::Point: EuclideanSpace,
{
    type Normals = M::Normals;
    fn normals(&self) -> &Self::Normals {
        (**self).normals()
    }
}

impl<M, I> Normals for WithIndices<M, I>
where
    M: Normals,
    M::Point: EuclideanSpace,
{
    type Normals = M::Normals;
    fn normals(&self) -> &Self::Normals {
        self.mesh.normals()
    }
}

impl<M, C> Normals for WithColors<M, C>
where
    M: Normals,
    M::Point: EuclideanSpace,
{
    type Normals = M::Normals;
    fn normals(&self) -> &Self::Normals {
        self.mesh.normals()
    }
}

impl<M, T, S> Normals for WithTexCoords<M, T, S>
where
    M: Normals,
    M::Point: EuclideanSpace,
{
    type Normals = M::Normals;
    fn normals(&self) -> &Self::Normals {
        self.mesh.normals()
    }
}

// Deref implementations for the mesh adaptor types to their inner mesh.

impl<M, I> Deref for WithIndices<M, I> {
    type Target = M;
    fn deref(&self) -> &Self::Target {
        &self.mesh
    }
}

impl<M, I> DerefMut for WithIndices<M, I> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mesh
    }
}

impl<M, C> Deref for WithColors<M, C> {
    type Target = M;
    fn deref(&self) -> &Self::Target {
        &self.mesh
    }
}

impl<M, C> DerefMut for WithColors<M, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mesh
    }
}

impl<M, T, S> Deref for WithTexCoords<M, T, S> {
    type Target = M;
    fn deref(&self) -> &Self::Target {
        &self.mesh
    }
}

impl<M, T, S> DerefMut for WithTexCoords<M, T, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mesh
    }
}

impl<M, N> Deref for WithNormals<M, N> {
    type Target = M;
    fn deref(&self) -> &Self::Target {
        &self.mesh
    }
}

impl<M, N> DerefMut for WithNormals<M, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mesh
    }
}

// Mesh constructors.

/// Create a simple base mesh from the given channel of vertex points.
pub fn from_points<P>(points: P) -> MeshPoints<P>
where
    P: Channel,
    P::Element: geom::Vertex,
{
    MeshPoints { points }
}

/// Combine the given mesh with the given channel of vertex indices.
pub fn with_indices<M, I>(mesh: M, indices: I) -> WithIndices<M, I>
where
    M: GetVertex,
    I: Channel<Element = usize>,
{
    WithIndices { mesh, indices }
}

/// Combine the given mesh with the given channel of vertex colors.
pub fn with_colors<M, C>(mesh: M, colors: C) -> WithColors<M, C>
where
    C: Channel,
{
    WithColors { mesh, colors }
}

/// Combine the given mesh with the given channel of vertex texture coordinates.
pub fn with_tex_coords<M, T, S>(mesh: M, tex_coords: T) -> WithTexCoords<M, T, S>
where
    T: Channel<Element = Point2<S>>,
    S: BaseFloat,
{
    let _tex_coord_scalar = PhantomData;
    WithTexCoords { mesh, tex_coords, _tex_coord_scalar }
}

/// Combine the given mesh with the given **Normals** channel.
pub fn with_normals<M, N>(mesh: M, normals: N) -> WithNormals<M, N>
where
    M: Points,
    M::Point: EuclideanSpace,
    N: Channel<Element = <M::Point as EuclideanSpace>::Diff>,
{
    WithNormals { mesh, normals }
}

// Mesh iterators.

/// An iterator yielding the raw vertices (with combined channels) of a mesh.
///
/// Requires that the mesh implements **GetVertex**.
///
/// Returns `None` when the inner mesh first returns `None` for a call to **GetVertex::get_vertex**.
#[derive(Clone, Debug)]
pub struct RawVertices<M> {
    index: usize,
    mesh: M,
}

/// An iterator yielding vertices in the order specified via the mesh's **Indices** channel.
///
/// Requires that the mesh implements **Indices** and **GetVertex**.
///
/// Returns `None` when a vertex has been yielded for every index in the **Indices** channel.
///
/// **Panics** if the **Indices** channel produces an index that is out of bounds of the mesh's
/// vertices.
#[derive(Clone, Debug)]
pub struct Vertices<M> {
    index: usize,
    mesh: M,
}

/// An iterator yielding triangles in the order specified via the mesh's **Indices** channel.
///
/// Requires that the mesh implements **Indices** and **GetVertex**.
///
/// **Panics** if the **Indices** channel produces an index that is out of bounds of the mesh's
pub type Triangles<M> = geom::tri::IterFromVertices<Vertices<M>>;

/// An iterator yielding the raw vertices (with combined channels) of a mesh.
///
/// Requires that the inner mesh implements **GetVertex**.
///
/// Returns `None` when the inner mesh first returns `None` for a call to **GetVertex::get_vertex**.
pub fn raw_vertices<M>(mesh: M) -> RawVertices<M>
where
    M: GetVertex,
{
    let index = 0;
    RawVertices { index, mesh }
}

/// Produce an iterator yielding vertices in the order specified via the mesh's **Indices**
/// channel.
///
/// Requires that the mesh implements **Indices** and **GetVertex**.
///
/// Returns `None` when a vertex has been yielded for every index in the **Indices** channel.
///
/// **Panics** if the **Indices** channel produces an index that is out of bounds of the mesh's
/// vertices.
pub fn vertices<M>(mesh: M) -> Vertices<M>
where
    M: Indices + GetVertex,
{
    let index = 0;
    Vertices { index, mesh }
}

/// Produce an iterator yielding triangles for every three vertices yielded in the order specified
/// via the mesh's **Indices** channel.
///
/// Requires that the mesh implements **Indices** and **GetVertex**.
///
/// Returns `None` when there are no longer enough vertex indices to produce a triangle.
///
/// **Panics** if the **Indices** channel produces an index that is out of bounds of the mesh's
/// vertices.
pub fn triangles<M>(mesh: M) -> Triangles<M>
where
    M: Indices + GetVertex,
{
    geom::tri::iter_from_vertices(vertices(mesh))
}

// The error message produced when the `Vertices` iterator panics due to an out of bound index.
const NO_VERTEX_FOR_INDEX: &'static str =
    "no vertex for the index produced by the mesh's indices channel";

impl<M> Iterator for RawVertices<M>
where
    M: GetVertex,
{
    type Item = M::Vertex;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(vertex) = self.mesh.get_vertex(self.index) {
            self.index += 1;
            return Some(vertex);
        }
        None
    }
}

impl<M> Iterator for Vertices<M>
where
    M: Indices + GetVertex,
{
    type Item = M::Vertex;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(&index) = self.mesh.indices().channel().get(self.index) {
            self.index += 1;
            let vertex = self.mesh.get_vertex(index).expect(NO_VERTEX_FOR_INDEX);
            return Some(vertex);
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<M> DoubleEndedIterator for Vertices<M>
where
    M: Indices + GetVertex,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let next_index = self.index + 1;
        let indices = self.mesh.indices().channel();
        if let Some(&index) = indices.get(indices.len() - next_index) {
            self.index = next_index;
            let vertex = self.mesh.get_vertex(index).expect(NO_VERTEX_FOR_INDEX);
            return Some(vertex);
        }
        None
    }
}

impl<M> ExactSizeIterator for Vertices<M>
where
    M: Indices + GetVertex,
{
    fn len(&self) -> usize {
        self.mesh.indices().channel().len() - self.index
    }
}
