use crate::draw::properties::spatial::{orientation, position};
use crate::draw::properties::{
    spatial, ColorScalar, Draw, Drawn, IntoDrawn, Primitive, Rgba, SetColor, SetOrientation,
    SetPosition,
};
use crate::draw::{self, Drawing};
use crate::geom::{self, Point2};
use crate::math::BaseFloat;

/// Properties related to drawing a **Line**.
#[derive(Clone, Debug)]
pub struct Line<S = geom::scalar::Default> {
    position: position::Properties<S>,
    orientation: orientation::Properties<S>,
    capped: geom::line::Capped<S>,
    color: Option<Rgba>,
}

/// The default resolution of the rounded line caps.
pub const DEFAULT_ROUND_RESOLUTION: usize = 25;

// Line-specific methods.

impl<S> Line<S>
where
    S: BaseFloat,
{
    /// Create a new **Line**. from its geometric parts.
    pub fn new(start: Point2<S>, end: Point2<S>, half_thickness: S) -> Self {
        let color = Default::default();
        let position = Default::default();
        let orientation = Default::default();
        let line = geom::Line::new(start, end, half_thickness);
        let cap = geom::line::Cap::Butt;
        let capped = geom::line::Capped { line, cap };
        Line {
            position,
            orientation,
            capped,
            color,
        }
    }

    /// Specify the thickness of the **Line**.
    pub fn thickness(self, thickness: S) -> Self {
        self.half_thickness(thickness / (S::one() + S::one()))
    }

    /// Specify half the thickness of the **Line**.
    ///
    /// As the half-thickness is used more commonly within **Line** geometric calculations, this
    /// can be *slightly* more efficient than the full `thickness` method.
    pub fn half_thickness(mut self, half_thickness: S) -> Self {
        self.capped.line.half_thickness = half_thickness.abs();
        self
    }

    /// Specify the `start` point for the line.
    pub fn start(mut self, start: Point2<S>) -> Self {
        self.capped.line.start = start;
        self
    }

    /// Specify the `end` point for the line.
    pub fn end(mut self, end: Point2<S>) -> Self {
        self.capped.line.end = end;
        self
    }

    /// Use the given four points as the vertices (corners) of the quad.
    pub fn points(self, start: Point2<S>, end: Point2<S>) -> Self {
        self.start(start).end(end)
    }

    /// Draw rounded caps on the ends of the line.
    ///
    /// The radius of the semi-circle is equal to the line's `half_thickness`.
    pub fn caps_round(self) -> Self {
        self.caps_round_with_resolution(DEFAULT_ROUND_RESOLUTION)
    }

    /// Draw rounded caps on the ends of the line.
    ///
    /// The radius of the semi-circle is equal to the line's `half_thickness`.
    pub fn caps_round_with_resolution(mut self, resolution: usize) -> Self {
        self.capped.cap = geom::line::Cap::Round { resolution };
        self
    }

    /// Draw squared caps on the ends of the line.
    ///
    /// The length of the protrusion is equal to the line's `half_thickness`.
    pub fn caps_square(mut self) -> Self {
        self.capped.cap = geom::line::Cap::Square;
        self
    }
}

// Trait implementations.

impl<S> IntoDrawn<S> for Line<S>
where
    S: BaseFloat,
{
    type Vertices = draw::mesh::vertex::IterFromPoint2s<geom::line::CappedVertices<S>, S>;
    type Indices = geom::polygon::TriangleIndices;
    fn into_drawn(self, draw: Draw<S>) -> Drawn<S, Self::Vertices, Self::Indices> {
        let Line {
            capped,
            position,
            orientation,
            color,
        } = self;

        // The color.
        let color = color
            .or_else(|| {
                draw.theme(|theme| {
                    theme
                        .color
                        .primitive
                        .get(&draw::theme::Primitive::Line)
                        .map(|&c| c)
                })
            })
            .unwrap_or(draw.theme(|t| t.color.default));

        let dimensions = Default::default();
        let spatial = spatial::Properties {
            dimensions,
            position,
            orientation,
        };
        let points = capped.vertices();
        let indices = geom::polygon::triangle_indices(points.len());
        let vertices = draw::mesh::vertex::IterFromPoint2s::new(points, color);

        (spatial, vertices, indices)
    }
}

