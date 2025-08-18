use crate::color::conv::IntoLinSrgba;
use crate::draw::primitive::polygon::{self, PolygonInit, PolygonOptions, SetPolygon};
use crate::draw::primitive::Primitive;
use crate::draw::properties::spatial::{dimension, orientation, position};
use crate::draw::properties::{
    ColorScalar, LinSrgba, SetColor, SetDimensions, SetOrientation, SetPosition, SetStroke,
};
use crate::draw::{self, Drawing};
use crate::geom;
use crate::glam::Vec2;
use lyon::math::Point;
use lyon::tessellation::StrokeOptions;
use nannou_core::geom::point;
use nannou_core::prelude::{Vec2Rotate, abs};
use lyon::path::builder::SvgPathBuilder;

/// Properties related to drawing a **Rect**.
#[derive(Clone, Debug)]
pub struct Rect {
    dimensions: dimension::Properties,
    polygon: PolygonInit,
    corner_radius: Option<f32>,
}

/// The drawing context for a Rect.
pub type DrawingRect<'a> = Drawing<'a, Rect>;

// Trait implementations.

impl Rect {
    /// Stroke the outline with the given color.
    pub fn stroke<C>(self, color: C) -> Self
    where
        C: IntoLinSrgba<ColorScalar>,
    {
        self.stroke_color(color)
    }

    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = Some(abs(radius));
        self
    }

}


impl<'a> DrawingRect<'a> {
    /// Stroke the outline with the given color.
    pub fn stroke<C>(self, color: C) -> Self
    where
        C: IntoLinSrgba<ColorScalar>,
    {
        self.map_ty(|ty| ty.stroke(color))
    }

    
    pub fn corner_radius(self, radius: f32) -> Self {
        self.map_ty(|ty| ty.corner_radius(radius))
    }

}

impl draw::renderer::RenderPrimitive for Rect {
    fn render_primitive(
        self,
        ctxt: draw::renderer::RenderContext,
        mesh: &mut draw::Mesh,
    ) -> draw::renderer::PrimitiveRender {
        let Rect {
            polygon,
            dimensions,
            corner_radius,
        } = self;



        // If dimensions were specified, scale the points to those dimensions.
        let (maybe_x, maybe_y, maybe_z) = (dimensions.x, dimensions.y, dimensions.z);
        assert!(
            maybe_z.is_none(),
            "z dimension support for rect is unimplemented"
        );
        
        let w = maybe_x.unwrap_or(100.0);
        let h = maybe_y.unwrap_or(100.0);

        let rect = geom::Rect::from_wh([w, h].into());

        match corner_radius {
            Some(radius) => {
                let mut builder = lyon::path::Path::svg_builder();

                // unit vector parallel to segment, used for positioning arc points
                let mut segment_vector = Vec2::from([0.0, 1.0]); 
                let starting_point = rect.bottom_left() + segment_vector * radius;
                // start at lower left corner.
                builder.move_to (Point::new(starting_point.x, starting_point.y));
                let mut rotation = lyon::geom::Angle::radians(0.0);

                for vertex in rect.corners().vertices().map(Vec2::from) {
                    let arc_start = vertex - segment_vector * radius;
                    builder.line_to(Point::new(arc_start.x, arc_start.y));
                    // rotate segment vector by 90 degrees to the right
                    segment_vector = segment_vector.rotate(-std::f32::consts::PI/2.0);
                    let arc_end = vertex + segment_vector * radius;
                    builder.arc_to(
                        lyon::math::Vector::from([radius, radius]), 
                        rotation,
                        lyon::geom::ArcFlags { large_arc: false, sweep: false },
                        Point::new(arc_end.x, arc_end.y)
                    );
                    rotation += lyon::geom::Angle::frac_pi_2();

                }
                let path = builder.build();
                polygon::render_events_themed(
                    polygon.opts,
                    || (&path).into_iter(),
                    ctxt,
                    &draw::theme::Primitive::Ellipse,
                    mesh,
                );
            }
            None => {
                let points = rect.corners().vertices().map(Vec2::from);
                polygon::render_points_themed(
                    polygon.opts,
                    points.into_iter(),
                    ctxt,
                    &draw::theme::Primitive::Rect,
                    mesh,
                );
            }
        }
        draw::renderer::PrimitiveRender::default()
    }
}

impl From<geom::Rect<f32>> for Rect {
    fn from(r: geom::Rect<f32>) -> Self {
        let (x, y, w, h) = r.x_y_w_h();
        Self::default().x_y(x, y).w_h(w, h)
    }
}

impl Default for Rect {
    fn default() -> Self {
        let dimensions = <_>::default();
        let polygon = <_>::default();
        Rect {
            dimensions,
            polygon,
            corner_radius: None,
        }
    }
}

impl SetOrientation for Rect {
    fn properties(&mut self) -> &mut orientation::Properties {
        SetOrientation::properties(&mut self.polygon)
    }
}

impl SetPosition for Rect {
    fn properties(&mut self) -> &mut position::Properties {
        SetPosition::properties(&mut self.polygon)
    }
}

impl SetDimensions for Rect {
    fn properties(&mut self) -> &mut dimension::Properties {
        SetDimensions::properties(&mut self.dimensions)
    }
}

impl SetColor<ColorScalar> for Rect {
    fn rgba_mut(&mut self) -> &mut Option<LinSrgba> {
        SetColor::rgba_mut(&mut self.polygon)
    }
}

impl SetStroke for Rect {
    fn stroke_options_mut(&mut self) -> &mut StrokeOptions {
        SetStroke::stroke_options_mut(&mut self.polygon)
    }
}

impl SetPolygon for Rect {
    fn polygon_options_mut(&mut self) -> &mut PolygonOptions {
        SetPolygon::polygon_options_mut(&mut self.polygon)
    }
}


// Primitive conversions.

impl From<Rect> for Primitive {
    fn from(prim: Rect) -> Self {
        Primitive::Rect(prim)
    }
}

impl Into<Option<Rect>> for Primitive {
    fn into(self) -> Option<Rect> {
        match self {
            Primitive::Rect(prim) => Some(prim),
            _ => None,
        }
    }
}
