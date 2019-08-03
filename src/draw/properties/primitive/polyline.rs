use crate::color::conv::IntoLinSrgba;
use crate::draw::mesh::vertex::IntoVertex;
use crate::draw::properties::spatial::{self, orientation, position};
use crate::draw::properties::{Draw, Drawn, IntoDrawn, Primitive, SetOrientation, SetPosition};
use crate::draw::{self, Drawing};
use crate::geom::{self, pt2};
use crate::math::BaseFloat;
use lyon::tessellation::geometry_builder::{self, GeometryBuilder, GeometryBuilderError, VertexId};
use lyon::tessellation::{LineCap, LineJoin, StrokeOptions, StrokeVertex};
use std::cell::Cell;
use std::iter;
use std::ops;

/// A polyline prior to being initialised.
#[derive(Clone, Debug, Default)]
pub struct Vertexless {
    opts: StrokeOptions,
    close: bool,
}

/// Properties related to drawing a **Polyline**.
#[derive(Clone, Debug)]
pub struct Polyline<S = geom::scalar::Default> {
    position: position::Properties<S>,
    orientation: orientation::Properties<S>,
    vertex_data_ranges: draw::IntermediaryVertexDataRanges,
    index_range: ops::Range<usize>,
}

struct PolylineGeometryBuilder<'a, 'mesh, S = geom::scalar::Default> {
    builder: &'a mut draw::IntermediaryMeshBuilder<'mesh, S>,
    color: &'a Cell<draw::mesh::vertex::Color>,
}

impl Vertexless {
    /// The start line cap as specified by the SVG spec.
    pub fn start_cap(mut self, cap: LineCap) -> Self {
        self.opts.start_cap = cap;
        self
    }

    /// The end line cap as specified by the SVG spec.
    pub fn end_cap(mut self, cap: LineCap) -> Self {
        self.opts.end_cap = cap;
        self
    }

    /// The start and end line cap as specified by the SVG spec.
    pub fn caps(self, cap: LineCap) -> Self {
        self.start_cap(cap).end_cap(cap)
    }

    /// The stroke for each sub-path does not extend beyond its two endpoints. A zero length
    /// sub-path will therefore not have any stroke.
    pub fn start_cap_butt(self) -> Self {
        self.start_cap(LineCap::Butt)
    }

    /// At the end of each sub-path, the shape representing the stroke will be extended by a
    /// rectangle with the same width as the stroke width and whose length is half of the stroke
    /// width. If a sub-path has zero length, then the resulting effect is that the stroke for that
    /// sub-path consists solely of a square with side length equal to the stroke width, centered
    /// at the sub-path's point.
    pub fn start_cap_square(self) -> Self {
        self.start_cap(LineCap::Square)
    }

    /// At each end of each sub-path, the shape representing the stroke will be extended by a half
    /// circle with a radius equal to the stroke width. If a sub-path has zero length, then the
    /// resulting effect is that the stroke for that sub-path consists solely of a full circle
    /// centered at the sub-path's point.
    pub fn start_cap_round(self) -> Self {
        self.start_cap(LineCap::Round)
    }

    /// The stroke for each sub-path does not extend beyond its two endpoints. A zero length
    /// sub-path will therefore not have any stroke.
    pub fn end_cap_butt(self) -> Self {
        self.end_cap(LineCap::Butt)
    }

    /// At the end of each sub-path, the shape representing the stroke will be extended by a
    /// rectangle with the same width as the stroke width and whose length is half of the stroke
    /// width. If a sub-path has zero length, then the resulting effect is that the stroke for that
    /// sub-path consists solely of a square with side length equal to the stroke width, centered
    /// at the sub-path's point.
    pub fn end_cap_square(self) -> Self {
        self.end_cap(LineCap::Square)
    }

    /// At each end of each sub-path, the shape representing the stroke will be extended by a half
    /// circle with a radius equal to the stroke width. If a sub-path has zero length, then the
    /// resulting effect is that the stroke for that sub-path consists solely of a full circle
    /// centered at the sub-path's point.
    pub fn end_cap_round(self) -> Self {
        self.end_cap(LineCap::Round)
    }

    /// The stroke for each sub-path does not extend beyond its two endpoints. A zero length
    /// sub-path will therefore not have any stroke.
    pub fn caps_butt(self) -> Self {
        self.caps(LineCap::Butt)
    }

    /// At the end of each sub-path, the shape representing the stroke will be extended by a
    /// rectangle with the same width as the stroke width and whose length is half of the stroke
    /// width. If a sub-path has zero length, then the resulting effect is that the stroke for that
    /// sub-path consists solely of a square with side length equal to the stroke width, centered
    /// at the sub-path's point.
    pub fn caps_square(self) -> Self {
        self.caps(LineCap::Square)
    }

