use geom::{ellipse, Ellipse, Rect};
use math::{vec2, BaseFloat, EuclideanSpace, InnerSpace, Point2};
use std::f64::consts::PI;

/// A line cap with a rounded edge around a line's start or end.
#[derive(Clone, Debug)]
pub struct Round;

/// Produces the **ellipse::Section** describing this line cap.
pub fn ellipse_section<S>(
    line_corner_a: Point2<S>,
    line_corner_b: Point2<S>,
    resolution: usize,
) -> ellipse::Section<S>
where
    S: BaseFloat,
{
    let section_radians = S::from(PI).expect("could not cast from f64");
    let av = vec2(line_corner_a.x, line_corner_a.y);
    let bv = vec2(line_corner_b.x, line_corner_b.y);
    let direction = av - bv;
    let offset_radians = direction.y.atan2(direction.x);
    let side = direction.magnitude();
    let p = EuclideanSpace::midpoint(line_corner_a, line_corner_b);
    let rect = Rect::from_x_y_w_h(p.x, p.y, side, side);
    let ellipse = Ellipse { rect, resolution };
    let section = ellipse::Section { ellipse, section_radians, offset_radians };
    section
}
