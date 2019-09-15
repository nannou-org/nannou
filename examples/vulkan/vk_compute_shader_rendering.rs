use nannou::prelude::*;
use std::cell::RefCell;
use std::sync::Arc;

fn main() {
    nannou::app(model).run();
}

struct Model {
    render_pass: Arc<dyn vk::RenderPassAbstract + Send + Sync>,
    pipeline: Arc<dyn vk::GraphicsPipelineAbstract + Send + Sync>,
    vertex_buffer: Arc<vk::CpuAccessibleBuffer<[Vertex]>>,
    view_fbo: RefCell<ViewFbo>,
    descriptor_set: Arc<dyn vk::DescriptorSet + Send + Sync>,
    compute_pipeline: Arc<dyn vk::ComputePipelineAbstract + Send + Sync>,
    compute_set: Arc<dyn vk::DescriptorSet + Send + Sync>,
}

#[derive(Debug, Default, Clone)]
struct Vertex {
    position: [f32; 2],
}
vk::impl_vertex!(Vertex, position);

fn model(app: &App) -> Model {
    let window_size = vk::image::Dimensions::Dim2d {
        width: 512,
        height: 512,
    };

    app.new_window()
        .with_dimensions(window_size.width(), window_size.height())
        .view(view)
        .build()
        .unwrap();

    let device = app.main_window().swapchain().device().clone();
    let queue = app.main_window().swapchain_queue().clone();

    // compute pipeline for rendering the fractal into an image
    let image = vk::StorageImage::new(
        device.clone(),
        window_size,
        vk::Format::R8G8B8A8Unorm,
        Some(queue.family()),
    )
    .unwrap();

    let shader = fractal::cs::Shader::load(device.clone()).expect("failed to create shader module");

    let compute_pipeline = Arc::new(
        vk::ComputePipeline::new(device.clone(), &shader.main_entry_point(), &())
            .expect("failed to create compute pipeline"),
    );

    let compute_set = Arc::new(
        vk::PersistentDescriptorSet::start(compute_pipeline.clone(), 0)
            .add_image(image.clone())
            .unwrap()
            .build()
            .unwrap(),
    );

    // graphics pipeline for rendering the image on the screen
    // one triangle is enough for covering the whole screen
    // (the positions are chosen so that in the shader the corners of
    // the screen have the uv coordinates (0,0), (0,1), (1,1), (1,0) )
    let vertex_buffer = vk::CpuAccessibleBuffer::from_iter(
        device.clone(),
        vk::BufferUsage::all(),
        [
            Vertex {
                position: [-1.0, -1.0],
            },
            Vertex {
                position: [-1.0, 3.0],
            },
            Vertex {
                position: [3.0, -1.0],
            },
        ]
        .iter()
        .cloned(),
    )
    .unwrap();

    let vertex_shader = display_image::vs::Shader::load(device.clone()).unwrap();
    let fragment_shader = display_image::fs::Shader::load(device.clone()).unwrap();

    let render_pass = Arc::new(
        vk::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: nannou::frame::COLOR_FORMAT,
                    samples: app.main_window().msaa_samples(),
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
            .vertex_shader(vertex_shader.main_entry_point(), ())
            .triangle_strip()
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(fragment_shader.main_entry_point(), ())
            .blend_alpha_blending()
            .render_pass(vk::Subpass::from(render_pass.clone(), 0).unwrap())
            .build(device.clone())
            .unwrap(),
    );

    let sampler = vk::SamplerBuilder::new().build(device.clone()).unwrap();

    let descriptor_set = Arc::new(
        vk::PersistentDescriptorSet::start(pipeline.clone(), 0)
            .add_sampled_image(image.clone(), sampler.clone())
            .unwrap()
            .build()
            .unwrap(),
    );

    let view_fbo = RefCell::new(ViewFbo::default());

    Model {
        render_pass,
        pipeline,
        vertex_buffer,
        view_fbo,
        descriptor_set,
        compute_pipeline,
        compute_set,
    }
}

fn view(app: &App, model: &Model, frame: &Frame) {
    let [w, h] = frame.swapchain_image().dimensions();
    let viewport = vk::ViewportBuilder::new().build([w as _, h as _]);
    let dynamic_state = vk::DynamicState::default().viewports(vec![viewport]);

    // // Update view_fbo in case of resize.
    model
        .view_fbo
        .borrow_mut()
        .update(frame, model.render_pass.clone(), |builder, image| {
            builder.add(image)
        })
        .unwrap();

    let clear_values = vec![[0.0, 1.0, 0.0, 1.0].into()];

    let push_constants = fractal::cs::ty::PushConstantData { time: app.time };

    frame
        .add_commands()
        .dispatch(
            [w / 8, h / 8, 1],
            model.compute_pipeline.clone(),
            model.compute_set.clone(),
            push_constants,
        )
        .expect("failed to add `dispatch` command")
        .begin_render_pass(model.view_fbo.borrow().expect_inner(), false, clear_values)
        .unwrap()
        .draw(
            model.pipeline.clone(),
            &dynamic_state,
            vec![model.vertex_buffer.clone()],
            model.descriptor_set.clone(),
            (),
        )
        .unwrap()
        .end_render_pass()
        .expect("failed to add `end_render_pass` command");
}

mod fractal {
    pub mod cs {
        nannou::vk::shaders::shader! {
            ty: "compute",
            src: "
#version 450

layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

layout(set = 0, binding = 0, rgba8) uniform writeonly image2D img;

layout(push_constant) uniform PushConstantData {
    float time;
} pc;

void main() {
    vec2 norm_coordinates = (gl_GlobalInvocationID.xy + vec2(0.5)) / vec2(imageSize(img));

    float speed = 0.8;
    vec2 c = 0.7885*vec2(cos(speed*pc.time), sin(0.1+speed*pc.time));

    vec2 z = 4.*(norm_coordinates - vec2(0.5));
    float i;
    for (i = 0.0; i < 1.0; i += 0.005) {
        z = vec2(
            z.x * z.x - z.y * z.y + c.x,
            2 * z.x * z.y  + c.y
        );

        if (length(z) > 4.0) {
            break;
        }
    }

    i = float(i>0.05) * i; 
    vec4 color = vec4(vec3(i), 1.0);
    imageStore(img, ivec2(gl_GlobalInvocationID.xy), color);
}"
        }
    }
}

mod display_image {
    pub mod vs {
        nannou::vk::shaders::shader! {
        ty: "vertex",
            src: "
#version 450

layout(location = 0) in vec2 position;
layout(location = 0) out vec2 tex_coords;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    tex_coords = 0.5*position + vec2(0.5);
}"
        }
    }

    pub mod fs {
        nannou::vk::shaders::shader! {
        ty: "fragment",
            src: "
#version 450

layout(location = 0) in vec2 tex_coords;
layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) uniform sampler2D tex;

void main() {
    f_color = texture(tex, tex_coords);
}"
        }
    }
}
