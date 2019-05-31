use nannou::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// This must match the number of colours per
// pixel.
// RGBA = 4
// RGB = 3
// RG = 2 etc.
pub const NUM_COLOURS: usize = 4;

struct ScreenShot {
    render_pass: Arc<vk::RenderPassAbstract + Send + Sync>,
    pipeline_sample: Arc<vk::GraphicsPipelineAbstract + Send + Sync>,
    pipeline_record: Arc<vk::GraphicsPipelineAbstract + Send + Sync>,
    frame_buffer: Arc<vk::FramebufferAbstract + Send + Sync>,
    screenshot_buffer: Arc<vk::CpuAccessibleBuffer<[[u8; NUM_COLOURS]]>>,
    vertex_buffer: Arc<vk::ImmutableBuffer<[Vertex]>>,
    input_color: Arc<vk::AttachmentImage>,
    output_color: Arc<vk::AttachmentImage>,
}

// Hack to get around wait issue
pub struct FrameLock {
    num_shots: AtomicUsize,
    level: AtomicUsize,
    shots: [ScreenShot; 3],
}

#[derive(Debug, Clone)]
struct Vertex {
    position: [f32; 2],
}

vk::impl_vertex!(Vertex, position);

pub fn new(app: &App, dims: (usize, usize)) -> FrameLock {
    FrameLock {
        num_shots: AtomicUsize::new(0),
        level: AtomicUsize::new(0),
        shots: [
            ScreenShot::new(app, dims),
            ScreenShot::new(app, dims),
            ScreenShot::new(app, dims),
        ],
    }
}

impl FrameLock {
    pub fn take(&self, frame: &Frame) {
        if self.num_shots.load(Ordering::SeqCst) > 0 {
            self.num_shots.fetch_sub(1, Ordering::SeqCst);

            match self.level.load(Ordering::SeqCst) {
                0 => {
                    self.shots[1].take(frame);
                }
                1 => {
                    self.shots[2].take(frame);
                }
                2 => {
                    self.shots[0].take(frame);
                }
                _ => unreachable!(),
            }
        }
    }
    pub fn copy_frame(&self, frame: &Frame) {
        if self.num_shots.load(Ordering::SeqCst) > 0 {
            match self.level.load(Ordering::SeqCst) {
                0 => {
                    self.level.store(1, Ordering::SeqCst);
                    self.shots[0].copy_frame(frame);
                }
                1 => {
                    self.level.store(2, Ordering::SeqCst);
                    self.shots[1].copy_frame(frame);
                }
                2 => {
                    self.level.store(0, Ordering::SeqCst);
                    self.shots[2].copy_frame(frame);
                }
                _ => unreachable!(),
            }
        }
    }
}

impl ScreenShot {
    fn new(app: &App, dims: (usize, usize)) -> Self {
        let device = app.main_window().swapchain().device().clone();

        let buf = vec![[0u8; NUM_COLOURS]; dims.0 * dims.1];
        let screenshot_buffer = vk::CpuAccessibleBuffer::from_iter(
            device.clone(),
            vk::BufferUsage {
                storage_texel_buffer: true,
                ..vk::BufferUsage::none()
            },
            buf.into_iter(),
        )
        .expect("Failed to create screenshot buffer");

        let vertex_shader_sample = vs_sample::Shader::load(device.clone()).unwrap();
        let fragment_shader_sample = fs_sample::Shader::load(device.clone()).unwrap();
        let vertex_shader_record = vs_record::Shader::load(device.clone()).unwrap();
        let fragment_shader_record = fs_record::Shader::load(device.clone()).unwrap();

        let positions = [[-1.0, -1.0], [-1.0, 1.0], [1.0, -1.0], [1.0, 1.0]];
        let data = positions.iter().map(|&position| Vertex { position });
        let queue = app.main_window().swapchain_queue().clone();
        let usage = vk::BufferUsage::vertex_buffer();
        let (vertex_buffer, buffer_future) =
            vk::ImmutableBuffer::from_iter(data, usage, queue).unwrap();

        buffer_future
            .then_signal_fence_and_flush()
            .expect("failed to signal_fence_and_flush buffer and image creation future")
            .wait(None)
            .expect("failed to wait for buffer and image creation future");

        let render_pass = Arc::new(
            vk::ordered_passes_renderpass!(
                device.clone(),
                attachments: {
                    input_color: {
                        load: Load,
                        store: DontCare,
                        format: app.main_window().swapchain().format(),
                        samples: app.main_window().msaa_samples(),
                    },
                    output_color: {
                        load: DontCare,
                        store: DontCare,
                        format: app.main_window().swapchain().format(),
                        samples: 1,
                    }
                },
                passes: [
                {
                    color: [input_color],
                    depth_stencil: {},
                    input: [],
                    resolve: [output_color]
                },
                {
                    color: [],
                    depth_stencil: {},
                    input: [output_color]
                }
                ]
            )
            .unwrap(),
        );

        let pipeline_sample = Arc::new(
            vk::GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex>()
                .vertex_shader(vertex_shader_sample.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fragment_shader_sample.main_entry_point(), ())
                .render_pass(vk::Subpass::from(render_pass.clone(), 0).unwrap())
                .build(device.clone())
                .unwrap(),
        );

        let pipeline_record = Arc::new(
            vk::GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex>()
                .vertex_shader(vertex_shader_record.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fragment_shader_record.main_entry_point(), ())
                .render_pass(vk::Subpass::from(render_pass.clone(), 1).unwrap())
                .build(device.clone())
                .unwrap(),
        );

        let input_color = vk::AttachmentImage::multisampled_with_usage(
            device.clone(),
            [dims.0 as u32, dims.1 as u32],
            app.main_window().msaa_samples(),
            app.main_window().swapchain().format(),
            vk::ImageUsage {
                transfer_destination: true,
                color_attachment: true,
                ..vk::ImageUsage::none()
            },
        )
        .expect("Failed to create input image");

        let output_color = vk::AttachmentImage::with_usage(
            device.clone(),
            [dims.0 as u32, dims.1 as u32],
            app.main_window().swapchain().format(),
            vk::ImageUsage {
                transient_attachment: true,
                input_attachment: true,
                ..vk::ImageUsage::none()
            },
        )
        .expect("Failed to create input image");

        let frame_buffer = Arc::new(
            vk::Framebuffer::start(render_pass.clone())
                .add(input_color.clone())
                .expect("Failed to add input color to fbo")
                .add(output_color.clone())
                .expect("Failed to add output color to fbo")
                .build()
                .expect("Failed to build fbo"),
        );

        ScreenShot {
            render_pass,
            pipeline_sample,
            pipeline_record,
            frame_buffer,
            screenshot_buffer,
            vertex_buffer,
            input_color,
            output_color,
        }
    }

