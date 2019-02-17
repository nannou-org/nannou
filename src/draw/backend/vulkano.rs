//! The `vulkano` backend for rendering the contents of a **Draw**'s mesh.

use draw;
use frame::{Frame, ViewFbo};
use math::{BaseFloat, NumCast};
use std::error::Error as StdError;
use std::fmt;
use std::sync::Arc;
use vk::{self, DeviceOwned, DynamicStateBuilder, GpuFuture, RenderPassDesc};

/// A type used for rendering a **nannou::draw::Mesh** with a vulkan graphics pipeline.
pub struct Renderer {
    render_pass: Arc<vk::RenderPassAbstract + Send + Sync>,
    graphics_pipeline: Arc<vk::GraphicsPipelineAbstract + Send + Sync>,
    vertices: Vec<Vertex>,
    render_pass_images: Option<RenderPassImages>,
    view_fbo: ViewFbo,
}

/// The `Vertex` type passed to the vertex shader.
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    // /// The mode with which the `Vertex` will be drawn within the fragment shader.
    // ///
    // /// `0` for rendering text.
    // /// `1` for rendering an image.
    // /// `2` for rendering non-textured 2D geometry.
    // ///
    // /// If any other value is given, the fragment shader will not output any color.
    // pub mode: u32,
    /// The position of the vertex within vector space.
    ///
    /// [-1.0, 1.0, 0.0] is the leftmost, bottom position of the display.
    /// [1.0, -1.0, 0.0] is the rightmost, top position of the display.
    pub position: [f32; 3],
    /// A color associated with the `Vertex`.
    ///
    /// The way that the color is used depends on the `mode`.
    pub color: [f32; 4],
    /// The coordinates of the texture used by this `Vertex`.
    ///
    /// [0.0, 0.0] is the leftmost, bottom position of the texture.
    /// [1.0, 1.0] is the rightmost, top position of the texture.
    pub tex_coords: [f32; 2],
}

// /// Draw text from the text cache texture `tex` in the fragment shader.
// pub const MODE_TEXT: u32 = 0;
// /// Draw an image from the texture at `tex` in the fragment shader.
// pub const MODE_IMAGE: u32 = 1;
// /// Ignore `tex` and draw simple, colored 2D geometry.
// pub const MODE_GEOMETRY: u32 = 2;

// The images used within the `Draw` render pass.
struct RenderPassImages {
    depth: Arc<vk::AttachmentImage>,
}

/// Errors that might occur while building the **Renderer**.
#[derive(Debug)]
pub enum RendererCreationError {
    RenderPass(vk::RenderPassCreationError),
    GraphicsPipeline(GraphicsPipelineError),
}

/// Errors that might occur while building the **vk::GraphicsPipeline**.
#[derive(Debug)]
pub enum GraphicsPipelineError {
    Creation(vk::GraphicsPipelineCreationError),
    VertexShaderLoad(vk::OomError),
    FragmentShaderLoad(vk::OomError),
}

/// Errors that might occur while drawing to a framebuffer.
#[derive(Debug)]
pub enum DrawError {
    RenderPassCreation(vk::RenderPassCreationError),
    GraphicsPipelineCreation(GraphicsPipelineError),
    BufferCreation(vk::memory::DeviceMemoryAllocError),
    ImageCreation(vk::ImageCreationError),
    FramebufferCreation(vk::FramebufferCreationError),
    BeginRenderPass(vk::command_buffer::BeginRenderPassError),
    DrawIndexed(vk::command_buffer::DrawIndexedError),
}

mod vertex_impl {
    use super::Vertex;
    impl_vertex!(Vertex, position, color, tex_coords);
}

