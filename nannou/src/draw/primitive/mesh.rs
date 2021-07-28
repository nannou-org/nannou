use crate::color::conv::IntoLinSrgba;
use crate::draw::mesh::vertex::{self, Point, TexCoords, Vertex};
use crate::draw::primitive::Primitive;
use crate::draw::properties::spatial::{orientation, position};
use crate::draw::properties::{ColorScalar, LinSrgba, SetColor, SetOrientation, SetPosition};
use crate::draw::{self, Drawing};
use crate::geom;
use crate::wgpu;
use std::ops;

/// The mesh type prior to being initialised with vertices or indices.
#[derive(Clone, Debug, Default)]
pub struct Vertexless;

/// Properties related to drawing an arbitrary mesh of colours, geometry and texture.
#[derive(Clone, Debug)]
pub struct Mesh {
    position: position::Properties,
    orientation: orientation::Properties,
    vertex_range: ops::Range<usize>,
    index_range: ops::Range<usize>,
    vertex_mode: draw::renderer::VertexMode,
    fill_color: Option<FillColor>,
    texture_view: Option<wgpu::TextureView>,
}

#[derive(Clone, Debug, Default)]
pub struct FillColor(Option<LinSrgba>);

// A simple iterator for flattening a fixed-size array of indices.
struct FlattenIndices<I> {
    iter: I,
    index: usize,
    vertex_start_index: usize,
    current: [usize; 3],
}

pub type DrawingMesh<'a> = Drawing<'a, Mesh>;

impl Vertexless {
    /// Describe the mesh with a sequence of textured points.
    ///
    /// Each of the vertices must be represented as a tuple containing the point and tex
    /// coordinates in that order, e.g. `(point, tex_coords)`. `point` may be of any type that
    /// implements `Into<Point3>` and `tex_coords` may be of any type that implements
    /// `Into<Point2>`.
    pub fn points_textured<I, P, T>(
        self,
        inner_mesh: &mut draw::Mesh,
        texture_view: &dyn wgpu::ToTextureView,
        points: I,
    ) -> Mesh
    where
        I: IntoIterator<Item = (P, T)>,
        P: Into<Point>,
        T: Into<TexCoords>,
    {
        let points = points.into_iter().map(|(p, t)| {
            let point = p.into();
            let color = vertex::DEFAULT_VERTEX_COLOR;
            let tex_coords = t.into();
            ((point, color), tex_coords).into()
        });
        let vertex_mode = draw::renderer::VertexMode::Texture;
        self.points_inner(
            inner_mesh,
            points,
            vertex_mode,
            Some(texture_view.to_texture_view()),
        )
    }

    /// Describe the mesh with a sequence of colored points.
    ///
    /// Each of the points must be represented as a tuple containing the point and the color in
    /// that order, e.g. `(point, color)`. `point` may be of any type that implements
    /// `Into<Point3>` and `color` may be of any type that implements `IntoLinSrgba`.
    pub fn points_colored<I, P, C>(self, inner_mesh: &mut draw::Mesh, points: I) -> Mesh
    where
        I: IntoIterator<Item = (P, C)>,
        P: Into<Point>,
        C: IntoLinSrgba<ColorScalar>,
    {
        let vertices = points.into_iter().map(|(p, c)| {
            let point = p.into();
            let color = c.into_lin_srgba();
            let tex_coords = vertex::default_tex_coords();
            ((point, color), tex_coords).into()
        });
        let vertex_mode = draw::renderer::VertexMode::Color;
        self.points_inner(inner_mesh, vertices, vertex_mode, None)
    }

    /// Describe the mesh with a sequence of points.
    ///
    /// The given iterator may yield any type that can be converted directly into `Point3`s.
    ///
    /// This method assumes that the entire mesh should be coloured with a single colour. If a
    /// colour is not specified via one of the builder methods, a default colour will be retrieved
    /// from the inner `Theme`.
    pub fn points<I>(self, inner_mesh: &mut draw::Mesh, points: I) -> Mesh
    where
        I: IntoIterator,
        I::Item: Into<Point>,
    {
        let vertices = points.into_iter().map(|p| {
            let point = p.into();
            let color = vertex::DEFAULT_VERTEX_COLOR;
            let tex_coords = vertex::default_tex_coords();
            ((point, color), tex_coords).into()
        });
        let vertex_mode = draw::renderer::VertexMode::Color;
        let mut mesh = self.points_inner(inner_mesh, vertices, vertex_mode, None);
        mesh.fill_color = Some(FillColor(None));
        mesh
    }

