use crate::color::IntoLinSrgba;
use crate::draw::mesh::vertex::{Color, TexCoords};
use crate::draw::primitive::Primitive;
use crate::draw::properties::{
    ColorScalar, SetColor, SetDimensions, SetFill, SetOrientation, SetPosition, SetStroke,
};
use crate::draw::{self, Draw};
use crate::geom::{Point2, Point3};
use crate::glam::{Quat, Vec2, Vec3};
use lyon::path::PathEvent;
use lyon::tessellation::{FillOptions, LineCap, LineJoin, StrokeOptions};
use std::marker::PhantomData;

/// A **Drawing** in progress.
///
/// **Drawing** provides a way of chaining together method calls describing properties of the thing
/// that we are drawing. **Drawing** ends when the instance is **Drop**ped, at which point the
/// properties of the drawing are inserted into the **Draw** type.
///
/// When a **Drawing** begins, a node is immediately created for the drawing within the **Draw**'s
/// inner **geom::Graph**. This ensures the correct instantiation order is maintained within the
/// graph. As a result, each **Drawing** is associated with a single, unique node. Thus a
/// **Drawing** can be thought of as a way of specifying properties for a node.
#[derive(Debug)]
pub struct Drawing<'a, T> {
    // The `Draw` instance used to create this drawing.
    draw: &'a Draw,
    // The draw command index of the primitive being drawn.
    index: usize,
    // Whether or not the **Drawing** should attempt to finish the drawing on drop.
    finish_on_drop: bool,
    // The node type currently being drawn.
    _ty: PhantomData<T>,
}

/// Some context that may be optionally provided to primitives in the drawing implementation.
///
/// This is particularly useful for paths and meshes.
pub struct DrawingContext<'a> {
    /// The intermediary mesh for buffering yet-to-be-drawn paths and meshes.
    pub mesh: &'a mut draw::Mesh,
    /// A re-usable buffer for collecting path events.
    pub path_event_buffer: &'a mut Vec<PathEvent>,
    /// A re-usable buffer for collecting colored polyline points.
    pub path_points_colored_buffer: &'a mut Vec<(Point2, Color)>,
    /// A re-usable buffer for collecting textured polyline points.
    pub path_points_textured_buffer: &'a mut Vec<(Point2, TexCoords)>,
    /// A re-usable buffer for collecting text.
    pub text_buffer: &'a mut String,
}

/// Construct a new **Drawing** instance.
pub fn new<'a, T>(draw: &'a Draw, index: usize) -> Drawing<'a, T> {
    let _ty = PhantomData;
    let finish_on_drop = true;
    Drawing {
        draw,
        index,
        finish_on_drop,
        _ty,
    }
}

impl<'a, T> Drop for Drawing<'a, T> {
    fn drop(&mut self) {
        if self.finish_on_drop {
            self.finish_inner();
        }
    }
}

impl<'a> DrawingContext<'a> {
    // Initialise the DrawingContext from the draw's IntermediaryState.
    pub(crate) fn from_intermediary_state(state: &'a mut super::IntermediaryState) -> Self {
        let super::IntermediaryState {
            intermediary_mesh,
            path_event_buffer,
            path_points_colored_buffer,
            path_points_textured_buffer,
            text_buffer,
        } = state;
        DrawingContext {
            mesh: intermediary_mesh,
            path_event_buffer: path_event_buffer,
            path_points_colored_buffer: path_points_colored_buffer,
            path_points_textured_buffer: path_points_textured_buffer,
            text_buffer: text_buffer,
        }
    }
}

impl<'a, T> Drawing<'a, T> {
    // Shared between the **finish** method and the **Drawing**'s **Drop** implementation.
    //
    // 1. Create vertices based on node-specific position, points, etc.
    // 2. Insert edges into geom graph based on
    fn finish_inner(&mut self) {
        match self.draw.state.try_borrow_mut() {
            Err(err) => eprintln!("drawing failed to borrow state and finish: {}", err),
            Ok(mut state) => state.finish_drawing(self.index),
        }
    }

