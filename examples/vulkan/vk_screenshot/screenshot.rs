use nannou::prelude::*;
use std::slice;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::Arc;
use std::thread;

// This must match the number of colours per
// pixel.
// RGBA = 4
// RGB = 3
// RG = 2 etc.
pub const NUM_COLOURS: usize = 4;

type SaveImage = Option<Arc<vk::AttachmentImage>>;

struct ScreenShot {
    //render_pass: Arc<vk::RenderPassAbstract + Send + Sync>,
    pipeline: Arc<vk::ComputePipelineAbstract + Send + Sync>,
    //frame_buffer: Arc<vk::FramebufferAbstract + Send + Sync>,
    screenshot_buffer: Arc<vk::CpuAccessibleBuffer<[[u8; NUM_COLOURS]]>>,
    //vertex_buffer: Arc<vk::ImmutableBuffer<[Vertex]>>,
    output_image: Arc<vk::AttachmentImage>,
    sampler: Arc<vk::Sampler>,
    queue: Arc<vk::Queue>,
    dims: (usize, usize),
    num_images: usize,
}

// Hack to get around wait issue
pub struct Shots {
    num_shots: AtomicUsize,
    images_in: Receiver<SaveImage>,
    images_out: SyncSender<SaveImage>,
}

#[derive(Debug, Clone)]
struct Vertex {
    position: [f32; 2],
}

vk::impl_vertex!(Vertex, position);

pub fn new(app: &App, window_id: WindowId) -> Shots {
    let window = app.window(window_id).expect("Failed to get window");
    let queue = window.swapchain_queue().clone();
    let dims = {
        let d = window.inner_size_pixels();
        (d.0 as usize, d.1 as usize)
    };
    let (save_out, images_in) = mpsc::sync_channel(3);
    let (images_out, save_in) = mpsc::sync_channel(3);
    for _ in 0..2 {
        images_out.send(None).expect("Failed to start out channel");
    }
    for _ in 0..3 {
        let input_image = vk::AttachmentImage::with_usage(
            queue.device().clone(),
            [dims.0 as u32, dims.1 as u32],
            window.swapchain().format(),
            vk::ImageUsage {
                transfer_destination: true,
                sampled: true,
                ..vk::ImageUsage::none()
            },
        )
        .expect("Failed to create input image");
        save_out
            .send(Some(input_image))
            .expect("Failed to send initial images");
    }
    let screenshot = ScreenShot::new(queue, dims);
    thread::spawn({ || save_images(screenshot, save_in, save_out) });
    Shots {
        num_shots: AtomicUsize::new(0),
        images_in,
        images_out,
    }
}

impl Shots {
    pub fn capture(&self, frame: &Frame) {
        if self.num_shots.load(Ordering::SeqCst) > 0 {
            if let Ok(image) = self.images_in.recv() {
                match image {
                    Some(image) => {
                        copy_frame(frame, image.clone());
                        self.images_out.send(Some(image)).ok();
                        self.num_shots.fetch_sub(1, Ordering::SeqCst);
                    }
                    None => {
                        self.images_out.send(None).ok();
                    }
                }
            }
        }
    }
    pub fn take(&self) {
        self.num_shots.fetch_add(1, Ordering::SeqCst);
    }
}

