use nannou::prelude::*;
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::slice;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

// This must match the number of colours per
// pixel.
// RGBA = 4
// RGB = 3
// RG = 2 etc.
pub const NUM_COLOURS: usize = 4;

struct ScreenShot {
    pipeline: Arc<vk::ComputePipelineAbstract + Send + Sync>,
    screenshot_buffer: RefCell<Arc<vk::CpuAccessibleBuffer<[[u8; NUM_COLOURS]]>>>,
    output_image: RefCell<Arc<vk::StorageImage<vk::Format>>>,
    sampler: Arc<vk::Sampler>,
    queue: Arc<vk::Queue>,
    num_images: usize,
}

// Hack to get around wait issue
pub struct Shots {
    num_shots: Cell<usize>,
    frames_since_empty: Cell<usize>,
    images_in: Receiver<Arc<vk::AttachmentImage>>,
    images_out: Sender<Msg>,
    saving_thread: Option<JoinHandle<()>>,
}

enum Msg {
    Buffer(Arc<vk::AttachmentImage>),
    Flush,
    Kill,
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
    let (save_out, images_in) = mpsc::channel();
    let (images_out, save_in) = mpsc::channel();

    for _ in 0..3 {
        let input_image = new_input_image(queue.device().clone(), [dims.0 as u32, dims.1 as u32]);
        save_out
            .send(input_image)
            .expect("Failed to send initial images");
    }
    let screenshot = ScreenShot::new(queue, dims);
    let saving_thread = thread::spawn({ || save_images(screenshot, save_in, save_out) });
    let saving_thread = Some(saving_thread);
    Shots {
        num_shots: Cell::new(0),
        frames_since_empty: Cell::new(3),
        images_in,
        images_out,
        saving_thread,
    }
}

fn save_images(
    mut screenshot: ScreenShot,
    save_in: Receiver<Msg>,
    save_out: Sender<Arc<vk::AttachmentImage>>,
) {
    let mut q = VecDeque::new();
    while let Ok(msg) = save_in.recv() {
        match msg {
            Msg::Buffer(image) => {
                q.push_back(image);
                if q.len() > 2 {
                    let image = q.pop_front().unwrap();
                    screenshot.take(image.clone());
                    screenshot.save(image.dimensions());
                    save_out.send(image).ok();
                }
            }
            Msg::Flush => {
                while let Some(image) = q.pop_front() {
                    screenshot.take(image.clone());
                    screenshot.save(image.dimensions());
                    save_out.send(image).ok();
                }
            }
            Msg::Kill => {
                while let Some(image) = q.pop_front() {
                    screenshot.take(image.clone());
                    screenshot.save(image.dimensions());
                    save_out.send(image).ok();
                }
                return ();
            }
        }
    }
}

impl Shots {
    pub fn capture(&self, frame: &Frame) {
        let num_shots = self.num_shots.get();
        let mut frames_since_empty = self.frames_since_empty.get();
        if num_shots > 0 {
            if let Ok(mut image) = self.images_in.recv() {
                if frame.swapchain_image().dimensions() != image.dimensions() {
                    image = new_input_image(
                        frame.queue().device().clone(),
                        frame.swapchain_image().dimensions(),
                    );
                }
                copy_frame(frame, image.clone());
                self.images_out.send(Msg::Buffer(image)).ok();
                self.num_shots.set(num_shots - 1);
            }
            if num_shots == 1 {
                frames_since_empty = 0;
            }
        }
        if frames_since_empty == 2 {
            self.images_out.send(Msg::Flush).ok();
        }
        self.frames_since_empty.set(frames_since_empty + 1);
    }

    pub fn take(&self) {
        self.num_shots.set(self.num_shots.get() + 1);
    }

    // Call this in the exit function to make sure all images are written
    pub fn flush(mut self, wait: Duration) {
        thread::sleep(wait);
        self.images_out.send(Msg::Kill).ok();
        self.saving_thread.take().map(|t| t.join());
    }
}

impl ScreenShot {
    fn new(queue: Arc<vk::Queue>, dims: (usize, usize)) -> Self {
        assert!(queue.family().supports_compute());
        let device = queue.device().clone();
        let screenshot_buffer = RefCell::new(new_screenshot_buffer(device.clone(), dims));

        let compute_shader = cs::Shader::load(device.clone()).unwrap();

        let sampler = vk::SamplerBuilder::new()
            .mipmap_mode(vk::sampler::MipmapMode::Linear)
            .build(device.clone())
            .unwrap();

        let pipeline = Arc::new(
            vk::ComputePipeline::new(device.clone(), &compute_shader.main_entry_point(), &())
                .expect("Faile to create compute pipeline"),
        );

        let dims2d = vk::image::Dimensions::Dim2d {
            width: dims.0 as u32,
            height: dims.1 as u32,
        };
        let output_image = RefCell::new(new_output_image(device.clone(), dims2d));

        ScreenShot {
            pipeline,
            screenshot_buffer,
            output_image,
            sampler,
            queue,
            num_images: 0,
        }
    }

