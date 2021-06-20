use crate::color::conv::IntoLinSrgba;
use crate::draw;
use crate::draw::primitive::polygon::{self, PolygonInit, PolygonOptions, SetPolygon};
use crate::draw::primitive::Primitive;
use crate::draw::properties::spatial::{dimension, orientation, position};
use crate::draw::properties::{
    spatial, ColorScalar, LinSrgba, SetColor, SetDimensions, SetOrientation, SetPosition, SetStroke,
};
use crate::draw::Drawing;
use crate::geom;
use crate::glam::Vec2;
use lyon::tessellation::StrokeOptions;

/// Properties related to drawing an **Ellipse**.
#[derive(Clone, Debug, Default)]
pub struct Ellipse {
    dimensions: spatial::dimension::Properties,
    resolution: Option<f32>,
    polygon: PolygonInit,
}

/// The drawing context for an ellipse.
pub type DrawingEllipse<'a> = Drawing<'a, Ellipse>;

// Ellipse-specific methods.

impl Ellipse {
    /// Stroke the outline with the given color.
    pub fn stroke<C>(self, color: C) -> Self
    where
        C: IntoLinSrgba<ColorScalar>,
    {
        self.stroke_color(color)
    }

    /// Specify the width and height of the **Ellipse** via a given **radius**.
    pub fn radius(self, radius: f32) -> Self {
        let side = radius * 2.0;
        self.w_h(side, side)
    }

    /// The number of sides used to draw the ellipse.
    ///
    /// By default, ellipse does not use a resolution, but rather uses a stroke tolerance to
    /// determine how many vertices to use during tessellation.
    pub fn resolution(mut self, resolution: f32) -> Self {
        self.resolution = Some(resolution);
        self
    }
}

// Trait implementations.

impl draw::renderer::RenderPrimitive for Ellipse {
    fn render_primitive(
        self,
        ctxt: draw::renderer::RenderContext,
        mesh: &mut draw::Mesh,
    ) -> draw::renderer::PrimitiveRender {
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
                let rect = geom::Rect::from_w_h(w, h);
                let ellipse = geom::Ellipse::new(rect, resolution);
                let points = ellipse.circumference().map(Vec2::from);
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

impl SetOrientation for Ellipse {
    fn properties(&mut self) -> &mut orientation::Properties {
        SetOrientation::properties(&mut self.polygon)
    }
}

impl SetPosition for Ellipse {
    fn properties(&mut self) -> &mut position::Properties {
        SetPosition::properties(&mut self.polygon)
    }
}

impl SetDimensions for Ellipse {
    fn properties(&mut self) -> &mut dimension::Properties {
        SetDimensions::properties(&mut self.dimensions)
    }
}

impl SetColor<ColorScalar> for Ellipse {
    fn rgba_mut(&mut self) -> &mut Option<LinSrgba> {
        SetColor::rgba_mut(&mut self.polygon)
    }
}

impl SetStroke for Ellipse {
    fn stroke_options_mut(&mut self) -> &mut StrokeOptions {
        SetStroke::stroke_options_mut(&mut self.polygon)
    }
}

impl SetPolygon for Ellipse {
    fn polygon_options_mut(&mut self) -> &mut PolygonOptions {
        SetPolygon::polygon_options_mut(&mut self.polygon)
    }
}

// Primitive conversion.

impl From<Ellipse> for Primitive {
    fn from(prim: Ellipse) -> Self {
        Primitive::Ellipse(prim)
    }
}

impl Into<Option<Ellipse>> for Primitive {
    fn into(self) -> Option<Ellipse> {
        match self {
            Primitive::Ellipse(prim) => Some(prim),
            _ => None,
        }
    }
}

// Drawing methods.

impl<'a> DrawingEllipse<'a> {
    /// Stroke the outline with the given color.
    pub fn stroke<C>(self, color: C) -> Self
    where
        C: IntoLinSrgba<ColorScalar>,
    {
        self.map_ty(|ty| ty.stroke(color))
    }

    /// Specify the width and height of the **Ellipse** via a given **radius**.
    pub fn radius(self, radius: f32) -> Self {
        self.map_ty(|ty| ty.radius(radius))
    }

    /// The number of sides used to draw the ellipse.
    pub fn resolution(self, resolution: f32) -> Self {
        self.map_ty(|ty| ty.resolution(resolution))
    }
}
