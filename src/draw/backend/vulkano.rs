//! The `vulkano` backend for rendering the contents of a **Draw**'s mesh.

use draw;
use frame::Frame;
use math::{BaseFloat, NumCast};
use std::error::Error as StdError;
use std::fmt;
use std::sync::Arc;
use vulkano;
use vulkano::buffer::{BufferUsage, ImmutableBuffer};
use vulkano::command_buffer::{BeginRenderPassError, DrawIndexedError, DynamicState};
use vulkano::device::{Device, DeviceOwned};
use vulkano::format::{ClearValue, Format};
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract,
                           RenderPassCreationError, Subpass};
use vulkano::image::attachment::AttachmentImage;
use vulkano::image::ImageCreationError;
use vulkano::instance::PhysicalDevice;
use vulkano::memory::DeviceMemoryAllocError;
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::pipeline::viewport::Viewport;
use window::SwapchainImage;

/// A type used for rendering a **nannou::draw::Mesh** with a vulkan graphics pipeline.
pub struct Renderer {
    render_pass: Arc<RenderPassAbstract + Send + Sync>,
    graphics_pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
    framebuffers: Vec<Arc<FramebufferAbstract + Send + Sync>>,
    vertices: Vec<Vertex>,
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

/// Errors that might occur while building the **Renderer**.
#[derive(Debug)]
pub enum RendererCreationError {
    RenderPass(RenderPassCreationError),
    GraphicsPipeline(GraphicsPipelineError),
}

/// Errors that might occur while building the **GraphicsPipeline**.
#[derive(Debug)]
pub enum GraphicsPipelineError {
    Creation(vulkano::pipeline::GraphicsPipelineCreationError),
    VertexShaderLoad(vulkano::OomError),
    FragmentShaderLoad(vulkano::OomError),
}

/// Errors that might occur while drawing to a framebuffer.
#[derive(Debug)]
pub enum FramebufferCreationError {
    FramebufferCreation(vulkano::framebuffer::FramebufferCreationError),
    ImageCreation(ImageCreationError),
}

/// Errors that might occur while drawing to a framebuffer.
#[derive(Debug)]
pub enum DrawError {
    BufferCreation(DeviceMemoryAllocError),
    FramebufferCreation(FramebufferCreationError),
    BeginRenderPass(BeginRenderPassError),
    DrawIndexed(DrawIndexedError),
}

mod vertex_impl {
    use super::Vertex;
    impl_vertex!(Vertex, position, color, tex_coords);
}

mod vs {
    vulkano_shaders::shader!{
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
    vulkano_shaders::shader!{
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

/// The render pass used for the graphics pipeline.
pub fn render_pass(
    device: Arc<Device>,
    format: Format,
    msaa_samples: u32,
) -> Result<Arc<RenderPassAbstract + Send + Sync>, RenderPassCreationError> {
    let rp = single_pass_renderpass!(
        device,
        attachments: {
            // The msaa intermediary image.
            msaa: {
                load: Clear,
                store: DontCare,
                format: format,
                samples: msaa_samples,
            },
            // The final image that will be used for the swapchain.
            color: {
                load: DontCare,
                store: Store,
                format: format,
                samples: 1,
            }
        },
        pass: {
            // We use the attachment named `color` as the one and only color attachment.
            color: [msaa],
            // No depth-stencil attachment is indicated with empty brackets.
            depth_stencil: {}
            // Resolve the msaa image to the final color image.
            resolve: [color],
        }
    )?;
    Ok(Arc::new(rp))
}

/// The dynamic state for the renderer.
pub fn dynamic_state(viewport_dimensions: [f32; 2]) -> DynamicState {
    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: viewport_dimensions,
        depth_range: 0.0 .. 1.0,
    };
    let viewports = Some(vec![viewport]);
    let line_width = None;
    let scissors = None;
    DynamicState { line_width, viewports, scissors }
}

/// The graphics pipeline used by the renderer.
pub fn graphics_pipeline<R>(
    render_pass: R,
) -> Result<Arc<GraphicsPipelineAbstract + Send + Sync>, GraphicsPipelineError>
where
    R: RenderPassAbstract + Send + Sync + 'static,
{
    let device = render_pass.device().clone();
    let vs = vs::Shader::load(device.clone()).map_err(GraphicsPipelineError::VertexShaderLoad)?;
    let fs = fs::Shader::load(device.clone()).map_err(GraphicsPipelineError::FragmentShaderLoad)?;
    let subpass = Subpass::from(render_pass, 0).expect("no subpass for `id`");
    let pipeline = GraphicsPipeline::start()
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

/// Determine the number of samples to use for MSAA.
///
/// The target is 4, but we fall back to the limit if its lower.
pub fn msaa_samples(physical_device: &PhysicalDevice) -> u32 {
    const TARGET_SAMPLES: u32 = 4;
    let color = physical_device.limits().framebuffer_color_sample_counts();
    let depth = physical_device.limits().framebuffer_depth_sample_counts();
    let limit = std::cmp::min(color, depth);
    std::cmp::min(limit, TARGET_SAMPLES)
}

impl Renderer {
    /// Create the `Renderer`.
    ///
    /// This creates the `RenderPass` and `GraphicsPipeline` ready for drawing.
    pub fn new(device: Arc<Device>, format: Format) -> Result<Self, RendererCreationError> {
        let msaa_samples = msaa_samples(&device.physical_device());
        let render_pass = Arc::new(render_pass(device, format, msaa_samples)?)
            as Arc<RenderPassAbstract + Send + Sync>;
        let graphics_pipeline = Arc::new(graphics_pipeline(render_pass.clone())?)
            as Arc<GraphicsPipelineAbstract + Send + Sync>;
        let framebuffers = vec![];
        let vertices = vec![];
        Ok(Renderer { render_pass, graphics_pipeline, framebuffers, vertices })
    }

    /// Draw the given mesh to the given frame.
    ///
    /// TODO: Make this generic over any "framebuffer" type.
    pub fn draw_to_frame<S>(
        &mut self,
        draw: &draw::Draw<S>,
        dpi_factor: f32,
        frame: &Frame,
    ) -> Result<(), DrawError>
    where
        S: BaseFloat,
    {
        let clear_value = draw
            .state
            .borrow()
            .background_color
            .map(|c| [c.red, c.green, c.blue, c.alpha])
            .unwrap_or([0.2, 0.2, 0.2, 1.0]);

        let clear_msaa = clear_value.into();
        let clear_color = ClearValue::None;
        let clear_values = vec![clear_msaa, clear_color];

        let [w, h] = frame.swapchain_image().dimensions();
        let device = frame.swapchain_image().swapchain().device().clone();
        let queue = frame.queue().clone();

        // Create the vertex and index buffers.
        let map_vertex = |v| Vertex::from_mesh_vertex(v, w as _, h as _, dpi_factor);
        self.vertices.extend(draw.raw_vertices().map(map_vertex));
        let (vertex_buffer, _vb_future) = ImmutableBuffer::from_iter(
            self.vertices.drain(..),
            BufferUsage::all(),
            queue.clone(),
        )?;
        let (index_buffer, _ib_future) = ImmutableBuffer::from_iter(
            draw.inner_mesh().indices().iter().map(|&u| u as u32),
            BufferUsage::all(),
            queue.clone(),
        )?;

        // Create the dynamic state.
        let dynamic_state = dynamic_state([w as _, h as _]);

        // Create the framebuffer for the image.
        fn create_framebuffer(
            render_pass: Arc<RenderPassAbstract + Send + Sync>,
            swapchain_image: Arc<SwapchainImage>,
            msaa_samples: u32,
        ) -> Result<Arc<FramebufferAbstract + Send + Sync>, FramebufferCreationError> {
            let device = swapchain_image.swapchain().device().clone();
            let dimensions = swapchain_image.dimensions();
            let format = swapchain_image.swapchain().format();
            let msaa_image = AttachmentImage::transient_multisampled(
                device,
                dimensions,
                msaa_samples,
                format,
            )?;
            let fb = Framebuffer::start(render_pass)
                .add(msaa_image)?
                .add(swapchain_image)?
                .build()?;
            Ok(Arc::new(fb) as _)
        }

        // Update the framebuffers if necessary.
        while frame.swapchain_image_index() >= self.framebuffers.len() {
            let fb = create_framebuffer(
                self.render_pass.clone(),
                frame.swapchain_image().clone(),
                msaa_samples(&device.physical_device()),
            )?;
            self.framebuffers.push(Arc::new(fb));
        }

        // If the dimensions for the current framebuffer do not match, recreate it.
        {
            let fb = &mut self.framebuffers[frame.swapchain_image_index()];
            let [fb_w, fb_h, _] = fb.dimensions();
            if fb_w != w || fb_h != h {
                let new_fb = create_framebuffer(
                    self.render_pass.clone(),
                    frame.swapchain_image().clone(),
                    msaa_samples(&device.physical_device()),
                )?;
                *fb = Arc::new(new_fb);
            }
        }

        // Submit the draw commands.
        frame
            .add_commands()
            .begin_render_pass(
                self.framebuffers[frame.swapchain_image_index()].clone(),
                false,
                clear_values,
            )?
            .draw_indexed(
                self.graphics_pipeline.clone(),
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

impl From<RenderPassCreationError> for RendererCreationError {
    fn from(err: RenderPassCreationError) -> Self {
        RendererCreationError::RenderPass(err)
    }
}

impl From<GraphicsPipelineError> for RendererCreationError {
    fn from(err: GraphicsPipelineError) -> Self {
        RendererCreationError::GraphicsPipeline(err)
    }
}

impl From<vulkano::pipeline::GraphicsPipelineCreationError> for GraphicsPipelineError {
    fn from(err: vulkano::pipeline::GraphicsPipelineCreationError) -> Self {
        GraphicsPipelineError::Creation(err)
    }
}

impl From<vulkano::framebuffer::FramebufferCreationError> for FramebufferCreationError {
    fn from(err: vulkano::framebuffer::FramebufferCreationError) -> Self {
        FramebufferCreationError::FramebufferCreation(err)
    }
}

impl From<ImageCreationError> for FramebufferCreationError {
    fn from(err: ImageCreationError) -> Self {
        FramebufferCreationError::ImageCreation(err)
    }
}

impl From<DeviceMemoryAllocError> for DrawError {
    fn from(err: DeviceMemoryAllocError) -> Self {
        DrawError::BufferCreation(err)
    }
}

impl From<FramebufferCreationError> for DrawError {
    fn from(err: FramebufferCreationError) -> Self {
        DrawError::FramebufferCreation(err)
    }
}

impl From<BeginRenderPassError> for DrawError {
    fn from(err: BeginRenderPassError) -> Self {
        DrawError::BeginRenderPass(err)
    }
}

impl From<DrawIndexedError> for DrawError {
    fn from(err: DrawIndexedError) -> Self {
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

impl StdError for FramebufferCreationError {
    fn description(&self) -> &str {
        match *self {
            FramebufferCreationError::FramebufferCreation(ref err) => err.description(),
            FramebufferCreationError::ImageCreation(ref err) => err.description(),
        }
    }
}

impl StdError for DrawError {
    fn description(&self) -> &str {
        match *self {
            DrawError::BufferCreation(ref err) => err.description(),
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

impl fmt::Display for FramebufferCreationError {
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
            "Renderer ( render_pass, graphics_pipeline, framebuffers: {} )",
            self.framebuffers.len(),
        )
    }
}