    /// Complete the drawing and insert it into the parent **Draw** instance.
    ///
    /// This will be called when the **Drawing** is **Drop**ped if it has not yet been called.
    pub fn finish(mut self) {
        self.finish_inner()
    }

    // Map the given function onto the primitive stored within **Draw** at `index`.
    //
    // The functionn is only applied if the node has not yet been **Drawn**.
    fn map_primitive<F, T2>(mut self, map: F) -> Drawing<'a, T2>
    where
        F: FnOnce(Primitive) -> Primitive,
        T2: Into<Primitive>,
    {
        if let Ok(mut state) = self.draw.state.try_borrow_mut() {
            if let Some(mut primitive) = state.drawing.remove(&self.index) {
                primitive = map(primitive);
                state.drawing.insert(self.index, primitive);
            }
        }
        self.finish_on_drop = false;
        let Drawing { draw, index, .. } = self;
        Drawing {
            draw,
            index,
            finish_on_drop: true,
            _ty: PhantomData,
        }
    }

    // The same as `map_primitive` but also passes a mutable reference to the vertex data to the
    // map function. This is useful for types that may have an unknown number of arbitrary
    // vertices.
    fn map_primitive_with_context<F, T2>(mut self, map: F) -> Drawing<'a, T2>
    where
        F: FnOnce(Primitive, DrawingContext) -> Primitive,
        T2: Into<Primitive>,
    {
        if let Ok(mut state) = self.draw.state.try_borrow_mut() {
            if let Some(mut primitive) = state.drawing.remove(&self.index) {
                {
                    let mut intermediary_state = state.intermediary_state.borrow_mut();
                    let ctxt = DrawingContext::from_intermediary_state(&mut *intermediary_state);
                    primitive = map(primitive, ctxt);
                }
                state.drawing.insert(self.index, primitive);
            }
        }
        self.finish_on_drop = false;
        let Drawing { draw, index, .. } = self;
        Drawing {
            draw,
            index,
            finish_on_drop: true,
            _ty: PhantomData,
        }
    }

    /// Apply the given function to the type stored within **Draw**.
    ///
    /// The function is only applied if the node has not yet been **Drawn**.
    ///
    /// **Panics** if the primitive does not contain type **T**.
    pub fn map_ty<F, T2>(self, map: F) -> Drawing<'a, T2>
    where
        F: FnOnce(T) -> T2,
        T2: Into<Primitive>,
        Primitive: Into<Option<T>>,
    {
        self.map_primitive(|prim| {
            let maybe_ty: Option<T> = prim.into();
            let ty = maybe_ty.expect("expected `T` but primitive contained different type");
            let ty2 = map(ty);
            ty2.into()
        })
    }

    /// Apply the given function to the type stored within **Draw**.
    ///
    /// The function is only applied if the node has not yet been **Drawn**.
    ///
    /// **Panics** if the primitive does not contain type **T**.
    pub(crate) fn map_ty_with_context<F, T2>(self, map: F) -> Drawing<'a, T2>
    where
        F: FnOnce(T, DrawingContext) -> T2,
        T2: Into<Primitive>,
        Primitive: Into<Option<T>>,
    {
        self.map_primitive_with_context(|prim, ctxt| {
            let maybe_ty: Option<T> = prim.into();
            let ty = maybe_ty.expect("expected `T` but primitive contained different type");
            let ty2 = map(ty, ctxt);
            ty2.into()
        })
    }
}

// SetColor implementations.

