use crate::color::conv::IntoLinSrgba;
use crate::draw;
use crate::draw::primitive::polygon::{self, PolygonInit, PolygonOptions, SetPolygon};
use crate::draw::primitive::Primitive;
use crate::draw::properties::spatial::{dimension, orientation, position};
use crate::draw::properties::{
    spatial, ColorScalar, LinSrgba, SetColor, SetDimensions, SetOrientation, SetPosition, SetStroke,
};
use crate::draw::Drawing;
use crate::geom::{self, Vector2};
use crate::math::{BaseFloat, Zero};
use lyon::tessellation::StrokeOptions;

/// Properties related to drawing an **Ellipse**.
#[derive(Clone, Debug)]
pub struct Ellipse<S = geom::scalar::Default> {
    dimensions: spatial::dimension::Properties<S>,
    resolution: Option<usize>,
    polygon: PolygonInit<S>,
}

/// The drawing context for an ellipse.
pub type DrawingEllipse<'a, S = geom::scalar::Default> = Drawing<'a, Ellipse<S>, S>;

// Ellipse-specific methods.

impl<S> Ellipse<S>
where
    S: BaseFloat,
{
    /// Stroke the outline with the given color.
    pub fn stroke<C>(self, color: C) -> Self
    where
        C: IntoLinSrgba<ColorScalar>,
    {
        self.stroke_color(color)
    }

    /// Specify the width and height of the **Ellipse** via a given **radius**.
    pub fn radius(self, radius: S) -> Self {
        let side = radius * (S::one() + S::one());
        self.w_h(side, side)
    }

    /// The number of sides used to draw the ellipse.
    ///
    /// By default, ellipse does not use a resolution, but rather uses a stroke tolerance to
    /// determine how many vertices to use during tessellation.
    pub fn resolution(mut self, resolution: usize) -> Self {
        self.resolution = Some(resolution);
        self
    }
}

// Trait implementations.

impl<'e> draw::renderer::RenderPrimitive<'e> for Ellipse<f32> {
    fn render_primitive(
        self,
        ctxt: draw::renderer::RenderContext,
        mesh: &mut draw::Mesh,
    ) -> draw::renderer::PrimitiveRender<'e> {
        let Ellipse {
            dimensions,
            polygon,
            resolution,
        } = self;

        // First get the dimensions of the ellipse.
        let (maybe_x, maybe_y, maybe_z) = (dimensions.x, dimensions.y, dimensions.z);
        assert!(
            maybe_z.is_none(),
            "z dimension support for ellipse is unimplemented"
        );

        let w = maybe_x.map(f32::abs).unwrap_or(100.0);
        let h = maybe_y.map(f32::abs).unwrap_or(100.0);
        match resolution {
            None => {
                // Determine the transform to apply to all points.
                let radii = lyon::math::vector(w * 0.5, h * 0.5);
                if radii.square_length() > 0.0 {
                    let centre = lyon::math::point(0.0, 0.0);
                    let mut builder = lyon::path::Path::builder();
                    let sweep_angle = lyon::math::Angle::radians(std::f32::consts::PI * 2.0);
                    let x_rotation = lyon::math::Angle::radians(0.0);
                    let start = lyon::math::point(w * 0.5, 0.0);
                    builder.move_to(start);
                    builder.arc(centre, radii, sweep_angle, x_rotation);
                    let path = builder.build();
                    polygon::render_events_themed(
                        polygon.opts,
                        || (&path).into_iter(),
                        ctxt,
                        &draw::theme::Primitive::Ellipse,
                        mesh,
                    );
                }
            }
            Some(resolution) => {
                let rect = geom::Rect::from_wh(Vector2 { x: w, y: h });
                let ellipse = geom::Ellipse::new(rect, resolution);
                let points = ellipse.circumference();
                polygon::render_points_themed(
                    polygon.opts,
                    points,
                    ctxt,
                    &draw::theme::Primitive::Ellipse,
                    mesh,
                );
            }
        }

        draw::renderer::PrimitiveRender::default()
    }
}

impl<S> Default for Ellipse<S>
where
    S: Zero,
{
    fn default() -> Self {
        let dimensions = Default::default();
        let polygon = Default::default();
        let resolution = Default::default();
        Ellipse {
            dimensions,
            polygon,
            resolution,
        }
    }
}

impl<S> SetOrientation<S> for Ellipse<S> {
    fn properties(&mut self) -> &mut orientation::Properties<S> {
        SetOrientation::properties(&mut self.polygon)
    }
}

impl<S> SetPosition<S> for Ellipse<S> {
    fn properties(&mut self) -> &mut position::Properties<S> {
        SetPosition::properties(&mut self.polygon)
    }
}

impl<S> SetDimensions<S> for Ellipse<S> {
    fn properties(&mut self) -> &mut dimension::Properties<S> {
        SetDimensions::properties(&mut self.dimensions)
    }
}

impl<S> SetColor<ColorScalar> for Ellipse<S> {
    fn rgba_mut(&mut self) -> &mut Option<LinSrgba> {
        SetColor::rgba_mut(&mut self.polygon)
    }
}

impl<S> SetStroke for Ellipse<S> {
    fn stroke_options_mut(&mut self) -> &mut StrokeOptions {
        SetStroke::stroke_options_mut(&mut self.polygon)
    }
}

impl<S> SetPolygon<S> for Ellipse<S> {
    fn polygon_options_mut(&mut self) -> &mut PolygonOptions<S> {
        SetPolygon::polygon_options_mut(&mut self.polygon)
    }
}

// Primitive conversion.

impl<'e, S> From<Ellipse<S>> for Primitive<'e, S> {
    fn from(prim: Ellipse<S>) -> Self {
        Primitive::Ellipse(prim)
    }
}

impl<'e, S> Into<Option<Ellipse<S>>> for Primitive<'e, S> {
    fn into(self) -> Option<Ellipse<S>> {
        match self {
            Primitive::Ellipse(prim) => Some(prim),
            _ => None,
        }
    }
}

// Drawing methods.

impl<'e, S> DrawingEllipse<'e, S>
where
    S: BaseFloat,
{
    /// Stroke the outline with the given color.
    pub fn stroke<C>(self, color: C) -> Self
    where
        C: IntoLinSrgba<ColorScalar>,
    {
        self.map_ty(|ty| ty.stroke(color))
    }

    /// Specify the width and height of the **Ellipse** via a given **radius**.
    pub fn radius(self, radius: S) -> Self {
        self.map_ty(|ty| ty.radius(radius))
    }

    /// The number of sides used to draw the ellipse.
    pub fn resolution(self, resolution: usize) -> Self {
        self.map_ty(|ty| ty.resolution(resolution))
    }
}