    fn points_inner<I>(
        self,
        inner_mesh: &mut draw::Mesh,
        vertices: I,
        vertex_mode: draw::renderer::VertexMode,
        texture_view: Option<wgpu::TextureView>,
    ) -> Mesh
    where
        I: Iterator<Item = Vertex>,
    {
        let v_start = inner_mesh.points().len();
        let i_start = inner_mesh.indices().len();
        for (i, vertex) in vertices.enumerate() {
            inner_mesh.push_vertex(vertex);
            inner_mesh.push_index((v_start + i) as u32);
        }
        let v_end = inner_mesh.points().len();
        let i_end = inner_mesh.indices().len();
        Mesh::new(v_start..v_end, i_start..i_end, vertex_mode, texture_view)
    }

    /// Describe the mesh with a sequence of textured triangles.
    ///
    /// Each of the vertices must be represented as a tuple containing the point and tex
    /// coordinates in that order, e.g. `(point, tex_coords)`. `point` may be of any type that
    /// implements `Into<Point3>` and `tex_coords` may be of any type that implements
    /// `Into<Point2>`.
    pub fn tris_textured<I, P, T>(
        self,
        inner_mesh: &mut draw::Mesh,
        texture_view: &dyn wgpu::ToTextureView,
        tris: I,
    ) -> Mesh
    where
        I: IntoIterator<Item = geom::Tri<(P, T)>>,
        P: Into<Point>,
        T: Into<TexCoords>,
    {
        let points = tris
            .into_iter()
            .map(|t| t.map_vertices(|(p, t)| (p.into(), t.into())))
            .flat_map(geom::Tri::vertices);
        self.points_textured(inner_mesh, texture_view, points)
    }

    /// Describe the mesh with a sequence of colored triangles.
    ///
    /// Each of the vertices must be represented as a tuple containing the point and the color in
    /// that order, e.g. `(point, color)`. `point` may be of any type that implements `Into<Point3>`
    /// and `color` may be of any type that implements `IntoLinSrgba`.
    pub fn tris_colored<I, P, C>(self, inner_mesh: &mut draw::Mesh, tris: I) -> Mesh
    where
        I: IntoIterator<Item = geom::Tri<(P, C)>>,
        P: Into<Point>,
        C: IntoLinSrgba<ColorScalar>,
    {
        let points = tris
            .into_iter()
            .map(|t| t.map_vertices(|(p, c)| (p.into(), c.into_lin_srgba())))
            .flat_map(geom::Tri::vertices);
        self.points_colored(inner_mesh, points)
    }

    /// Describe the mesh with a sequence of triangles.
    ///
    /// Each triangle may be composed of any vertex type that may be converted directly into
    /// `Point3`s.
    ///
    /// This method assumes that the entire mesh should be coloured with a single colour. If a
    /// colour is not specified via one of the builder methods, a default colour will be retrieved
    /// from the inner `Theme`.
    pub fn tris<I, V>(self, inner_mesh: &mut draw::Mesh, tris: I) -> Mesh
    where
        I: IntoIterator<Item = geom::Tri<V>>,
        V: Into<Point>,
    {
        let points = tris
            .into_iter()
            .map(|t| t.map_vertices(Into::into))
            .flat_map(geom::Tri::vertices);
        self.points(inner_mesh, points)
    }

    /// Describe the mesh with the given indexed, textured points.
    ///
    /// Each trio of `indices` describes a single triangle made up of colored `points`.
    ///
    /// Each of the `points` must be represented as a tuple containing the point and the texture
    /// coordinates in that order, e.g. `(point, tex_coords)`. `point` may be of any type that
    /// implements `Into<Point3>` and `tex_coords` may be of any type that implements
    /// `Into<Point2>`.
    pub fn indexed_textured<V, I, P, T>(
        self,
        inner_mesh: &mut draw::Mesh,
        texture_view: &dyn wgpu::ToTextureView,
        points: V,
        indices: I,
    ) -> Mesh
    where
        V: IntoIterator<Item = (P, T)>,
        I: IntoIterator<Item = usize>,
        P: Into<Point>,
        T: Into<TexCoords>,
    {
        let vertices = points.into_iter().map(|(p, t)| {
            let point = p.into();
            let color = vertex::DEFAULT_VERTEX_COLOR;
            let tex_coords = t.into();
            ((point, color), tex_coords).into()
        });
        let vertex_mode = draw::renderer::VertexMode::Texture;
        self.indexed_inner(
            inner_mesh,
            vertices,
            indices,
            vertex_mode,
            Some(texture_view.to_texture_view()),
        )
    }