mod vs {
    crate::vk::shaders::shader!{
        ty: "vertex",
        src: "
#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 color;
layout(location = 2) in vec2 tex_coords;
// in uint mode;

layout(location = 0) out vec4 v_color;
layout(location = 1) out vec2 v_tex_coords;
// flat out uint v_mode;

void main() {
    gl_Position = vec4(position, 1.0);
    v_color = color;
    v_tex_coords = tex_coords;
    // v_mode = mode;
}
"
    }
}

mod fs {
    crate::vk::shaders::shader!{
        ty: "fragment",
        src: "
#version 450
// uniform sampler2D tex;

layout(location = 0) in vec4 v_color;
layout(location = 1) in vec2 v_tex_coords;
// flat in uint v_mode;

layout(location = 0) out vec4 f_color;

void main() {
    // // Text
    // if (v_mode == uint(0)) {
    //     f_color = v_color * vec4(1.0, 1.0, 1.0, texture(tex, v_tex_coords).r);

    // // Image
    // } else if (v_mode == uint(1)) {
    //     f_color = texture(tex, v_tex_coords);

    // // 2D Geometry
    // } else if (v_mode == uint(2)) {
    //     f_color = v_color;
    // }

    f_color = v_color;
}
"
    }
}

impl Vertex {
    /// Create a vertex from the given mesh vertex.
    pub fn from_mesh_vertex<S>(
        v: draw::mesh::Vertex<S>,
        framebuffer_width: f32,
        framebuffer_height: f32,
        dpi_factor: f32,
    ) -> Self
    where
        S: BaseFloat,
    {
        let point = v.point();
        let x_f: f32 = NumCast::from(point.x).unwrap();
        let y_f: f32 = NumCast::from(point.y).unwrap();
        let z_f: f32 = NumCast::from(point.z).unwrap();
        // Map coords from (-fb_dim, +fb_dim) to (-1.0, 1.0)
        // In vulkan, *y* increases in the downwards direction, so we negate it.
        let x = 2.0 * x_f * dpi_factor / framebuffer_width;
        let y = -(2.0 * y_f * dpi_factor / framebuffer_height);
        let z = 2.0 * z_f * dpi_factor / framebuffer_height;
        let tex_x = NumCast::from(v.tex_coords.x).unwrap();
        let tex_y = NumCast::from(v.tex_coords.y).unwrap();
        let position = [x, y, z];
        let color = [v.color.red, v.color.green, v.color.blue, v.color.alpha];
        let tex_coords = [tex_x, tex_y];
        Vertex {
            position,
            color,
            tex_coords,
        }
    }
}

impl RenderPassImages {
    // Create all the necessary images for the `RenderPass` besides the swapchain image which will
    // be provided by the `Frame`.
    fn new(
        device: Arc<vk::Device>,
        dimensions: [u32; 2],
        depth_format: vk::Format,
        msaa_samples: u32,
    ) -> Result<Self, vk::ImageCreationError> {
        let depth = vk::AttachmentImage::transient_multisampled(
            device,
            dimensions,
            msaa_samples,
            depth_format,
        )?;
        Ok(RenderPassImages {
            depth,
        })
    }
}

impl Renderer {
    /// Create the `Renderer`.
    ///
    /// This creates the `RenderPass` and `vk::GraphicsPipeline` ready for drawing.
    pub fn new(
        device: Arc<vk::Device>,
        color_format: vk::Format,
        depth_format: vk::Format,
        msaa_samples: u32,
    ) -> Result<Self, RendererCreationError> {
        let load_op = vk::LoadOp::Load;
        let render_pass = Arc::new(
            create_render_pass(device, color_format, depth_format, load_op, msaa_samples)?
        ) as Arc<vk::RenderPassAbstract + Send + Sync>;
        let graphics_pipeline = create_graphics_pipeline(render_pass.clone())?
            as Arc<vk::GraphicsPipelineAbstract + Send + Sync>;
        let vertices = vec![];
        let render_pass_images = None;
        let view_fbo = ViewFbo::default();
        Ok(Renderer {
            render_pass,
            graphics_pipeline,
            vertices,
            render_pass_images,
            view_fbo,
        })
    }