impl<'a, T> Drawing<'a, T>
where
    T: SetColor<ColorScalar> + Into<Primitive>,
    Primitive: Into<Option<T>>,
{
    /// Specify a color.
    ///
    /// This method supports any color type that can be converted into RGBA.
    ///
    /// Colors that have no alpha channel will be given an opaque alpha channel value `1.0`.
    pub fn color<C>(self, color: C) -> Self
    where
        C: IntoLinSrgba<ColorScalar>,
    {
        self.map_ty(|ty| SetColor::color(ty, color))
    }

    /// Specify the color via red, green and blue channels.
    pub fn rgb(self, r: ColorScalar, g: ColorScalar, b: ColorScalar) -> Self {
        self.map_ty(|ty| SetColor::rgb(ty, r, g, b))
    }

    /// Specify the color via red, green and blue channels as bytes
    pub fn rgb8(self, r: u8, g: u8, b: u8) -> Self {
        self.map_ty(|ty| SetColor::rgb8(ty, r, g, b))
    }

    /// Specify the color via red, green, blue and alpha channels.
    pub fn rgba(self, r: ColorScalar, g: ColorScalar, b: ColorScalar, a: ColorScalar) -> Self {
        self.map_ty(|ty| SetColor::rgba(ty, r, g, b, a))
    }

    /// Specify the color via red, green, blue and alpha channels as bytes.
    pub fn rgba8(self, r: u8, g: u8, b: u8, a: u8) -> Self {
        self.map_ty(|ty| SetColor::rgba8(ty, r, g, b, a))
    }

    /// Specify the color via hue, saturation and luminance.
    ///
    /// If you're looking for HSVA or HSBA, use the `hsva` method instead.
    ///
    /// The given hue expects a value between `0.0` and `1.0` where `0.0` is 0 degress and `1.0` is
    /// 360 degrees (or 2 PI radians).
    ///
    /// See the [wikipedia entry](https://en.wikipedia.org/wiki/HSL_and_HSV) for more details on
    /// this color space.
    pub fn hsl(self, h: ColorScalar, s: ColorScalar, l: ColorScalar) -> Self {
        self.map_ty(|ty| SetColor::hsl(ty, h, s, l))
    }

    /// Specify the color via hue, saturation, luminance and an alpha channel.
    ///
    /// If you're looking for HSVA or HSBA, use the `hsva` method instead.
    ///
    /// The given hue expects a value between `0.0` and `1.0` where `0.0` is 0 degress and `1.0` is
    /// 360 degrees (or 2 PI radians).
    ///
    /// See the [wikipedia entry](https://en.wikipedia.org/wiki/HSL_and_HSV) for more details on
    /// this color space.
    pub fn hsla(self, h: ColorScalar, s: ColorScalar, l: ColorScalar, a: ColorScalar) -> Self {
        self.map_ty(|ty| SetColor::hsla(ty, h, s, l, a))
    }

    /// Specify the color via hue, saturation and *value* (brightness).
    ///
    /// This is sometimes also known as "hsb".
    ///
    /// The given hue expects a value between `0.0` and `1.0` where `0.0` is 0 degress and `1.0` is
    /// 360 degrees (or 2 PI radians).
    ///
    /// See the [wikipedia entry](https://en.wikipedia.org/wiki/HSL_and_HSV) for more details on
    /// this color space.
    pub fn hsv(self, h: ColorScalar, s: ColorScalar, v: ColorScalar) -> Self {
        self.map_ty(|ty| SetColor::hsv(ty, h, s, v))
    }

    /// Specify the color via hue, saturation, *value* (brightness) and an alpha channel.
    ///
    /// This is sometimes also known as "hsba".
    ///
    /// The given hue expects a value between `0.0` and `1.0` where `0.0` is 0 degress and `1.0` is
    /// 360 degrees (or 2 PI radians).
    ///
    /// See the [wikipedia entry](https://en.wikipedia.org/wiki/HSL_and_HSV) for more details on
    /// this color space.
    pub fn hsva(self, h: ColorScalar, s: ColorScalar, v: ColorScalar, a: ColorScalar) -> Self {
        self.map_ty(|ty| SetColor::hsva(ty, h, s, v, a))
    }

    /// Specify the color as gray scale
    ///
    /// The given g expects a value between `0.0` and `1.0` where `0.0` is black and `1.0` is white
    pub fn gray(self, g: ColorScalar) -> Self {
        self.map_ty(|ty| SetColor::gray(ty, g))
    }
}