    /// Describe the mesh with the given indexed, colored points.
    ///
    /// Each trio of `indices` describes a single triangle made up of colored `points`.
    ///
    /// Each of the `points` must be represented as a tuple containing the point and the color in
    /// that order, e.g. `(point, color)`. `point` may be of any type that implements
    /// `Into<Point3>` and `color` may be of any type that implements `IntoLinSrgba`.
    pub fn indexed_colored<V, I, P, C>(
        self,
        inner_mesh: &mut draw::Mesh,
        points: V,
        indices: I,
    ) -> Mesh
    where
        V: IntoIterator<Item = (P, C)>,
        I: IntoIterator<Item = usize>,
        P: Into<Point>,
        C: IntoLinSrgba<ColorScalar>,
    {
        let vertices = points.into_iter().map(|(p, c)| {
            let point = p.into();
            let color = c.into_lin_srgba();
            let tex_coords = vertex::default_tex_coords();
            ((point, color), tex_coords).into()
        });
        let vertex_mode = draw::renderer::VertexMode::Color;
        self.indexed_inner(inner_mesh, vertices, indices, vertex_mode, None)
    }

    /// Describe the mesh with the given indexed points.
    ///
    /// Each trio of `indices` describes a single triangle made up of `points`.
    ///
    /// Each point may be any type that may be converted directly into the `Point3` type.
    pub fn indexed<V, I>(self, inner_mesh: &mut draw::Mesh, points: V, indices: I) -> Mesh
    where
        V: IntoIterator,
        V::Item: Into<Point>,
        I: IntoIterator<Item = usize>,
    {
        let vertices = points.into_iter().map(|p| {
            let point = p.into();
            let color = vertex::DEFAULT_VERTEX_COLOR;
            let tex_coords = vertex::default_tex_coords();
            ((point, color), tex_coords).into()
        });
        let vertex_mode = draw::renderer::VertexMode::Color;
        let mut mesh = self.indexed_inner(inner_mesh, vertices, indices, vertex_mode, None);
        mesh.fill_color = Some(FillColor(None));
        mesh
    }

    fn indexed_inner<V, I>(
        self,
        inner_mesh: &mut draw::Mesh,
        vertices: V,
        indices: I,
        vertex_mode: draw::renderer::VertexMode,
        texture_view: Option<wgpu::TextureView>,
    ) -> Mesh
    where
        V: IntoIterator<Item = Vertex>,
        I: IntoIterator<Item = usize>,
    {
        let v_start = inner_mesh.points().len();
        let i_start = inner_mesh.indices().len();
        inner_mesh.extend_vertices(vertices);
        inner_mesh.extend_indices(indices.into_iter().map(|ix| ix as u32));
        let v_end = inner_mesh.points().len();
        let i_end = inner_mesh.indices().len();
        Mesh::new(v_start..v_end, i_start..i_end, vertex_mode, texture_view)
    }
}

impl Mesh {
    // Initialise a new `Mesh` with its ranges into the intermediary mesh, ready for drawing.
    fn new(
        vertex_range: ops::Range<usize>,
        index_range: ops::Range<usize>,
        vertex_mode: draw::renderer::VertexMode,
        texture_view: Option<wgpu::TextureView>,
    ) -> Self {
        let orientation = Default::default();
        let position = Default::default();
        let fill_color = None;
        Mesh {
            orientation,
            position,
            vertex_range,
            index_range,
            vertex_mode,
            fill_color,
            texture_view,
        }
    }
}

