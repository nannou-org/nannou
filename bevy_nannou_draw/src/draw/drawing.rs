use crate::draw::primitive::Primitive;
use crate::draw::properties::{
    SetColor, SetDimensions, SetFill, SetOrientation, SetPosition, SetStroke,
};
use crate::draw::{self, Draw};
use bevy::prelude::*;
use lyon::path::PathEvent;
use lyon::tessellation::{FillOptions, LineCap, LineJoin, StrokeOptions};
use std::marker::PhantomData;
use bevy::color::palettes::basic::RED;
use bevy::pbr::{ExtendedMaterial, OpaqueRendererMethod};
use bevy::render::render_resource::PrimitiveTopology::TriangleList;
use lyon::lyon_tessellation::{FillTessellator, StrokeTessellator};
use crate::draw::mesh::MeshExt;
use crate::draw::properties::material::SetMaterial;
use crate::draw::render::{GlyphCache, RenderContext, RenderPrimitive};
use crate::render::{NannouMaterial, NannouMesh};

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
pub struct Drawing<'a, 'w, T: Clone, M>
where
    T: Into<Primitive> + Clone,
    M: Material + Default,
{
    // The `Draw` instance used to create this drawing.
    draw: &'a Draw<'w, M>,
    // The entity for this drawing.
    entity: Entity,
    // The primitive for this drawing.
    primitive: T,
    material: M,
    // Whether or not the **Drawing** should attempt to finish the drawing on drop.
    finish_on_drop: bool,
}

/// Some context that may be optionally provided to primitives in the drawing implementation.
///
/// This is particularly useful for paths and meshes.
pub struct DrawingContext<'a> {
    /// The intermediary mesh for buffering yet-to-be-drawn paths and meshes.
    pub mesh: &'a mut Mesh,
    /// A re-usable buffer for collecting path events.
    pub path_event_buffer: &'a mut Vec<PathEvent>,
    /// A re-usable buffer for collecting colored polyline points.
    pub path_points_colored_buffer: &'a mut Vec<(Vec2, Color)>,
    /// A re-usable buffer for collecting textured polyline points.
    pub path_points_textured_buffer: &'a mut Vec<(Vec2, Vec2)>,
    /// A re-usable buffer for collecting text.
    pub text_buffer: &'a mut String,
}

/// Construct a new **Drawing** instance.
pub fn new<'a, 'w, T: Clone, M: Material>(
    draw: &'a Draw<'w, M>,
    entity: Entity,
    primitive: T,
    material: M,
) -> Drawing<'a, 'w, T, M>
where
    T: Into<Primitive> + Clone,
    M: Material + Default,
{
    let finish_on_drop = true;
    Drawing {
        draw,
        entity,
        finish_on_drop,
        primitive,
        material,
    }
}

impl<'a, 'w, T, M> Drop for Drawing<'a, 'w, T, M>
where
    T: Into<Primitive> + Clone,
    M: Material + Default,
{
    fn drop(&mut self) {
        if self.finish_on_drop {
            let draw_state = self.draw.state.write().expect("failed to lock draw state");
            let intermediary_state = draw_state
                .intermediary_state
                .read()
                .expect("failed to lock intermediary state");

            let mut fill_tessellator = FillTessellator::new();
            let mut stroke_tessellator = StrokeTessellator::new();

            let mut glyph_cache = self.draw.world_mut().resource_mut::<GlyphCache>();

            let ctxt = RenderContext {
                intermediary_mesh: &intermediary_state.intermediary_mesh,
                path_event_buffer: &intermediary_state.path_event_buffer,
                path_points_colored_buffer: &intermediary_state.path_points_colored_buffer,
                path_points_textured_buffer: &intermediary_state
                    .path_points_textured_buffer,
                text_buffer: &intermediary_state.text_buffer,
                theme: &draw_state.theme,
                transform: &self.draw.context.transform,
                fill_tessellator: &mut fill_tessellator,
                stroke_tessellator: &mut stroke_tessellator,
                glyph_cache: &mut glyph_cache,
                // TODO: read from window
                output_attachment_size: Vec2::new(100.0, 100.0),
                output_attachment_scale_factor: 1.0,
            };

            let mut mesh = Mesh::init_with_topology(TriangleList);
            let primitive = self.primitive.clone().into();
            primitive.render_primitive(ctxt, &mut mesh);

            let material = self.draw.world_mut()
                .resource_mut::<Assets<M>>()
                .add(self.material.clone());

            mesh = mesh.with_inserted_attribute(Mesh::ATTRIBUTE_UV_0,
                vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]]
            );
            let mesh = self.draw.world_mut()
                .resource_mut::<Assets<Mesh>>()
                .add(mesh);


            self.draw.world_mut()
                .entity_mut(self.entity)
                .insert((
                    NannouMesh,
                    MaterialMeshBundle::<M> {
                        mesh: mesh.clone(),
                        material: material.clone(),
                        transform: Transform::from_translation(Vec3::new(
                            0.0,
                            0.0,
                            0.0,
                        )),
                        ..default()
                    },
                    ))
                .insert(self.draw.context.clone());
        }
    }
}

