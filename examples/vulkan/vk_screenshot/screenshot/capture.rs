use super::{new_input_image, new_output_image, Buffer};
use nannou::prelude::*;
use std::cell::RefCell;
use std::sync::Arc;

pub(crate) struct FrameCapture {
    resolve_rp: Arc<dyn vk::RenderPassAbstract + Send + Sync>,
    sample_rp: Arc<dyn vk::RenderPassAbstract + Send + Sync>,
    resolve_fbo: RefCell<vk::Fbo>,
    sample_fbo: RefCell<vk::Fbo>,
    inter_color: Arc<vk::AttachmentImage>,
    output_image: Arc<vk::AttachmentImage>,
    //output_image: Arc<vk::StorageImage<vk::Format>>,
    descriptor_set: RefCell<
        vk::FixedSizeDescriptorSetsPool<Arc<dyn vk::GraphicsPipelineAbstract + Send + Sync>>,
    >,
    sample_pipeline: Arc<dyn vk::GraphicsPipelineAbstract + Send + Sync>,
    vertex_buffer: Arc<vk::CpuAccessibleBuffer<[Vertex]>>,
    sampler: Arc<vk::Sampler>,
}

#[derive(Debug, Default, Clone)]
struct Vertex {
    position: [f32; 2],
}

vk::impl_vertex!(Vertex, position);

impl FrameCapture {
    pub(crate) fn new(device: Arc<vk::Device>, msaa_samples: u32, dims: [u32; 2]) -> Self {
        let (resolve_rp, sample_rp) = create_render_pass(device.clone(), msaa_samples);
        let sample_pipeline = create_sample_pipeline(device.clone(), sample_rp.clone());
        let descriptor_set = create_descriptor_set(sample_pipeline.clone());
        let descriptor_set = RefCell::new(descriptor_set);
        let sampler = vk::SamplerBuilder::new().build(device.clone()).unwrap();
        FrameCapture {
            resolve_rp,
            sample_rp,
            resolve_fbo: Default::default(),
            sample_fbo: Default::default(),
            inter_color: new_input_image(device.clone(), dims),
            descriptor_set,
            sample_pipeline,
            sampler,
            vertex_buffer: create_vertex_buffer(device.clone()),
            output_image: new_output_image(device.clone(), dims),
        }
    }

    pub(crate) fn capture(&self, frame: &Frame, screenshot_buffer: Buffer) {
        let [w, h] = frame.swapchain_image().dimensions();
        let dims = [w, h, 1];
        let viewport = vk::ViewportBuilder::new().build([w as _, h as _]);
        let dynamic_state = vk::DynamicState::default().viewports(vec![viewport]);
        // Copy image in a pass so that we can resolve if needed
        self.resolve_fbo
            .borrow_mut()
            .update(self.resolve_rp.clone(), dims, |builder| {
                builder
                    .add(frame.image().clone())
                    .expect("Failed to add frame image")
                    .add(self.inter_color.clone())
            })
            .expect("Failed to add inter image");
        self.sample_fbo
            .borrow_mut()
            .update(self.sample_rp.clone(), dims, |builder| {
                builder.add(self.output_image.clone())
            })
            .expect("Failed to add output image");
        let clear_values = vec![vk::ClearValue::None, vk::ClearValue::None];
        let clear_value_sample = vec![vk::ClearValue::None];
        let set = self
            .descriptor_set
            .borrow_mut()
            .next()
            .add_sampled_image(self.inter_color.clone(), self.sampler.clone())
            .expect("Failed to add sampler")
            .build()
            .expect("Failed to build descriptor set");
        frame
            .add_commands()
            .begin_render_pass(
                self.resolve_fbo
                    .borrow()
                    .as_ref()
                    .expect("Failed to get resolve_fbo")
                    .clone(),
                false,
                clear_values.clone(),
            )
            .expect("failed to begin render pass for screenshot copy")
            .end_render_pass()
            .expect("failed to add `end_render_pass` command")
            .begin_render_pass(
                self.sample_fbo
                    .borrow()
                    .as_ref()
                    .expect("Failed to get resolve_fbo")
                    .clone(),
                false,
                clear_value_sample,
            )
            .expect("failed to begin render pass for screenshot copy")
            .draw(
                self.sample_pipeline.clone(),
                &dynamic_state,
                vec![self.vertex_buffer.clone()],
                set,
                (),
            )
            .expect("Failed to draw sample pass")
            .end_render_pass()
            .expect("failed to add `end_render_pass` command")
            .copy_image_to_buffer(self.output_image.clone(), screenshot_buffer.buffer.clone())
            .expect("Failed to copy image to buffer");
        /*
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
            */
    }