    fn take(&self, frame: &Frame) {
        let queue = frame.queue().clone();
        let device = queue.device();
        let [w, h] = frame.swapchain_image().dimensions();
        let viewport = vk::ViewportBuilder::new().build([w as _, h as _]);
        let dynamic_state = vk::DynamicState::default().viewports(vec![viewport]);

        /*
        let desciptor_set_sample = Arc::new(
            vk::PersistentDescriptorSet::start(self.pipeline_sample.clone(), 0)
                .add_sampled_image(self.inter_image.clone(), self.sampler.clone())
                .expect("Failed to add frame image")
                .build()
                .expect("Failed to build sampled desciptor set"),
        );
        */
        let buf_view =
            vk::BufferView::new(self.screenshot_buffer.clone(), vk::format::R8G8B8A8Uint)
                .expect("Failed to make buffer view");
        let desciptor_set_record = Arc::new(
            vk::PersistentDescriptorSet::start(self.pipeline_record.clone(), 0)
                .add_image(self.output_color.clone())
                .expect("Failed to add output_color image to descriptor set")
                .add_buffer_view(buf_view)
                .expect("Failed to add screenshot buffer")
                .build()
                .expect("Failed to build record desciptor set"),
        );
        let clear_values = vec![vk::ClearValue::None, vk::ClearValue::None];
        let command_buffer =
            vk::AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family())
                .expect("Failed to create command buffer")
                // TODO this copy is really dumb but I can't get
                // around it without the frame having sampled usage
                .begin_render_pass(self.frame_buffer.clone(), false, clear_values)
                .unwrap()
                .draw(
                    self.pipeline_sample.clone(),
                    &dynamic_state,
                    vec![self.vertex_buffer.clone()],
                    //desciptor_set_sample,
                    (),
                    (),
                )
                .unwrap()
                .next_subpass(false)
                .unwrap()
                .draw(
                    self.pipeline_record.clone(),
                    &dynamic_state,
                    vec![self.vertex_buffer.clone()],
                    desciptor_set_record,
                    (),
                )
                .unwrap()
                .end_render_pass()
                .expect("failed to add `end_render_pass` command")
                .build()
                .expect("Failed to build command buffer");
        vk::sync::now(device.clone())
            .then_execute(queue.clone(), command_buffer)
            .expect("Failed to execute command buffer")
            .then_signal_fence_and_flush()
            .expect("Failed to signal fence")
            .wait(None)
            .expect("Failed to wait on future");
    }

    fn copy_frame(&self, frame: &Frame) {
        let [w, h] = frame.swapchain_image().dimensions();
        frame
            .add_commands()
            .copy_image(
                frame.image().clone(),
                [0, 0, 0],
                0,
                0,
                self.input_color.clone(),
                [0, 0, 0],
                0,
                0,
                [w, h, 1],
                1,
            )
            .expect("Failed to copy image");
    }

    fn save(&self) {}
}

mod vs_sample {
    nannou::vk::shaders::shader! {
    ty: "vertex",
        src: "
#version 450

layout(location = 0) in vec2 position;
//layout(location = 0) out vec2 tex_coords;

void main() {
    gl_Position = vec4(position, 0.0, 0.0);
    //tex_coords = position + vec2(0.5);
}"
    }
}

mod fs_sample {
    nannou::vk::shaders::shader! {
    ty: "fragment",
        src: "
#version 450

layout(location = 0) out vec4 f_color;
//layout(location = 0) in vec2 tex_coords;
//layout(set = 0, binding = 0) uniform sampler2D tex;

void main() {
   //f_color = texture(tex, tex_coords);
   f_color = vec4(0.0);
}"
    }
}

mod vs_record {
    nannou::vk::shaders::shader! {
    ty: "vertex",
        src: "
#version 450

layout(location = 0) in vec2 position;
layout(location = 0) out vec2 tex_coords;

void main() {
    gl_Position = vec4(position, 0.0, 0.0);
    tex_coords = position + vec2(0.5);
}"
    }
}

mod fs_record {
    nannou::vk::shaders::shader! {
    ty: "fragment",
        src: "
#version 450

layout(location = 0) in vec2 tex_coords;
layout(input_attachment_index = 0, set = 0, binding = 0) uniform subpassInput in_color;
layout(set = 0, binding = 1, rgba8) uniform imageBuffer StorageTexelBuffer;

void main() {
    imageStore(StorageTexelBuffer, int(tex_coords.x * tex_coords.y), subpassLoad(in_color).rgba);
}"
    }
}
