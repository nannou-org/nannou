use geom::{vec2, Point2, Quad};
use math::{BaseFloat, InnerSpace};

/// A line cap with a square edge that extends past the end of the start or end by half the line's
/// thickness.
#[derive(Clone, Debug)]
pub struct Square;

/// Produces the **Quad** describing this line cap.
///
/// `protrusion` is the distance past the line end over which the **Quad** should protrude. This is
/// normally equal to half of the line's thickness.
pub fn quad<S>(line_corner_a: Point2<S>, line_corner_b: Point2<S>, protrusion: S) -> Quad<Point2<S>>
where
    S: BaseFloat,
{
    let direction = line_corner_b - line_corner_a;
    let unit = direction.normalize();
    let normal = vec2(-unit.y, unit.x);
    let n = normal.normalize_to(protrusion);
    let c = line_corner_b + n;
    let d = line_corner_a + n;
    Quad([line_corner_a, line_corner_b, c, d])
}
