//! A simple API for drawing 2D and 3D graphics.
//!
//! See the [**Draw** type](./struct.Draw.html) for more details.

use crate::geom::{self, Point2};
use crate::glam::{vec3, EulerRot, Mat4, Quat, Vec2, Vec3};
use crate::math::{deg_to_rad, turns_to_rad};
use crate::wgpu;
use lyon::path::PathEvent;
use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::mem;
use std::rc::Rc;

pub use self::background::Background;
pub use self::drawing::{Drawing, DrawingContext};
use self::mesh::vertex::{Color, TexCoords};
pub use self::mesh::Mesh;
use self::primitive::Primitive;
pub use self::renderer::{Builder as RendererBuilder, Renderer};
pub use self::theme::Theme;

pub mod background;
mod drawing;
pub mod mesh;
pub mod primitive;
pub mod properties;
pub mod renderer;
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
pub struct Draw {
    /// The state of the **Draw**.
    ///
    /// State is shared between this **Draw** instance and all other **Draw** instances that were
    /// produced by cloning or changing transform, scissor or blend mode.
    ///
    /// We use a `RefCell` in order to avoid requiring a `mut` handle to a `draw`. The primary
    /// purpose of a **Draw** is to be an easy-as-possible, high-level API for drawing stuff. In
    /// order to be friendlier to new users, we want to avoid them having to think about mutability
    /// and focus on creativity. Rust-lang nuances can come later.
    state: Rc<RefCell<State>>,
    /// The current context of this **Draw** instance.
    context: Context,
}

/// The current **Transform**, alpha **BlendState** and **Scissor** of a **Draw** instance.
#[derive(Clone, Debug, PartialEq)]
pub struct Context {
    pub transform: Mat4,
    pub blend: wgpu::BlendState,
    pub scissor: Scissor,
    // TODO: Consider changing `PolygonMode` (added as of wgpu 0.7) rather than `PrimitiveTopology`
    // here.
    pub topology: wgpu::PrimitiveTopology,
    pub sampler: wgpu::SamplerDescriptor<'static>,
}

/// Commands generated by drawings.
///
/// During rendering, the list of **DrawCommand**s are converted into a list of **RenderCommands**
/// that are directly associated with encodable render pass commands.
#[derive(Clone, Debug)]
pub enum DrawCommand {
    /// Draw a primitive.
    Primitive(Primitive),
    /// A change in the rendering context occurred.
    Context(Context),
}

/// The scissor for a **Draw**'s render context.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Scissor {
    /// The extent of the scissor matches the bounds of the target texture.
    Full,
    /// Crop the view to the given rect.
    Rect(geom::Rect),
    /// The scissor has no overlap with the previous window, resulting in nothing to draw.
    NoOverlap,
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
    /// The last context used to draw an image, used to detect changes and emit commands for them.
    last_draw_context: Option<Context>,
    /// If `Some`, the **Draw** should first clear the frame's texture with the given color.
    background_color: Option<properties::LinSrgba>,
    /// Primitives that are in the process of being drawn.
    ///
    /// Keys are indices into the `draw_commands` Vec.
    drawing: HashMap<usize, Primitive>,
    /// The list of recorded draw commands.
    ///
    /// An element may be `None` if it is a primitive in the process of being drawn.
    draw_commands: Vec<Option<DrawCommand>>,
    /// State made accessible via the `DrawingContext`.
    intermediary_state: RefCell<IntermediaryState>,
    /// The theme containing default values.
    theme: Theme,
}

/// State made accessible via the `DrawingContext`.
#[derive(Clone, Debug)]
pub struct IntermediaryState {
    /// Buffers of vertex data that may be re-used for paths, meshes, etc between view calls.
    intermediary_mesh: Mesh,
    /// A re-usable buffer for collecting path events.
    path_event_buffer: Vec<PathEvent>,
    /// A re-usable buffer for collecting colored polyline points.
    path_points_colored_buffer: Vec<(Point2, Color)>,
    /// A re-usable buffer for collecting textured polyline points.
    path_points_textured_buffer: Vec<(Point2, TexCoords)>,
    /// A buffer containing all text.
    text_buffer: String,
}