impl<'a> DrawingContext<'a> {
    // Initialise the DrawingContext from the draw's IntermediaryState.
    pub(crate) fn from_intermediary_state(state: &'a mut super::IntermediaryState) -> Self {
        let super::IntermediaryState {
            ref mut intermediary_mesh,
            ref mut path_event_buffer,
            ref mut path_points_colored_buffer,
            ref mut path_points_textured_buffer,
            ref mut text_buffer,
        } = *state;
        DrawingContext {
            mesh: intermediary_mesh,
            path_event_buffer: path_event_buffer,
            path_points_colored_buffer: path_points_colored_buffer,
            path_points_textured_buffer: path_points_textured_buffer,
            text_buffer: text_buffer,
        }
    }
}

impl <'a, 'w, T, M> Drawing<'a, 'w, T, M>
where
    T: Into<Primitive> + Clone,
    M: Material + Default,
{
    // Shared between the **finish** method and the **Drawing**'s **Drop** implementation.
    //
    // 1. Create vertices based on node-specific position, points, etc.
    // 2. Insert edges into geom graph based on
    fn finish_inner(&mut self) {
        // match self.draw.state.try_write() {
        //     Err(err) => eprintln!("drawing failed to borrow state and finish: {}", err),
        //     Ok(mut state) => state.finish_drawing(self.entity),
        // }
    }

    /// Complete the drawing and insert it into the parent **Draw** instance.
    ///
    /// This will be called when the **Drawing** is **Drop**ped if it has not yet been called.
    pub fn finish(mut self) {
        self.finish_inner()
    }

    pub fn fragment_shader<const FS: &'static str>(self) -> Drawing<'a, 'w, T, ExtendedMaterial<StandardMaterial, NannouMaterial<"", FS>>> {
        let Self {
            draw,
            entity,
            ref primitive,
            ref material,
            ..
        } = self;
        let material = ExtendedMaterial {
            base: StandardMaterial {
                // can be used in forward or deferred mode.
                opaque_render_method: OpaqueRendererMethod::Auto,
                // see the fragment shader `extended_material.wgsl` for more info.
                // Note: to run in deferred mode, you must also add a `DeferredPrepass` component to the camera and either
                // change the above to `OpaqueRendererMethod::Deferred` or add the `DefaultOpaqueRendererMethod` resource.
                // base_color: RED.into(),
                ..Default::default()
            },
            extension: NannouMaterial::<"", FS>::default(),
        };

        Drawing::<'a, 'w, T, ExtendedMaterial<StandardMaterial, NannouMaterial<"", FS>>> {
            draw: draw.material(),
            entity,
            primitive: primitive.clone(),
            material,
            finish_on_drop: true,
        }
    }

    // Map the given function onto the primitive stored within **Draw** at `index`.
    //
    // The functionn is only applied if the node has not yet been **Drawn**.
    fn map_primitive<F, T2>(mut self, map: F) -> Drawing<'a, 'w, T2, M>
    where
        F: FnOnce(Primitive) -> T2,
        T2: Into<Primitive> + Clone,
    {
        let Drawing {
            draw,
            entity,
            ref material,
            ref primitive,
            ..
        } = self;
        let primitive = map(primitive.clone().into());
        self.finish_on_drop = false;

        Drawing {
            draw,
            entity,
            primitive,
            material: material.clone(),
            finish_on_drop: true,
        }
    }

    // The same as `map_primitive` but also passes a mutable reference to the vertex data to the
    // map function. This is useful for types that may have an unknown number of arbitrary
    // vertices.
    fn map_primitive_with_context<F, T2>(mut self, map: F) -> Drawing<'a, 'w, T2, M>
    where
        F: FnOnce(Primitive, DrawingContext) -> T2,
        T2: Into<Primitive> + Clone,
    {
        let mut state = self.draw.state.write().unwrap();
        let mut intermediary_state = state
            .intermediary_state
            .write()
            .expect("intermediary state lock poisoned");
        let ctxt = DrawingContext::from_intermediary_state(&mut intermediary_state);
        let primitive = map(self.primitive.clone().into(), ctxt);
        self.finish_on_drop = false;
        let Drawing {
            draw,
            entity,
            ref material,
            ..
        } = self;
        Drawing {
            draw,
            entity,
            primitive,
            material: material.clone(),
            finish_on_drop: true,
        }
    }

    /// Apply the given function to the type stored within **Draw**.
    ///
    /// The function is only applied if the node has not yet been **Drawn**.
    ///
    /// **Panics** if the primitive does not contain type **T**.
    pub fn map_ty<F, T2>(self, map: F) -> Drawing<'a, 'w, T2, M>
    where
        F: FnOnce(T) -> T2,
        T2: Into<Primitive> + Clone,
        Primitive: Into<Option<T>>,
    {
        self.map_primitive(|prim| {
            let maybe_ty: Option<T> = prim.into();
            let ty = maybe_ty.expect("expected `T` but primitive contained different type");
            let ty2 = map(ty);
            ty2
        })
    }

    /// Apply the given function to the type stored within **Draw**.
    ///
    /// The function is only applied if the node has not yet been **Drawn**.
    ///
    /// **Panics** if the primitive does not contain type **T**.
    pub(crate) fn map_ty_with_context<F, T2>(self, map: F) -> Drawing<'a, 'w, T2, M>
    where
        F: FnOnce(T, DrawingContext) -> T2,
        T2: Into<Primitive> + Clone,
        Primitive: Into<Option<T>>,
    {
        self.map_primitive_with_context(|prim, ctxt| {
            let maybe_ty: Option<T> = prim.into();
            let ty = maybe_ty.expect("expected `T` but primitive contained different type");
            let ty2 = map(ty, ctxt);
            ty2
        })
    }
}