    /// Draw the given mesh to the given frame.
    ///
    /// TODO: Make this generic over any "framebuffer" type.
    pub fn draw_to_frame<S>(
        &mut self,
        draw: &draw::Draw<S>,
        dpi_factor: f32,
        frame: &Frame,
        depth_format: vk::Format,
    ) -> Result<(), DrawError>
    where
        S: BaseFloat,
    {
        let Renderer {
            ref mut render_pass,
            ref mut graphics_pipeline,
            ref mut vertices,
            ref mut render_pass_images,
            ref mut view_fbo,
        } = *self;

        // Retrieve the color/depth image load op and clear values based on the bg color.
        let bg_color = draw.state.borrow().background_color;
        let (load_op, clear_color, clear_depth) = match bg_color {
            None => (vk::LoadOp::Load, vk::ClearValue::None, vk::ClearValue::None),
            Some(color) => {
                let clear_color = [color.red, color.green, color.blue, color.alpha].into();
                let clear_depth = 1f32.into();
                (vk::LoadOp::Clear, clear_color, clear_depth)
            },
        };

        // Ensure that the render pass has the correct load op. If not, recreate it.
        let recreate_render_pass = render_pass
            .attachment_descs()
            .next()
            .map(|desc| desc.load != load_op)
            .unwrap_or(true);

        let device = frame.queue().device().clone();
        let color_format = frame.image_format();
        let msaa_samples = frame.image_msaa_samples();

        // If necessary, recreate the render pass and in turn the graphics pipeline.
        if recreate_render_pass {
            *render_pass = create_render_pass(
                device.clone(),
                color_format,
                depth_format,
                load_op,
                msaa_samples,
            )?;
            *graphics_pipeline = create_graphics_pipeline(render_pass.clone())?;
        }

        // Prepare clear values.
        let clear_values = vec![
            clear_color,
            clear_depth,
        ];

        let image_dims = frame.image().dimensions();
        let [img_w, img_h] = image_dims;
        let queue = frame.queue().clone();

        // Create the vertex and index buffers.
        let map_vertex = |v| Vertex::from_mesh_vertex(v, img_w as _, img_h as _, dpi_factor);
        vertices.extend(draw.raw_vertices().map(map_vertex));
        let (vertex_buffer, vb_future) = vk::ImmutableBuffer::from_iter(
            vertices.drain(..),
            vk::BufferUsage::vertex_buffer(),
            queue.clone(),
        )?;
        let (index_buffer, ib_future) = vk::ImmutableBuffer::from_iter(
            draw.inner_mesh().indices().iter().map(|&u| u as u32),
            vk::BufferUsage::index_buffer(),
            queue.clone(),
        )?;

        // Create (or recreate) the render pass images if necessary.
        let recreate_images = render_pass_images
            .as_ref()
            .map(|imgs| image_dims != imgs.depth.dimensions())
            .unwrap_or(true);
        if recreate_images {
            *render_pass_images = Some(RenderPassImages::new(
                device.clone(),
                image_dims,
                depth_format,
                msaa_samples,
            )?);
        }

        // Safe to `unwrap` here as we have ensured that `render_pass_images` is `Some` above.
        let render_pass_images = render_pass_images.as_mut().expect("render_pass_images is `None`");

        // Ensure framebuffers are up to date with the frame's swapchain image and render pass.
        view_fbo.update(&frame, render_pass.clone(), |builder, image| {
            builder
                .add(image)?
                .add(render_pass_images.depth.clone())
        }).unwrap();

        // Create the dynamic state.
        let dynamic_state = dynamic_state([img_w as _, img_h as _]);

        vb_future
            .join(ib_future)
            .then_signal_fence_and_flush()
            .expect("`then_signal_fence_and_flush` failed")
            .wait(None)
            .expect("failed to wait for `vb` and `ib` futures");

        // Submit the draw commands.
        frame
            .add_commands()
            .begin_render_pass(view_fbo.expect_inner(), false, clear_values)?
            .draw_indexed(
                graphics_pipeline.clone(),
                &dynamic_state,
                vec![vertex_buffer],
                index_buffer,
                (),
                (),
            )?
            .end_render_pass()
            .expect("failed to add `end_render_pass` command");

        Ok(())
    }
}

