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
    pipeline: Arc<vk::ComputePipelineAbstract + Send + Sync>,
    screenshot_buffer: Arc<vk::CpuAccessibleBuffer<[[u8; NUM_COLOURS]]>>,
    output_image: Arc<vk::StorageImage<vk::Format>>,
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
                transfer_destination: true,
                ..vk::BufferUsage::none()
            },
            buf.into_iter(),
        )
        .expect("Failed to create screenshot buffer");

        let compute_shader = cs::Shader::load(device.clone()).unwrap();

        let sampler = vk::SamplerBuilder::new()
            .mipmap_mode(vk::sampler::MipmapMode::Linear)
            .build(device.clone())
            .unwrap();


        let pipeline = Arc::new(
            vk::ComputePipeline::new(device.clone(), &compute_shader.main_entry_point(), &())
                .expect("Faile to create compute pipeline"),
        );

        let output_image = vk::StorageImage::with_usage(
            device.clone(),
            vk::image::Dimensions::Dim2d{ width: dims.0 as u32, height: dims.1 as u32},
            vk::Format::R8G8B8A8Uint,
            vk::ImageUsage {
                transfer_source: true,
                storage: true,
                ..vk::ImageUsage::none()
            },
            device.active_queue_families(),
        )
        .expect("Failed to create output image");

        ScreenShot {
            pipeline,
            screenshot_buffer,
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
        let command_buffer = vk::AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family())
                .expect("Failed to create command buffer")
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


// TODO width and height are hard coded and should be passed in as
// push constants
mod cs {
    nannou::vk::shaders::shader! {
    ty: "compute",
        src: "
#version 450
const uint WIDTH = 1366;
const uint HEIGHT = 600;

layout(local_size_x = 32, local_size_y = 32, local_size_z = 1) in;

layout(set = 0, binding = 0) uniform sampler2D tex;

layout(set = 0, binding = 1, rgba8ui) uniform uimage2D storage_image;

void main() {
    if(gl_GlobalInvocationID.x >= WIDTH || gl_GlobalInvocationID.y >= HEIGHT) {
        return;
    }
    vec4 t = texture(tex, vec2(gl_GlobalInvocationID.x / float(WIDTH), gl_GlobalInvocationID.y / float(HEIGHT)));
    uvec4 col = uvec4(uint(255*t.x), uint(255*t.y), uint(255*t.z), uint(255*t.w));
    imageStore(storage_image, ivec2(gl_GlobalInvocationID.xy), col);
}"
    }
}
