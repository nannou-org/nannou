use super::{new_input_image, new_output_image, Buffer};
use nannou::prelude::*;
use std::cell::RefCell;
use std::sync::Arc;
use vk::image::Dimensions;

pub(crate) struct FrameCapture {
    render_pass: Arc<dyn vk::RenderPassAbstract + Send + Sync>,
    resolve_fbo: RefCell<vk::Fbo>,
    inter_color: Arc<vk::AttachmentImage>,
    output_image: Arc<vk::StorageImage<vk::Format>>,
}

impl FrameCapture {
    pub(crate) fn new(device: Arc<vk::Device>, msaa_samples: u32, dims: [u32; 2]) -> Self {
        FrameCapture {
            render_pass: create_render_pass(device.clone(), msaa_samples),
            resolve_fbo: Default::default(),
            inter_color: new_input_image(device.clone(), dims),
            output_image: new_output_image(
                device,
                Dimensions::Dim2d {
                    width: dims[0],
                    height: dims[1],
                },
            ),
        }
    }

    pub(crate) fn capture(&self, frame: &Frame, screenshot_buffer: Buffer) {
        let [w, h] = frame.swapchain_image().dimensions();
        let dims = [w, h, 1];
        // Copy image in a pass so that we can resolve if needed
        self.resolve_fbo
            .borrow_mut()
            .update(self.render_pass.clone(), dims, |builder| {
                builder
                    .add(frame.image().clone())
                    .expect("Failed to add frame image")
                    .add(self.inter_color.clone())
            })
            .expect("Failed to add input image");
        let clear_values = vec![vk::ClearValue::None, vk::ClearValue::None];
        frame
            .add_commands()
            .begin_render_pass(
                self.resolve_fbo
                    .borrow()
                    .as_ref()
                    .expect("Failed to get resolve_fbo")
                    .clone(),
                false,
                clear_values,
            )
            .expect("failed to begin render pass for screenshot copy")
            .end_render_pass()
            .expect("failed to add `end_render_pass` command");
        let [w, h] = self.inter_color.dimensions();
        let src = self.inter_color.clone();
        let src_tl = [0; 3];
        let src_br = [w as i32, h as i32, 1];
        let src_base_layer = 0;
        let src_mip_level = 0;
        let dst = self.output_image.clone();
        let dst_tl = [0; 3];
        let dst_br = [w as i32, h as i32, 1];
        let dst_base_layer = 0;
        let dst_mip_level = 0;
        let layer_count = 1;
        let filter = vk::sampler::Filter::Linear;
        frame
            .add_commands()
            .blit_image(
                src,
                src_tl,
                src_br,
                src_base_layer,
                src_mip_level,
                dst,
                dst_tl,
                dst_br,
                dst_base_layer,
                dst_mip_level,
                layer_count,
                filter,
            )
            .expect("failed to blit linear sRGBA image to swapchain image")
            .copy_image_to_buffer(self.output_image.clone(), screenshot_buffer.buffer.clone())
            .expect("Failed to copy image to buffer");
    }

    pub(crate) fn clear(&self) {
        let mut fb = self.resolve_fbo.borrow_mut();
        *fb = Default::default();
    }

    pub(crate) fn update_images(&mut self, device: Arc<vk::Device>, dims: (usize, usize)) {
        self.inter_color = new_input_image(device.clone(), [dims.0 as u32, dims.1 as u32]);
        self.output_image = new_output_image(
            device,
            Dimensions::Dim2d {
                width: dims.0 as u32,
                height: dims.1 as u32,
            },
        );
    }
}

fn create_render_pass(
    device: Arc<vk::Device>,
    msaa_samples: u32,
) -> Arc<dyn vk::RenderPassAbstract + Send + Sync> {
    let rp = vk::single_pass_renderpass!(
        device,
        attachments: {
            frame_image: {
                load: Load,
                store: Store,
                format: nannou::frame::COLOR_FORMAT,
                samples: msaa_samples,
            },
            resolve_image: {
                load: DontCare,
                store: Store,
                //format: super::INPUT_IMAGE_FORMAT,
                //format: nannou::frame::COLOR_FORMAT,

                format: vk::Format::R8G8B8A8Uint,
                samples: 1,
            }
        },
        pass: {
            color: [frame_image],
            depth_stencil: {}
            resolve: [resolve_image],
        }
    )
    .expect("Failed to create resolve renderpass");
    let rp = Arc::new(rp) as Arc<dyn vk::RenderPassAbstract + Send + Sync>;
    rp
}
