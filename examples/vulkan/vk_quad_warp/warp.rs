use nannou::prelude::*;
use std::cell::RefCell;
use std::sync::Arc;
use crate::homography;

use nannou::vulkano::buffer::{BufferUsage, ImmutableBuffer};
use nannou::vulkano::command_buffer::DynamicState;
use nannou::vulkano::descriptor::descriptor_set::{PersistentDescriptorSet};
use nannou::vulkano::device::DeviceOwned;
use nannou::vulkano::framebuffer::{RenderPassAbstract, Subpass};
use nannou::vulkano::pipeline::viewport::Viewport;
use nannou::vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use nannou::vulkano::sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode};
use nannou::vulkano::image::attachment::AttachmentImage;
use nannou::vulkano::sync::GpuFuture;
use nannou::math::Matrix4;
use nannou::vulkano::buffer::CpuBufferPool;

use crate::Model;

pub struct Warp {
    render_pass: Arc<RenderPassAbstract + Send + Sync>,
    pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
    framebuffer: RefCell<ViewFramebuffer>,
    uniform_buffer: CpuBufferPool<vs::ty::Data>,
    sampler: Arc<Sampler>,
}

#[derive(Debug, Clone)]
struct Vertex {
    position: [f32; 3],
    v_tex_coords: [f32; 2],
    
}

nannou::vulkano::impl_vertex!(Vertex, position, v_tex_coords);

pub(crate) fn warp(app: &App) -> Warp {
    let device = app.main_window().swapchain().device().clone();

    let uniform_buffer = CpuBufferPool::<vs::ty::Data>::new(device.clone(), BufferUsage::all());
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
                    samples: app.main_window().msaa_samples(),
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

    let sampler = Sampler::new(
        device.clone(),
        Filter::Linear,
        Filter::Linear,
        MipmapMode::Linear,
        SamplerAddressMode::ClampToEdge,
        SamplerAddressMode::ClampToEdge,
        SamplerAddressMode::ClampToEdge,
        0.0,
        1.0,
        0.0,
        1.0,
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

    let framebuffer = RefCell::new(ViewFramebuffer::default());

    Warp {
        render_pass,
        pipeline,
        framebuffer,
        sampler,
        uniform_buffer,
    }
}

pub(crate) fn view(app: &App, model: &Model, inter_image: Arc<AttachmentImage>, frame: Frame) -> Frame {
    let Model {
        warp,
        controls,
        ..
    } = model;
    
    let [w, h] = frame.swapchain_image().dimensions();
    let half_w = w as f32 / 2.0;
    let half_h = h as f32 / 2.0;
    let ref corners = controls.corners;
    
    let remap = | a: &Point2| -> Point2 {
        pt2(a.x / half_w as f32, a.y / half_h as f32)
    };

    let tl = remap(&corners.top_left.pos);
    let tr = remap(&corners.top_right.pos);
    let bl = remap(&corners.bottom_left.pos);
    let br = remap(&corners.bottom_right.pos);

    let src_dims = [
    [-1.0, -1.0],
    [1.0, -1.0],
    [1.0, 1.0],
    [-1.0, 1.0]];
    let dst_dims = [[tl.x, -tl.y],
    [tr.x, -tr.y],
    [br.x, -br.y],
    [bl.x, -bl.y]];

    let h_matrix = homography::find_homography(src_dims, dst_dims);
    let h_matrix: &Matrix4<f32> = From::from(&h_matrix);
    let h_matrix = h_matrix.clone();

    let uniform_data = vs::ty::Data {
        homography: h_matrix.into(),
    };

    let (vertex_buffer, buffer_future) = ImmutableBuffer::from_iter(
        [
        Vertex {
            position: [-1.0, -1.0, 0.0],
            v_tex_coords: [0.0, 0.0],
            
        },
        Vertex {
            position: [-1.0, 1.0, 0.0],
            v_tex_coords: [0.0, 1.0],
        },
        Vertex {
            position: [1.0, -1.0, 0.0],
            v_tex_coords: [1.0, 0.0],
        },
        Vertex {
            position: [1.0, 1.0, 0.0],
            v_tex_coords: [1.0, 1.0],
        },
        ]
        .iter()
        .cloned(),
        BufferUsage::all(),
        app.window(frame.window_id())
        .expect("no window for frame's window_id")
        .swapchain_queue()
        .clone(),
        )
            .unwrap();


    buffer_future
        .then_signal_fence_and_flush()
        .expect("failed to signal_fence_and_flush buffer and image creation future")
        .wait(None)
        .expect("failed to wait for buffer and image creation future");

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

    // Update framebuffer in case of window resize.
    warp.framebuffer.borrow_mut()
        .update(&frame, warp.render_pass.clone(), |builder, image| builder.add(image))
        .expect("framebuffer failed to create");

    let clear_values = vec![[0.0, 1.0, 0.0, 1.0].into()];

    let uniform_buffer_slice = warp.uniform_buffer.next(uniform_data).unwrap();
    
    let desciptor_set = Arc::new(
        PersistentDescriptorSet::start(warp.pipeline.clone(), 0)
            .add_sampled_image(inter_image.clone(), warp.sampler.clone())
            .expect("Failed to create desciptor set")
            .add_buffer(uniform_buffer_slice)
            .unwrap()
            .build()
            .expect("Failed to build desciptor set"),
    );

    frame
        .add_commands()
        .begin_render_pass(
            warp.framebuffer.borrow().as_ref().unwrap().clone(),
            false,
            clear_values,
        )
        .expect("Failed to start render pass")
        .draw(
            warp.pipeline.clone(),
            &dynamic_state,
            vec![vertex_buffer.clone()],
            desciptor_set.clone(),
            (),
        )
        .expect("Failed to draw")
        .end_render_pass()
        .expect("failed to add `end_render_pass` command");

    frame
}

mod vs {
    nannou::vulkano_shaders::shader! {
    ty: "vertex",
        src: "
#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 v_tex_coords;

layout(location = 0) out vec2 tex_coords;

layout(set = 0, binding = 1) uniform Data {
    mat4 homography;
} uniforms;

void main() {
    vec4 pos = uniforms.homography * vec4(position, 1.0);
    gl_Position = pos;
    tex_coords = v_tex_coords;
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

layout(set = 0, binding = 0) uniform sampler2D tex;

void main() {
   f_color = texture(tex, tex_coords);
}"
    }
}