    /// At each end of each sub-path, the shape representing the stroke will be extended by a half
    /// circle with a radius equal to the stroke width. If a sub-path has zero length, then the
    /// resulting effect is that the stroke for that sub-path consists solely of a full circle
    /// centered at the sub-path's point.
    pub fn caps_round(self) -> Self {
        self.caps(LineCap::Round)
    }

    /// Draw a line between the end point and the start point, closing the polyline.
    ///
    /// Default value is `false`.
    pub fn close(mut self) -> Self {
        self.close = true;
        self
    }

    /// The way in which lines are joined at the vertices, matching the SVG spec.
    ///
    /// Default value is `MiterClip`.
    pub fn join(mut self, join: LineJoin) -> Self {
        self.opts.line_join = join;
        self
    }

    /// A sharp corner is to be used to join path segments.
    pub fn join_miter(self) -> Self {
        self.join(LineJoin::Miter)
    }

    /// Same as a `join_miter`, but if the miter limit is exceeded, the miter is clipped at a miter
    /// length equal to the miter limit value multiplied by the stroke width.
    pub fn join_miter_clip(self) -> Self {
        self.join(LineJoin::MiterClip)
    }

    /// A round corner is to be used to join path segments.
    pub fn join_round(self) -> Self {
        self.join(LineJoin::Round)
    }

    /// A bevelled corner is to be used to join path segments. The bevel shape is a triangle that
    /// fills the area between the two stroked segments.
    pub fn join_bevel(self) -> Self {
        self.join(LineJoin::Bevel)
    }

    /// The total thickness (aka width) of the line.
    pub fn thickness(mut self, thickness: f32) -> Self {
        self.opts.line_width = thickness;
        self
    }

    /// Describes the limit before miter lines will clip, as described in the SVG spec.
    ///
    /// Must be greater than or equal to `1.0`.
    pub fn miter_limit(mut self, limit: f32) -> Self {
        self.opts.miter_limit = limit;
        self
    }

    /// Maximum allowed distance to the path when building an approximation.
    pub fn tolerance(mut self, tolerance: f32) -> Self {
        self.opts.tolerance = tolerance;
        self
    }

    /// Specify the full set of stroke options for the polyline tessellation.
    pub fn stroke_options(mut self, opts: StrokeOptions) -> Self {
        self.opts = opts;
        self
    }

    /// Draw a polyline whose path is defined by the given list of vertices.
    pub(crate) fn vertices<S, I>(
        self,
        mesh: &mut draw::IntermediaryMesh<S>,
        vertices: I,
    ) -> Polyline<S>
    where
        S: BaseFloat,
        I: IntoIterator,
        I::Item: IntoVertex<S>,
    {
        let color = Cell::new(crate::color::WHITE.into_lin_srgba());
        let color = &color;
        let v_iter = vertices.into_iter().map(IntoVertex::into_vertex).map(|v| {
            color.set(v.color);
            let p: geom::Point3 = v.cast().expect("failed to cast point");
            lyon::tessellation::math::Point::new(p.x, p.y)
        });
        let mut builder = mesh.builder();
        let res = lyon::tessellation::basic_shapes::stroke_polyline(
            v_iter,
            self.close,
            &self.opts,
            &mut PolylineGeometryBuilder {
                builder: &mut builder,
                color,
            },
        );
        if let Err(err) = res {
            eprintln!("failed to tessellate polyline: {:?}", err);
        }
        Polyline::new(builder.vertex_data_ranges(), builder.index_range())
    }
}

impl<S> Polyline<S>
where
    S: BaseFloat,
{
    // Initialise a new `Polyline` with its ranges into the intermediary mesh, ready for drawing.
    fn new(
        vertex_data_ranges: draw::IntermediaryVertexDataRanges,
        index_range: ops::Range<usize>,
    ) -> Self {
        let orientation = Default::default();
        let position = Default::default();
        Polyline {
            orientation,
            position,
            vertex_data_ranges,
            index_range,
        }
    }
}