impl ScreenShot {
    fn new(queue: Arc<vk::Queue>, dims: (usize, usize)) -> Self {
        assert!(queue.family().supports_compute());
        let device = queue.device().clone();
        let buf = vec![[0u8; NUM_COLOURS]; dims.0 * dims.1];
        let screenshot_buffer = vk::CpuAccessibleBuffer::from_iter(
            device.clone(),
            vk::BufferUsage {
                storage_buffer: true,
                ..vk::BufferUsage::none()
            },
            buf.into_iter(),
        )
        .expect("Failed to create screenshot buffer");

        let vertex_shader_record = vs_record::Shader::load(device.clone()).unwrap();
        let fragment_shader_record = fs_record::Shader::load(device.clone()).unwrap();
        let compute_shader = cs::Shader::load(device.clone()).unwrap();

        let positions = [[-1.0, -1.0], [-1.0, 1.0], [1.0, -1.0], [1.0, 1.0]];
        let data = positions.iter().map(|&position| Vertex { position });
        let usage = vk::BufferUsage::vertex_buffer();
        let (vertex_buffer, buffer_future) =
            vk::ImmutableBuffer::from_iter(data, usage, queue.clone()).unwrap();

        let sampler = vk::SamplerBuilder::new()
            .mipmap_mode(vk::sampler::MipmapMode::Linear)
            .build(device.clone())
            .unwrap();
        buffer_future
            .then_signal_fence_and_flush()
            .expect("failed to signal_fence_and_flush buffer and image creation future")
            .wait(None)
            .expect("failed to wait for buffer and image creation future");

        /*
        let render_pass = Arc::new(
            vk::single_pass_renderpass!(
                device.clone(),
                attachments: {
                color: {
                    load: DontCare,
                    store: Store,
                    format: vk::Format::R8G8B8A8Uint,
                    samples: 1,
                }
                },
                pass: {
                    color: [color],
                    depth_stencil: {}
                }
            )
            .unwrap(),
        );

        let pipeline = Arc::new(
            vk::GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex>()
                .vertex_shader(vertex_shader_record.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fragment_shader_record.main_entry_point(), ())
                .render_pass(vk::Subpass::from(render_pass.clone(), 0).unwrap())
                .build(device.clone())
                .unwrap(),
        );
        */

        let pipeline = Arc::new(
            vk::ComputePipeline::new(device.clone(), &compute_shader.main_entry_point(), &())
                .expect("Faile to create compute pipeline"),
        );

        let output_image = vk::AttachmentImage::with_usage(
            device.clone(),
            [dims.0 as u32, dims.1 as u32],
            vk::Format::R8G8B8A8Srgb,
            vk::ImageUsage {
                transfer_source: true,
                storage: true,
                ..vk::ImageUsage::none()
            },
        )
        .expect("Failed to create input image");
        /*
        let frame_buffer = Arc::new(
            vk::Framebuffer::start(render_pass.clone())
                .add(output_image.clone())
                .expect("Failed to add uint image")
                .build()
                .expect("Failed to build fbo"),
        );
        */

        ScreenShot {
            //render_pass,
            pipeline,
            //frame_buffer,
            screenshot_buffer,
            //vertex_buffer,
            output_image,
            sampler,
            queue,
            dims,
            num_images: 0,
        }
    }

    fn take(&mut self, frame_capture: Arc<vk::AttachmentImage>) {
        let queue = &self.queue;
        let device = queue.device();
        let [w, h] = frame_capture.dimensions();
        self.dims = (w as usize, h as usize);
        let viewport = vk::ViewportBuilder::new().build([w as _, h as _]);
        let dynamic_state = vk::DynamicState::default().viewports(vec![viewport]);

        /*
        let buf_view =
            vk::BufferView::new(self.screenshot_buffer.clone(), vk::format::R8G8B8A8Uint)
                .expect("Failed to make buffer view");
                */

        // TODO shoudn't be Persistent
        let desciptor_set = Arc::new(
            vk::PersistentDescriptorSet::start(self.pipeline.clone(), 0)
                .add_sampled_image(frame_capture.clone(), self.sampler.clone())
                .expect("Failed to add output_color image to descriptor set")
                .add_image(self.output_image.clone())
                .expect("Failed to add output image")
                //.add_buffer_view(buf_view)
                // .expect("Failed to add screenshot buffer")
                .build()
                .expect("Failed to build record desciptor set"),
        );
        //let clear_values = vec![vk::format::ClearValue::None];
        let command_buffer = vk::AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family())
                .expect("Failed to create command buffer")
                /*
                .begin_render_pass(self.frame_buffer.clone(), false, clear_values)
                .unwrap()
                .draw(
                    self.pipeline.clone(),
                    &dynamic_state,
                    vec![self.vertex_buffer.clone()],
                    desciptor_set_record,
                    (),
                )
                .unwrap()
                .end_render_pass()
                .expect("failed to add `end_render_pass` command")
                .copy_image_to_buffer(self.output_image.clone(), self.screenshot_buffer.clone())
                .expect("Failed to copy image to buffer")
                */
                // TODO this should be the dims from the actual image
                // for this run.
                .dispatch([(self.dims.0 as f32 / 32.0).ceil() as u32, (self.dims.1 as f32 / 32.0).ceil() as u32, 1],
                          self.pipeline.clone(),
                          desciptor_set.clone(),
                          ())
                .expect("Failed to add descriptor set")
                .copy_image_to_buffer(self.output_image.clone(), self.screenshot_buffer.clone())
                .expect("Failed to copy image to buffer")
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

