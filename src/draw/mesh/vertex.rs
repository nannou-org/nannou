use color;
use geom;
use math::{BaseFloat, Point2, Point3, Vector3};
use mesh::vertex::{WithColor, WithTexCoords};
use std::marker::PhantomData;

pub type Point<S> = Point3<S>;
pub type Color = color::Rgba;
pub type TexCoords<S> = Point2<S>;
pub type Normal<S> = Vector3<S>;

/// Types that can be converted into a `draw::mesh::vertex::Point`.
pub trait IntoPoint<S> {
    /// Convert self into a `Point`.
    fn into_point(self) -> Point<S>;
}

impl<S> IntoPoint<S> for Point<S> {
    fn into_point(self) -> Self {
        self
    }
}

impl<S> IntoPoint<S> for Point2<S>
where
    S: BaseFloat,
{
    fn into_point(self) -> Point<S> {
        let Point2 { x, y } = self;
        let z = S::zero();
        Point3 { x, y, z }
    }
}

impl<S> IntoPoint<S> for (S, S)
where
    S: BaseFloat,
{
    fn into_point(self) -> Point<S> {
        let (x, y) = self;
        let z = S::zero();
        Point3 { x, y, z }
    }
}

impl<S> IntoPoint<S> for (S, S, S) {
    fn into_point(self) -> Point<S> {
        let (x, y, z) = self;
        Point3 { x, y, z }
    }
}

impl<S> IntoPoint<S> for [S; 2]
where
    S: BaseFloat,
{
    fn into_point(self) -> Point<S> {
        let x = self[0];
        let y = self[1];
        let z = S::zero();
        Point3 { x, y, z }
    }
}

impl<S> IntoPoint<S> for [S; 3]
where
    S: Copy,
{
    fn into_point(self) -> Point<S> {
        let x = self[0];
        let y = self[1];
        let z = self[2];
        Point3 { x, y, z }
    }
}

/// The vertex type produced by the **draw::Mesh**'s inner **MeshType**.
pub type Vertex<S> = WithTexCoords<WithColor<Point<S>, Color>, TexCoords<S>>;

/// Simplified constructor for a **draw::mesh::Vertex**.
pub fn new<S>(point: Point<S>, color: Color, tex_coords: TexCoords<S>) -> Vertex<S> {
    WithTexCoords {
        tex_coords,
        vertex: WithColor {
            color,
            vertex: point,
        },
    }
}

/// Default texture coordinates, for the case where a type is not textured.
pub fn default_tex_coords<S>() -> TexCoords<S>
where
    S: BaseFloat,
{
    Point2 {
        x: S::zero(),
        y: S::zero(),
    }
}

impl<S> Vertex<S> {
    /// Borrow the inner **Point**.
    pub fn point(&self) -> &Point<S> {
        &self.vertex.vertex
    }

    /// Mutably borrow the inner **Point**.
    pub fn point_mut(&mut self) -> &mut Point<S> {
        &mut self.vertex.vertex
    }
}

/// A type that converts an iterator yielding colored points to an iterator yielding **Vertex**s.
///
/// Default values are used for tex_coords.
#[derive(Clone, Debug)]
pub struct IterFromColoredPoints<I, S = geom::DefaultScalar> {
    colored_points: I,
    _scalar: PhantomData<S>,
}

impl<I, S> IterFromColoredPoints<I, S> {
    /// Produce an iterator that converts an iterator yielding colored points to an iterator
    /// yielding **Vertex**s.
    ///
    /// The default value of `(0.0, 0.0)` is used for tex_coords.
    pub fn new<P>(colored_points: P) -> Self
    where
        P: IntoIterator<IntoIter = I, Item = WithColor<Point<S>, Color>>,
        I: Iterator<Item = WithColor<Point<S>, Color>>,
    {
        let colored_points = colored_points.into_iter();
        let _scalar = PhantomData;
        IterFromColoredPoints {
            colored_points,
            _scalar,
        }
    }
}

