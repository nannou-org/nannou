//! A simple API for drawing 2D and 3D graphics.
//!
//! See the [**Draw** type](./struct.Draw.html) for more details.

use bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

pub use self::background::Background;
pub use self::drawing::{Drawing, DrawingContext};
use self::primitive::Primitive;
pub use self::theme::Theme;
use crate::draw::mesh::MeshExt;
use crate::render::DefaultNannouMaterial;
use bevy::prelude::*;
use bevy::render::render_resource as wgpu;
use lyon::path::PathEvent;

pub mod background;
mod drawing;
pub mod mesh;
pub mod primitive;
pub mod properties;
pub mod render;
pub mod theme;

/// A simple API for drawing 2D and 3D graphics.
///
/// **Draw** provides a simple way to compose together geometry and text with custom colours and
/// textures and draw them to the screen. A suite of methods have been provided for drawing
/// polygons, paths, meshes, text and textures in an accessible-yet-efficient manner.
///
/// **Draw** can also be used to create new **Draw** instances that refer to the same inner draw
/// state but are slightly different from one another. E.g. `draw.rotate(radians)` produces a new
/// **Draw** instance where all drawings will be rotated by the given amount. `draw.x(x)` produces
/// a new **Draw** instance where all drawings are translated along the *x* axis by the given
/// amount.
///
/// See the [draw](https://github.com/nannou-org/nannou/blob/master/examples) examples for a
/// variety of demonstrations of how the **Draw** type can be used!
#[derive(Clone, Debug)]
pub struct Draw<'w, M = DefaultNannouMaterial>
    where M: Material + Default
{
    /// The state of the **Draw**.
    ///
    /// State is shared between this **Draw** instance and all other **Draw** instances that were
    /// produced by cloning or changing transform, scissor or blend mode.
    ///
    /// We use a `RefCell` in order to avoid requiring a `mut` handle to a `draw`. The primary
    /// purpose of a **Draw** is to be an easy-as-possible, high-level API for drawing stuff. In
    /// order to be friendlier to new users, we want to avoid them having to think about mutability
    /// and focus on creativity. Rust-lang nuances can come later.
    pub state: Arc<RwLock<State>>,
    pub(crate) world: Rc<RefCell<UnsafeWorldCell<'w>>>,
    /// The current context of this **Draw** instance.
    context: Context,
    /// The window entity to which this **Draw** instance is associated.
    window: Entity,
    /// The material to use for drawing.
    _material: PhantomData<M>,
}

#[derive(Component)]
pub struct BackgroundColor(pub Color);

impl <'w, M> Drop for Draw<'w, M>
    where M: Material + Default
{
    fn drop(&mut self) {
        let state = self.state.read().expect("lock poisoned");
        if let Some(background_color)  = state.background_color {
            self.world_mut()
                .entity_mut(self.window)
                .insert(BackgroundColor(background_color));
        }
    }
}

/// The current **Transform**, alpha **BlendState** and **Scissor** of a **Draw** instance.
#[derive(Component, Clone, Debug, PartialEq)]
pub struct Context {
    // TODO: figure out how to fixup camera via transform
    pub transform: Mat4,
    pub blend: wgpu::BlendState,
    pub polygon_mode: wgpu::PolygonMode,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            transform: Mat4::IDENTITY,
            blend: wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
            },
            polygon_mode: wgpu::PolygonMode::Fill,
        }
    }
}

/// The inner state of the **Draw** type.
///
/// The **Draw** type stores its **State** behind a **RefCell** - a type used for moving mutability
/// checks from compile time to runtime. We do this in order to avoid requiring a `mut` handle to a
/// `draw`. The primary purpose of a **Draw** is to be an easy-as-possible, high-level API for
/// drawing stuff. In order to be friendlier to new users, we want to avoid requiring them to think
/// about mutability and instead focus on creativity. Rust-lang nuances can come later.
#[derive(Clone, Debug)]
pub struct State {
    /// If `Some`, the **Draw** should first clear the frame's texture with the given color.
    pub background_color: Option<Color>,
    /// State made accessible via the `DrawingContext`.
    pub intermediary_state: Arc<RwLock<IntermediaryState>>,
    /// The theme containing default values.
    pub theme: Theme,
    /// The current count of drawings.
    pub count: usize,
}

/// State made accessible via the `DrawingContext`.
#[derive(Clone, Debug)]
pub struct IntermediaryState {
    /// Buffers of vertex data that may be re-used for paths, meshes, etc between view calls.
    pub intermediary_mesh: Mesh,
    /// A re-usable buffer for collecting path events.
    pub path_event_buffer: Vec<PathEvent>,
    /// A re-usable buffer for collecting colored polyline points.
    pub path_points_colored_buffer: Vec<(Vec2, Color)>,
    /// A re-usable buffer for collecting textured polyline points.
    pub path_points_textured_buffer: Vec<(Vec2, Vec2)>,
    /// A buffer containing all text.
    pub text_buffer: String,
}

