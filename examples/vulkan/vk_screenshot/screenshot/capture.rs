use nannou::prelude::*;
use std::cell::RefCell;
use std::sync::Arc;

pub(crate) struct FrameCapture {
    render_pass: Arc<dyn vk::RenderPassAbstract + Send + Sync>,
    resolve_fbo: RefCell<vk::Fbo>,
}

impl FrameCapture {
    pub(crate) fn new(device: Arc<vk::Device>, msaa_samples: u32) -> Self {
        FrameCapture {
            render_pass: create_render_pass(device, msaa_samples),
            resolve_fbo: Default::default(),
        }
    }

    pub(crate) fn capture(&self, frame: &Frame, input_color: Arc<vk::AttachmentImage>) {
        let [w, h] = frame.swapchain_image().dimensions();
        let dims = [w, h, 1];
        // Copy image in a pass so that we can resolve if needed
        self.resolve_fbo
            .borrow_mut()
            .update(self.render_pass.clone(), dims, |builder| {
                builder
                    .add(frame.image().clone())
                    .expect("Failed to add frame image")
                    .add(input_color.clone())
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
        /*
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
            */
    }

    pub(crate) fn clear(&self) {
        let mut fb = self.resolve_fbo.borrow_mut();
        *fb = Default::default();
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
                format: nannou::frame::COLOR_FORMAT,
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