impl<'a, S> Drawing<'a, Vertexless, S>
where
    S: BaseFloat,
{
    /// The start line cap as specified by the SVG spec.
    pub fn start_cap(self, cap: LineCap) -> Self {
        self.map_ty(|ty| ty.start_cap(cap))
    }

    /// The end line cap as specified by the SVG spec.
    pub fn end_cap(self, cap: LineCap) -> Self {
        self.map_ty(|ty| ty.end_cap(cap))
    }

    /// The start and end line cap as specified by the SVG spec.
    pub fn caps(self, cap: LineCap) -> Self {
        self.map_ty(|ty| ty.caps(cap))
    }

    /// The stroke for each sub-path does not extend beyond its two endpoints. A zero length
    /// sub-path will therefore not have any stroke.
    pub fn start_cap_butt(self) -> Self {
        self.map_ty(|ty| ty.start_cap_butt())
    }

    /// At the end of each sub-path, the shape representing the stroke will be extended by a
    /// rectangle with the same width as the stroke width and whose length is half of the stroke
    /// width. If a sub-path has zero length, then the resulting effect is that the stroke for that
    /// sub-path consists solely of a square with side length equal to the stroke width, centered
    /// at the sub-path's point.
    pub fn start_cap_square(self) -> Self {
        self.map_ty(|ty| ty.start_cap_square())
    }

    /// At each end of each sub-path, the shape representing the stroke will be extended by a half
    /// circle with a radius equal to the stroke width. If a sub-path has zero length, then the
    /// resulting effect is that the stroke for that sub-path consists solely of a full circle
    /// centered at the sub-path's point.
    pub fn start_cap_round(self) -> Self {
        self.map_ty(|ty| ty.start_cap_round())
    }

    /// The stroke for each sub-path does not extend beyond its two endpoints. A zero length
    /// sub-path will therefore not have any stroke.
    pub fn end_cap_butt(self) -> Self {
        self.map_ty(|ty| ty.end_cap_butt())
    }

    /// At the end of each sub-path, the shape representing the stroke will be extended by a
    /// rectangle with the same width as the stroke width and whose length is half of the stroke
    /// width. If a sub-path has zero length, then the resulting effect is that the stroke for that
    /// sub-path consists solely of a square with side length equal to the stroke width, centered
    /// at the sub-path's point.
    pub fn end_cap_square(self) -> Self {
        self.map_ty(|ty| ty.end_cap_square())
    }

    /// At each end of each sub-path, the shape representing the stroke will be extended by a half
    /// circle with a radius equal to the stroke width. If a sub-path has zero length, then the
    /// resulting effect is that the stroke for that sub-path consists solely of a full circle
    /// centered at the sub-path's point.
    pub fn end_cap_round(self) -> Self {
        self.map_ty(|ty| ty.end_cap_round())
    }

    /// The stroke for each sub-path does not extend beyond its two endpoints. A zero length
    /// sub-path will therefore not have any stroke.
    pub fn caps_butt(self) -> Self {
        self.map_ty(|ty| ty.caps_butt())
    }

    /// At the end of each sub-path, the shape representing the stroke will be extended by a
    /// rectangle with the same width as the stroke width and whose length is half of the stroke
    /// width. If a sub-path has zero length, then the resulting effect is that the stroke for that
    /// sub-path consists solely of a square with side length equal to the stroke width, centered
    /// at the sub-path's point.
    pub fn caps_square(self) -> Self {
        self.map_ty(|ty| ty.caps_square())
    }

    /// At each end of each sub-path, the shape representing the stroke will be extended by a half
    /// circle with a radius equal to the stroke width. If a sub-path has zero length, then the
    /// resulting effect is that the stroke for that sub-path consists solely of a full circle
    /// centered at the sub-path's point.
    pub fn caps_round(self) -> Self {
        self.map_ty(|ty| ty.caps_round())
    }

    /// Draw a line between the end point and the start point, closing the polyline.
    ///
    /// Default value is `false`.
    pub fn close(self) -> Self {
        self.map_ty(|ty| ty.close())
    }

    /// The way in which lines are joined at the vertices, matching the SVG spec.
    ///
    /// Default value is `MiterClip`.
    pub fn join(self, join: LineJoin) -> Self {
        self.map_ty(|ty| ty.join(join))
    }

    /// A sharp corner is to be used to join path segments.
    pub fn join_miter(self) -> Self {
        self.map_ty(|ty| ty.join_miter())
    }

    /// Same as a `join_miter`, but if the miter limit is exceeded, the miter is clipped at a miter
    /// length equal to the miter limit value multiplied by the stroke width.
    pub fn join_miter_clip(self) -> Self {
        self.map_ty(|ty| ty.join_miter_clip())
    }

    /// A round corner is to be used to join path segments.
    pub fn join_round(self) -> Self {
        self.map_ty(|ty| ty.join_round())
    }

    /// A bevelled corner is to be used to join path segments. The bevel shape is a triangle that
    /// fills the area between the two stroked segments.
    pub fn join_bevel(self) -> Self {
        self.map_ty(|ty| ty.join_bevel())
    }

    /// The total thickness (aka width) of the line.
    pub fn thickness(self, thickness: f32) -> Self {
        self.map_ty(|ty| ty.thickness(thickness))
    }

    /// Describes the limit before miter lines will clip, as described in the SVG spec.
    ///
    /// Must be greater than or equal to `1.0`.
    pub fn miter_limit(self, limit: f32) -> Self {
        self.map_ty(|ty| ty.miter_limit(limit))
    }

    /// Maximum allowed distance to the path when building an approximation.
    pub fn tolerance(self, tolerance: f32) -> Self {
        self.map_ty(|ty| ty.tolerance(tolerance))
    }

    /// Specify the full set of stroke options for the polyline tessellation.
    pub fn stroke_options(self, opts: StrokeOptions) -> Self {
        self.map_ty(|ty| ty.stroke_options(opts))
    }

    /// Describe the polyline with some "half_thickness" of the line and the given sequence of
    /// vertices.
    pub fn vertices<I>(self, vertices: I) -> Drawing<'a, Polyline<S>, S>
    where
        I: IntoIterator,
        I::Item: IntoVertex<S>,
    {
        self.map_ty_with_vertices(|ty, mesh| ty.vertices(mesh, vertices))
    }
}

