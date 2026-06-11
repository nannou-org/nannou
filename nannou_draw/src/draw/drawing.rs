use std::marker::PhantomData;

use bevy::asset::UntypedAssetId;
use bevy::prelude::*;
use lyon::path::PathEvent;
use lyon::tessellation::{FillOptions, LineCap, LineJoin, StrokeOptions};
use uuid::Uuid;

use crate::draw::primitive::Primitive;
use crate::draw::properties::{
    SetColor, SetDimensions, SetFill, SetOrientation, SetPosition, SetStroke, color, fill,
    spatial::{dimension, orientation, position},
    stroke,
};
use crate::draw::{Draw, DrawCommand, DrawRef};
use crate::render::{DefaultNannouShaderModel, ErasedShaderModel, ShaderModel};

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
///
/// The type parameter `T` is a zero-cost marker: the primitive itself is stored type-erased as a
/// [`Primitive`] within the **Draw**'s state, and `T` only determines which builder methods are
/// available (via bounds like `T: SetColor`) and tracks type-state transitions (e.g.
/// `PathInit -> PathFill -> Path`). The method bodies delegate to non-generic functions that
/// mutate the stored primitive in place, keeping monomorphisation in downstream crates to a
/// minimum.
pub struct Drawing<'a, T> {
    // The `Draw` instance used to create this drawing.
    pub(crate) draw: DrawRef<'a>,
    // The draw command index of the primitive being drawn.
    pub(crate) index: usize,
    // The draw command index of the shader model being used.
    pub(crate) shader_model_index: usize,
    // Whether the **Drawing** should attempt to finish the drawing on drop.
    pub(crate) finish_on_drop: bool,
    // The node type currently being drawn.
    _ty: PhantomData<T>,
}

/// Some context that may be optionally provided to primitives in the drawing implementation.
///
/// This is particularly useful for paths and meshes.
pub struct DrawingContext<'a> {
    /// The intermediary mesh for buffering yet-to-be-drawn paths and meshes.
    pub mesh: &'a mut Mesh,
    /// A re-usable buffer for collecting path events.
    pub path_event_buffer: &'a mut Vec<PathEvent>,
    /// A re-usable buffer for collecting polyline points vertex data.
    pub path_points_vertex_buffer: &'a mut Vec<(Vec2, Color, Vec2)>,
    /// A re-usable buffer for collecting text.
    pub text_buffer: &'a mut String,
}

/// Construct a new **Drawing** instance.
pub fn new<T>(draw: &Draw, index: usize, model_index: usize) -> Drawing<'_, T>
where
    T: Into<Primitive>,
{
    let _ty = PhantomData;
    let finish_on_drop = true;
    Drawing {
        draw: DrawRef::Borrowed(draw),
        index,
        shader_model_index: model_index,
        finish_on_drop,
        _ty,
    }
}

impl<'a, T> Drop for Drawing<'a, T> {
    fn drop(&mut self) {
        if self.finish_on_drop {
            finish_drawing(&self.draw, self.index, self.shader_model_index)
        }
    }
}

impl<'a> DrawingContext<'a> {
    // Initialise the DrawingContext from the draw's IntermediaryState.
    pub(crate) fn from_intermediary_state(state: &'a mut super::IntermediaryState) -> Self {
        let super::IntermediaryState {
            ref mut intermediary_mesh,
            ref mut path_event_buffer,
            ref mut path_points_vertex_buffer,
            ref mut text_buffer,
        } = *state;
        DrawingContext {
            mesh: intermediary_mesh,
            path_event_buffer,
            path_points_vertex_buffer,
            text_buffer,
        }
    }
}

impl<'a, T> Drawing<'a, T> {
    /// Complete the drawing and insert it into the parent **Draw** instance.
    ///
    /// This will be called when the **Drawing** is **Drop**ped if it has not yet been called.
    pub fn finish(mut self) {
        self.finish_on_drop = false;
        finish_drawing(&self.draw, self.index, self.shader_model_index)
    }