// SetDimensions implementations.

impl<'a, T> Drawing<'a, T>
where
    T: SetDimensions + Into<Primitive>,
    Primitive: Into<Option<T>>,
{
    /// Set the absolute width for the node.
    pub fn width(self, w: f32) -> Self {
        self.map_ty(|ty| SetDimensions::width(ty, w))
    }

    /// Set the absolute height for the node.
    pub fn height(self, h: f32) -> Self {
        self.map_ty(|ty| SetDimensions::height(ty, h))
    }

    /// Set the absolute depth for the node.
    pub fn depth(self, d: f32) -> Self {
        self.map_ty(|ty| SetDimensions::depth(ty, d))
    }

    /// Short-hand for the **width** method.
    pub fn w(self, w: f32) -> Self {
        self.map_ty(|ty| SetDimensions::w(ty, w))
    }

    /// Short-hand for the **height** method.
    pub fn h(self, h: f32) -> Self {
        self.map_ty(|ty| SetDimensions::h(ty, h))
    }

    /// Short-hand for the **depth** method.
    pub fn d(self, d: f32) -> Self {
        self.map_ty(|ty| SetDimensions::d(ty, d))
    }

    /// Set the **x** and **y** dimensions for the node.
    pub fn wh(self, v: Vec2) -> Self {
        self.map_ty(|ty| SetDimensions::wh(ty, v))
    }

    /// Set the **x**, **y** and **z** dimensions for the node.
    pub fn whd(self, v: Vec3) -> Self {
        self.map_ty(|ty| SetDimensions::whd(ty, v))
    }

    /// Set the width and height for the node.
    pub fn w_h(self, x: f32, y: f32) -> Self {
        self.map_ty(|ty| SetDimensions::w_h(ty, x, y))
    }

    /// Set the width and height for the node.
    pub fn w_h_d(self, x: f32, y: f32, z: f32) -> Self {
        self.map_ty(|ty| SetDimensions::w_h_d(ty, x, y, z))
    }
}

// SetPosition methods.

impl<'a, T> Drawing<'a, T>
where
    T: SetPosition + Into<Primitive>,
    Primitive: Into<Option<T>>,
{
    /// Build with the given **Absolute** **Position** along the *x* axis.
    pub fn x(self, x: f32) -> Self {
        self.map_ty(|ty| SetPosition::x(ty, x))
    }

    /// Build with the given **Absolute** **Position** along the *y* axis.
    pub fn y(self, y: f32) -> Self {
        self.map_ty(|ty| SetPosition::y(ty, y))
    }

    /// Build with the given **Absolute** **Position** along the *z* axis.
    pub fn z(self, z: f32) -> Self {
        self.map_ty(|ty| SetPosition::z(ty, z))
    }

    /// Set the **Position** with some two-dimensional point.
    pub fn xy(self, p: Point2) -> Self {
        self.map_ty(|ty| SetPosition::xy(ty, p))
    }

    /// Set the **Position** with some three-dimensional point.
    pub fn xyz(self, p: Point3) -> Self {
        self.map_ty(|ty| SetPosition::xyz(ty, p))
    }

    /// Set the **Position** with *x* *y* coordinates.
    pub fn x_y(self, x: f32, y: f32) -> Self {
        self.map_ty(|ty| SetPosition::x_y(ty, x, y))
    }

    /// Set the **Position** with *x* *y* *z* coordinates.
    pub fn x_y_z(self, x: f32, y: f32, z: f32) -> Self {
        self.map_ty(|ty| SetPosition::x_y_z(ty, x, y, z))
    }
}

// SetOrientation methods.