/// The render pass used for the graphics pipeline.
pub fn create_render_pass(
    device: Arc<vk::Device>,
    color_format: vk::Format,
    depth_format: vk::Format,
    load_op: vk::LoadOp,
    msaa_samples: u32,
) -> Result<Arc<vk::RenderPassAbstract + Send + Sync>, vk::RenderPassCreationError> {
    // TODO: Remove this in favour of a nannou-specific, dynamic `RenderPassDesc` implementation.
    match load_op {
        vk::LoadOp::Clear => {
            create_render_pass_clear(device, color_format, depth_format, msaa_samples)
        }
        vk::LoadOp::Load => {
            create_render_pass_load(device, color_format, depth_format, msaa_samples)
        }
        vk::LoadOp::DontCare => unreachable!(),
    }
}

/// Create a render pass that uses `vk::LoadOp::Clear` for the multisampled color and depth
/// attachments.
pub fn create_render_pass_clear(
    device: Arc<vk::Device>,
    color_format: vk::Format,
    depth_format: vk::Format,
    msaa_samples: u32,
) -> Result<Arc<vk::RenderPassAbstract + Send + Sync>, vk::RenderPassCreationError> {
    let rp = single_pass_renderpass!(
        device,
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: color_format,
                samples: msaa_samples,
            },
            depth: {
                load: Clear,
                store: Store,
                format: depth_format,
                samples: msaa_samples,
                initial_layout: ImageLayout::Undefined,
                final_layout: ImageLayout::DepthStencilAttachmentOptimal,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {depth}
        }
    )?;
    Ok(Arc::new(rp))
}

/// Create a render pass that uses `vk::LoadOp::Clear` for the multisampled color and depth
/// attachments.
pub fn create_render_pass_load(
    device: Arc<vk::Device>,
    color_format: vk::Format,
    depth_format: vk::Format,
    msaa_samples: u32,
) -> Result<Arc<vk::RenderPassAbstract + Send + Sync>, vk::RenderPassCreationError> {
    let rp = single_pass_renderpass!(
        device,
        attachments: {
            color: {
                load: Load,
                store: Store,
                format: color_format,
                samples: msaa_samples,
            },
            depth: {
                load: Load,
                store: Store,
                format: depth_format,
                samples: msaa_samples,
                initial_layout: ImageLayout::Undefined,
                final_layout: ImageLayout::DepthStencilAttachmentOptimal,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {depth}
        }
    )?;
    Ok(Arc::new(rp))
}

/// The dynamic state for the renderer.
pub fn dynamic_state(viewport_dimensions: [f32; 2]) -> vk::DynamicState {
    let viewport = vk::ViewportBuilder::new().build(viewport_dimensions);
    vk::DynamicState::default().viewports(vec![viewport])
}

/// The graphics pipeline used by the renderer.
pub fn create_graphics_pipeline<R>(
    render_pass: R,
) -> Result<Arc<vk::GraphicsPipelineAbstract + Send + Sync>, GraphicsPipelineError>
where
    R: vk::RenderPassAbstract + Send + Sync + 'static,
{
    let device = render_pass.device().clone();
    let vs = vs::Shader::load(device.clone()).map_err(GraphicsPipelineError::VertexShaderLoad)?;
    let fs = fs::Shader::load(device.clone()).map_err(GraphicsPipelineError::FragmentShaderLoad)?;
    let subpass = vk::Subpass::from(render_pass, 0).expect("no subpass for `id`");
    let pipeline = vk::GraphicsPipeline::start()
        //.sample_shading_enabled(1.0)
        .vertex_input_single_buffer::<Vertex>()
        .vertex_shader(vs.main_entry_point(), ())
        .triangle_list()
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs.main_entry_point(), ())
        .blend_alpha_blending()
        .render_pass(subpass)
        .build(device)?;
    Ok(Arc::new(pipeline))
}

