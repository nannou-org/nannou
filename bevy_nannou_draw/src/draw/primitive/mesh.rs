use crate::draw::mesh::MeshExt;
use crate::draw::primitive::{Primitive, Vertex};
use crate::draw::properties::spatial::{orientation, position};
use crate::draw::properties::{SetColor, SetOrientation, SetPosition};
use crate::draw::{self, Drawing};
use bevy::prelude::*;
use bevy::render::mesh::Indices;
use std::ops;
use nannou_core::geom;

/// The mesh type prior to being initialised with vertices or indices.
#[derive(Clone, Debug, Default)]
pub struct Vertexless<M> {
    _marker: std::marker::PhantomData<M>,
}

/// Properties related to drawing an arbitrary mesh of colours, geometry and texture.
#[derive(Clone, Debug)]
pub struct PrimitiveMesh<M: Material> {
    position: position::Properties,
    orientation: orientation::Properties,
    vertex_range: ops::Range<usize>,
    index_range: ops::Range<usize>,
    fill_color: Option<FillColor>,
    texture_handle: Option<Handle<Image>>,
    material: M,
}

#[derive(Clone, Debug, Default)]
struct FillColor(Option<Color>);

// A simple iterator for flattening a fixed-size array of indices.
struct FlattenIndices<I> {
    iter: I,
    index: usize,
    vertex_start_index: usize,
    current: [usize; 3],
}

pub type DrawingMesh<'a, M: Material> = Drawing<'a, PrimitiveMesh<M>>;

impl <M:Material> Vertexless<M> {
    /// Describe the mesh with a sequence of textured points.
    ///
    /// Each of the vertices must be represented as a tuple containing the point and tex
    /// coordinates in that order, e.g. `(point, tex_coords)`. `point` may be of any type that
    /// implements `Into<Vec3>` and `tex_coords` may be of any type that implements
    /// `Into<Vec2>`.
    pub fn points_textured<I, P, T>(
        self,
        inner_mesh: &mut Mesh,
        texture_handle: Handle<Image>,
        points: I,
    ) -> PrimitiveMesh<M>
    where
        I: IntoIterator<Item = (P, T)>,
        P: Into<Vec3>,
        T: Into<Vec2>,
    {
        let points = points.into_iter().map(|(p, t)| {
            let point = p.into();
            let color = Color::default();
            let tex_coords = t.into();
            (point, color, tex_coords)
        });
        self.points_inner(inner_mesh, points, Some(texture_handle))
    }

    /// Describe the mesh with a sequence of colored points.
    ///
    /// Each of the points must be represented as a tuple containing the point and the color in
    /// that order, e.g. `(point, color)`. `point` may be of any type that implements
    /// `Into<Vec3>` and `color` may be of any type that implements `IntoColor`.
    pub fn points_colored<I, P, C>(self, inner_mesh: &mut Mesh, points: I) -> PrimitiveMesh<M>
    where
        I: IntoIterator<Item = (P, C)>,
        P: Into<Vec3>,
        C: Into<Color>,
    {
        let vertices = points.into_iter().map(|(p, c)| {
            let point = p.into();
            let color = c.into();
            let tex_coords = Vec2::ZERO;
            (point, color, tex_coords)
        });
        self.points_inner(inner_mesh, vertices, None)
    }

    /// Describe the mesh with a sequence of points.
    ///
    /// The given iterator may yield any type that can be converted directly into `Vec3`s.
    ///
    /// This method assumes that the entire mesh should be coloured with a single colour. If a
    /// colour is not specified via one of the builder methods, a default colour will be retrieved
    /// from the inner `Theme`.
    pub fn points<I>(self, inner_mesh: &mut Mesh, points: I) -> PrimitiveMesh<M>
    where
        I: IntoIterator,
        I::Item: Into<Vec3>,
    {
        let vertices = points.into_iter().map(|p| {
            let point = p.into();
            let color = Color::default();
            let tex_coords = Vec2::ZERO;
            (point, color, tex_coords)
        });
        let mut mesh = self.points_inner(inner_mesh, vertices, None);
        mesh.fill_color = Some(FillColor(None));
        mesh
    }

