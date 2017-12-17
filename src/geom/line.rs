use geom::{quad, tri, Tri};
use math::prelude::*;
use math::{two, vec2, BaseFloat, Point2};

/// Given two points and half the line thickness, return the four corners of the rectangle
/// describing the line.
pub fn quad_corners<S>(a: Point2<S>, b: Point2<S>, half_thickness: S) -> [Point2<S>; 4]
where
    S: BaseFloat,
{
    let direction = b - a;
    let unit = direction.normalize();
    let normal = vec2(-unit.y, unit.x);
    let n = normal.normalize_to(half_thickness);
    let r1 = [a.x - n.x, a.y - n.y].into();
    let r2 = [a.x + n.x, a.y + n.y].into();
    let r3 = [b.x - n.x, b.y - n.y].into();
    let r4 = [b.x + n.x, b.y + n.y].into();
    [r1, r2, r3, r4]
}

/// Given two points and half the line thickness, return the two triangles that describe the line.
pub fn triangles<S>(a: Point2<S>, b: Point2<S>, half_thickness: S) -> [Tri<Point2<S>>; 2]
where
    S: BaseFloat,
{
    let r = quad_corners(a, b, half_thickness);
    let (t1, t2) = quad::triangles(&r);
    [t1, t2]
}

/// Describes whether or not the given point touches the line described by *a -> b* with the given
/// thickness.
///
/// If so, the `Tri` containing the point will be returned.
///
/// `None` is returned otherwise.
pub fn contains<S>(
    a: Point2<S>,
    b: Point2<S>,
    thickness: S,
    point: &Point2<S>,
) -> Option<Tri<Point2<S>>>
where
    S: BaseFloat,
{
    let half_thickness = thickness / two::<S>();
    let tris = triangles(a, b, half_thickness);
    tri::iter_contains(tris.iter(), point).map(|&t| t)
}
