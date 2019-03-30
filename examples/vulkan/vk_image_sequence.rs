extern crate nannou;

use nannou::prelude::*;
use std::cell::RefCell;
use std::sync::Arc;

fn main() {
    nannou::app(model).run();
}

struct Model {
    render_pass: Arc<vk::RenderPassAbstract + Send + Sync>,
    pipeline: Arc<vk::GraphicsPipelineAbstract + Send + Sync>,
    vertex_buffer: Arc<vk::CpuAccessibleBuffer<[Vertex]>>,
    view_fbo: RefCell<ViewFbo>,
    desciptor_set: Arc<vk::DescriptorSet + Send + Sync>,
}

#[derive(Debug, Clone)]
struct Vertex {
    position: [f32; 2],
}

vk::impl_vertex!(Vertex, position);

fn model(app: &App) -> Model {
    app.new_window()
        .with_dimensions(220, 220)
        .view(view)
        .build()
        .unwrap();

    let device = app.main_window().swapchain().device().clone();

    let positions = [[-1.0, -1.0], [-1.0, 1.0], [1.0, -1.0], [1.0, 1.0]];
    let data = positions.iter().map(|&position| Vertex { position });
    let usage = vk::BufferUsage::all();
    let vertex_buffer = vk::CpuAccessibleBuffer::from_iter(device.clone(), usage, data).unwrap();

    let vertex_shader = vs::Shader::load(device.clone()).unwrap();
    let fragment_shader = fs::Shader::load(device.clone()).unwrap();

    let render_pass = Arc::new(
        vk::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: app.main_window().swapchain().format(),
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

    // Load the images from a directory.
    let (texture, _tex_future) = {
        let sequence_path = app.assets_path().unwrap().join("images").join("sequence");
        let mut images = vec![];
        let (mut width, mut height) = (0, 0);
        for entry in std::fs::read_dir(sequence_path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let image = match image::open(&path) {
                Ok(img) => img.to_rgba(),
                Err(err) => {
                    eprintln!("failed to open {} as an image: {}", path.display(), err);
                    continue;
                }
            };
            let (w, h) = image.dimensions();
            width = w;
            height = h;
            let image_data = image.into_raw().clone();
            images.push((path, image_data));
        }
        images.sort();
        let array_layers = images.len() as u32;
        let image_data: Vec<_> = images
            .iter()
            .flat_map(|(_, img)| img.iter().cloned())
            .collect();
        vk::ImmutableImage::from_iter(
            image_data.into_iter(),
            vk::image::Dimensions::Dim2dArray {
                width,
                height,
                array_layers,
            },
            vk::Format::R8G8B8A8Srgb,
            app.main_window().swapchain_queue().clone(),
        )
        .unwrap()
    };

    let sampler = vk::SamplerBuilder::new().build(device.clone()).unwrap();

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

    let desciptor_set = Arc::new(
        vk::PersistentDescriptorSet::start(pipeline.clone(), 0)
            .add_sampled_image(texture.clone(), sampler.clone())
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
        desciptor_set,
    }
}

fn view(app: &App, model: &Model, frame: Frame) -> Frame {
    let [w, h] = frame.swapchain_image().dimensions();
    let viewport = vk::ViewportBuilder::new().build([w as _, h as _]);
    let dynamic_state = vk::DynamicState::default().viewports(vec![viewport]);

    // Update view_fbo in case of window resize.
    model.view_fbo.borrow_mut()
        .update(&frame, model.render_pass.clone(), |builder, image| builder.add(image))
        .unwrap();

    let clear_values = vec![[0.0, 1.0, 0.0, 1.0].into()];

    let push_constants = fs::ty::PushConstantData {
        sequence_idx: (app.time * 124.0) as i32 % 86,
        time: app.time * 20.0,
    };

    frame
        .add_commands()
        .begin_render_pass(
            model.view_fbo.borrow().expect_inner(),
            false,
            clear_values,
        )
        .unwrap()
        .draw(
            model.pipeline.clone(),
            &dynamic_state,
            vec![model.vertex_buffer.clone()],
            model.desciptor_set.clone(),
            push_constants,
        )
        .unwrap()
        .end_render_pass()
        .expect("failed to add `end_render_pass` command");

    let draw = app.draw();
    let win = app.window_rect();
    let t = app.time;
    draw.ellipse()
        .x_y(app.mouse.x * t.cos(), app.mouse.y)
        .radius(win.w() * 0.125 * t.sin())
        .rgba(1.0, 0.0, 0.0, 0.4);

    frame
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
layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) uniform sampler2DArray tex;

layout(push_constant) uniform PushConstantData {
  int sequence_idx;
  float time;
} pc;

void main() {
    vec4 c = vec4( abs(tex_coords.x + sin(pc.time)), tex_coords.x, tex_coords.y * abs(cos(pc.time)), 1.0);    
    f_color = texture(tex, vec3(tex_coords, pc.sequence_idx)) + (c*0.6);
}"
    }
}