    fn points_inner<I>(
        self,
        inner_mesh: &mut Mesh,
        vertices: I,
        texture_handle: Option<Handle<Image>>,
    ) -> PrimitiveMesh<M>
    where
        I: Iterator<Item = Vertex>,
    {
        let v_start = inner_mesh.count_vertices();
        let i_start = inner_mesh.count_indices();
        for (i, (point, color, tex_coords)) in vertices.enumerate() {
            inner_mesh.points_mut().push(point.to_array());
            inner_mesh.colors_mut().push(color.linear().to_f32_array());
            inner_mesh.tex_coords_mut().push(tex_coords.to_array());
            inner_mesh.normals_mut().push([0.0, 0.0, 1.0]);
            inner_mesh.push_index(i as u32);
        }
        let v_end = inner_mesh.count_vertices();
        let i_end = inner_mesh.count_indices();
        PrimitiveMesh::new(v_start..v_end, i_start..i_end, texture_handle)
    }

    /// Describe the mesh with a sequence of textured triangles.
    ///
    /// Each of the vertices must be represented as a tuple containing the point and tex
    /// coordinates in that order, e.g. `(point, tex_coords)`. `point` may be of any type that
    /// implements `Into<Vec3>` and `tex_coords` may be of any type that implements
    /// `Into<Vec2>`.
    pub fn tris_textured<I, P, T>(
        self,
        inner_mesh: &mut Mesh,
        texture_handle: Handle<Image>,
        tris: I,
    ) -> PrimitiveMesh<M>
    where
        I: IntoIterator<Item = geom::Tri<(P, T)>>,
        P: Into<Vec3>,
        T: Into<Vec2>,
    {
        let points = tris
            .into_iter()
            .map(|t| t.map_vertices(|(p, t)| (p.into(), t.into())))
            .flat_map(geom::Tri::vertices);
        self.points_textured(inner_mesh, texture_handle, points)
    }

    /// Describe the mesh with a sequence of colored triangles.
    ///
    /// Each of the vertices must be represented as a tuple containing the point and the color in
    /// that order, e.g. `(point, color)`. `point` may be of any type that implements `Into<Vec3>`
    /// and `color` may be of any type that implements `IntoColor`.
    pub fn tris_colored<I, P, C>(self, inner_mesh: &mut Mesh, tris: I) -> PrimitiveMesh<M>
    where
        I: IntoIterator<Item = geom::Tri<(P, C)>>,
        P: Into<Vec3>,
        C: Into<Color>,
    {
        let points = tris
            .into_iter()
            .map(|t| t.map_vertices(|(p, c)| (p.into(), c.into())))
            .flat_map(geom::Tri::vertices);
        self.points_colored(inner_mesh, points)
    }

    /// Describe the mesh with a sequence of triangles.
    ///
    /// Each triangle may be composed of any vertex type that may be converted directly into
    /// `Vec3`s.
    ///
    /// This method assumes that the entire mesh should be coloured with a single colour. If a
    /// colour is not specified via one of the builder methods, a default colour will be retrieved
    /// from the inner `Theme`.
    pub fn tris<I, V>(self, inner_mesh: &mut Mesh, tris: I) -> PrimitiveMesh<M>
    where
        I: IntoIterator<Item = geom::Tri<V>>,
        V: Into<Vec3>,
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
    /// implements `Into<Vec3>` and `tex_coords` may be of any type that implements
    /// `Into<Vec2>`.
    pub fn indexed_textured<V, I, P, T>(
        self,
        inner_mesh: &mut Mesh,
        texture_handle: Handle<Image>,
        points: V,
        indices: I,
    ) -> PrimitiveMesh<M>
    where
        V: IntoIterator<Item = (P, T)>,
        I: IntoIterator<Item = usize>,
        P: Into<Vec3>,
        T: Into<Vec2>,
    {
        let vertices = points.into_iter().map(|(p, t)| {
            let point = p.into();
            let color = Color::default();
            let tex_coords = t.into();
            (point, color, tex_coords)
        });
        self.indexed_inner(
            inner_mesh,
            vertices,
            indices,
            Some(texture_handle),
        )
    }