impl<'a, T> Drawing<'a, T>
where
    T: SetOrientation + Into<Primitive>,
    Primitive: Into<Option<T>>,
{
    /// Describe orientation via the vector that points to the given target.
    pub fn look_at(self, target: Point3) -> Self {
        self.map_ty(|ty| SetOrientation::look_at(ty, target))
    }

    /// Specify the orientation around the *x* axis as an absolute value in radians.
    pub fn x_radians(self, x: f32) -> Self {
        self.map_ty(|ty| SetOrientation::x_radians(ty, x))
    }

    /// Specify the orientation around the *y* axis as an absolute value in radians.
    pub fn y_radians(self, y: f32) -> Self {
        self.map_ty(|ty| SetOrientation::y_radians(ty, y))
    }

    /// Specify the orientation around the *z* axis as an absolute value in radians.
    pub fn z_radians(self, z: f32) -> Self {
        self.map_ty(|ty| SetOrientation::z_radians(ty, z))
    }

    /// Specify the orientation around the *x* axis as an absolute value in degrees.
    pub fn x_degrees(self, x: f32) -> Self {
        self.map_ty(|ty| SetOrientation::x_degrees(ty, x))
    }

    /// Specify the orientation around the *y* axis as an absolute value in degrees.
    pub fn y_degrees(self, y: f32) -> Self {
        self.map_ty(|ty| SetOrientation::y_degrees(ty, y))
    }

    /// Specify the orientation around the *z* axis as an absolute value in degrees.
    pub fn z_degrees(self, z: f32) -> Self {
        self.map_ty(|ty| SetOrientation::z_degrees(ty, z))
    }

    /// Specify the orientation around the *x* axis as a number of turns around the axis.
    pub fn x_turns(self, x: f32) -> Self {
        self.map_ty(|ty| SetOrientation::x_turns(ty, x))
    }

    /// Specify the orientation around the *y* axis as a number of turns around the axis.
    pub fn y_turns(self, y: f32) -> Self {
        self.map_ty(|ty| SetOrientation::y_turns(ty, y))
    }

    /// Specify the orientation around the *z* axis as a number of turns around the axis.
    pub fn z_turns(self, z: f32) -> Self {
        self.map_ty(|ty| SetOrientation::z_turns(ty, z))
    }

    /// Specify the orientation along each axis with the given **Vector** of radians.
    ///
    /// This has the same affect as calling `self.x_radians(v.x).y_radians(v.y).z_radians(v.z)`.
    pub fn radians(self, v: Vec3) -> Self {
        self.map_ty(|ty| SetOrientation::radians(ty, v))
    }

    /// Specify the orientation along each axis with the given **Vector** of degrees.
    ///
    /// This has the same affect as calling `self.x_degrees(v.x).y_degrees(v.y).z_degrees(v.z)`.
    pub fn degrees(self, v: Vec3) -> Self {
        self.map_ty(|ty| SetOrientation::degrees(ty, v))
    }

    /// Specify the orientation along each axis with the given **Vector** of "turns".
    ///
    /// This has the same affect as calling `self.x_turns(v.x).y_turns(v.y).z_turns(v.z)`.
    pub fn turns(self, v: Vec3) -> Self {
        self.map_ty(|ty| SetOrientation::turns(ty, v))
    }

    /// Specify the orientation with the given **Euler**.
    ///
    /// The euler must be specified in radians.
    pub fn euler(self, e: Vec3) -> Self {
        self.map_ty(|ty| SetOrientation::euler(ty, e))
    }

    /// Specify the orientation with the given **Quaternion**.
    pub fn quaternion(self, q: Quat) -> Self {
        self.map_ty(|ty| SetOrientation::quaternion(ty, q))
    }

    // Higher level methods.

    /// Specify the "pitch" of the orientation in radians.
    ///
    /// This has the same effect as calling `x_radians`.
    pub fn pitch(self, pitch: f32) -> Self {
        self.map_ty(|ty| SetOrientation::pitch(ty, pitch))
    }

    /// Specify the "yaw" of the orientation in radians.
    ///
    /// This has the same effect as calling `y_radians`.
    pub fn yaw(self, yaw: f32) -> Self {
        self.map_ty(|ty| SetOrientation::yaw(ty, yaw))
    }

    /// Specify the "roll" of the orientation in radians.
    ///
    /// This has the same effect as calling `z_radians`.
    pub fn roll(self, roll: f32) -> Self {
        self.map_ty(|ty| SetOrientation::roll(ty, roll))
    }

    /// Assuming we're looking at a 2D plane, positive values cause a clockwise rotation where the
    /// given value is specified in radians.
    ///
    /// This is equivalent to calling the `z_radians` or `roll` methods.
    pub fn rotate(self, radians: f32) -> Self {
        self.map_ty(|ty| SetOrientation::rotate(ty, radians))
    }
}