    // Consume the drawing, producing an equivalent drawing for the next type-state `U`.
    //
    // The conversion of the stored primitive itself is expected to have already been performed
    // (e.g. via `with_primitive`).
    pub(crate) fn transition<U>(mut self) -> Drawing<'a, U> {
        self.finish_on_drop = false;
        let Drawing {
            ref draw,
            index,
            shader_model_index,
            ..
        } = self;
        Drawing {
            draw: draw.clone(),
            index,
            shader_model_index,
            finish_on_drop: true,
            _ty: PhantomData,
        }
    }

    /// Map the parent's shader model to a new instance, drawing this primitive and any
    /// subsequent drawings with the result.
    ///
    /// The active shader model must be of type `SM`; if it is not, a warning is emitted and the
    /// mapping is skipped.
    pub fn map_shader_model<SM, F>(self, map: F) -> Drawing<'a, T>
    where
        SM: ShaderModel,
        F: FnOnce(SM) -> SM,
    {
        let shader_model = {
            let state = self.draw.state.read().unwrap();
            state.shader_models[&self.draw.shader_model]
                .as_any()
                .downcast_ref::<SM>()
                .cloned()
        };
        match shader_model {
            Some(shader_model) => {
                let shader_model = map(shader_model);
                self.with_new_shader_model(Box::new(shader_model))
            }
            None => {
                bevy::log::warn_once!(
                    "`map_shader_model`: the active shader model is not of the requested type; \
                     the mapping has been skipped"
                );
                self
            }
        }
    }

    // Insert the given model under a fresh id, drawing this primitive and any subsequent
    // drawings with it. Takes ownership over the draw instance clone.
    fn with_new_shader_model(mut self, model: Box<dyn ErasedShaderModel>) -> Drawing<'a, T> {
        self.finish_on_drop = false;

        let Drawing {
            ref draw,
            index,
            shader_model_index,
            ..
        } = self;

        let new_id = UntypedAssetId::Uuid {
            type_id: model.as_any().type_id(),
            uuid: Uuid::new_v4(),
        };

        let state = draw.state.clone();
        let mut state = state.write().unwrap();
        state.shader_models.insert(new_id.clone(), model);
        // Mark the last shader model as the new model so that further drawings use the same model
        // as the parent draw ref.
        state.last_shader_model = Some(new_id.clone());

        let draw = Draw {
            state: draw.state.clone(),
            context: draw.context.clone(),
            shader_model: new_id.clone(),
            window: draw.window,
            text_cx: draw.text_cx.clone(),
        };

        Drawing {
            draw: DrawRef::Owned(draw),
            index,
            shader_model_index,
            finish_on_drop: true,
            _ty: PhantomData,
        }
    }
}

// SetColor implementations.

