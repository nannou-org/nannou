//! Items related to the geometry that joins two lines together.
//!
//! Line joins describe the style of geometry used to fill the space at which two lines would
//! intersect. The following diagram shows the space occuppied by a line join between A and B.
//!
//!
//! ```ignore
//!                                  Line join fills this space.
//!                                  This one is a "miter" join.
//!
//!                                             /
//!                                            /
//!                          Line A           /
//!
//!           ^   ------------------------xxxx        ^
//!           |                           xxxxx       | half_thickness
//! thickness |   ------------------------xxxxxx      v
//!           |                           xxxx  \
//!           v   ------------------------x  \   \
//!                                       \   \   \
//!                                        \   \   \   Line B
//!                                         \   \   \
//!                                          \   \   \
//!                                           \   \   \
//! ```
//!
//! Nannou provides three common types of line joins:
//!
//! - [**miter**](./miter/index.html): Extends the stroke to where the edges on each side bisect.
//!   This is the default join type.
//! - [**round**](./round/index.html): Rounds the outside edge with a circle the diameter of the
//!   thickness.
//! - [**bevel**](./bevel/index.html): Cuts the outside edge off where a circle the diameter of the
//!   thickness intersects

use geom::Line;
use math::{BaseFloat, Point2};

pub mod miter;
pub mod round;
pub mod bevel;

/// Given three points that describe two lines and half the thickness of the line, return the two
/// points that intersect on either side of the line.
///
/// The following diagram describes the expected arguments as well as the left `L` and right `R`
/// fields of the tuple result.
///
/// ```ignore
///    -------------------------------    L       ^
///                                               | half_thickness
///  a -------------------------------  b         v
///                                         \
///    ------------------------------R   \   \
///                                   \   \   \
///            Line 1                  \   \   \
///                                     \   \   \
///                            Line 2    \   \   \
///                                       \   \   \
///
///                                            c
/// ```
///
/// ## Example
///
/// ```
/// extern crate nannou;
///
/// use nannou::prelude::*;
/// use nannou::geom::line;
///
/// fn main() {
///     // Find `L` and `R`.
///     //
///     //         |  c  |  ^
///     //         |  |  |  |
///     //         |  |  |  | 2.0
///     // --------L  |  |  |
///     //            |  |  |
///     // a----------b  |  v
///     //               |
///     // --------------R
///     //
///     // <---------->
///     //     2.0
///
///     let a = pt2(0.0, 0.0);
///     let b = pt2(2.0, 0.0);
///     let c = pt2(2.0, 2.0);
///     let half_thickness = 1.0;
///     let result = line::join::intersections(a, b, c, half_thickness);
///     assert_eq!(Some((pt2(1.0, 1.0), pt2(3.0, -1.0))), result);
/// }
/// ```
pub fn intersections<S>(
    a: Point2<S>,
    b: Point2<S>,
    c: Point2<S>,
    half_thickness: S,
) -> Option<(Point2<S>, Point2<S>)>
where
    S: BaseFloat,
{
    let ab = Line { start: a, end: b, half_thickness };
    let bc = Line { start: b, end: c, half_thickness };
    let ab_corners = ab.quad_corners();
    let bc_corners = bc.quad_corners();
    let (ar, al, bl_ab, br_ab) = ab_corners.into();
    let (br_bc, bl_bc, cl, cr) = bc_corners.into();
    let il = match intersect((al, bl_ab), (cl, bl_bc)) {
        Some(il) => il,
        None => return None,
    };
    let ir = intersect((ar, br_ab), (cr, br_bc))
        .expect("no intersection due to parallel lines");
    Some((il, ir))
}

/// The point of intersection between two straight lines.
///
/// Returns `None` if the two lines are parallel.
///
/// ```ignore
///
///                 b
///
///                   \
///                    \
///                     \
/// a -------------------X------------------ a
///                       \
///                        \
///                         \
///                          \
///                           \
///                            \
///                             \
///                              \
///
///                                b
///
/// ```
///
/// ## Example
///
/// ```
/// extern crate nannou;
///
/// use nannou::prelude::*;
/// use nannou::geom::line;
///
/// fn main() {
///     let a = (pt2(4.0, 0.0), pt2(6.0, 10.0));
///     let b = (pt2(0.0, 3.0), pt2(10.0, 7.0));
///     assert_eq!(Some(pt2(5.0, 5.0)), line::join::intersect(a, b));
///
///     let a = (pt2(0.0, 1.0), pt2(3.0, 1.0));
///     let b = (pt2(2.0, 2.0), pt2(4.0, 2.0));
///     assert_eq!(None, line::join::intersect(a, b));
///
///     let a = (pt2(0.0, 0.0), pt2(2.0, 2.0));
///     let b = (pt2(0.0, 10.0), pt2(10.0, 0.0));
///     assert_eq!(Some(pt2(5.0, 5.0)), line::join::intersect(a, b));
///     assert_ne!(Some(pt2(4.9, 5.1)), line::join::intersect(a, b));
/// }
/// ```
pub fn intersect<S>(a: (Point2<S>, Point2<S>), b: (Point2<S>, Point2<S>)) -> Option<Point2<S>>
where
    S: BaseFloat,
{
    let (a1, a2) = a;
    let (b1, b2) = b;
    let determinant = |a, b, c, d| a * d - b * c;
    let det_a = determinant(a1.x, a1.y, a2.x, a2.y);
    let det_b = determinant(b1.x, b1.y, b2.x, b2.y);
    let axd = a1.x - a2.x;
    let bxd = b1.x - b2.x;
    let ayd = a1.y - a2.y;
    let byd = b1.y - b2.y;
    let x_nom = determinant(det_a, axd, det_b, bxd);
    let y_nom = determinant(det_a, ayd, det_b, byd);
    let denom = determinant(axd, ayd, bxd, byd);
    if denom == S::zero() {
        return None;
    }
    let x = x_nom / denom;
    let y = y_nom / denom;
    let intersection = Point2 { x, y };
    Some(intersection)
}