impl IntermediaryState {
    pub fn reset(&mut self) {
        self.intermediary_mesh.clear();
        self.path_event_buffer.clear();
        self.path_points_colored_buffer.clear();
        self.path_points_textured_buffer.clear();
        self.text_buffer.clear();
    }

    /// Allows inspecting the intermediary mesh
    pub fn intermediary_mesh(&self) -> &Mesh {
        &self.intermediary_mesh
    }
    /// Allows inspecting the path event buffer
    pub fn path_event_buffer(&self) -> &[PathEvent] {
        &self.path_event_buffer
    }
    /// Allows inspecting the path points colored buffer
    pub fn path_points_colored_buffer(&self) -> &[(Point2, Color)] {
        &self.path_points_colored_buffer
    }
    /// Allows inspecting the path points textured buffer
    pub fn path_points_textured_buffer(&self) -> &[(Point2, TexCoords)] {
        &self.path_points_textured_buffer
    }
    /// Allows inspecting the text buffer
    pub fn text_buffer(&self) -> &str {
        self.text_buffer.as_str()
    }
}

impl State {
    // Resets all state within the `Draw` instance.
    fn reset(&mut self) {
        self.background_color = None;
        self.last_draw_context = None;
        self.drawing.clear();
        self.draw_commands.clear();
        self.intermediary_state.borrow_mut().reset();
    }

    // Drain any remaining `drawing`s and insert them as draw commands.
    fn finish_remaining_drawings(&mut self) {
        let mut drawing = mem::replace(&mut self.drawing, Default::default());
        for (index, primitive) in drawing.drain() {
            self.insert_draw_command(index, primitive);
        }
        mem::swap(&mut self.drawing, &mut drawing);
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
            *elem = Some(DrawCommand::Primitive(prim));
        }
    }

    /// Allows inspecting the intermediary_state.
    pub fn intermediary_state(&self) -> Ref<IntermediaryState> {
        self.intermediary_state.borrow()
    }

    /// Allows inspecting the theme
    pub fn theme(&self) -> &Theme {
        &self.theme
    }
}

impl Draw {
    /// Create a new **Draw** instance.
    ///
    /// This is the same as calling **Draw::default**.
    pub fn new() -> Self {
        Self::default()
    }

    /// Resets all state within the `Draw` instance.
    pub fn reset(&self) {
        self.state.borrow_mut().reset();
    }

    /// Returns a reference to the state
    pub fn state(&self) -> Ref<State> {
        self.state.borrow()
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
        self.scale_axes(vec3(s, s, s))
    }

    /// Produce a new **Draw** instance where the contents are scaled by the given amount across
    /// each axis.
    pub fn scale_axes(&self, v: Vec3) -> Self {
        self.transform(Mat4::from_scale(v))
    }

    /// Produce a new **Draw** instance where the contents are scaled by the given amount along the
    /// x axis
    pub fn scale_x(&self, s: f32) -> Self {
        self.scale_axes(vec3(s, 1.0, 1.0))
    }

    /// Produce a new **Draw** instance where the contents are scaled by the given amount along the
    /// y axis
    pub fn scale_y(&self, s: f32) -> Self {
        self.scale_axes(vec3(1.0, s, 1.0))
    }

    /// Produce a new **Draw** instance where the contents are scaled by the given amount along the
    /// z axis
    pub fn scale_z(&self, s: f32) -> Self {
        self.scale_axes(vec3(1.0, 1.0, s))
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
        self.radians(vec3(x, 0.0, 0.0))
    }

    /// Specify the orientation around the *y* axis in radians.
    pub fn y_radians(&self, y: f32) -> Self {
        self.radians(vec3(0.0, y, 0.0))
    }

    /// Specify the orientation around the *z* axis in radians.
    pub fn z_radians(&self, z: f32) -> Self {
        self.radians(vec3(0.0, 0.0, z))
    }

    /// Specify the orientation along each axis with the given **Vector** of degrees.
    pub fn degrees(&self, v: Vec3) -> Self {
        self.radians(vec3(deg_to_rad(v.x), deg_to_rad(v.y), deg_to_rad(v.z)))
    }

    /// Specify the orientation around the *x* axis in degrees.
    pub fn x_degrees(&self, x: f32) -> Self {
        self.x_radians(deg_to_rad(x))
    }