    /// Describe the mesh with the given indexed, colored points.
    ///
    /// Each trio of `indices` describes a single triangle made up of colored `points`.
    ///
    /// Each of the `points` must be represented as a tuple containing the point and the color in
    /// that order, e.g. `(point, color)`. `point` may be of any type that implements
    /// `Into<Vec3>` and `color` may be of any type that implements `IntoColor`.
    pub fn indexed_colored<V, I, P, C>(
        self,
        inner_mesh: &mut Mesh,
        points: V,
        indices: I,
    ) -> PrimitiveMesh<M>
    where
        V: IntoIterator<Item = (P, C)>,
        I: IntoIterator<Item = usize>,
        P: Into<Vec3>,
        C: Into<Color>,
    {
        let vertices = points.into_iter().map(|(p, c)| {
            let point = p.into();
            let color = c.into();
            let tex_coords = Vec2::ZERO;
            (point, color, tex_coords)
        });
        self.indexed_inner(inner_mesh, vertices, indices, None)
    }

    /// Describe the mesh with the given indexed points.
    ///
    /// Each trio of `indices` describes a single triangle made up of `points`.
    ///
    /// Each point may be any type that may be converted directly into the `Vec3` type.
    pub fn indexed<V, I>(self, inner_mesh: &mut Mesh, points: V, indices: I) -> PrimitiveMesh<M>
    where
        V: IntoIterator,
        V::Item: Into<Vec3>,
        I: IntoIterator<Item = usize>,
    {
        let vertices = points.into_iter().map(|p| {
            let point = p.into();
            let color = Color::default();
            let tex_coords = Vec2::ZERO;
            (point, color, tex_coords)
        });
        let mut mesh = self.indexed_inner(inner_mesh, vertices, indices, None);
        mesh.fill_color = Some(FillColor(None));
        mesh
    }

    fn indexed_inner<V, I>(
        self,
        inner_mesh: &mut Mesh,
        vertices: V,
        indices: I,
        texture_handle: Option<Handle<Image>>,
    ) -> PrimitiveMesh<M>
    where
        V: IntoIterator<Item = Vertex>,
        I: IntoIterator<Item = usize>,
    {
        let v_start = inner_mesh.count_vertices();
        let i_start = inner_mesh.count_indices();

        for (point, color, tex_coords) in vertices.into_iter() {
            inner_mesh.points_mut().push(point.to_array());
            inner_mesh.colors_mut().push(color.linear().to_f32_array());
            inner_mesh.tex_coords_mut().push(tex_coords.to_array());
            inner_mesh.normals_mut().push([0.0, 0.0, 1.0]);
        }
        for index in indices {
            inner_mesh.push_index(index as u32);
        }

        let v_end = inner_mesh.count_vertices();
        let i_end = inner_mesh.count_indices();
        PrimitiveMesh::new(v_start..v_end, i_start..i_end, texture_handle)
    }
}

impl <M:Material> PrimitiveMesh<M> {
    // Initialise a new `Mesh` with its ranges into the intermediary mesh, ready for drawing.
    fn new(
        vertex_range: ops::Range<usize>,
        index_range: ops::Range<usize>,
        texture_handle: Option<Handle<Image>>,
    ) -> Self {
        let orientation = Default::default();
        let position = Default::default();
        let fill_color = None;
        PrimitiveMesh {
            orientation,
            position,
            vertex_range,
            index_range,
            fill_color,
            texture_handle: texture_handle,
        }
    }
}