// SetColor implementations.

impl<'a, 'w, T, M> Drawing<'a, 'w, T, M>
where
    T: SetColor + Into<Primitive> + Clone,
    M: Material + Default,
    Primitive: Into<Option<T>>,
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
        self.map_ty(|ty| SetColor::color(ty, color))
    }

    /// Specify the color via red, green and blue channels.
    pub fn rgb(self, r: f32, g: f32, b: f32) -> Self {
        self.map_ty(|ty| SetColor::rgb(ty, r, g, b))
    }

    /// Specify the color via red, green and blue channels as bytes
    pub fn rgb8(self, r: u8, g: u8, b: u8) -> Self {
        self.map_ty(|ty| SetColor::rgb8(ty, r, g, b))
    }

    /// Specify the color via red, green, blue and alpha channels.
    pub fn rgba(self, r: f32, g: f32, b: f32, a: f32) -> Self {
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
    pub fn hsl(self, h: f32, s: f32, l: f32) -> Self {
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
    pub fn hsla(self, h: f32, s: f32, l: f32, a: f32) -> Self {
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
    pub fn hsv(self, h: f32, s: f32, v: f32) -> Self {
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
    pub fn hsva(self, h: f32, s: f32, v: f32, a: f32) -> Self {
        self.map_ty(|ty| SetColor::hsva(ty, h, s, v, a))
    }

    /// Specify the color as gray scale
    ///
    /// The given g expects a value between `0.0` and `1.0` where `0.0` is black and `1.0` is white
    pub fn gray(self, g: f32) -> Self {
        self.map_ty(|ty| SetColor::gray(ty, g))
    }
}

// SetDimensions implementations.

impl<'a, 'w, T, M> Drawing<'a, 'w, T, M>
where
    T: SetDimensions + Into<Primitive> + Clone,
    M: Material + Default,
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

impl<'a, 'w, T, M> Drawing<'a, 'w, T, M>
where
    T: SetPosition + Into<Primitive> + Clone,
    M: Material + Default,
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
    pub fn xy(self, p: Vec2) -> Self {
        self.map_ty(|ty| SetPosition::xy(ty, p))
    }

    /// Set the **Position** with some three-dimensional point.
    pub fn xyz(self, p: Vec3) -> Self {
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

impl<'a, 'w, T, M> Drawing<'a, 'w, T, M>
where
    T: SetOrientation + Into<Primitive> + Clone,
    M: Material + Default,
    Primitive: Into<Option<T>>,
{
    /// Describe orientation via the vector that points to the given target.
    pub fn look_at(self, target: Vec3) -> Self {
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

impl<'a, 'w, T, M> Drawing<'a, 'w, T, M>
where
    T: SetFill + Into<Primitive> + Clone,
    M: Material + Default,
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

impl<'a, 'w, T, M> Drawing<'a, 'w, T, M>
where
    T: SetStroke + Into<Primitive> + Clone,
    M: Material + Default,
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


impl<'a, 'w, T, M> Drawing<'a, 'w, T, M>
    where
        T: SetMaterial<M> + Into<Primitive> + Clone,
        M: Material + Default,
        Primitive: Into<Option<T>>,
{

}