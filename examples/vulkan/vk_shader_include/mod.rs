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
}

#[derive(Clone, Debug, Default)]
struct Vertex {
    position: [f32; 2],
}

vk::impl_vertex!(Vertex, position);

fn model(app: &App) -> Model {
    app.new_window()
        .with_dimensions(1024, 512)
        .with_title("nannou")
        .view(view)
        .build()
        .unwrap();

    let device = app.main_window().swapchain().device().clone();

    let vertex_buffer = vk::CpuAccessibleBuffer::from_iter(
        device.clone(),
        vk::BufferUsage::all(),
        [
            Vertex {
                position: [-1.0, -1.0],
            },
            Vertex {
                position: [-1.0, 1.0],
            },
            Vertex {
                position: [1.0, -1.0],
            },
            Vertex {
                position: [1.0, 1.0],
            },
        ]
        .iter()
        .cloned(),
    )
    .unwrap();

    let vertex_shader = vs::Shader::load(device.clone()).unwrap();
    let fragment_shader = fs::Shader::load(device.clone()).unwrap();

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

    let view_fbo = RefCell::new(ViewFbo::default());

    Model {
        render_pass,
        pipeline,
        vertex_buffer,
        view_fbo,
    }
}

fn view(app: &App, model: &Model, frame: &Frame) {
    let [w, h] = frame.swapchain_image().dimensions();
    let viewport = vk::ViewportBuilder::new().build([w as _, h as _]);
    let dynamic_state = vk::DynamicState::default().viewports(vec![viewport]);

    // Update the view_fbo in case of window resize.
    model
        .view_fbo
        .borrow_mut()
        .update(frame, model.render_pass.clone(), |builder, image| {
            builder.add(image)
        })
        .unwrap();

    let clear_values = vec![[0.0, 1.0, 0.0, 1.0].into()];

    let push_constants = fs::ty::PushConstantData {
        time: app.time,
        width: w as f32,
        height: h as f32,
    };

    frame
        .add_commands()
        .begin_render_pass(model.view_fbo.borrow().expect_inner(), false, clear_values)
        .unwrap()
        .draw(
            model.pipeline.clone(),
            &dynamic_state,
            vec![model.vertex_buffer.clone()],
            (),
            push_constants,
        )
        .unwrap()
        .end_render_pass()
        .expect("failed to add `end_render_pass` command");
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
    // We declare what directories to search for when using the `#include <...>`
    // syntax. Specified directories have descending priorities based on their order.
    include: [ "examples/vulkan/vk_shader_include/common_shaders" ],
        src: "
#version 450
// Substitutes this line with the contents of the file `lfos.glsl` found in one of the standard
// `include` directories specified above.
// Note, that relative inclusion (`#include \"...\"`), although it falls back to standard
// inclusion, should not be used for **embedded** shader source, as it may be misleading and/or
// confusing.
#include <lfos.glsl>

layout(location = 0) in vec2 tex_coords;
layout(location = 0) out vec4 f_color;

layout(push_constant) uniform PushConstantData {
    float time;
    float width;
    float height;
} pc;

float circle(in vec2 _st, in float _radius){
    vec2 dist = _st-vec2(0.5);
	return 1.-smoothstep(_radius-(_radius*0.01),
                         _radius+(_radius*0.01),
                         dot(dist,dist)*4.0);
}

void main() {

    // Background
	vec3 bg = vec3(0.8,0.9,0.9);

    float aspect = pc.width / pc.height;
    vec2 center = vec2(0.5,0.5);
    float radius = 0.25 * aspect;

    // Here we use the 'lfo' function imported from our include shader
    float x = tex_coords.x + lfo(2, pc.time * 3.0) * 0.7;
    vec3 color = vec3(lfo(3,pc.time * 6.0),0.0,0.9);
    vec3 shape = vec3(circle(vec2((x * aspect) - 0.45, tex_coords.y), radius) );
    shape *= color;

    // Blend the two
	f_color = vec4( vec3(mix(shape, bg, 0.5)),1.0 );
}"
    }
}