    /// Specify the orientation around the *y* axis in degrees.
    pub fn y_degrees(&self, y: f32) -> Self {
        self.y_radians(deg_to_rad(y))
    }

    /// Specify the orientation around the *z* axis in degrees.
    pub fn z_degrees(&self, z: f32) -> Self {
        self.z_radians(deg_to_rad(z))
    }

    /// Specify the orientation along each axis with the given **Vector** of degrees.
    pub fn turns(&self, v: Vec3) -> Self {
        self.radians(vec3(
            turns_to_rad(v.x),
            turns_to_rad(v.y),
            turns_to_rad(v.z),
        ))
    }

    /// Specify the orientation around the *x* axis as a number of turns around the axis.
    pub fn x_turns(&self, x: f32) -> Self {
        self.x_radians(turns_to_rad(x))
    }

    /// Specify the orientation around the *y* axis as a number of turns around the axis.
    pub fn y_turns(&self, y: f32) -> Self {
        self.y_radians(turns_to_rad(y))
    }

    /// Specify the orientation around the *z* axis as a number of turns around the axis.
    pub fn z_turns(&self, z: f32) -> Self {
        self.z_radians(turns_to_rad(z))
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

    /// Produce a new **Draw** instance that will be cropped to the given rectangle.
    ///
    /// If the current **Draw** instance already contains a scissor, the result will be the overlap
    /// between the original scissor and the new one.
    pub fn scissor(&self, scissor: geom::Rect<f32>) -> Self {
        let mut context = self.context.clone();
        context.scissor = match context.scissor {
            Scissor::Full => Scissor::Rect(scissor),
            Scissor::Rect(rect) => rect
                .overlap(scissor)
                .map(Scissor::Rect)
                .unwrap_or(Scissor::NoOverlap),
            Scissor::NoOverlap => Scissor::NoOverlap,
        };
        self.context(context)
    }

    /// Produce a new **Draw** instance.
    ///
    /// All drawing that occurs on the new instance will be rendered as a "wireframe" between all
    /// vertices.
    ///
    /// This will cause the **draw::Renderer** to switch render pipelines in order to use the
    /// **LineList** primitive topology. The switch will only occur if this topology was not
    /// already enabled.
    pub fn line_mode(&self) -> Self {
        self.primitive_topology(wgpu::PrimitiveTopology::LineList)
    }

    /// Produce a new **Draw** instance.
    ///
    /// All drawing that occurs on the new instance will be rendered as points on the vertices.
    ///
    /// This will cause the **draw::Renderer** to switch render pipelines in order to use the
    /// **PointList** primitive topology. The switch will only occur if this topology was not
    /// already enabled.
    pub fn point_mode(&self) -> Self {
        self.primitive_topology(wgpu::PrimitiveTopology::PointList)
    }

    /// Produce a new **Draw** instance.
    ///
    /// All drawing that occurs on the new instance will be rendered as triangles on the vertices.
    ///
    /// This will cause the **draw::Renderer** to switch render pipelines in order to use the
    /// **TriangleList** primitive topology. The switch will only occur if this topology was not
    /// already enabled.
    ///
    /// This is the default primitive topology mode.
    pub fn triangle_mode(&self) -> Self {
        self.primitive_topology(wgpu::PrimitiveTopology::TriangleList)
    }

    /// Produce a new **Draw** instance where all textures and textured vertices drawn will be
    /// sampled via a sampler of the given descriptor.
    pub fn sampler(&self, desc: wgpu::SamplerDescriptor<'static>) -> Self {
        let mut context = self.context.clone();
        context.sampler = desc;
        self.context(context)
    }

    /// Specify the primitive topology to use within the render pipeline.
    ///
    /// This method is shared between the `line_mode`, `point_mode` and `triangle_mode` methods.
    fn primitive_topology(&self, topology: wgpu::PrimitiveTopology) -> Self {
        let mut context = self.context.clone();
        context.topology = topology;
        self.context(context)
    }

    /// Produce a new **Draw** instance with the given context.
    fn context(&self, context: Context) -> Self {
        let state = self.state.clone();
        Draw { state, context }
    }

    // Primitives.

    /// Specify a color with which the background should be cleared.
    pub fn background(&self) -> Background {
        background::new(self)
    }

    /// Add the given type to be drawn.
    pub fn a<T>(&self, primitive: T) -> Drawing<T>
    where
        T: Into<Primitive>,
        Primitive: Into<Option<T>>,
    {
        let index = {
            let mut state = self.state.borrow_mut();
            // If drawing with a different context, insert the necessary command to update it.
            if state.last_draw_context.as_ref() != Some(&self.context) {
                state
                    .draw_commands
                    .push(Some(DrawCommand::Context(self.context.clone())));
                state.last_draw_context = Some(self.context.clone());
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
    pub fn path(&self) -> Drawing<primitive::PathInit> {
        self.a(Default::default())
    }

    /// Begin drawing an **Ellipse**.
    pub fn ellipse(&self) -> Drawing<primitive::Ellipse> {
        self.a(Default::default())
    }

    /// Begin drawing a **Line**.
    pub fn line(&self) -> Drawing<primitive::Line> {
        self.a(Default::default())
    }

    /// Begin drawing an **Arrow**.
    pub fn arrow(&self) -> Drawing<primitive::Arrow> {
        self.a(Default::default())
    }

    /// Begin drawing a **Quad**.
    pub fn quad(&self) -> Drawing<primitive::Quad> {
        self.a(Default::default())
    }

    /// Begin drawing a **Rect**.
    pub fn rect(&self) -> Drawing<primitive::Rect> {
        self.a(Default::default())
    }

    /// Begin drawing a **Triangle**.
    pub fn tri(&self) -> Drawing<primitive::Tri> {
        self.a(Default::default())
    }

    /// Begin drawing a **Polygon**.
    pub fn polygon(&self) -> Drawing<primitive::PolygonInit> {
        self.a(Default::default())
    }

    /// Begin drawing a **Mesh**.
    pub fn mesh(&self) -> Drawing<primitive::mesh::Vertexless> {
        self.a(Default::default())
    }

    /// Begin drawing a **Polyline**.
    ///
    /// Note that this is simply short-hand for `draw.path().stroke()`
    pub fn polyline(&self) -> Drawing<primitive::PathStroke> {
        self.path().stroke()
    }

    /// Begin drawing a **Text**.
    pub fn text(&self, s: &str) -> Drawing<primitive::Text> {
        let text = {
            let state = self.state.borrow();
            let mut intermediary_state = state.intermediary_state.borrow_mut();
            let ctxt = DrawingContext::from_intermediary_state(&mut *intermediary_state);
            primitive::text::Text::new(ctxt, s)
        };
        self.a(text)
    }

    /// Begin drawing a **Texture**.
    pub fn texture(&self, view: &dyn wgpu::ToTextureView) -> Drawing<primitive::Texture> {
        self.a(primitive::Texture::new(view))
    }

    /// Finish any drawings-in-progress and produce an iterator draining the inner draw commands
    /// and yielding them by value.
    pub fn drain_commands(&self) -> impl Iterator<Item = DrawCommand> {
        self.finish_remaining_drawings();
        let cmds = {
            let mut state = self.state.borrow_mut();
            let empty = Vec::with_capacity(state.draw_commands.len());
            std::mem::replace(&mut state.draw_commands, empty)
        };
        cmds.into_iter().filter_map(|opt| opt)
    }

    /// Drain any remaining `drawing`s and convert them to draw commands.
    pub fn finish_remaining_drawings(&self) {
        self.state.borrow_mut().finish_remaining_drawings()
    }
}

impl Default for IntermediaryState {
    fn default() -> Self {
        let intermediary_mesh = Default::default();
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
        let intermediary_state = RefCell::new(Default::default());
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

impl Default for Draw {
    fn default() -> Self {
        let state: Rc<RefCell<State>> = Rc::new(RefCell::new(Default::default()));
        let context = Default::default();
        Draw { state, context }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self {
            transform: Mat4::IDENTITY,
            blend: wgpu::BlendState {
                color: wgpu::RenderPipelineBuilder::DEFAULT_COLOR_BLEND,
                alpha: wgpu::RenderPipelineBuilder::DEFAULT_ALPHA_BLEND,
            },
            scissor: Scissor::Full,
            topology: wgpu::RenderPipelineBuilder::DEFAULT_PRIMITIVE_TOPOLOGY,
            sampler: wgpu::SamplerBuilder::new().into_descriptor(),
        }
    }
}