// Error Implementations

impl From<vk::RenderPassCreationError> for RendererCreationError {
    fn from(err: vk::RenderPassCreationError) -> Self {
        RendererCreationError::RenderPass(err)
    }
}

impl From<GraphicsPipelineError> for RendererCreationError {
    fn from(err: GraphicsPipelineError) -> Self {
        RendererCreationError::GraphicsPipeline(err)
    }
}

impl From<vk::GraphicsPipelineCreationError> for GraphicsPipelineError {
    fn from(err: vk::GraphicsPipelineCreationError) -> Self {
        GraphicsPipelineError::Creation(err)
    }
}

impl From<vk::RenderPassCreationError> for DrawError {
    fn from(err: vk::RenderPassCreationError) -> Self {
        DrawError::RenderPassCreation(err)
    }
}

impl From<GraphicsPipelineError> for DrawError {
    fn from(err: GraphicsPipelineError) -> Self {
        DrawError::GraphicsPipelineCreation(err)
    }
}

impl From<vk::memory::DeviceMemoryAllocError> for DrawError {
    fn from(err: vk::memory::DeviceMemoryAllocError) -> Self {
        DrawError::BufferCreation(err)
    }
}

impl From<vk::ImageCreationError> for DrawError {
    fn from(err: vk::ImageCreationError) -> Self {
        DrawError::ImageCreation(err)
    }
}

impl From<vk::FramebufferCreationError> for DrawError {
    fn from(err: vk::FramebufferCreationError) -> Self {
        DrawError::FramebufferCreation(err)
    }
}

impl From<vk::command_buffer::BeginRenderPassError> for DrawError {
    fn from(err: vk::command_buffer::BeginRenderPassError) -> Self {
        DrawError::BeginRenderPass(err)
    }
}

impl From<vk::command_buffer::DrawIndexedError> for DrawError {
    fn from(err: vk::command_buffer::DrawIndexedError) -> Self {
        DrawError::DrawIndexed(err)
    }
}

impl StdError for RendererCreationError {
    fn description(&self) -> &str {
        match *self {
            RendererCreationError::RenderPass(ref err) => err.description(),
            RendererCreationError::GraphicsPipeline(ref err) => err.description(),
        }
    }
}

impl StdError for GraphicsPipelineError {
    fn description(&self) -> &str {
        match *self {
            GraphicsPipelineError::Creation(ref err) => err.description(),
            GraphicsPipelineError::VertexShaderLoad(ref err) => err.description(),
            GraphicsPipelineError::FragmentShaderLoad(ref err) => err.description(),
        }
    }
}

impl StdError for DrawError {
    fn description(&self) -> &str {
        match *self {
            DrawError::RenderPassCreation(ref err) => err.description(),
            DrawError::GraphicsPipelineCreation(ref err) => err.description(),
            DrawError::BufferCreation(ref err) => err.description(),
            DrawError::ImageCreation(ref err) => err.description(),
            DrawError::FramebufferCreation(ref err) => err.description(),
            DrawError::BeginRenderPass(ref err) => err.description(),
            DrawError::DrawIndexed(ref err) => err.description(),
        }
    }
}

impl fmt::Display for RendererCreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl fmt::Display for GraphicsPipelineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl fmt::Display for DrawError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl fmt::Debug for Renderer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Renderer ( render_pass, graphics_pipeline, framebuffer: {} )",
            if self.view_fbo.is_some() { "Some" } else { "None" },
        )
    }
}