// SetFill methods

impl<'a, T> Drawing<'a, T>
where
    T: SetFill + Into<Primitive>,
    Primitive: Into<Option<T>>,
{
    /// Specify the whole set of fill tessellation options.
    pub fn fill_opts(self, opts: FillOptions) -> Self {
        self.map_ty(|ty| ty.fill_opts(opts))
    }

    /// Maximum allowed distance to the path when building an approximation.
    pub fn fill_tolerance(self, tolerance: f32) -> Self {
        self.map_ty(|ty| ty.fill_tolerance(tolerance))
    }

    /// Specify the rule used to determine what is inside and what is outside of the shape.
    ///
    /// Currently, only the `EvenOdd` rule is implemented.
    pub fn fill_rule(self, rule: lyon::tessellation::FillRule) -> Self {
        self.map_ty(|ty| ty.fill_rule(rule))
    }

    /// Whether to perform a vertical or horizontal traversal of the geometry.
    ///
    /// Default value: `Vertical`.
    pub fn fill_sweep_orientation(self, orientation: lyon::tessellation::Orientation) -> Self {
        self.map_ty(|ty| ty.fill_sweep_orientation(orientation))
    }

    /// A fast path to avoid some expensive operations if the path is known to not have any
    /// self-intersections.
    ///
    /// Do not set this to `false` if the path may have intersecting edges else the tessellator may
    /// panic or produce incorrect results. In doubt, do not change the default value.
    ///
    /// Default value: `true`.
    pub fn handle_intersections(self, b: bool) -> Self {
        self.map_ty(|ty| ty.handle_intersections(b))
    }
}

// SetStroke methods

