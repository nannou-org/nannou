//! Items related to working with paths for 2D geometry and vector graphics.
//!
//! This module attempts to provide abstractions around the various `Path` and `Builder` types
//! offerred by `lyon` in a way that interoperates a little more fluidly and consistently with the
//! rest of nannou's API.

use crate::geom::Point2;
use crate::glam::Vec2;

/// A wrapper around a 2D lyon path exposing a nannou-friendly API.
pub struct Path {
    path: lyon::path::Path,
}

/// A type used for building a 2D lyon path.
pub struct Builder {
    builder: lyon::path::Builder,
}

impl Path {
    /// Begin building a new path.
    pub fn builder() -> Builder {
        Builder::new()
    }

    /// Create an empty path.
    pub fn new() -> Self {
        lyon::path::Path::new().into()
    }

    /// Returns a lyon view on this **Path**.
    pub fn as_slice(&self) -> lyon::path::PathSlice {
        self.path.as_slice()
    }

    /// Returns a slice over an endpoint's custom attributes.
    pub fn attributes(&self, endpoint: lyon::path::EndpointId) -> &[f32] {
        self.path.attributes(endpoint)
    }

    /// Iterates over the entire **Path** yielding **PathEvent**s.
    pub fn iter(&self) -> lyon::path::Iter {
        self.path.iter()
    }

    /// Iterates over the endpoint and control point ids of the **Path**.
    pub fn id_iter(&self) -> lyon::path::IdIter {
        self.path.id_iter()
    }

    /// Iterate over points alongside their attributes.
    pub fn iter_with_attributes(&self) -> lyon::path::IterWithAttributes {
        self.path.iter_with_attributes()
    }

    /// Applies a transform to all endpoints and control points of this path and returns the
    /// result.
    pub fn transformed<T>(&self, transform: &T) -> Self
    where
        T: lyon::geom::traits::Transformation<f32>,
    {
        self.path.transformed(transform).into()
    }

    /// Reversed version of this path with edge loops specified in the opposite order.
    pub fn reversed(&self) -> Self {
        self.path.reversed().into()
    }

    /// Concatenate two paths.
    pub fn merge(&self, other: &Self) -> Self {
        self.path.merge(&other.path).into()
    }
}

impl Builder {
    /// Begin building a new path.
    pub fn new() -> Self {
        lyon::path::Builder::new().into()
    }

    /// Build a path with the given capacity for the inner path event storage.
    pub fn with_capacity(points: usize, edges: usize) -> Self {
        lyon::path::Builder::with_capacity(points, edges).into()
    }

    /// Returns a lyon builder that supports SVG commands.
    pub fn with_svg(self) -> lyon::path::builder::SvgPathBuilder<Self> {
        lyon::path::builder::SvgPathBuilder::new(self)
    }

    /// Returns a lyon builder that approximates all curves with sequences of line segments.
    pub fn flattened(self, tolerance: f32) -> lyon::path::builder::FlatteningBuilder<Self> {
        lyon::path::builder::FlatteningBuilder::new(self, tolerance)
    }

    /// Sets the position in preparation for the next sub-path.
    ///
    /// If the current sub-path contains edges, this ends the sub-path without closing it.
    pub fn move_to(mut self, to: Point2) -> Self {
        self.builder.move_to(to.to_array().into());
        self
    }

    /// Adds a line segment to the current sub-path and sets the current position.
    pub fn line_to(mut self, to: Point2) -> Self {
        self.builder.line_to(to.to_array().into());
        self
    }

    /// Closes the current sub path and sets the current position to the first position of the
    /// current sub-path.
    pub fn close(mut self) -> Self {
        self.builder.close();
        self
    }

    /// Add a quadratic bezier curve to the path.
    pub fn quadratic_bezier_to(mut self, ctrl: Point2, to: Point2) -> Self {
        self.builder
            .quadratic_bezier_to(ctrl.to_array().into(), to.to_array().into());
        self
    }

    /// Add a cubic bezier curve to the path.
    pub fn cubic_bezier_to(mut self, ctrl1: Point2, ctrl2: Point2, to: Point2) -> Self {
        self.builder.cubic_bezier_to(
            ctrl1.to_array().into(),
            ctrl2.to_array().into(),
            to.to_array().into(),
        );
        self
    }

    /// Add an arc to the path.
    pub fn arc(
        mut self,
        center: Point2,
        radii: Vec2,
        sweep_angle_radians: f32,
        x_rotation_radians: f32,
    ) -> Self {
        self.builder.arc(
            center.to_array().into(),
            radii.to_array().into(),
            lyon::math::Angle::radians(sweep_angle_radians),
            lyon::math::Angle::radians(x_rotation_radians),
        );
        self
    }

