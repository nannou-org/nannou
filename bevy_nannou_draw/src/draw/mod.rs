//! A simple API for drawing 2D and 3D graphics.
//!
//! See the [**Draw** type](./struct.Draw.html) for more details.

use bevy::ecs::world::Command;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, RwLock};

pub use self::background::Background;
pub use self::drawing::{Drawing, DrawingContext};
use self::primitive::Primitive;
pub use self::theme::Theme;
use crate::changed::Cd;
use crate::draw::mesh::MeshExt;
use crate::draw::render::{GlyphCache, RenderContext, RenderPrimitive};
use crate::render::{DefaultNannouMaterial, NannouMesh, NannouRender};
use bevy::prelude::*;
use bevy::render::render_resource as wgpu;
use bevy::utils::HashMap;
use lyon::path::PathEvent;
use lyon::tessellation::{FillTessellator, StrokeTessellator};

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
#[derive(Clone)]
pub struct Draw<M = DefaultNannouMaterial>
where
    M: Material + Default,
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

    /// The current context of this **Draw** instance.
    context: DrawContext,
    /// The current material of this **Draw** instance.
    material: Arc<RwLock<Cd<M>>>,
    /// The window to which this **Draw** instance is associated.
    window: Entity
}

#[derive(Clone)]
pub enum DrawRef<'a, M>
where
    M: Material + Default,
{
    Borrowed(&'a Draw<M>),
    Owned(Draw<M>),
}

impl<'a, M> Deref for DrawRef<'a, M>
where
    M: Material + Default,
{
    type Target = Draw<M>;

    fn deref(&self) -> &Self::Target {
        match self {
            DrawRef::Borrowed(draw) => *draw,
            DrawRef::Owned(draw) => draw,
        }
    }
}

/// The current **Transform**, alpha **BlendState** and **Scissor** of a **Draw** instance.
#[derive(Clone, Debug, PartialEq)]
pub struct DrawContext {
    // TODO: figure out how to fixup camera via transform
    pub transform: Mat4,
}

impl Default for DrawContext {
    fn default() -> Self {
        Self {
            transform: Mat4::IDENTITY,
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
pub struct State {
    /// The last context used to draw an image, used to detect changes and emit commands for them.
    last_draw_context: Option<DrawContext>,
    /// If `Some`, the **Draw** should first clear the frame's texture with the given color.
    background_color: Option<Color>,
    /// Primitives that are in the process of being drawn.
    ///
    /// Keys are indices into the `draw_commands` Vec.
    drawing: HashMap<usize, Primitive>,
    /// The list of recorded draw commands.
    ///
    /// An element may be `None` if it is a primitive in the process of being drawn.
    pub(crate) draw_commands: Vec<Option<Box<dyn FnOnce(&mut World) + Sync + Send + 'static>>>,
    /// State made accessible via the `DrawingContext`.
    intermediary_state: Arc<RwLock<IntermediaryState>>,
    /// The theme containing default values.
    theme: Theme,
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
        self.drawing.clear();
        self.draw_commands.clear();
        self.intermediary_state.write().unwrap().reset();
    }

    // Drain any remaining `drawing`s and insert them as draw commands.
    fn finish_remaining_drawings(&mut self) {
        let mut drawing = std::mem::replace(&mut self.drawing, Default::default());
        for (index, primitive) in drawing.drain() {
            self.insert_draw_command(index, primitive);
        }
        std::mem::swap(&mut self.drawing, &mut drawing);
    }

    // Finish the drawing at the given node index if it is not yet complete.
    pub(crate) fn finish_drawing(&mut self, index: usize) {
        if let Some(primitive) = self.drawing.remove(&index) {
            self.insert_draw_command(index, primitive);
        }
    }

    // Insert the draw primitive command at the given index.
    fn insert_draw_command(&mut self, index: usize, prim: Primitive) {
        if let Some(elem) = self.draw_commands.get_mut(index) {
            let intermediary_state = self.intermediary_state.clone();
            let theme = self.theme.clone();
            *elem = Some(Box::new(move |world: &mut World| {
                let mut fill_tessellator = FillTessellator::new();
                let mut stroke_tessellator = StrokeTessellator::new();
                let intermediary_state = intermediary_state.read().unwrap();

                world.resource_scope(|world, mut render: Mut<NannouRender>| {
                    world.resource_scope(|world, mut meshes: Mut<Assets<Mesh>>| {
                        let mesh = &render.mesh;
                        let mesh = meshes.get_mut(mesh).unwrap();
                        world.resource_scope(|world, mut glyph_cache: Mut<GlyphCache>| {
                            let ctxt = RenderContext {
                                intermediary_mesh: &intermediary_state.intermediary_mesh,
                                path_event_buffer: &intermediary_state.path_event_buffer,
                                path_points_colored_buffer: &intermediary_state.path_points_colored_buffer,
                                path_points_textured_buffer: &intermediary_state.path_points_textured_buffer,
                                text_buffer: &intermediary_state.text_buffer,
                                theme: &theme,
                                transform: &render.draw_context.transform,
                                fill_tessellator: &mut fill_tessellator,
                                stroke_tessellator: &mut stroke_tessellator,
                                glyph_cache: &mut glyph_cache,
                                // TODO: read from window
                                output_attachment_size: Vec2::new(100.0, 100.0),
                                output_attachment_scale_factor: 1.0,
                            };

                            let primitive: Primitive = prim.into();
                            primitive.render_primitive(ctxt, mesh);
                        })
                    });
                });
            }));
        }
    }
}

impl<M> Draw<M>
where
    M: Material + Default,
{
    pub fn new(window: Entity) -> Self {
        Draw {
            state: Default::default(),
            context: Default::default(),
            material: Arc::new(RwLock::new(Cd::new_true(M::default()))),
            window
        }
    }

    /// Resets all state within the `Draw` instance.
    pub fn reset(&mut self) {
        self.material = Arc::new(RwLock::new(Cd::new_true(M::default())));
        self.state.write().unwrap().reset();
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

    /// Produce a new **Draw** instance with the given context.
    fn context(&self, context: DrawContext) -> Draw<M> {
        let state = self.state.clone();
        let material = self.material.clone();
        let window = self.window;
        Draw {
            state,
            context,
            material,
            window,
        }
    }

    /// Produce a new **Draw** instance with a new material type.
    fn material<M2: Material + Default>(&self) -> Draw<M2> {
        let mut context = self.context.clone();
        let DrawContext { transform, .. } = context;
        let context = DrawContext { transform };
        let state = self.state.clone();
        let window = self.window;
        Draw {
            state,
            context,
            material: Arc::new(RwLock::new(Cd::new_true(M2::default()))),
            window,
        }
    }

    // Primitives.

    /// Specify a color with which the background should be cleared.
    pub fn background<'a>(&'a self) -> Background<'a, M> {
        background::new(self)
    }

    /// Add the given type to be drawn.
    pub fn a<T>(&self, primitive: T) -> Drawing<T, M>
    where
        T: Into<Primitive>,
        Primitive: Into<Option<T>>,
    {
        let index = {
            let mut state = self.state.write().unwrap();
            if self.material.read().unwrap().changed() {
                let material = self.material.read().unwrap().clone();
                state
                    .draw_commands
                    .push(Some(Box::new(move |world: &mut World| {
                        let mut materials = world.resource_mut::<Assets<M>>();
                        let material = materials.add(material);
                        let mut meshes = world.resource_mut::<Assets<Mesh>>();
                        let mesh = meshes.add(Mesh::init());

                        let entity = world.spawn((MaterialMeshBundle {
                            mesh: mesh.clone(),
                            material: material.clone(),
                            ..Default::default()
                        }, NannouMesh)).id();

                        let mut render = world.get_resource_or_insert_with::<NannouRender>(|| {
                            NannouRender {
                                mesh: mesh.clone(),
                                entity: entity,
                                draw_context: DrawContext::default(),
                            }
                        });
                        render.mesh = mesh;
                        render.entity = entity;
                    })));
            }

            // If drawing with a different context, insert the necessary command to update it.
            if state.last_draw_context.as_ref() != Some(&self.context) {
                let context = self.context.clone();
                state
                    .draw_commands
                    .push(Some(Box::new(move |world: &mut World| {
                        let mut render = world.resource_mut::<NannouRender>();
                        render.draw_context = context;
                    })));
            }
            // The primitive will be inserted in the next element.
            let index = state.draw_commands.len();
            let primitive: Primitive = primitive.into();
            state.draw_commands.push(None);
            state.drawing.insert(index, primitive);
            index
        };
        drawing::new(self, index)
    }

    /// Begin drawing a **Path**.
    pub fn path<'a>(&'a self) -> Drawing<'a, primitive::PathInit, M> {
        self.a(Default::default())
    }

    /// Begin drawing an **Ellipse**.
    pub fn ellipse<'a>(&'a self) -> Drawing<'a, primitive::Ellipse, M> {
        self.a(Default::default())
    }

    /// Begin drawing a **Line**.
    pub fn line<'a>(&'a self) -> Drawing<'a, primitive::Line, M> {
        self.a(Default::default())
    }

    /// Begin drawing an **Arrow**.
    pub fn arrow<'a>(&'a self) -> Drawing<'a, primitive::Arrow, M> {
        self.a(Default::default())
    }

    /// Begin drawing a **Quad**.
    pub fn quad<'a>(&'a self) -> Drawing<'a, primitive::Quad, M> {
        self.a(Default::default())
    }

    /// Begin drawing a **Rect**.
    pub fn rect<'a>(&'a self) -> Drawing<'a, primitive::Rect, M> {
        self.a(Default::default())
    }

    /// Begin drawing a **Triangle**.
    pub fn tri<'a>(&'a self) -> Drawing<'a, primitive::Tri, M> {
        self.a(Default::default())
    }

    /// Begin drawing a **Polygon**.
    pub fn polygon<'a>(&'a self) -> Drawing<'a, primitive::PolygonInit, M> {
        self.a(Default::default())
    }

    /// Begin drawing a **Mesh**.
    pub fn mesh<'a>(&'a self) -> Drawing<'a, primitive::mesh::Vertexless, M> {
        self.a(Default::default())
    }

    /// Begin drawing a **Polyline**.
    ///
    /// Note that this is simply short-hand for `draw.path().stroke()`
    pub fn polyline<'a>(&'a self) -> Drawing<'a, primitive::PathStroke, M> {
        self.path().stroke()
    }

    /// Begin drawing a **Text**.
    pub fn text<'a>(&'a self, s: &str) -> Drawing<'a, primitive::Text, M> {
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
    ) -> Drawing<'a, primitive::Texture, M> {
        self.a(primitive::Texture::new(texture_handle, texture))
    }
}

impl Draw<DefaultNannouMaterial> {
    /// Produce a new **Draw** instance that will draw with the given alpha blend descriptor.
    pub fn alpha_blend(&self, blend_descriptor: wgpu::BlendComponent) -> Self {
        // self.material.extension.
        todo!()
    }

    /// Produce a new **Draw** instance that will draw with the given color blend descriptor.
    pub fn color_blend(&self, blend_descriptor: wgpu::BlendComponent) -> Self {
        todo!()
    }

    /// Short-hand for `color_blend`, the common use-case.
    pub fn blend(&self, blend_descriptor: wgpu::BlendComponent) -> Self {
        self.color_blend(blend_descriptor)
    }

    /// Produce a new **Draw** instance that will use the given polygon mode.
    pub fn polygon_mode(&self, polygon_mode: wgpu::PolygonMode) -> Self {
        todo!()
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
        let last_draw_context = None;
        let background_color = Default::default();
        let draw_commands = Default::default();
        let drawing = Default::default();
        let intermediary_state = Arc::new(Default::default());
        let theme = Default::default();
        State {
            last_draw_context,
            draw_commands,
            drawing,
            intermediary_state,
            theme,
            background_color,
        }
    }
}
