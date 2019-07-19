//! Vertex types yielded by the mesh adaptors and their implementations.

use crate::draw::properties::IntoLinSrgba;
use crate::geom::graph::node::{self, ApplyTransform};
use crate::geom::{self, Point2, Point3};
use crate::math::BaseFloat;
use std::ops::{Deref, DerefMut};

/// A vertex with a specified color.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct WithColor<V, C> {
    pub vertex: V,
    pub color: C,
}

/// A vertex with some specified texture coordinates.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct WithTexCoords<V, T = Point2<geom::scalar::Default>> {
    pub vertex: V,
    pub tex_coords: T,
}

/// A vertex with its normal vector.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct WithNormal<V, N = geom::vertex::Default> {
    pub vertex: V,
    pub normal: N,
}

// Node Transform application implementations.

impl<S, V, C> ApplyTransform<S> for WithColor<V, C>
where
    V: ApplyTransform<S>,
    S: BaseFloat,
{
    fn apply_transform(self, transform: &node::PreparedTransform<S>) -> Self {
        let WithColor { mut vertex, color } = self;
        vertex = vertex.apply_transform(transform);
        WithColor { vertex, color }
    }
}

impl<S, V, T> ApplyTransform<S> for WithTexCoords<V, T>
where
    V: ApplyTransform<S>,
    S: BaseFloat,
{
    fn apply_transform(self, transform: &node::PreparedTransform<S>) -> Self {
        let WithTexCoords {
            mut vertex,
            tex_coords,
        } = self;
        vertex = vertex.apply_transform(transform);
        WithTexCoords { vertex, tex_coords }
    }
}

impl<S, V, N> ApplyTransform<S> for WithNormal<V, N>
where
    V: ApplyTransform<S>,
    S: BaseFloat,
{
    fn apply_transform(self, _transform: &node::PreparedTransform<S>) -> Self {
        //let WithNormal { mut vertex, mut normal } = self;
        //vertex = vertex.apply_transform(transform);
        // TODO: Apply transform to the `normal`.
        unimplemented!();
        //WithNormal { vertex, normal }
    }
}

// Deref implementations for each vertex adaptor to their inner vertex type.

impl<V, C> Deref for WithColor<V, C> {
    type Target = V;
    fn deref(&self) -> &Self::Target {
        &self.vertex
    }
}

impl<V, C> DerefMut for WithColor<V, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vertex
    }
}

impl<V, T> Deref for WithTexCoords<V, T> {
    type Target = V;
    fn deref(&self) -> &Self::Target {
        &self.vertex
    }
}

impl<V, T> DerefMut for WithTexCoords<V, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vertex
    }
}

impl<V, N> Deref for WithNormal<V, N> {
    type Target = V;
    fn deref(&self) -> &Self::Target {
        &self.vertex
    }
}

impl<V, N> DerefMut for WithNormal<V, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vertex
    }
}

// Geometry vertex implementations.

impl<V, C> geom::Vertex for WithColor<V, C>
where
    V: geom::Vertex,
    C: Clone + Copy + PartialEq,
{
    type Scalar = V::Scalar;
}

impl<V, T> geom::Vertex for WithTexCoords<V, T>
where
    V: geom::Vertex,
    T: Clone + Copy + PartialEq,
{
    type Scalar = V::Scalar;
}

impl<V, N> geom::Vertex for WithNormal<V, N>
where
    V: geom::Vertex,
    N: Clone + Copy + PartialEq,
{
    type Scalar = V::Scalar;
}

impl<V, C> geom::Vertex2d for WithColor<V, C>
where
    V: geom::Vertex2d,
    Self: geom::Vertex<Scalar = V::Scalar>,
{
    fn point2(self) -> Point2<Self::Scalar> {
        self.vertex.point2()
    }
}

impl<V, T> geom::Vertex2d for WithTexCoords<V, T>
where
    V: geom::Vertex2d,
    Self: geom::Vertex<Scalar = V::Scalar>,
{
    fn point2(self) -> Point2<Self::Scalar> {
        self.vertex.point2()
    }
}

impl<V, N> geom::Vertex2d for WithNormal<V, N>
where
    V: geom::Vertex2d,
    Self: geom::Vertex<Scalar = V::Scalar>,
{
    fn point2(self) -> Point2<Self::Scalar> {
        self.vertex.point2()
    }
}

impl<V, C> geom::Vertex3d for WithColor<V, C>
where
    V: geom::Vertex3d,
    Self: geom::Vertex<Scalar = V::Scalar>,
{
    fn point3(self) -> Point3<Self::Scalar> {
        self.vertex.point3()
    }
}

impl<V, T> geom::Vertex3d for WithTexCoords<V, T>
where
    V: geom::Vertex3d,
    Self: geom::Vertex<Scalar = V::Scalar>,
{
    fn point3(self) -> Point3<Self::Scalar> {
        self.vertex.point3()
    }
}

impl<V, N> geom::Vertex3d for WithNormal<V, N>
where
    V: geom::Vertex3d,
    Self: geom::Vertex<Scalar = V::Scalar>,
{
    fn point3(self) -> Point3<Self::Scalar> {
        self.vertex.point3()
    }
}

// For converting from a tuples to vertices.

impl<A, V, B, C> From<(A, B)> for WithColor<V, C>
where
    A: Into<V>,
    B: IntoLinSrgba<f32>,
    C: From<crate::color::LinSrgba<f32>>,
{
    fn from((vertex, color): (A, B)) -> Self {
        let vertex = vertex.into();
        // TODO: Using `into_lin_srgba` solely because palette's conversion implementations (e.g.
        // `From` and `Into`) are not exhaustive. Using this gives more flexibility in terms of
        // supported color conversions, but these conversions should really be added upstream to
        // palette itself.
        let lin_srgba = color.into_lin_srgba();
        let color = lin_srgba.into();
        WithColor { vertex, color }
    }
}

impl<A, V, T> From<(A, T)> for WithTexCoords<V, T>
where
    A: Into<V>,
{
    fn from((vertex, tex_coords): (A, T)) -> Self {
        let vertex = vertex.into();
        WithTexCoords { vertex, tex_coords }
    }
}

impl<A, V, N> From<(A, N)> for WithNormal<V, N>
where
    A: Into<V>,
{
    fn from((vertex, normal): (A, N)) -> Self {
        let vertex = vertex.into();
        WithNormal { vertex, normal }
    }
}

#[test]
fn test_tuple_conv() {
    use crate::color::named::GREEN;
    let _: Point2<_> = [0.0, 0.0].into();
    let _: WithColor<Point2<_>, _> = ([0.0, 0.0], GREEN).into();
}