    /// Add a closed polygon.
    pub fn polygon(mut self, points: &[Point2]) -> Self {
        self.builder.polygon(point_slice_nannou_to_lyon(points));
        self
    }

    /// Returns the current position of the head of the path.
    pub fn position(&self) -> Point2 {
        let p = self.builder.current_position();
        [p.x, p.y].into()
    }

    /// Build the path and return it.
    pub fn build(self) -> Path {
        self.builder.build().into()
    }

    /// Access to the inner `lyon::path::Builder`.
    pub fn inner(&self) -> &lyon::path::Builder {
        &self.builder
    }

    /// Mutable access to the inner `lyon::path::Builder`.
    pub fn inner_mut(&mut self) -> &mut lyon::path::Builder {
        &mut self.builder
    }
}

// lyon builder traits

impl lyon::path::builder::Build for Builder {
    type PathType = Path;

    fn build(self) -> Self::PathType {
        self.builder.build().into()
    }

    fn build_and_reset(&mut self) -> Self::PathType {
        self.builder.build_and_reset().into()
    }
}

impl lyon::path::builder::FlatPathBuilder for Builder {
    fn move_to(&mut self, to: lyon::math::Point) {
        self.builder.move_to(to);
    }

    fn line_to(&mut self, to: lyon::math::Point) {
        self.builder.line_to(to);
    }

    fn close(&mut self) {
        self.builder.close();
    }

    fn current_position(&self) -> lyon::math::Point {
        self.builder.current_position()
    }
}

impl lyon::path::builder::PathBuilder for Builder {
    fn quadratic_bezier_to(&mut self, ctrl: lyon::math::Point, to: lyon::math::Point) {
        self.builder.quadratic_bezier_to(ctrl, to);
    }

    fn cubic_bezier_to(
        &mut self,
        ctrl1: lyon::math::Point,
        ctrl2: lyon::math::Point,
        to: lyon::math::Point,
    ) {
        self.builder.cubic_bezier_to(ctrl1, ctrl2, to);
    }

    fn arc(
        &mut self,
        center: lyon::math::Point,
        radii: lyon::math::Vector,
        sweep_angle: lyon::math::Angle,
        x_rotation: lyon::math::Angle,
    ) {
        self.builder.arc(center, radii, sweep_angle, x_rotation);
    }

    fn path_event(&mut self, event: lyon::path::PathEvent) {
        self.builder.path_event(event);
    }
}

impl lyon::path::builder::PolygonBuilder for Builder {
    fn polygon(&mut self, points: &[lyon::math::Point]) {
        self.builder.polygon(points);
    }
}

// Indexing

impl std::ops::Index<lyon::path::ControlPointId> for Path {
    type Output = Point2;
    fn index(&self, id: lyon::path::ControlPointId) -> &Self::Output {
        point_lyon_to_nannou(self.path.index(id))
    }
}

impl std::ops::Index<lyon::path::EndpointId> for Path {
    type Output = Point2;
    fn index(&self, id: lyon::path::EndpointId) -> &Self::Output {
        point_lyon_to_nannou(self.path.index(id))
    }
}

// Path iteration

impl<'a> IntoIterator for &'a Path {
    type Item = lyon::path::PathEvent;
    type IntoIter = lyon::path::Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

// Conversions

impl From<lyon::path::Path> for Path {
    fn from(path: lyon::path::Path) -> Self {
        Path { path }
    }
}

impl From<lyon::path::Builder> for Builder {
    fn from(builder: lyon::path::Builder) -> Self {
        Builder { builder }
    }
}

impl Into<lyon::path::Path> for Path {
    fn into(self) -> lyon::path::Path {
        self.path
    }
}

impl Into<lyon::path::Builder> for Builder {
    fn into(self) -> lyon::path::Builder {
        self.builder
    }
}

impl<'a> Into<lyon::path::PathSlice<'a>> for &'a Path {
    fn into(self) -> lyon::path::PathSlice<'a> {
        self.as_slice()
    }
}

// Simplified constructors

/// Begin building a path.
pub fn path() -> Builder {
    Builder::new()
}

/// Build a path with the given capacity for the inner path event storage.
pub fn path_with_capacity(points: usize, edges: usize) -> Builder {
    Builder::with_capacity(points, edges)
}

// Conversions between slice types.
//
// The following conversions are safe as both `Point2` and `lyon::path::Point` have the same size,
// fields and `repr(C)` layout.

fn point_slice_nannou_to_lyon(ps: &[Point2]) -> &[lyon::math::Point] {
    unsafe { std::mem::transmute(ps) }
}

fn point_lyon_to_nannou(p: &lyon::math::Point) -> &Point2 {
    unsafe { std::mem::transmute(p) }
}