impl<'a> Drawing<'a, Vertexless> {
    /// Describe the mesh with a sequence of points.
    ///
    /// The given iterator may yield any type that can be converted directly into `Point3`s.
    ///
    /// This method assumes that the entire mesh should be coloured with a single colour. If a
    /// colour is not specified via one of the builder methods, a default colour will be retrieved
    /// from the inner `Theme`.
    pub fn points<I>(self, points: I) -> DrawingMesh<'a>
    where
        I: IntoIterator,
        I::Item: Into<Point>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.points(ctxt.mesh, points))
    }

    /// Describe the mesh with a sequence of colored points.
    ///
    /// Each of the points must be represented as a tuple containing the point and the color in
    /// that order, e.g. `(point, color)`. `point` may be of any type that implements
    /// `Into<Point3>` and `color` may be of any type that implements `IntoLinSrgba`.
    pub fn points_colored<I, P, C>(self, points: I) -> DrawingMesh<'a>
    where
        I: IntoIterator<Item = (P, C)>,
        P: Into<Point>,
        C: IntoLinSrgba<ColorScalar>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.points_colored(ctxt.mesh, points))
    }

    /// Describe the mesh with a sequence of textured points.
    ///
    /// Each of the vertices must be represented as a tuple containing the point and tex
    /// coordinates in that order, e.g. `(point, tex_coords)`. `point` may be of any type that
    /// implements `Into<Point3>` and `tex_coords` may be of any type that implements
    /// `Into<Point2>`.
    pub fn points_textured<I, P, T>(
        self,
        view: &dyn wgpu::ToTextureView,
        points: I,
    ) -> DrawingMesh<'a>
    where
        I: IntoIterator<Item = (P, T)>,
        P: Into<Point>,
        T: Into<TexCoords>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.points_textured(ctxt.mesh, view, points))
    }

    /// Describe the mesh with a sequence of triangles.
    ///
    /// Each triangle may be composed of any vertex type that may be converted directly into
    /// `Point3`s.
    ///
    /// This method assumes that the entire mesh should be coloured with a single colour. If a
    /// colour is not specified via one of the builder methods, a default colour will be retrieved
    /// from the inner `Theme`.
    pub fn tris<I, V>(self, tris: I) -> DrawingMesh<'a>
    where
        I: IntoIterator<Item = geom::Tri<V>>,
        V: Into<Point>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.tris(ctxt.mesh, tris))
    }

    /// Describe the mesh with a sequence of colored triangles.
    ///
    /// Each of the vertices must be represented as a tuple containing the point and the color in
    /// that order, e.g. `(point, color)`. `point` may be of any type that implements `Into<Point3>`
    /// and `color` may be of any type that implements `IntoLinSrgba`.
    pub fn tris_colored<I, P, C>(self, tris: I) -> DrawingMesh<'a>
    where
        I: IntoIterator<Item = geom::Tri<(P, C)>>,
        P: Into<Point>,
        C: IntoLinSrgba<ColorScalar>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.tris_colored(ctxt.mesh, tris))
    }

    /// Describe the mesh with a sequence of textured triangles.
    ///
    /// Each of the vertices must be represented as a tuple containing the point and tex
    /// coordinates in that order, e.g. `(point, tex_coords)`. `point` may be of any type that
    /// implements `Into<Point3>` and `tex_coords` may be of any type that implements
    /// `Into<Point2>`.
    pub fn tris_textured<I, P, T>(self, view: &dyn wgpu::ToTextureView, tris: I) -> DrawingMesh<'a>
    where
        I: IntoIterator<Item = geom::Tri<(P, T)>>,
        P: Into<Point>,
        T: Into<TexCoords>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.tris_textured(ctxt.mesh, view, tris))
    }

    /// Describe the mesh with the given indexed points.
    ///
    /// Each trio of `indices` describes a single triangle made up of `points`.
    ///
    /// Each point may be any type that may be converted directly into the `Point3` type.
    pub fn indexed<V, I>(self, points: V, indices: I) -> DrawingMesh<'a>
    where
        V: IntoIterator,
        V::Item: Into<Point>,
        I: IntoIterator<Item = usize>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.indexed(ctxt.mesh, points, indices))
    }

    /// Describe the mesh with the given indexed, colored points.
    ///
    /// Each trio of `indices` describes a single triangle made up of colored `points`.
    ///
    /// Each of the `points` must be represented as a tuple containing the point and the color in
    /// that order, e.g. `(point, color)`. `point` may be of any type that implements
    /// `Into<Point3>` and `color` may be of any type that implements `IntoLinSrgba`.
    pub fn indexed_colored<V, I, P, C>(self, points: V, indices: I) -> DrawingMesh<'a>
    where
        V: IntoIterator<Item = (P, C)>,
        I: IntoIterator<Item = usize>,
        P: Into<Point>,
        C: IntoLinSrgba<ColorScalar>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.indexed_colored(ctxt.mesh, points, indices))
    }

    /// Describe the mesh with the given indexed, textured points.
    ///
    /// Each trio of `indices` describes a single triangle made up of colored `points`.
    ///
    /// Each of the `points` must be represented as a tuple containing the point and the texture
    /// coordinates in that order, e.g. `(point, tex_coords)`. `point` may be of any type that
    /// implements `Into<Point3>` and `tex_coords` may be of any type that implements
    /// `Into<Point2>`.
    pub fn indexed_textured<V, I, P, T>(
        self,
        view: &dyn wgpu::ToTextureView,
        points: V,
        indices: I,
    ) -> DrawingMesh<'a>
    where
        V: IntoIterator<Item = (P, T)>,
        I: IntoIterator<Item = usize>,
        P: Into<Point>,
        T: Into<TexCoords>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.indexed_textured(ctxt.mesh, view, points, indices))
    }
}