impl<'a, M: Material> Drawing<'a, Vertexless<M>> {
    /// Describe the mesh with a sequence of points.
    ///
    /// The given iterator may yield any type that can be converted directly into `Vec3`s.
    ///
    /// This method assumes that the entire mesh should be coloured with a single colour. If a
    /// colour is not specified via one of the builder methods, a default colour will be retrieved
    /// from the inner `Theme`.
    pub fn points<I>(self, points: I) -> DrawingMesh<'a, M>
    where
        I: IntoIterator,
        I::Item: Into<Vec3>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.points(ctxt.mesh, points))
    }

    /// Describe the mesh with a sequence of colored points.
    ///
    /// Each of the points must be represented as a tuple containing the point and the color in
    /// that order, e.g. `(point, color)`. `point` may be of any type that implements
    /// `Into<Vec3>` and `color` may be of any type that implements `IntoColor`.
    pub fn points_colored<I, P, C>(self, points: I) -> DrawingMesh<'a, M>
    where
        I: IntoIterator<Item = (P, C)>,
        P: Into<Vec3>,
        C: Into<Color>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.points_colored(ctxt.mesh, points))
    }

    /// Describe the mesh with a sequence of textured points.
    ///
    /// Each of the vertices must be represented as a tuple containing the point and tex
    /// coordinates in that order, e.g. `(point, tex_coords)`. `point` may be of any type that
    /// implements `Into<Vec3>` and `tex_coords` may be of any type that implements
    /// `Into<Vec2>`.
    pub fn points_textured<I, P, T>(
        self,
        texture_handle: Handle<Image>,
        points: I,
    ) -> DrawingMesh<'a, M>
    where
        I: IntoIterator<Item = (P, T)>,
        P: Into<Vec3>,
        T: Into<Vec2>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.points_textured(ctxt.mesh, texture_handle, points))
    }

    /// Describe the mesh with a sequence of triangles.
    ///
    /// Each triangle may be composed of any vertex type that may be converted directly into
    /// `Vec3`s.
    ///
    /// This method assumes that the entire mesh should be coloured with a single colour. If a
    /// colour is not specified via one of the builder methods, a default colour will be retrieved
    /// from the inner `Theme`.
    pub fn tris<I, V>(self, tris: I) -> DrawingMesh<'a, M>
    where
        I: IntoIterator<Item = geom::Tri<V>>,
        V: Into<Vec3>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.tris(ctxt.mesh, tris))
    }

    /// Describe the mesh with a sequence of colored triangles.
    ///
    /// Each of the vertices must be represented as a tuple containing the point and the color in
    /// that order, e.g. `(point, color)`. `point` may be of any type that implements `Into<Vec3>`
    /// and `color` may be of any type that implements `IntoColor`.
    pub fn tris_colored<I, P, C>(self, tris: I) -> DrawingMesh<'a, M>
    where
        I: IntoIterator<Item = geom::Tri<(P, C)>>,
        P: Into<Vec3>,
        C: Into<Color>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.tris_colored(ctxt.mesh, tris))
    }

    /// Describe the mesh with a sequence of textured triangles.
    ///
    /// Each of the vertices must be represented as a tuple containing the point and tex
    /// coordinates in that order, e.g. `(point, tex_coords)`. `point` may be of any type that
    /// implements `Into<Vec3>` and `tex_coords` may be of any type that implements
    /// `Into<Vec2>`.
    pub fn tris_textured<I, P, T>(self, texture_handle: Handle<Image>, tris: I) -> DrawingMesh<'a, M>
    where
        I: IntoIterator<Item = geom::Tri<(P, T)>>,
        P: Into<Vec3>,
        T: Into<Vec2>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.tris_textured(ctxt.mesh, texture_handle, tris))
    }

    /// Describe the mesh with the given indexed points.
    ///
    /// Each trio of `indices` describes a single triangle made up of `points`.
    ///
    /// Each point may be any type that may be converted directly into the `Vec3` type.
    pub fn indexed<V, I>(self, points: V, indices: I) -> DrawingMesh<'a, M>
    where
        V: IntoIterator,
        V::Item: Into<Vec3>,
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
    /// `Into<Vec3>` and `color` may be of any type that implements `IntoColor`.
    pub fn indexed_colored<V, I, P, C>(self, points: V, indices: I) -> DrawingMesh<'a, M>
    where
        V: IntoIterator<Item = (P, C)>,
        I: IntoIterator<Item = usize>,
        P: Into<Vec3>,
        C: Into<Color>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.indexed_colored(ctxt.mesh, points, indices))
    }

    /// Describe the mesh with the given indexed, textured points.
    ///
    /// Each trio of `indices` describes a single triangle made up of colored `points`.
    ///
    /// Each of the `points` must be represented as a tuple containing the point and the texture
    /// coordinates in that order, e.g. `(point, tex_coords)`. `point` may be of any type that
    /// implements `Into<Vec3>` and `tex_coords` may be of any type that implements
    /// `Into<Vec2>`.
    pub fn indexed_textured<V, I, P, T>(
        self,
        texture_handle: Handle<Image>,
        points: V,
        indices: I,
    ) -> DrawingMesh<'a, M>
    where
        V: IntoIterator<Item = (P, T)>,
        I: IntoIterator<Item = usize>,
        P: Into<Vec3>,
        T: Into<Vec2>,
    {
        self.map_ty_with_context(|ty, ctxt| {
            ty.indexed_textured(ctxt.mesh, texture_handle, points, indices)
        })
    }
}