    fn take(&mut self, frame_capture: Arc<vk::AttachmentImage>) {
        let queue = &self.queue;
        let device = queue.device();
        let [w, h] = frame_capture.dimensions();

        let storage_dims = self.output_image.borrow().dimensions();
        let frame_dims = vk::image::Dimensions::Dim2d {
            width: w,
            height: h,
        };
        if storage_dims != frame_dims {
            self.output_image
                .replace(new_output_image(self.queue.device().clone(), frame_dims));
            self.screenshot_buffer.replace(new_screenshot_buffer(
                self.queue.device().clone(),
                (w as usize, h as usize),
            ));
        }

        // TODO shoudn't be Persistent
        let desciptor_set = Arc::new(
            vk::PersistentDescriptorSet::start(self.pipeline.clone(), 0)
                .add_sampled_image(frame_capture.clone(), self.sampler.clone())
                .expect("Failed to add output_color image to descriptor set")
                .add_image(self.output_image.borrow().clone())
                .expect("Failed to add output image")
                //.add_buffer_view(buf_view)
                // .expect("Failed to add screenshot buffer")
                .build()
                .expect("Failed to build record desciptor set"),
        );
        let push_constants = cs::ty::PushConstantData {
            width: w,
            height: h,
        };
        let command_buffer =
            vk::AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family())
                .expect("Failed to create command buffer")
                // TODO this should be the dims from the actual image
                // for this run.
                .dispatch(
                    [
                        (w as f32 / 32.0).ceil() as u32,
                        (h as f32 / 32.0).ceil() as u32,
                        1,
                    ],
                    self.pipeline.clone(),
                    desciptor_set.clone(),
                    push_constants,
                )
                .expect("Failed to dispatch")
                .copy_image_to_buffer(
                    self.output_image.borrow().clone(),
                    self.screenshot_buffer.borrow().clone(),
                )
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

    fn save(&mut self, dims: [u32; 2]) {
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
        if let Ok(screenshot_buffer) = self.screenshot_buffer.borrow().read() {
            self.num_images += 1;
            let screenshot_path = format!(
                "{}{}",
                env!("CARGO_MANIFEST_DIR"),
                format!("/screenshot{}.png", self.num_images)
            );
            write(
                &(*screenshot_buffer),
                screenshot_path,
                (dims[0] as usize, dims[1] as usize),
            );
        }
    }
}

fn new_input_image(device: Arc<vk::Device>, dims: [u32; 2]) -> Arc<vk::AttachmentImage> {
    vk::AttachmentImage::with_usage(
        device,
        dims,
        // TODO this needs to check if the swapchain is in BGRA or RGBA
        vk::Format::B8G8R8A8Unorm,
        vk::ImageUsage {
            transfer_destination: true,
            sampled: true,
            ..vk::ImageUsage::none()
        },
    )
    .expect("Failed to create input image")
}

fn new_output_image(
    device: Arc<vk::Device>,
    dims: vk::image::Dimensions,
) -> Arc<vk::StorageImage<vk::Format>> {
    vk::StorageImage::with_usage(
        device.clone(),
        dims,
        vk::Format::R8G8B8A8Uint,
        vk::ImageUsage {
            transfer_source: true,
            storage: true,
            ..vk::ImageUsage::none()
        },
        device.active_queue_families(),
    )
    .expect("Failed to create output image")
}

fn new_screenshot_buffer(
    device: Arc<vk::Device>,
    dims: (usize, usize),
) -> Arc<vk::CpuAccessibleBuffer<[[u8; NUM_COLOURS]]>> {
    let buf = vec![[0u8; NUM_COLOURS]; dims.0 * dims.1];
    vk::CpuAccessibleBuffer::from_iter(
        device.clone(),
        vk::BufferUsage {
            transfer_destination: true,
            ..vk::BufferUsage::none()
        },
        buf.into_iter(),
    )
    .expect("Failed to create screenshot buffer")
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

layout(local_size_x = 32, local_size_y = 32, local_size_z = 1) in;

layout(set = 0, binding = 0) uniform sampler2D tex;

layout(set = 0, binding = 1, rgba8ui) uniform uimage2D storage_image;

layout(push_constant) uniform PushConstantData {
    uint width;
    uint height;
} pc;

void main() {
    if(gl_GlobalInvocationID.x >= pc.width || gl_GlobalInvocationID.y >= pc.height) {
        return;
    }
    vec4 t = texture(tex, vec2(gl_GlobalInvocationID.x / float(pc.width), gl_GlobalInvocationID.y / float(pc.height)));
    uvec4 col = uvec4(uint(255*t.x), uint(255*t.y), uint(255*t.z), uint(255*t.w));
    imageStore(storage_image, ivec2(gl_GlobalInvocationID.xy), col);
}"
    }
}