    fn save(&mut self) {
        fn write(
            screenshot_buffer: &[[u8; NUM_COLOURS]],
            screenshot_path: String,
            dims: (usize, usize),
        ) {
            dbg!(&screenshot_buffer[..10]);
            let buf: &[u8] = unsafe {
                slice::from_raw_parts(
                    &screenshot_buffer[0] as *const u8,
                    NUM_COLOURS * dims.0 * dims.1,
                )
            };

            // It is vital that ColorType(bit_depth) matches the
            // type that is used in the screenshot buffer
            nannou::image::save_buffer(
                screenshot_path,
                buf,
                dims.0 as u32,
                dims.1 as u32,
                nannou::image::ColorType::RGBA(8),
            )
            .expect("Failed to save image");
        }
        if let Ok(screenshot_buffer) = self.screenshot_buffer.read() {
            self.num_images += 1;
            let screenshot_path = format!(
                "{}{}",
                env!("CARGO_MANIFEST_DIR"),
                format!("/screenshot{}.png", self.num_images)
            );
            write(&(*screenshot_buffer), screenshot_path, self.dims);
        }
    }
}

fn save_images(
    mut screenshot: ScreenShot,
    save_in: Receiver<SaveImage>,
    save_out: SyncSender<SaveImage>,
) {
    while let Ok(image) = save_in.recv() {
        match image {
            Some(image) => {
                screenshot.take(image.clone());
                screenshot.save();
                save_out.send(Some(image)).ok();
            }
            None => {
                save_out.send(None).ok();
            }
        }
    }
}

fn copy_frame(frame: &Frame, input_color: Arc<vk::AttachmentImage>) {
    let [w, h] = frame.swapchain_image().dimensions();
    frame
        .add_commands()
        .copy_image(
            frame.swapchain_image().clone(),
            [0, 0, 0],
            0,
            0,
            input_color.clone(),
            [0, 0, 0],
            0,
            0,
            [w, h, 1],
            1,
        )
        .expect("Failed to copy image");
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
layout(set = 0, binding = 0) uniform sampler2D tex;
layout(location = 0) out vec4 f_color;

void main() {
    //f_color = texture(tex, tex_coords);
    f_color = vec4(1.0);
}"
    }
}

// TODO might pass in more workgroups then necessary
mod cs {
    nannou::vk::shaders::shader! {
    ty: "compute",
        src: "
#version 450

layout(local_size_x = 32, local_size_y = 32, local_size_z = 1) in;

layout(set = 0, binding = 0) uniform sampler2D tex;

layout(set = 0, binding = 1, rgba8) uniform image2D storage_image;

void main() {
    vec4 t = texture(tex, vec2(gl_LocalInvocationID.x / 32.0, gl_LocalInvocationID.y / 32.0));
    imageStore(storage_image, ivec2(gl_GlobalInvocationID.xy), t);
}"
    }
}