    pub(crate) fn clear(&self) {
        let mut fb = self.resolve_fbo.borrow_mut();
        *fb = Default::default();
    }

    pub(crate) fn update_images(&mut self, device: Arc<vk::Device>, dims: (usize, usize)) {
        self.inter_color = new_input_image(device.clone(), [dims.0 as u32, dims.1 as u32]);
        self.output_image = new_output_image(device.clone(), [dims.0 as u32, dims.1 as u32]);
    }
}

fn create_render_pass(
    device: Arc<vk::Device>,
    msaa_samples: u32,
) -> (
    Arc<dyn vk::RenderPassAbstract + Send + Sync>,
    Arc<dyn vk::RenderPassAbstract + Send + Sync>,
) {
    let resolve_rp = vk::single_pass_renderpass!(
        device.clone(),
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

                //format: vk::Format::R8G8B8A8Uint,
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
    let sample_rp = vk::single_pass_renderpass!(
        device,
        attachments: {
            output_image: {
                load: DontCare,
                store: Store,
                format: vk::Format::R8G8B8A8Uint,
                samples: 1,
            }
        },
        pass: {
            color: [output_image],
            depth_stencil: {}
            resolve: [],
        }
    )
    .expect("Failed to create resolve renderpass");
    /*
    let rp = vulkano::ordered_passes_renderpass!(
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

                //format: vk::Format::R8G8B8A8Uint,
                samples: 1,
            },
            output_image: {
                load: DontCare,
                store: Store,
                format: vk::Format::R8G8B8A8Uint,
                samples: 1,
            }
        },
        passes: [
            {
                color: [frame_image],
                depth_stencil: {},
                input: [],
                resolve: [resolve_image]
            },
            {
                color: [output_image],
                depth_stencil: {},
                input: [],
                resolve: []
            }
        ]
    )
    .expect("Failed to create resolve renderpass");
    let rp = Arc::new(rp) as Arc<dyn vk::RenderPassAbstract + Send + Sync>;
    rp*/
    (Arc::new(resolve_rp), Arc::new(sample_rp))
}

fn create_sample_pipeline(
    device: Arc<vk::Device>,
    render_pass: Arc<dyn vk::RenderPassAbstract + Send + Sync>,
) -> Arc<dyn vk::GraphicsPipelineAbstract + Send + Sync> {
    let vertex_shader = vs::Shader::load(device.clone()).unwrap();
    let fragment_shader = fs::Shader::load(device.clone()).unwrap();
    Arc::new(
        vk::GraphicsPipeline::start()
            .vertex_input_single_buffer::<Vertex>()
            .vertex_shader(vertex_shader.main_entry_point(), ())
            .triangle_strip()
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(fragment_shader.main_entry_point(), ())
            .render_pass(vk::Subpass::from(render_pass, 0).unwrap())
            .build(device)
            .unwrap(),
    )
}

fn create_vertex_buffer(device: Arc<vk::Device>) -> Arc<vk::CpuAccessibleBuffer<[Vertex]>> {
    let positions = [[-1.0, -1.0], [-1.0, 1.0], [1.0, -1.0], [1.0, 1.0]];
    let vertices = positions.iter().map(|&position| Vertex { position });
    vk::CpuAccessibleBuffer::from_iter(device.clone(), vk::BufferUsage::all(), vertices).unwrap()
}

fn create_descriptor_set(
    graphics_pipeline: Arc<dyn vk::GraphicsPipelineAbstract + Send + Sync>,
) -> vk::FixedSizeDescriptorSetsPool<Arc<dyn vk::GraphicsPipelineAbstract + Send + Sync>> {
    let pool = vk::FixedSizeDescriptorSetsPool::new(graphics_pipeline, 0);
    pool
}

mod vs {
    nannou::vk::shaders::shader! {
    ty: "vertex",
        src: "
#version 450
layout(location = 0) in vec2 position;
layout(location = 0) out vec2 tex_coords;
void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    tex_coords = position + vec2(0.5);
}"
    }
}

mod fs {
    nannou::vk::shaders::shader! {
    ty: "fragment",
        src: "
#version 450
layout(location = 0) in vec2 tex_coords;
layout(location = 0) out uvec4 f_color;
layout(set = 0, binding = 0) uniform sampler2D tex;
void main() {
    float max = 255;
    vec4 col = texture(tex, tex_coords);
    f_color = uvec4(
    uint(max * pow(col.r, 1.0 / 2.2)),
    uint(max * pow(col.g, 1.0 / 2.2)),
    uint(max * pow(col.b, 1.0 / 2.2)),
    uint(max * col.a));
}"
    }
}