#[derive(Component, Debug)]
pub struct DrawIndex(pub usize);

impl IntermediaryState {
    pub fn reset(&mut self) {
        self.intermediary_mesh.clear();
        self.path_event_buffer.clear();
        self.path_points_colored_buffer.clear();
        self.path_points_textured_buffer.clear();
        self.text_buffer.clear();
    }
}

impl State {
    // Resets all state within the `Draw` instance.
    fn reset(&mut self) {
        self.background_color = None;
        self.intermediary_state
            .write()
            .expect("lock poisoned")
            .reset();
    }
}

impl<'w, M> Draw<'w, M>
where
    M: Material + Default,
{
    pub fn new(world: Rc<RefCell<UnsafeWorldCell<'w>>>, window: Entity) -> Self {
        Draw {
            state: Default::default(),
            world,
            context: Default::default(),
            window,
            _material: Default::default(),
        }
    }

    /// Resets all state within the `Draw` instance.
    pub fn reset(&self) {
        self.state.write().expect("lock poisoned").reset();
    }

    // Context changes.

    /// Produce a new **Draw** instance transformed by the given transform matrix.
    ///
    /// The resulting **Draw** instance will be have a transform equal to the new transform applied
    /// to the existing transform.
    pub fn transform(&self, transform_matrix: Mat4) -> Self {
        let mut context = self.context.clone();
        context.transform = context.transform * transform_matrix;
        self.context(context)
    }

    /// Translate the position of the origin by the given translation vector.
    pub fn translate(&self, v: Vec3) -> Self {
        self.transform(Mat4::from_translation(v))
    }

    /// Translate the position of the origin by the given translation vector.
    ///
    /// This method is short for `translate`.
    pub fn xyz(&self, v: Vec3) -> Self {
        self.translate(v)
    }

    /// Translate the position of the origin by the given translation vector.
    pub fn xy(&self, v: Vec2) -> Self {
        self.xyz(v.extend(0.0))
    }

    /// Translate the position of the origin by the given amount across each axis.
    pub fn x_y_z(&self, x: f32, y: f32, z: f32) -> Self {
        self.xyz([x, y, z].into())
    }

    /// Translate the position of the origin by the given amount across each axis.
    pub fn x_y(&self, x: f32, y: f32) -> Self {
        self.xy([x, y].into())
    }

    /// Translate the position of the origin along the x axis.
    pub fn x(&self, x: f32) -> Self {
        self.x_y(x, 0.0)
    }

    /// Translate the position of the origin along the y axis.
    pub fn y(&self, y: f32) -> Self {
        self.x_y(0.0, y)
    }

    /// Translate the position of the origin along the z axis.
    pub fn z(&self, z: f32) -> Self {
        self.x_y_z(0.0, 0.0, z)
    }

    /// Produce a new **Draw** instance where the contents are scaled uniformly by the given value.
    pub fn scale(&self, s: f32) -> Self {
        self.scale_axes(Vec3::new(s, s, s))
    }

    /// Produce a new **Draw** instance where the contents are scaled by the given amount across
    /// each axis.
    pub fn scale_axes(&self, v: Vec3) -> Self {
        self.transform(Mat4::from_scale(v))
    }

    /// Produce a new **Draw** instance where the contents are scaled by the given amount along the
    /// x axis
    pub fn scale_x(&self, s: f32) -> Self {
        self.scale_axes(Vec3::new(s, 1.0, 1.0))
    }

    /// Produce a new **Draw** instance where the contents are scaled by the given amount along the
    /// y axis
    pub fn scale_y(&self, s: f32) -> Self {
        self.scale_axes(Vec3::new(1.0, s, 1.0))
    }

    /// Produce a new **Draw** instance where the contents are scaled by the given amount along the
    /// z axis
    pub fn scale_z(&self, s: f32) -> Self {
        self.scale_axes(Vec3::new(1.0, 1.0, s))
    }

    /// The given vector is interpreted as a Euler angle in radians and a transform is applied
    /// accordingly.
    pub fn euler(&self, euler: Vec3) -> Self {
        self.transform(Mat4::from_euler(EulerRot::XYZ, euler.x, euler.y, euler.z))
    }

    /// Specify the orientation with the given **Quaternion**.
    pub fn quaternion(&self, q: Quat) -> Self {
        self.transform(Mat4::from_quat(q))
    }

    /// Specify the orientation along each axis with the given **Vector** of radians.
    ///
    /// This currently has the same affect as calling `euler`.
    pub fn radians(&self, v: Vec3) -> Self {
        self.euler(v)
    }

    /// Specify the orientation around the *x* axis in radians.
    pub fn x_radians(&self, x: f32) -> Self {
        self.radians(Vec3::new(x, 0.0, 0.0))
    }

    /// Specify the orientation around the *y* axis in radians.
    pub fn y_radians(&self, y: f32) -> Self {
        self.radians(Vec3::new(0.0, y, 0.0))
    }

    /// Specify the orientation around the *z* axis in radians.
    pub fn z_radians(&self, z: f32) -> Self {
        self.radians(Vec3::new(0.0, 0.0, z))
    }

    /// Specify the orientation along each axis with the given **Vector** of degrees.
    pub fn degrees(&self, v: Vec3) -> Self {
        self.radians(Vec3::new(
            v.x.to_radians(),
            v.y.to_radians(),
            v.z.to_radians(),
        ))
    }

    /// Specify the orientation around the *x* axis in degrees.
    pub fn x_degrees(&self, x: f32) -> Self {
        self.x_radians(x.to_radians())
    }

    /// Specify the orientation around the *y* axis in degrees.
    pub fn y_degrees(&self, y: f32) -> Self {
        self.y_radians(y.to_radians())
    }

    /// Specify the orientation around the *z* axis in degrees.
    pub fn z_degrees(&self, z: f32) -> Self {
        self.z_radians(z.to_radians())
    }

    /// Specify the orientation along each axis with the given **Vector** of degrees.
    pub fn turns(&self, v: Vec3) -> Self {
        self.radians(v * std::f32::consts::TAU)
    }

    /// Specify the orientation around the *x* axis as a number of turns around the axis.
    pub fn x_turns(&self, x: f32) -> Self {
        self.x_radians(x * std::f32::consts::TAU)
    }

    /// Specify the orientation around the *y* axis as a number of turns around the axis.
    pub fn y_turns(&self, y: f32) -> Self {
        self.y_radians(y * std::f32::consts::TAU)
    }

    /// Specify the orientation around the *z* axis as a number of turns around the axis.
    pub fn z_turns(&self, z: f32) -> Self {
        self.z_radians(z * std::f32::consts::TAU)
    }

    /// Specify the "pitch" of the orientation in radians.
    ///
    /// This has the same effect as calling `x_radians`.
    pub fn pitch(&self, pitch: f32) -> Self {
        self.x_radians(pitch)
    }

    /// Specify the "yaw" of the orientation in radians.
    ///
    /// This has the same effect as calling `y_radians`.
    pub fn yaw(&self, yaw: f32) -> Self {
        self.y_radians(yaw)
    }

    /// Specify the "roll" of the orientation in radians.
    ///
    /// This has the same effect as calling `z_radians`.
    pub fn roll(&self, roll: f32) -> Self {
        self.z_radians(roll)
    }

    /// Assuming we're looking at a 2D plane, positive values cause a clockwise rotation where the
    /// given value is specified in radians.
    ///
    /// This is equivalent to calling the `z_radians` or `roll` methods.
    pub fn rotate(&self, radians: f32) -> Self {
        self.z_radians(radians)
    }

    /// Produce a new **Draw** instance that will draw with the given alpha blend descriptor.
    pub fn alpha_blend(&self, blend_descriptor: wgpu::BlendComponent) -> Self {
        let mut context = self.context.clone();
        context.blend.alpha = blend_descriptor;
        self.context(context)
    }

    /// Produce a new **Draw** instance that will draw with the given color blend descriptor.
    pub fn color_blend(&self, blend_descriptor: wgpu::BlendComponent) -> Self {
        let mut context = self.context.clone();
        context.blend.color = blend_descriptor;
        self.context(context)
    }

    /// Short-hand for `color_blend`, the common use-case.
    pub fn blend(&self, blend_descriptor: wgpu::BlendComponent) -> Self {
        self.color_blend(blend_descriptor)
    }

    /// Produce a new **Draw** instance that will use the given polygon mode.
    pub fn polygon_mode(&self, polygon_mode: wgpu::PolygonMode) -> Self {
        let mut context = self.context.clone();
        context.polygon_mode = polygon_mode;
        self.context(context)
    }

    /// Produce a new **Draw** instance with the given context.
    fn context(&self, context: Context) -> Self {
        let state = self.state.clone();
        Draw {
            state,
            context,
            world: self.world.clone(),
            _material: PhantomData,
            window: self.window,
        }
    }

    /// Produce a new **Draw** instance with a new material type.
    fn material<M2: Material + Default>(&self) -> &Draw<'w, M2> {
        unsafe { std::mem::transmute(self) }
    }

    // Primitives.

    /// Specify a color with which the background should be cleared.
    pub fn background<'a>(&'a self) -> Background<'a, 'w, M> {
        background::new(self)
    }

    /// Add the given type to be drawn.
    pub fn a<'a, T>(&'a self, primitive: T) -> Drawing<'a, 'w, T, M>
    where
        T: Into<Primitive> + Clone,
        Primitive: Into<Option<T>>,
    {
        let mut state = self.state.write().expect("lock poisoned");
        state.count += 1;
        let index = state.count;
        let entity = self.world_mut().spawn(DrawIndex(index)).id();
        drawing::new(self, entity, primitive, M::default())
    }

    /// Begin drawing a **Path**.
    pub fn path<'a>(&'a self) -> Drawing<'a, 'w, primitive::PathInit, M> {
        self.a(Default::default())
    }

    /// Begin drawing an **Ellipse**.
    pub fn ellipse<'a>(&'a self) -> Drawing<'a, 'w, primitive::Ellipse, M> {
        self.a(Default::default())
    }

    /// Begin drawing a **Line**.
    pub fn line<'a>(&'a self) -> Drawing<'a, 'w, primitive::Line, M> {
        self.a(Default::default())
    }

    /// Begin drawing an **Arrow**.
    pub fn arrow<'a>(&'a self) -> Drawing<'a, 'w, primitive::Arrow, M> {
        self.a(Default::default())
    }

    /// Begin drawing a **Quad**.
    pub fn quad<'a>(&'a self) -> Drawing<'a, 'w, primitive::Quad, M> {
        self.a(Default::default())
    }

    /// Begin drawing a **Rect**.
    pub fn rect<'a>(&'a self) -> Drawing<'a, 'w, primitive::Rect, M> {
        self.a(Default::default())
    }

    /// Begin drawing a **Triangle**.
    pub fn tri<'a>(&'a self) -> Drawing<'a, 'w, primitive::Tri, M> {
        self.a(Default::default())
    }

    /// Begin drawing a **Polygon**.
    pub fn polygon<'a>(&'a self) -> Drawing<'a, 'w, primitive::PolygonInit, M> {
        self.a(Default::default())
    }

    /// Begin drawing a **Mesh**.
    pub fn mesh<'a>(&'a self) -> Drawing<'a, 'w, primitive::mesh::Vertexless, M> {
        self.a(Default::default())
    }

    /// Begin drawing a **Polyline**.
    ///
    /// Note that this is simply short-hand for `draw.path().stroke()`
    pub fn polyline<'a>(&'a self) -> Drawing<'a, 'w, primitive::PathStroke, M> {
        self.path().stroke()
    }

    /// Begin drawing a **Text**.
    pub fn text<'a>(&'a self, s: &str) -> Drawing<'a, 'w, primitive::Text, M> {
        let text = {
            let state = self.state.read().expect("lock poisoned");
            let mut intermediary_state = state.intermediary_state.write().expect("lock poisoned");
            let ctxt = DrawingContext::from_intermediary_state(&mut *intermediary_state);
            primitive::text::Text::new(ctxt, s)
        };
        self.a(text)
    }

    /// Begin drawing a **Texture**.
    // TODO: this api sucks, because it requires the user to wait for the texture to load before
    // they can draw it. We should provide a way to draw a texture without waiting for it to load.
    // This is mostly due to the image size being unknown until the texture is loaded.
    pub fn texture<'a>(
        &'a self,
        texture_handle: Handle<Image>,
        texture: Image,
    ) -> Drawing<'a, 'w, primitive::Texture, M> {
        self.a(primitive::Texture::new(texture_handle, texture))
    }

    pub(crate) fn world(&self) -> &World {
        unsafe { self.world.borrow().world() }
    }

    pub(crate) fn world_mut(&self) -> &mut World {
        unsafe { self.world.borrow_mut().world_mut() }
    }
}

impl Default for IntermediaryState {
    fn default() -> Self {
        let intermediary_mesh = Mesh::init();
        let path_event_buffer = Default::default();
        let path_points_colored_buffer = Default::default();
        let path_points_textured_buffer = Default::default();
        let text_buffer = Default::default();
        IntermediaryState {
            intermediary_mesh,
            path_event_buffer,
            path_points_colored_buffer,
            path_points_textured_buffer,
            text_buffer,
        }
    }
}

impl Default for State {
    fn default() -> Self {
        let background_color = Default::default();
        let intermediary_state = Arc::new(RwLock::new(Default::default()));
        let theme = Default::default();
        State {
            intermediary_state,
            theme,
            background_color,
            count: 0,
        }
    }
}
