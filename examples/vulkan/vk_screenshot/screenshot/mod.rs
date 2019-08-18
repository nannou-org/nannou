use nannou::prelude::*;
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::slice;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

mod capture;

// This must match the number of colours per
// pixel.
// RGBA = 4
// RGB = 3
// RG = 2 etc.
pub const NUM_COLOURS: usize = 4;

#[derive(Clone)]
pub(crate) struct Buffer {
    buffer: Arc<vk::CpuAccessibleBuffer<[[u8; NUM_COLOURS]]>>,
    dims: (usize, usize),
}

struct ShotWriter {
    num_images: usize,
}

// Hack to get around wait issue
pub struct Shots {
    num_shots: Cell<usize>,
    frames_since_empty: Cell<usize>,
    images_in: Receiver<Buffer>,
    images_out: Sender<Msg>,
    saving_thread: Option<JoinHandle<()>>,
    frame_capture: RefCell<capture::FrameCapture>,
}

enum Msg {
    Buffer(Buffer),
    Flush,
    Kill,
}

#[derive(Default, Debug, Clone)]
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
        let output_image = Buffer {
            buffer: new_screenshot_buffer(queue.device().clone(), (dims.0, dims.1)),
            dims,
        };
        save_out
            .send(output_image)
            .expect("Failed to send initial images");
    }
    let shot_writer = ShotWriter { num_images: 0 };
    let saving_thread = thread::spawn({ || save_images(shot_writer, save_in, save_out) });
    let saving_thread = Some(saving_thread);
    let frame_capture = RefCell::new(capture::FrameCapture::new(
        queue.device().clone(),
        window.msaa_samples(),
        [dims.0 as u32, dims.1 as u32],
    ));
    Shots {
        num_shots: Cell::new(0),
        frames_since_empty: Cell::new(3),
        images_in,
        images_out,
        saving_thread,
        frame_capture,
    }
}

fn save_images(mut screenshot: ShotWriter, save_in: Receiver<Msg>, save_out: Sender<Buffer>) {
    let mut q = VecDeque::new();
    while let Ok(msg) = save_in.recv() {
        match msg {
            Msg::Buffer(image) => {
                q.push_back(image);
                if q.len() > 2 {
                    let image = q.pop_front().unwrap();
                    screenshot.save(image.clone());
                    save_out.send(image).ok();
                }
            }
            Msg::Flush => {
                while let Some(image) = q.pop_front() {
                    screenshot.save(image.clone());
                    save_out.send(image).ok();
                }
            }
            Msg::Kill => {
                while let Some(image) = q.pop_front() {
                    screenshot.save(image.clone());
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
        self.frame_capture.borrow().clear();
        if num_shots > 0 {
            if let Ok(mut image) = self.images_in.recv() {
                let [w, h] = frame.swapchain_image().dimensions();
                let swap_dims = (w as usize, h as usize);
                if swap_dims != image.dims {
                    image = Buffer {
                        buffer: new_screenshot_buffer(frame.queue().device().clone(), swap_dims),
                        dims: swap_dims,
                    };
                    self.frame_capture
                        .borrow_mut()
                        .update_images(frame.queue().device().clone(), swap_dims);
                }
                self.frame_capture.borrow().capture(frame, image.clone());
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

impl ShotWriter {
    fn save(&mut self, screenshot_buffer: Buffer) {
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
        if let Ok(buffer) = screenshot_buffer.buffer.read() {
            self.num_images += 1;
            let screenshot_path = format!(
                "{}{}",
                env!("CARGO_MANIFEST_DIR"),
                format!("/screenshot{}.png", self.num_images)
            );
            write(&(*buffer), screenshot_path, screenshot_buffer.dims);
        }
    }
}

fn new_input_image(device: Arc<vk::Device>, dims: [u32; 2]) -> Arc<vk::AttachmentImage> {
    vk::AttachmentImage::with_usage(
        device,
        dims,
        // TODO this needs to check if the swapchain is in BGRA or RGBA
        //INPUT_IMAGE_FORMAT,
        nannou::frame::COLOR_FORMAT,
        //vk::Format::R8G8B8A8Uint,
        vk::ImageUsage {
            transfer_source: true,
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
            transfer_destination: true,
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