impl<'a, 'mesh, S> GeometryBuilder<StrokeVertex> for PolylineGeometryBuilder<'a, 'mesh, S>
where
    S: BaseFloat,
{
    fn begin_geometry(&mut self) {
        self.builder.begin_geometry();
    }

    fn end_geometry(&mut self) -> geometry_builder::Count {
        self.builder.end_geometry()
    }

    fn add_vertex(&mut self, v: StrokeVertex) -> Result<VertexId, GeometryBuilderError> {
        let color = self.color.get();
        let point = pt2(v.position.x, v.position.y)
            .cast()
            .expect("failed to cast point");
        let v: draw::mesh::Vertex<S> = (point, color).into_vertex();
        self.builder.add_vertex(v)
    }

    fn add_triangle(&mut self, a: VertexId, b: VertexId, c: VertexId) {
        self.builder.add_triangle(a, b, c);
    }

    fn abort_geometry(&mut self) {
        self.builder.abort_geometry();
    }
}

impl<S> IntoDrawn<S> for Vertexless
where
    S: BaseFloat,
{
    type Vertices = iter::Empty<draw::mesh::Vertex<S>>;
    type Indices = iter::Empty<usize>;
    fn into_drawn(self, _draw: Draw<S>) -> Drawn<S, Self::Vertices, Self::Indices> {
        let properties = Default::default();
        let vertices = iter::empty();
        let indices = iter::empty();
        (properties, vertices, indices)
    }
}

impl<S> IntoDrawn<S> for Polyline<S>
where
    S: BaseFloat,
{
    type Vertices = draw::properties::VerticesFromRanges;
    type Indices = draw::properties::IndicesFromRange;
    fn into_drawn(self, _draw: Draw<S>) -> Drawn<S, Self::Vertices, Self::Indices> {
        let Polyline {
            orientation,
            position,
            vertex_data_ranges,
            index_range,
        } = self;
        let dimensions = spatial::dimension::Properties::default();
        let spatial = spatial::Properties {
            dimensions,
            orientation,
            position,
        };
        let vertices = draw::properties::VerticesFromRanges {
            ranges: vertex_data_ranges,
            fill_color: None,
        };
        let indices = draw::properties::IndicesFromRange { range: index_range };
        (spatial, vertices, indices)
    }
}

impl<S> SetOrientation<S> for Polyline<S> {
    fn properties(&mut self) -> &mut orientation::Properties<S> {
        SetOrientation::properties(&mut self.orientation)
    }
}

impl<S> SetPosition<S> for Polyline<S> {
    fn properties(&mut self) -> &mut position::Properties<S> {
        SetPosition::properties(&mut self.position)
    }
}

impl<S> From<Vertexless> for Primitive<S> {
    fn from(prim: Vertexless) -> Self {
        Primitive::PolylineVertexless(prim)
    }
}

impl<S> From<Polyline<S>> for Primitive<S> {
    fn from(prim: Polyline<S>) -> Self {
        Primitive::Polyline(prim)
    }
}

impl<S> Into<Option<Vertexless>> for Primitive<S> {
    fn into(self) -> Option<Vertexless> {
        match self {
            Primitive::PolylineVertexless(prim) => Some(prim),
            _ => None,
        }
    }
}

impl<S> Into<Option<Polyline<S>>> for Primitive<S> {
    fn into(self) -> Option<Polyline<S>> {
        match self {
            Primitive::Polyline(prim) => Some(prim),
            _ => None,
        }
    }
}