impl<'a, T> Drawing<'a, T>
where
    T: SetColor,
{
    /// Specify a color.
    ///
    /// This method supports any color type that can be converted into RGBA.
    ///
    /// Colors that have no alpha channel will be given an opaque alpha channel value `1.0`.
    pub fn color<C>(self, color: C) -> Self
    where
        C: Into<Color>,
    {
        color::set_color(&self.draw, self.index, color.into());
        self
    }

    /// Specify the color via red, green and blue channels.
    pub fn srgb(self, r: f32, g: f32, b: f32) -> Self {
        self.color(Color::srgb(r, g, b))
    }

    /// Specify the color via red, green and blue channels as bytes
    pub fn srgb_u8(self, r: u8, g: u8, b: u8) -> Self {
        self.color(Color::srgb_u8(r, g, b))
    }

    /// Specify the color via red, green, blue and alpha channels.
    pub fn srgba(self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.color(Color::srgba(r, g, b, a))
    }

    /// Specify the color via red, green, blue and alpha channels as bytes.
    pub fn srgba_u8(self, r: u8, g: u8, b: u8, a: u8) -> Self {
        self.color(Color::srgba_u8(r, g, b, a))
    }

    pub fn linear_rgb(self, r: f32, g: f32, b: f32) -> Self {
        self.color(Color::linear_rgb(r, g, b))
    }

    pub fn linear_rgba(self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.color(Color::linear_rgba(r, g, b, a))
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
    pub fn hsl(self, h: f32, s: f32, l: f32) -> Self {
        self.color(Color::hsl(h * 360.0, s, l))
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
    pub fn hsla(self, h: f32, s: f32, l: f32, a: f32) -> Self {
        self.color(Color::hsla(h * 360.0, s, l, a))
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
    pub fn hsv(self, h: f32, s: f32, v: f32) -> Self {
        self.color(Color::hsv(h * 360.0, s, v))
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
    pub fn hsva(self, h: f32, s: f32, v: f32, a: f32) -> Self {
        self.color(Color::hsva(h * 360.0, s, v, a))
    }

    pub fn hwb(self, h: f32, w: f32, b: f32) -> Self {
        self.color(Color::hwb(h * 360.0, w, b))
    }

    pub fn hwba(self, h: f32, w: f32, b: f32, a: f32) -> Self {
        self.color(Color::hwba(h * 360.0, w, b, a))
    }

    pub fn lab(self, l: f32, a: f32, b: f32) -> Self {
        self.color(Color::lab(l, a, b))
    }

    pub fn laba(self, l: f32, a: f32, b: f32, alpha: f32) -> Self {
        self.color(Color::laba(l, a, b, alpha))
    }

    pub fn lch(self, l: f32, c: f32, h: f32) -> Self {
        self.color(Color::lch(l, c, h))
    }

    pub fn lcha(self, l: f32, c: f32, h: f32, alpha: f32) -> Self {
        self.color(Color::lcha(l, c, h, alpha))
    }

    pub fn oklab(self, l: f32, a: f32, b: f32) -> Self {
        self.color(Color::oklab(l, a, b))
    }

    pub fn oklaba(self, l: f32, a: f32, b: f32, alpha: f32) -> Self {
        self.color(Color::oklaba(l, a, b, alpha))
    }

    pub fn oklch(self, l: f32, c: f32, h: f32) -> Self {
        self.color(Color::oklch(l, c, h))
    }

    pub fn oklcha(self, l: f32, c: f32, h: f32, alpha: f32) -> Self {
        self.color(Color::oklcha(l, c, h, alpha))
    }

    pub fn cie_xyz(self, x: f32, y: f32, z: f32) -> Self {
        self.color(Color::xyz(x, y, z))
    }

    pub fn cie_xyza(self, x: f32, y: f32, z: f32, alpha: f32) -> Self {
        self.color(Color::xyza(x, y, z, alpha))
    }

    /// Specify the color as gray scale
    ///
    /// The given g expects a value between `0.0` and `1.0` where `0.0` is black and `1.0` is white
    pub fn gray(self, g: f32) -> Self {
        self.color(Color::srgb(g, g, g))
    }
}

// SetDimensions implementations.

impl<'a, T> Drawing<'a, T>
where
    T: SetDimensions,
{
    /// Set the absolute width for the node.
    pub fn width(self, w: f32) -> Self {
        dimension::set_dimensions(&self.draw, self.index, Some(w), None, None);
        self
    }

    /// Set the absolute height for the node.
    pub fn height(self, h: f32) -> Self {
        dimension::set_dimensions(&self.draw, self.index, None, Some(h), None);
        self
    }

    /// Set the absolute depth for the node.
    pub fn depth(self, d: f32) -> Self {
        dimension::set_dimensions(&self.draw, self.index, None, None, Some(d));
        self
    }

    /// Short-hand for the **width** method.
    pub fn w(self, w: f32) -> Self {
        self.width(w)
    }

    /// Short-hand for the **height** method.
    pub fn h(self, h: f32) -> Self {
        self.height(h)
    }

    /// Short-hand for the **depth** method.
    pub fn d(self, d: f32) -> Self {
        self.depth(d)
    }

    /// Set the **x** and **y** dimensions for the node.
    pub fn wh(self, v: Vec2) -> Self {
        dimension::set_dimensions(&self.draw, self.index, Some(v.x), Some(v.y), None);
        self
    }

    /// Set the **x**, **y** and **z** dimensions for the node.
    pub fn whd(self, v: Vec3) -> Self {
        dimension::set_dimensions(&self.draw, self.index, Some(v.x), Some(v.y), Some(v.z));
        self
    }

    /// Set the width and height for the node.
    pub fn w_h(self, x: f32, y: f32) -> Self {
        dimension::set_dimensions(&self.draw, self.index, Some(x), Some(y), None);
        self
    }

    /// Set the width and height for the node.
    pub fn w_h_d(self, x: f32, y: f32, z: f32) -> Self {
        dimension::set_dimensions(&self.draw, self.index, Some(x), Some(y), Some(z));
        self
    }
}

// SetPosition methods.

impl<'a, T> Drawing<'a, T>
where
    T: SetPosition,
{
    /// Build with the given **Absolute** **Position** along the *x* axis.
    pub fn x(self, x: f32) -> Self {
        position::set_position(&self.draw, self.index, Some(x), None, None);
        self
    }

    /// Build with the given **Absolute** **Position** along the *y* axis.
    pub fn y(self, y: f32) -> Self {
        position::set_position(&self.draw, self.index, None, Some(y), None);
        self
    }

    /// Build with the given **Absolute** **Position** along the *z* axis.
    pub fn z(self, z: f32) -> Self {
        position::set_position(&self.draw, self.index, None, None, Some(z));
        self
    }

    /// Set the **Position** with some two-dimensional point.
    pub fn xy(self, p: Vec2) -> Self {
        position::set_position(&self.draw, self.index, Some(p.x), Some(p.y), None);
        self
    }

    /// Set the **Position** with some three-dimensional point.
    pub fn xyz(self, p: Vec3) -> Self {
        position::set_position(&self.draw, self.index, Some(p.x), Some(p.y), Some(p.z));
        self
    }

    /// Set the **Position** with *x* *y* coordinates.
    pub fn x_y(self, x: f32, y: f32) -> Self {
        position::set_position(&self.draw, self.index, Some(x), Some(y), None);
        self
    }

    /// Set the **Position** with *x* *y* *z* coordinates.
    pub fn x_y_z(self, x: f32, y: f32, z: f32) -> Self {
        position::set_position(&self.draw, self.index, Some(x), Some(y), Some(z));
        self
    }
}

// SetOrientation methods.

impl<'a, T> Drawing<'a, T>
where
    T: SetOrientation,
{
    /// Describe orientation via the vector that points to the given target.
    pub fn look_at(self, target: Vec3) -> Self {
        orientation::set_orientation(&self.draw, self.index, orientation::Update::LookAt(target));
        self
    }

    /// Specify the orientation around the *x* axis as an absolute value in radians.
    pub fn x_radians(self, x: f32) -> Self {
        orientation::set_orientation(&self.draw, self.index, orientation::Update::XRadians(x));
        self
    }

    /// Specify the orientation around the *y* axis as an absolute value in radians.
    pub fn y_radians(self, y: f32) -> Self {
        orientation::set_orientation(&self.draw, self.index, orientation::Update::YRadians(y));
        self
    }

    /// Specify the orientation around the *z* axis as an absolute value in radians.
    pub fn z_radians(self, z: f32) -> Self {
        orientation::set_orientation(&self.draw, self.index, orientation::Update::ZRadians(z));
        self
    }

    /// Specify the orientation around the *x* axis as an absolute value in degrees.
    pub fn x_degrees(self, x: f32) -> Self {
        self.x_radians(x.to_radians())
    }

    /// Specify the orientation around the *y* axis as an absolute value in degrees.
    pub fn y_degrees(self, y: f32) -> Self {
        self.y_radians(y.to_radians())
    }

    /// Specify the orientation around the *z* axis as an absolute value in degrees.
    pub fn z_degrees(self, z: f32) -> Self {
        self.z_radians(z.to_radians())
    }

    /// Specify the orientation around the *x* axis as a number of turns around the axis.
    pub fn x_turns(self, x: f32) -> Self {
        self.x_radians(x * std::f32::consts::TAU)
    }

    /// Specify the orientation around the *y* axis as a number of turns around the axis.
    pub fn y_turns(self, y: f32) -> Self {
        self.y_radians(y * std::f32::consts::TAU)
    }

    /// Specify the orientation around the *z* axis as a number of turns around the axis.
    pub fn z_turns(self, z: f32) -> Self {
        self.z_radians(z * std::f32::consts::TAU)
    }

    /// Specify the orientation along each axis with the given **Vector** of radians.
    ///
    /// This has the same affect as calling `self.x_radians(v.x).y_radians(v.y).z_radians(v.z)`.
    pub fn radians(self, v: Vec3) -> Self {
        orientation::set_orientation(&self.draw, self.index, orientation::Update::Radians(v));
        self
    }

    /// Specify the orientation along each axis with the given **Vector** of degrees.
    ///
    /// This has the same affect as calling `self.x_degrees(v.x).y_degrees(v.y).z_degrees(v.z)`.
    pub fn degrees(self, v: Vec3) -> Self {
        self.radians(Vec3::new(
            v.x.to_radians(),
            v.y.to_radians(),
            v.z.to_radians(),
        ))
    }

    /// Specify the orientation along each axis with the given **Vector** of "turns".
    ///
    /// This has the same affect as calling `self.x_turns(v.x).y_turns(v.y).z_turns(v.z)`.
    pub fn turns(self, v: Vec3) -> Self {
        self.radians(v * std::f32::consts::TAU)
    }

    /// Specify the orientation with the given **Euler**.
    ///
    /// The euler must be specified in radians.
    pub fn euler(self, e: Vec3) -> Self {
        self.radians(e)
    }

    /// Specify the orientation with the given **Quaternion**.
    pub fn quaternion(self, q: Quat) -> Self {
        orientation::set_orientation(&self.draw, self.index, orientation::Update::Quat(q));
        self
    }

    // Higher level methods.

    /// Specify the "pitch" of the orientation in radians.
    ///
    /// This has the same effect as calling `x_radians`.
    pub fn pitch(self, pitch: f32) -> Self {
        self.x_radians(pitch)
    }

    /// Specify the "yaw" of the orientation in radians.
    ///
    /// This has the same effect as calling `y_radians`.
    pub fn yaw(self, yaw: f32) -> Self {
        self.y_radians(yaw)
    }

    /// Specify the "roll" of the orientation in radians.
    ///
    /// This has the same effect as calling `z_radians`.
    pub fn roll(self, roll: f32) -> Self {
        self.z_radians(roll)
    }

    /// Assuming we're looking at a 2D plane, positive values cause a clockwise rotation where the
    /// given value is specified in radians.
    ///
    /// This is equivalent to calling the `z_radians` or `roll` methods.
    pub fn rotate(self, radians: f32) -> Self {
        self.z_radians(radians)
    }
}

// SetFill methods

impl<'a, T> Drawing<'a, T>
where
    T: SetFill,
{
    /// Specify the whole set of fill tessellation options.
    pub fn fill_opts(self, opts: FillOptions) -> Self {
        fill::set_fill(&self.draw, self.index, fill::Update::Opts(opts));
        self
    }

    /// Maximum allowed distance to the path when building an approximation.
    pub fn fill_tolerance(self, tolerance: f32) -> Self {
        fill::set_fill(&self.draw, self.index, fill::Update::Tolerance(tolerance));
        self
    }

    /// Specify the rule used to determine what is inside and what is outside of the shape.
    ///
    /// Currently, only the `EvenOdd` rule is implemented.
    pub fn fill_rule(self, rule: lyon::tessellation::FillRule) -> Self {
        fill::set_fill(&self.draw, self.index, fill::Update::Rule(rule));
        self
    }

    /// Whether to perform a vertical or horizontal traversal of the geometry.
    ///
    /// Default value: `Vertical`.
    pub fn fill_sweep_orientation(self, orientation: lyon::tessellation::Orientation) -> Self {
        fill::set_fill(
            &self.draw,
            self.index,
            fill::Update::SweepOrientation(orientation),
        );
        self
    }

    /// A fast path to avoid some expensive operations if the path is known to not have any
    /// self-intersections.
    ///
    /// Do not set this to `false` if the path may have intersecting edges else the tessellator may
    /// panic or produce incorrect results. In doubt, do not change the default value.
    ///
    /// Default value: `true`.
    pub fn handle_intersections(self, b: bool) -> Self {
        fill::set_fill(&self.draw, self.index, fill::Update::HandleIntersections(b));
        self
    }
}

// SetStroke methods

impl<'a, T> Drawing<'a, T>
where
    T: SetStroke,
{
    /// The start line cap as specified by the SVG spec.
    pub fn start_cap(self, cap: LineCap) -> Self {
        stroke::set_stroke(&self.draw, self.index, stroke::Update::StartCap(cap));
        self
    }

    /// The end line cap as specified by the SVG spec.
    pub fn end_cap(self, cap: LineCap) -> Self {
        stroke::set_stroke(&self.draw, self.index, stroke::Update::EndCap(cap));
        self
    }

    /// The start and end line cap as specified by the SVG spec.
    pub fn caps(self, cap: LineCap) -> Self {
        stroke::set_stroke(&self.draw, self.index, stroke::Update::Caps(cap));
        self
    }

    /// The stroke for each sub-path does not extend beyond its two endpoints. A zero length
    /// sub-path will therefore not have any stroke.
    pub fn start_cap_butt(self) -> Self {
        self.start_cap(LineCap::Butt)
    }

    /// At the end of each sub-path, the shape representing the stroke will be extended by a
    /// rectangle with the same width as the stroke width and whose length is half of the stroke
    /// width. If a sub-path has zero length, then the resulting effect is that the stroke for that
    /// sub-path consists solely of a square with side length equal to the stroke width, centered
    /// at the sub-path's point.
    pub fn start_cap_square(self) -> Self {
        self.start_cap(LineCap::Square)
    }

    /// At each end of each sub-path, the shape representing the stroke will be extended by a half
    /// circle with a radius equal to the stroke width. If a sub-path has zero length, then the
    /// resulting effect is that the stroke for that sub-path consists solely of a full circle
    /// centered at the sub-path's point.
    pub fn start_cap_round(self) -> Self {
        self.start_cap(LineCap::Round)
    }

    /// The stroke for each sub-path does not extend beyond its two endpoints. A zero length
    /// sub-path will therefore not have any stroke.
    pub fn end_cap_butt(self) -> Self {
        self.end_cap(LineCap::Butt)
    }

    /// At the end of each sub-path, the shape representing the stroke will be extended by a
    /// rectangle with the same width as the stroke width and whose length is half of the stroke
    /// width. If a sub-path has zero length, then the resulting effect is that the stroke for that
    /// sub-path consists solely of a square with side length equal to the stroke width, centered
    /// at the sub-path's point.
    pub fn end_cap_square(self) -> Self {
        self.end_cap(LineCap::Square)
    }

    /// At each end of each sub-path, the shape representing the stroke will be extended by a half
    /// circle with a radius equal to the stroke width. If a sub-path has zero length, then the
    /// resulting effect is that the stroke for that sub-path consists solely of a full circle
    /// centered at the sub-path's point.
    pub fn end_cap_round(self) -> Self {
        self.end_cap(LineCap::Round)
    }

    /// The stroke for each sub-path does not extend beyond its two endpoints. A zero length
    /// sub-path will therefore not have any stroke.
    pub fn caps_butt(self) -> Self {
        self.caps(LineCap::Butt)
    }

    /// At the end of each sub-path, the shape representing the stroke will be extended by a
    /// rectangle with the same width as the stroke width and whose length is half of the stroke
    /// width. If a sub-path has zero length, then the resulting effect is that the stroke for that
    /// sub-path consists solely of a square with side length equal to the stroke width, centered
    /// at the sub-path's point.
    pub fn caps_square(self) -> Self {
        self.caps(LineCap::Square)
    }

    /// At each end of each sub-path, the shape representing the stroke will be extended by a half
    /// circle with a radius equal to the stroke width. If a sub-path has zero length, then the
    /// resulting effect is that the stroke for that sub-path consists solely of a full circle
    /// centered at the sub-path's point.
    pub fn caps_round(self) -> Self {
        self.caps(LineCap::Round)
    }

    /// The way in which lines are joined at the vertices, matching the SVG spec.
    ///
    /// Default value is `MiterClip`.
    pub fn join(self, join: LineJoin) -> Self {
        stroke::set_stroke(&self.draw, self.index, stroke::Update::Join(join));
        self
    }

    /// A sharp corner is to be used to join path segments.
    pub fn join_miter(self) -> Self {
        self.join(LineJoin::Miter)
    }

    /// Same as a `join_miter`, but if the miter limit is exceeded, the miter is clipped at a miter
    /// length equal to the miter limit value multiplied by the stroke width.
    pub fn join_miter_clip(self) -> Self {
        self.join(LineJoin::MiterClip)
    }

    /// A round corner is to be used to join path segments.
    pub fn join_round(self) -> Self {
        self.join(LineJoin::Round)
    }

    /// A bevelled corner is to be used to join path segments. The bevel shape is a triangle that
    /// fills the area between the two stroked segments.
    pub fn join_bevel(self) -> Self {
        self.join(LineJoin::Bevel)
    }

    /// The total stroke_weight (aka width) of the line.
    pub fn stroke_weight(self, stroke_weight: f32) -> Self {
        stroke::set_stroke(
            &self.draw,
            self.index,
            stroke::Update::Weight(stroke_weight),
        );
        self
    }

    /// Describes the limit before miter lines will clip, as described in the SVG spec.
    ///
    /// Must be greater than or equal to `1.0`.
    pub fn miter_limit(self, limit: f32) -> Self {
        stroke::set_stroke(&self.draw, self.index, stroke::Update::MiterLimit(limit));
        self
    }

    /// Maximum allowed distance to the path when building an approximation.
    pub fn stroke_tolerance(self, tolerance: f32) -> Self {
        stroke::set_stroke(&self.draw, self.index, stroke::Update::Tolerance(tolerance));
        self
    }

    /// Specify the full set of stroke options for the path tessellation.
    pub fn stroke_opts(self, opts: StrokeOptions) -> Self {
        stroke::set_stroke(&self.draw, self.index, stroke::Update::Opts(opts));
        self
    }
}

impl<'a, T> Drawing<'a, T> {
    /// Set the base color of the shader model used to draw this primitive.
    ///
    /// This only applies to the default nannou shader model; if a custom shader model is active,
    /// a warning is emitted and the color is ignored.
    pub fn base_color<C: Into<Color>>(self, color: C) -> Self {
        let color = color.into();
        self.map_shader_model(|mut model: DefaultNannouShaderModel| {
            model.color = color;
            model
        })
    }

    /// Set the texture of the shader model used to draw this primitive.
    ///
    /// The texture is applied via [`ShaderModel::set_texture`], so this works with any shader
    /// model type. Models without a texture slot ignore it.
    pub fn texture(self, texture: &Handle<Image>) -> Self {
        let mut model = {
            let state = self.draw.state.read().unwrap();
            state.shader_models[&self.draw.shader_model].clone_erased()
        };
        model.set_texture_erased(texture.clone());
        self.with_new_shader_model(model)
    }
}
// Finish the drawing at the given index.
//
// Shared between the **finish** method and the **Drawing**'s **Drop** implementation.
fn finish_drawing(draw: &DrawRef, index: usize, shader_model_index: usize) {
    match draw.state.try_write() {
        Err(err) => eprintln!("drawing failed to borrow state and finish: {}", err),
        Ok(mut state) => {
            // If we are "Owned", that means we mutated our shader model and so need to
            // spawn a new entity just for this primitive.
            if let DrawRef::Owned(draw) = draw {
                let id = draw.shader_model.clone();
                let shader_model_cmd = state
                    .draw_commands
                    .get_mut(shader_model_index)
                    .expect("expected a valid shader model index");
                if shader_model_cmd.is_none() {
                    *shader_model_cmd = Some(DrawCommand::ShaderModel(id));
                }
            }
            state.finish_drawing(index);
        }
    }
}

// Mutate the primitive stored within **Draw** at `index` in place.
//
// The function is only applied if the node has not yet been **Drawn** and the draw state is not
// locked elsewhere.
pub(crate) fn with_primitive(draw: &Draw, index: usize, f: impl FnOnce(&mut Primitive)) {
    if let Ok(mut state) = draw.state.try_write() {
        if let Some(primitive) = state.drawing.get_mut(&index) {
            f(primitive);
        }
    }
}

// The same as `with_primitive`, but passes ownership of the primitive to the given function
// along with mutable access to the intermediary vertex buffers. This is useful for type-state
// transitions and for primitives with an arbitrary number of vertices.
pub(crate) fn with_primitive_ctxt(
    draw: &Draw,
    index: usize,
    f: impl FnOnce(Primitive, DrawingContext) -> Primitive,
) {
    if let Ok(mut state) = draw.state.try_write() {
        let state = &mut *state;
        if let Some(primitive) = state.drawing.get_mut(&index) {
            let prim = std::mem::take(primitive);
            let new = {
                let mut intermediary_state = state.intermediary_state.write().unwrap();
                let ctxt = DrawingContext::from_intermediary_state(&mut intermediary_state);
                f(prim, ctxt)
            };
            *primitive = new;
        }
    }
}