impl <M: Material> draw::render::RenderPrimitive for PrimitiveMesh<M> {
    fn render_primitive(
        self,
        ctxt: draw::render::RenderContext,
        mesh: &mut Mesh,
    ) -> draw::render::PrimitiveRender {
        let PrimitiveMesh {
            orientation,
            position,
            vertex_range,
            index_range,
            fill_color,
            texture_handle: texture_handle,
        } = self;

        // Determine the transform to apply to vertices.
        let global_transform = *ctxt.transform;
        let local_transform = position.transform() * orientation.transform();
        let transform = global_transform * local_transform;

        // We need to update the indices to point to where vertices will be in the new mesh.
        let old_mesh_vertex_start = vertex_range.start as u32;
        let new_mesh_vertex_start = mesh.count_vertices() as u32;
        let indices = index_range
            .map(|i| ctxt.intermediary_mesh.get_index(i))
            .map(|i| new_mesh_vertex_start + i - old_mesh_vertex_start);

        // A small function for transforming a point via the transform matrix.
        let transform_point = |p: Vec3| -> Vec3 { transform.transform_point3(p) };

        // Color the vertices based on whether or not we should fill, then extend the mesh!
        match fill_color {
            Some(fill) => {
                let theme_prim = draw::theme::Primitive::Mesh;
                let color = fill.0.unwrap_or_else(|| ctxt.theme.fill(&theme_prim));
                let vertices = vertex_range.map(|i| {
                    let point = transform_point(ctxt.intermediary_mesh.points()[i].into());
                    let tex_coords: Vec2 = ctxt.intermediary_mesh.tex_coords()[i].into();
                    (point, color, tex_coords)
                });

                for (point, color, tex_coords) in vertices {
                    mesh.points_mut().push(point.to_array());
                    mesh.colors_mut().push(color.linear().to_f32_array());
                    mesh.tex_coords_mut().push(tex_coords.to_array());
                    mesh.normals_mut().push([0.0, 0.0, 1.0]);
                }
                for index in indices {
                    mesh.push_index(index);
                }
            }
            None => {
                let vertices = vertex_range.map(|i| {
                    let point = transform_point(ctxt.intermediary_mesh.points()[i].into());
                    let [r, g, b, a] = ctxt.intermediary_mesh.colors()[i];
                    let color = Color::LinearRgba(LinearRgba::new(r, g, b, a));
                    let tex_coords: Vec2 = ctxt.intermediary_mesh.tex_coords()[i].into();
                    (point, color, tex_coords)
                });

                for (point, color, tex_coords) in vertices {
                    mesh.points_mut().push(point.to_array());
                    mesh.colors_mut().push(color.linear().to_f32_array());
                    mesh.tex_coords_mut().push(tex_coords.to_array());
                    mesh.normals_mut().push([0.0, 0.0, 1.0]);
                }
                for index in indices {
                    mesh.push_index(index);
                }
            }
        }

        draw::render::PrimitiveRender {
            texture_handle,
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

impl <M: Material> SetOrientation for PrimitiveMesh<M> {
    fn properties(&mut self) -> &mut orientation::Properties {
        SetOrientation::properties(&mut self.orientation)
    }
}

impl <M: Material> SetPosition for PrimitiveMesh<M> {
    fn properties(&mut self) -> &mut position::Properties {
        SetPosition::properties(&mut self.position)
    }
}

impl <M: Material> SetColor for PrimitiveMesh<M> {
    fn color_mut(&mut self) -> &mut Option<Color> {
        &mut self.fill_color.get_or_insert_with(Default::default).0
    }
}

impl <M: Material> From<Vertexless<M>> for Primitive {
    fn from(prim: Vertexless<M>) -> Self {
        Primitive::MeshVertexless(prim)
    }
}

impl <M: Material> From<PrimitiveMesh<M>> for Primitive {
    fn from(prim: PrimitiveMesh<M>) -> Self {
        Primitive::Mesh(prim)
    }
}

impl <M: Material> Into<Option<Vertexless<M>>> for Primitive {
    fn into(self) -> Option<Vertexless<M>> {
        match self {
            Primitive::MeshVertexless(prim) => Some(prim),
            _ => None,
        }
    }
}

impl <M: Material> Into<Option<PrimitiveMesh<M>>> for Primitive {
    fn into(self) -> Option<PrimitiveMesh<M>> {
        match self {
            Primitive::Mesh(prim) => Some(prim),
            _ => None,
        }
    }
}