impl<'a, T> Drawing<'a, T>
where
    T: SetStroke + Into<Primitive>,
    Primitive: Into<Option<T>>,
{
    /// The start line cap as specified by the SVG spec.
    pub fn start_cap(self, cap: LineCap) -> Self {
        self.map_ty(|ty| ty.start_cap(cap))
    }

    /// The end line cap as specified by the SVG spec.
    pub fn end_cap(self, cap: LineCap) -> Self {
        self.map_ty(|ty| ty.end_cap(cap))
    }

    /// The start and end line cap as specified by the SVG spec.
    pub fn caps(self, cap: LineCap) -> Self {
        self.map_ty(|ty| ty.caps(cap))
    }

    /// The stroke for each sub-path does not extend beyond its two endpoints. A zero length
    /// sub-path will therefore not have any stroke.
    pub fn start_cap_butt(self) -> Self {
        self.map_ty(|ty| ty.start_cap_butt())
    }

    /// At the end of each sub-path, the shape representing the stroke will be extended by a
    /// rectangle with the same width as the stroke width and whose length is half of the stroke
    /// width. If a sub-path has zero length, then the resulting effect is that the stroke for that
    /// sub-path consists solely of a square with side length equal to the stroke width, centered
    /// at the sub-path's point.
    pub fn start_cap_square(self) -> Self {
        self.map_ty(|ty| ty.start_cap_square())
    }

    /// At each end of each sub-path, the shape representing the stroke will be extended by a half
    /// circle with a radius equal to the stroke width. If a sub-path has zero length, then the
    /// resulting effect is that the stroke for that sub-path consists solely of a full circle
    /// centered at the sub-path's point.
    pub fn start_cap_round(self) -> Self {
        self.map_ty(|ty| ty.start_cap_round())
    }

    /// The stroke for each sub-path does not extend beyond its two endpoints. A zero length
    /// sub-path will therefore not have any stroke.
    pub fn end_cap_butt(self) -> Self {
        self.map_ty(|ty| ty.end_cap_butt())
    }

    /// At the end of each sub-path, the shape representing the stroke will be extended by a
    /// rectangle with the same width as the stroke width and whose length is half of the stroke
    /// width. If a sub-path has zero length, then the resulting effect is that the stroke for that
    /// sub-path consists solely of a square with side length equal to the stroke width, centered
    /// at the sub-path's point.
    pub fn end_cap_square(self) -> Self {
        self.map_ty(|ty| ty.end_cap_square())
    }

    /// At each end of each sub-path, the shape representing the stroke will be extended by a half
    /// circle with a radius equal to the stroke width. If a sub-path has zero length, then the
    /// resulting effect is that the stroke for that sub-path consists solely of a full circle
    /// centered at the sub-path's point.
    pub fn end_cap_round(self) -> Self {
        self.map_ty(|ty| ty.end_cap_round())
    }

    /// The stroke for each sub-path does not extend beyond its two endpoints. A zero length
    /// sub-path will therefore not have any stroke.
    pub fn caps_butt(self) -> Self {
        self.map_ty(|ty| ty.caps_butt())
    }

    /// At the end of each sub-path, the shape representing the stroke will be extended by a
    /// rectangle with the same width as the stroke width and whose length is half of the stroke
    /// width. If a sub-path has zero length, then the resulting effect is that the stroke for that
    /// sub-path consists solely of a square with side length equal to the stroke width, centered
    /// at the sub-path's point.
    pub fn caps_square(self) -> Self {
        self.map_ty(|ty| ty.caps_square())
    }

    /// At each end of each sub-path, the shape representing the stroke will be extended by a half
    /// circle with a radius equal to the stroke width. If a sub-path has zero length, then the
    /// resulting effect is that the stroke for that sub-path consists solely of a full circle
    /// centered at the sub-path's point.
    pub fn caps_round(self) -> Self {
        self.map_ty(|ty| ty.caps_round())
    }

    /// The way in which lines are joined at the vertices, matching the SVG spec.
    ///
    /// Default value is `MiterClip`.
    pub fn join(self, join: LineJoin) -> Self {
        self.map_ty(|ty| ty.join(join))
    }

    /// A sharp corner is to be used to join path segments.
    pub fn join_miter(self) -> Self {
        self.map_ty(|ty| ty.join_miter())
    }

    /// Same as a `join_miter`, but if the miter limit is exceeded, the miter is clipped at a miter
    /// length equal to the miter limit value multiplied by the stroke width.
    pub fn join_miter_clip(self) -> Self {
        self.map_ty(|ty| ty.join_miter_clip())
    }

    /// A round corner is to be used to join path segments.
    pub fn join_round(self) -> Self {
        self.map_ty(|ty| ty.join_round())
    }

    /// A bevelled corner is to be used to join path segments. The bevel shape is a triangle that
    /// fills the area between the two stroked segments.
    pub fn join_bevel(self) -> Self {
        self.map_ty(|ty| ty.join_bevel())
    }

    /// The total stroke_weight (aka width) of the line.
    pub fn stroke_weight(self, stroke_weight: f32) -> Self {
        self.map_ty(|ty| ty.stroke_weight(stroke_weight))
    }

    /// Describes the limit before miter lines will clip, as described in the SVG spec.
    ///
    /// Must be greater than or equal to `1.0`.
    pub fn miter_limit(self, limit: f32) -> Self {
        self.map_ty(|ty| ty.miter_limit(limit))
    }

    /// Maximum allowed distance to the path when building an approximation.
    pub fn stroke_tolerance(self, tolerance: f32) -> Self {
        self.map_ty(|ty| ty.stroke_tolerance(tolerance))
    }

    /// Specify the full set of stroke options for the path tessellation.
    pub fn stroke_opts(self, opts: StrokeOptions) -> Self {
        self.map_ty(|ty| ty.stroke_opts(opts))
    }
}
