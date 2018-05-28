use draw::{self, Drawing};
use draw::properties::{spatial, ColorScalar, Draw, Drawn, IntoDrawn, Primitive, Rgba, SetColor, SetDimensions, SetOrientation, SetPosition};
use draw::properties::spatial::{dimension, orientation, position};
use geom;
use math::{BaseFloat, ElementWise, Point2, Vector2};
use std::{iter, slice};

/// Properties related to drawing a **Line**.
#[derive(Clone, Debug)]
pub struct Line<S = geom::DefaultScalar> {
    line: geom::Line<S>,
    spatial: spatial::Properties<S>,
    color: Option<Rgba>,
}
// Line-specific methods.

impl<S> Line<S>
where
    S: BaseFloat,
{
    /// Create a new **Line**. from its geometric parts.
    pub fn new(start: Point2<S>, end: Point2<S>, half_thickness: S) -> Self {
        let color = Default::default();
        let spatial = Default::default();
        let line = geom::Line::new(start, end, half_thickness);
        Line { line, spatial, color }
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
        self.line.half_thickness = half_thickness;
        self
    }

    /// Specify the `start` point for the line.
    pub fn start(mut self, start: Point2<S>) -> Self {
        self.line.start = start;
        self
    }

    /// Specify the `end` point for the line.
    pub fn end(mut self, end: Point2<S>) -> Self {
        self.line.end = end;
        self
    }

    /// Use the given four points as the vertices (corners) of the quad.
    pub fn points(self, start: Point2<S>, end: Point2<S>) -> Self {
        self.start(start).end(end)
    }
}

// Trait implementations.

impl<S> IntoDrawn<S> for Line<S>
where
    S: BaseFloat,
{
    type Vertices = draw::mesh::vertex::IterFromPoint2s<geom::line::Vertices<S>, S>;
    type Indices = iter::Cloned<slice::Iter<'static, usize>>;
    fn into_drawn(self, draw: Draw<S>) -> Drawn<S, Self::Vertices, Self::Indices> {
        let Line {
            line,
            spatial,
            color,
        } = self;

        // If dimensions were specified, scale the points to those dimensions.
        let (maybe_x, maybe_y, maybe_z) = spatial.dimensions.to_scalars(&draw);
        assert!(
            maybe_z.is_none(),
            "z dimension support for ellipse is unimplemented"
        );

        let mut quad = line.quad_corners();
        if maybe_x.is_some() || maybe_y.is_some() {
            let rect = line.bounding_rect();
            let centroid = line.centroid();
            let x_scale = maybe_x.map(|x| x / rect.w()).unwrap_or_else(S::one);
            let y_scale = maybe_y.map(|y| y / rect.h()).unwrap_or_else(S::one);
            let scale = Vector2 { x: x_scale, y: y_scale };
            let (a, b, c, d) = quad.into();
            let translate = |v: Point2<S>| centroid + ((v - centroid).mul_element_wise(scale));
            let new_a = translate(a);
            let new_b = translate(b);
            let new_c = translate(c);
            let new_d = translate(d);
            quad = geom::Quad([new_a, new_b, new_c, new_d]);
        }

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

        let points = quad.vertices();
        let vertices = draw::mesh::vertex::IterFromPoint2s::new(points, color);
        let indices = geom::quad::TRIANGLE_INDICES.iter().cloned();

        (spatial, vertices, indices)
    }
}

impl<S> From<geom::Line<S>> for Line<S>
where
    S: BaseFloat,
{
    fn from(line: geom::Line<S>) -> Self {
        let spatial = <_>::default();
        let color = <_>::default();
        Line { line, spatial, color }
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
        SetOrientation::properties(&mut self.spatial)
    }
}

impl<S> SetPosition<S> for Line<S> {
    fn properties(&mut self) -> &mut position::Properties<S> {
        SetPosition::properties(&mut self.spatial)
    }
}

impl<S> SetDimensions<S> for Line<S> {
    fn properties(&mut self) -> &mut dimension::Properties<S> {
        SetDimensions::properties(&mut self.spatial)
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
}
