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
    nannou::app(model)
        .event(event)
        .view(view)
        .run();
}

struct Model {
    _window: WindowId,
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
    let _window = app
        .new_window()
        .with_dimensions(512, 512)
        .with_title("nannou")
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

    let mut textures = Vec::new();
    let mut samplers = Vec::new();
    let num_images = 4;
    for i in 0..num_images {
        let (texture, _tex_future) = {
            let logo_path = app
                .assets_path()
                .unwrap()
                .join("images")
                .join(format!("nature_{}.jpg", i as i32 + 1));
            let image = image::open(logo_path).unwrap().to_rgba();
            let (width, height) = image.dimensions();
            let image_data = image.into_raw().clone();
            ImmutableImage::from_iter(
                image_data.iter().cloned(),
                Dimensions::Dim2d { width, height },
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

        textures.push(texture);
        samplers.push(sampler);
    }

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
            .add_sampled_image(textures[0].clone(), samplers[0].clone())
            .unwrap()
            .add_sampled_image(textures[1].clone(), samplers[1].clone())
            .unwrap()
            .add_sampled_image(textures[2].clone(), samplers[2].clone())
            .unwrap()
            .add_sampled_image(textures[3].clone(), samplers[3].clone())
            .unwrap()
            .build()
            .unwrap(),
    );

    let framebuffers = RefCell::new(Vec::new());

    Model {
        _window,
        render_pass,
        pipeline,
        vertex_buffer,
        framebuffers,
        desciptor_set,
    }
}

fn event(_app: &App, model: Model, event: Event) -> Model {
    if let Event::Update(_update) = event {}
    model
}

fn view(_app: &App, model: &Model, frame: Frame) -> Frame {
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

    while frame.swapchain_image_index() >= model.framebuffers.borrow().len() {
        let fb =
            create_framebuffer(model.render_pass.clone(), frame.swapchain_image().clone()).unwrap();
        model.framebuffers.borrow_mut().push(Arc::new(fb));
    }

    if frame.swapchain_image_is_new() {
        let fb = &mut model.framebuffers.borrow_mut()[frame.swapchain_image_index()];
        let new_fb =
            create_framebuffer(model.render_pass.clone(), frame.swapchain_image().clone()).unwrap();
        *fb = Arc::new(new_fb);
    }

    let clear_values = vec![[0.0, 1.0, 0.0, 1.0].into()];

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
            (),
        )
        .unwrap()
        .end_render_pass()
        .expect("failed to add `end_render_pass` command");

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

layout(set = 0, binding = 0) uniform sampler2D tex1;
layout(set = 0, binding = 1) uniform sampler2D tex2;
layout(set = 0, binding = 2) uniform sampler2D tex3;
layout(set = 0, binding = 3) uniform sampler2D tex4;

void main() {
    // Texture coordinates
    vec2 uv = tex_coords;
    uv.y *= -1.0;
    float aspect = uv.x / uv.y;

    float squares = pow(2.0,2.0);    
    float sw = sqrt(squares) / aspect;
    float sh = sqrt(squares);

    float vx = mod(uv.x * sw * aspect, 1.0);
    float vy = mod(uv.y * sh*-1.0, 1.0);

    float b = float(int(mod(uv.y*sh,2.0)));
	float a = float(int(mod(uv.x*sw * aspect + b,4.0)));

    vec4 c = vec4(0.0);
    if(a == 0) {
        c += texture(tex1, vec2(vx,vy));
    } else if(a == 1) {
        c += texture(tex2, vec2(vx,vy));    
    } else if(a == 2) {
        c += texture(tex3, vec2(vx,vy));    
    } else if(a == 3) {
        c += texture(tex4, vec2(vx,vy));    
    }
    f_color = c;
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