impl draw::renderer::RenderPrimitive for Mesh {
    fn render_primitive(
        self,
        ctxt: draw::renderer::RenderContext,
        mesh: &mut draw::Mesh,
    ) -> draw::renderer::PrimitiveRender {
        let Mesh {
            orientation,
            position,
            vertex_range,
            index_range,
            vertex_mode,
            fill_color,
            texture_view,
        } = self;

        // Determine the transform to apply to vertices.
        let global_transform = *ctxt.transform;
        let local_transform = position.transform() * orientation.transform();
        let transform = global_transform * local_transform;

        // We need to update the indices to point to where vertices will be in the new mesh.
        let old_mesh_vertex_start = vertex_range.start as u32;
        let new_mesh_vertex_start = mesh.raw_vertex_count() as u32;
        let indices = index_range
            .map(|i| ctxt.intermediary_mesh.indices()[i])
            .map(|i| new_mesh_vertex_start + i - old_mesh_vertex_start);

        // A small function for transforming a point via the transform matrix.
        let transform_point = |p: geom::Point3| -> geom::Point3 { transform.transform_point3(p) };

        // Color the vertices based on whether or not we should fill, then extend the mesh!
        match fill_color {
            Some(fill) => {
                let theme_primitive = draw::theme::Primitive::Mesh;
                let color =
                    ctxt.theme
                        .resolve_color(fill.0, theme_primitive, draw::theme::ColorType::Fill);
                let vertices = vertex_range.map(|i| {
                    let point = transform_point(ctxt.intermediary_mesh.points()[i]);
                    let tex_coords = ctxt.intermediary_mesh.tex_coords()[i];
                    ((point, color), tex_coords).into()
                });
                mesh.extend(vertices, indices);
            }
            None => {
                let vertices = vertex_range.map(|i| {
                    let point = transform_point(ctxt.intermediary_mesh.points()[i]);
                    let color = ctxt.intermediary_mesh.colors()[i];
                    let tex_coords = ctxt.intermediary_mesh.tex_coords()[i];
                    ((point, color), tex_coords).into()
                });
                mesh.extend(vertices, indices);
            }
        }

        draw::renderer::PrimitiveRender {
            texture_view,
            vertex_mode,
        }
    }
}

impl<I> Iterator for FlattenIndices<I>
where
    I: Iterator<Item = [usize; 3]>,
{
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.index < self.current.len() {
                let ix = self.current[self.index];
                self.index += 1;
                return Some(self.vertex_start_index + ix);
            }
            match self.iter.next() {
                None => return None,
                Some(trio) => {
                    self.current = trio;
                    self.index = 0;
                }
            }
        }
    }
}

impl SetOrientation for Mesh {
    fn properties(&mut self) -> &mut orientation::Properties {
        SetOrientation::properties(&mut self.orientation)
    }
}

impl SetPosition for Mesh {
    fn properties(&mut self) -> &mut position::Properties {
        SetPosition::properties(&mut self.position)
    }
}

impl SetColor<ColorScalar> for Mesh {
    fn rgba_mut(&mut self) -> &mut Option<LinSrgba> {
        &mut self.fill_color.get_or_insert_with(Default::default).0
    }
}

impl From<Vertexless> for Primitive {
    fn from(prim: Vertexless) -> Self {
        Primitive::MeshVertexless(prim)
    }
}

impl From<Mesh> for Primitive {
    fn from(prim: Mesh) -> Self {
        Primitive::Mesh(prim)
    }
}

impl Into<Option<Vertexless>> for Primitive {
    fn into(self) -> Option<Vertexless> {
        match self {
            Primitive::MeshVertexless(prim) => Some(prim),
            _ => None,
        }
    }
}

impl Into<Option<Mesh>> for Primitive {
    fn into(self) -> Option<Mesh> {
        match self {
            Primitive::Mesh(prim) => Some(prim),
            _ => None,
        }
    }
}
