extern crate nannou;

use nannou::prelude::*;
use nannou::vulkano;
use std::cell::RefCell;
use std::sync::Arc;

use nannou::vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use nannou::vulkano::command_buffer::DynamicState;
use nannou::vulkano::descriptor::descriptor_set::{DescriptorSet, PersistentDescriptorSet};
use nannou::vulkano::device::DeviceOwned;
use nannou::vulkano::format::Format;
use nannou::vulkano::framebuffer::{
    Framebuffer, FramebufferAbstract, FramebufferCreationError, RenderPassAbstract, Subpass,
};
use nannou::vulkano::image::{Dimensions, ImmutableImage};
use nannou::vulkano::pipeline::viewport::Viewport;
use nannou::vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use nannou::vulkano::sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode};

fn main() {
    nannou::app(model).run();
}

struct Model {
    render_pass: Arc<RenderPassAbstract + Send + Sync>,
    pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
    vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
    framebuffers: RefCell<Vec<Arc<FramebufferAbstract + Send + Sync>>>,
    desciptor_set: Arc<DescriptorSet + Send + Sync>,
}

#[derive(Debug, Clone)]
struct Vertex {
    position: [f32; 2],
}

nannou::vulkano::impl_vertex!(Vertex, position);

fn model(app: &App) -> Model {
    app.new_window()
        .with_dimensions(220, 220)
        .view(view)
        .build()
        .unwrap();

    let device = app.main_window().swapchain().device().clone();

    let vertex_buffer = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::all(),
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
        nannou::vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: app.main_window().swapchain().format(),
                    samples: 1,
                    initial_layout: ImageLayout::PresentSrc,
                    final_layout: ImageLayout::PresentSrc,
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
        ImmutableImage::from_iter(
            image_data.into_iter(),
            Dimensions::Dim2dArray {
                width,
                height,
                array_layers,
            },
            Format::R8G8B8A8Srgb,
            app.main_window().swapchain_queue().clone(),
        )
        .unwrap()
    };

    let sampler = Sampler::new(
        device.clone(),
        Filter::Linear,
        Filter::Linear,
        MipmapMode::Nearest,
        SamplerAddressMode::ClampToEdge,
        SamplerAddressMode::ClampToEdge,
        SamplerAddressMode::ClampToEdge,
        0.0,
        1.0,
        0.0,
        0.0,
    )
    .unwrap();

    let pipeline = Arc::new(
        GraphicsPipeline::start()
            .vertex_input_single_buffer::<Vertex>()
            .vertex_shader(vertex_shader.main_entry_point(), ())
            .triangle_strip()
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(fragment_shader.main_entry_point(), ())
            .blend_alpha_blending()
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
            .build(device.clone())
            .unwrap(),
    );

    let desciptor_set = Arc::new(
        PersistentDescriptorSet::start(pipeline.clone(), 0)
            .add_sampled_image(texture.clone(), sampler.clone())
            .unwrap()
            .build()
            .unwrap(),
    );

    let framebuffers = RefCell::new(Vec::new());

    Model {
        render_pass,
        pipeline,
        vertex_buffer,
        framebuffers,
        desciptor_set,
    }
}

fn view(app: &App, model: &Model, frame: Frame) -> Frame {
    let [w, h] = frame.swapchain_image().dimensions();
    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [w as _, h as _],
        depth_range: 0.0..1.0,
    };
    let dynamic_state = DynamicState {
        line_width: None,
        viewports: Some(vec![viewport]),
        scissors: None,
    };

    // Update the framebuffers if necessary.
    while frame.swapchain_image_index() >= model.framebuffers.borrow().len() {
        let fb =
            create_framebuffer(model.render_pass.clone(), frame.swapchain_image().clone()).unwrap();
        model.framebuffers.borrow_mut().push(Arc::new(fb));
    }

    // If the dimensions for the current framebuffer do not match, recreate it.
    if frame.swapchain_image_is_new() {
        let fb = &mut model.framebuffers.borrow_mut()[frame.swapchain_image_index()];
        let new_fb =
            create_framebuffer(model.render_pass.clone(), frame.swapchain_image().clone()).unwrap();
        *fb = Arc::new(new_fb);
    }

    let clear_values = vec![[0.0, 1.0, 0.0, 1.0].into()];

    let push_constants = fs::ty::PushConstantData {
        sequence_idx: (app.time * 124.0) as i32 % 86,
        time: app.time * 20.0,
    };

    frame
        .add_commands()
        .begin_render_pass(
            model.framebuffers.borrow()[frame.swapchain_image_index()].clone(),
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
    nannou::vulkano_shaders::shader! {
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
    nannou::vulkano_shaders::shader! {
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

// Create the framebuffer for the image.
fn create_framebuffer(
    render_pass: Arc<RenderPassAbstract + Send + Sync>,
    swapchain_image: Arc<nannou::window::SwapchainImage>,
) -> Result<Arc<FramebufferAbstract + Send + Sync>, FramebufferCreationError> {
    let fb = Framebuffer::start(render_pass)
        .add(swapchain_image)?
        .build()?;
    Ok(Arc::new(fb) as _)
}
