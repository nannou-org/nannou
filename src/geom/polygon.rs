use geom::tri::{self, Tri};

/// An iterator that triangulates a polygon represented by a sequence of points describing its
/// edges.
#[derive(Clone)]
pub struct Triangles<I>
where
    I: Iterator,
{
    first: I::Item,
    prev: I::Item,
    points: I,
}

/// Triangulate the polygon given as a list of `Point`s describing its sides.
///
/// Returns `None` if the given iterator yields less than two points.
pub fn triangles<I>(points: I) -> Option<Triangles<I::IntoIter>>
where
    I: IntoIterator,
    I::Item: tri::Vertex2d,
{
    let mut points = points.into_iter();
    let first = match points.next() {
        Some(p) => p,
        None => return None,
    };
    let prev = match points.next() {
        Some(p) => p,
        None => return None,
    };
    Some(Triangles {
        first: first,
        prev: prev,
        points: points,
    })
}

impl<I> Iterator for Triangles<I>
where
    I: Iterator,
    I::Item: tri::Vertex2d,
{
    type Item = Tri<I::Item>;
    fn next(&mut self) -> Option<Self::Item> {
        self.points.next().map(|point| {
            let t = Tri([self.first, self.prev, point]);
            self.prev = point;
            t
        })
    }
}

/// Returns `true` if the given `Point` is over the polygon described by the given series of
/// points.
pub fn contains<I>(points: I, point: I::Item) -> Option<Tri<I::Item>>
where
    I: IntoIterator,
    I::Item: tri::Vertex2d,
{
    triangles(points).and_then(|ts| tri::iter_contains(ts, point))
}