impl<S> From<geom::Line<S>> for Line<S>
where
    S: BaseFloat,
{
    fn from(line: geom::Line<S>) -> Self {
        let cap = <_>::default();
        let capped = geom::line::Capped { line, cap };
        capped.into()
    }
}

impl<S> From<geom::line::Capped<S>> for Line<S>
where
    S: BaseFloat,
{
    fn from(capped: geom::line::Capped<S>) -> Self {
        let position = <_>::default();
        let orientation = <_>::default();
        let color = <_>::default();
        Line {
            capped,
            position,
            orientation,
            color,
        }
    }
}

impl<S> Default for Line<S>
where
    S: BaseFloat,
{
    fn default() -> Self {
        // Create a quad pointing towards 0.0 radians.
        let zero = S::zero();
        let fifty = S::from(50.0).unwrap();
        let half_thickness = S::one();
        let left = -fifty;
        let right = fifty;
        let a = Point2 { x: left, y: zero };
        let b = Point2 { x: right, y: zero };
        Line::new(a, b, half_thickness)
    }
}

impl<S> SetOrientation<S> for Line<S> {
    fn properties(&mut self) -> &mut orientation::Properties<S> {
        SetOrientation::properties(&mut self.orientation)
    }
}

impl<S> SetPosition<S> for Line<S> {
    fn properties(&mut self) -> &mut position::Properties<S> {
        SetPosition::properties(&mut self.position)
    }
}

impl<S> SetColor<ColorScalar> for Line<S> {
    fn rgba_mut(&mut self) -> &mut Option<Rgba> {
        SetColor::rgba_mut(&mut self.color)
    }
}

// Primitive conversions.

impl<S> From<Line<S>> for Primitive<S> {
    fn from(prim: Line<S>) -> Self {
        Primitive::Line(prim)
    }
}

impl<S> Into<Option<Line<S>>> for Primitive<S> {
    fn into(self) -> Option<Line<S>> {
        match self {
            Primitive::Line(prim) => Some(prim),
            _ => None,
        }
    }
}

// Drawing methods.

impl<'a, S> Drawing<'a, Line<S>, S>
where
    S: BaseFloat,
{
    /// Specify the thickness of the **Line**.
    pub fn thickness(self, thickness: S) -> Self {
        self.map_ty(|ty| ty.thickness(thickness))
    }

    /// Specify half the thickness of the **Line**.
    ///
    /// As the half-thickness is used more commonly within **Line** geometric calculations, this
    /// can be *slightly* more efficient than the full `thickness` method.
    pub fn half_thickness(self, half_thickness: S) -> Self {
        self.map_ty(|ty| ty.half_thickness(half_thickness))
    }

    /// Specify the `start` point for the line.
    pub fn start(self, start: Point2<S>) -> Self {
        self.map_ty(|ty| ty.start(start))
    }

    /// Specify the `end` point for the line.
    pub fn end(self, end: Point2<S>) -> Self {
        self.map_ty(|ty| ty.end(end))
    }

    /// Use the given four points as the vertices (corners) of the quad.
    pub fn points(self, start: Point2<S>, end: Point2<S>) -> Self {
        self.map_ty(|ty| ty.points(start, end))
    }

    /// Draw rounded caps on the ends of the line.
    ///
    /// The radius of the semi-circle is equal to the line's `half_thickness`.
    pub fn caps_round(self) -> Self {
        self.map_ty(|ty| ty.caps_round())
    }

    /// Draw rounded caps on the ends of the line.
    ///
    /// The radius of the semi-circle is equal to the line's `half_thickness`.
    pub fn caps_round_with_resolution(self, resolution: usize) -> Self {
        self.map_ty(|ty| ty.caps_round_with_resolution(resolution))
    }

    /// Draw squared caps on the ends of the line.
    ///
    /// The length of the protrusion is equal to the line's `half_thickness`.
    pub fn caps_square(self) -> Self {
        self.map_ty(|ty| ty.caps_square())
    }
}