impl<I, S> Iterator for IterFromColoredPoints<I, S>
where
    I: Iterator<Item = WithColor<Point<S>, Color>>,
    S: BaseFloat,
{
    type Item = Vertex<S>;
    fn next(&mut self) -> Option<Self::Item> {
        self.colored_points.next().map(|vertex| {
            let tex_coords = default_tex_coords();
            let vertex = WithTexCoords { tex_coords, vertex };
            vertex
        })
    }
}

/// A type that converts an iterator yielding points to an iterator yielding **Vertex**s.
///
/// The given `default_color` is used to color every vertex.
///
/// The default value of `(0.0, 0.0)` is used for tex_coords.
#[derive(Clone, Debug)]
pub struct IterFromPoints<I, S = geom::DefaultScalar> {
    points: I,
    default_color: Color,
    _scalar: PhantomData<S>,
}

impl<I, S> IterFromPoints<I, S> {
    /// Produce an iterator that converts an iterator yielding points to an iterator yielding
    /// **Vertex**s.
    ///
    /// The given `default_color` is used to color every vertex.
    ///
    /// The default value of `(0.0, 0.0)` is used for tex_coords.
    pub fn new<P>(points: P, default_color: Color) -> Self
    where
        P: IntoIterator<IntoIter = I, Item = Point<S>>,
        I: Iterator<Item = Point3<S>>,
    {
        let points = points.into_iter();
        let _scalar = PhantomData;
        IterFromPoints {
            points,
            default_color,
            _scalar,
        }
    }
}

impl<I, S> Iterator for IterFromPoints<I, S>
where
    I: Iterator<Item = Point<S>>,
    S: BaseFloat,
{
    type Item = Vertex<S>;
    fn next(&mut self) -> Option<Self::Item> {
        self.points.next().map(|vertex| {
            let color = self.default_color;
            let vertex = WithColor { vertex, color };
            let tex_coords = default_tex_coords();
            let vertex = WithTexCoords { vertex, tex_coords };
            vertex
        })
    }
}

/// A type that converts an iterator yielding 2D points to an iterator yielding **Vertex**s.
///
/// The `z` position for each vertex will be `0.0`.
///
/// The given `default_color` is used to color every vertex.
///
/// The default value of `(0.0, 0.0)` is used for tex_coords.
#[derive(Clone, Debug)]
pub struct IterFromPoint2s<I, S = geom::DefaultScalar> {
    points: I,
    default_color: Color,
    _scalar: PhantomData<S>,
}

impl<I, S> IterFromPoint2s<I, S> {
    /// A type that converts an iterator yielding 2D points to an iterator yielding **Vertex**s.
    ///
    /// The `z` position for each vertex will be `0.0`.
    ///
    /// The given `default_color` is used to color every vertex.
    ///
    /// The default value of `(0.0, 0.0)` is used for tex_coords.
    pub fn new<P>(points: P, default_color: Color) -> Self
    where
        P: IntoIterator<IntoIter = I, Item = Point2<S>>,
        I: Iterator<Item = Point2<S>>,
    {
        let points = points.into_iter();
        let _scalar = PhantomData;
        IterFromPoint2s {
            points,
            default_color,
            _scalar,
        }
    }
}

impl<I, S> Iterator for IterFromPoint2s<I, S>
where
    I: Iterator<Item = Point2<S>>,
    S: BaseFloat,
{
    type Item = Vertex<S>;
    fn next(&mut self) -> Option<Self::Item> {
        self.points.next().map(|Point2 { x, y }| {
            let vertex = Point3 { x, y, z: S::zero() };
            let color = self.default_color;
            let vertex = WithColor { vertex, color };
            let tex_coords = default_tex_coords();
            let vertex = WithTexCoords { vertex, tex_coords };
            vertex
        })
    }
}
