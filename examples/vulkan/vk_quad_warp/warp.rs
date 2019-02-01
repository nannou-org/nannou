use nannou::prelude::*;
use nannou::vulkano;
use std::cell::RefCell;
use std::sync::Arc;
use crate::homography;

use nannou::vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, ImmutableBuffer};
use nannou::vulkano::command_buffer::DynamicState;
use nannou::vulkano::descriptor::descriptor_set::{DescriptorSet, PersistentDescriptorSet};
use nannou::vulkano::device::DeviceOwned;
use nannou::vulkano::format::Format;
use nannou::vulkano::framebuffer::{RenderPassAbstract, Subpass};
use nannou::vulkano::image::{Dimensions, ImmutableImage};
use nannou::vulkano::pipeline::viewport::Viewport;
use nannou::vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use nannou::vulkano::sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode};
use nannou::window::SwapchainFramebuffers;
use nannou::vulkano::image::attachment::AttachmentImage;
use nannou::vulkano::sync::GpuFuture;
use nannou::math::Matrix4;
use nannou::math::cgmath;
use nannou::vulkano::buffer::CpuBufferPool;
use std::cmp::Ordering::Equal;

use crate::Model;

pub struct Warp {
    render_pass: Arc<RenderPassAbstract + Send + Sync>,
    pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
    framebuffers: RefCell<SwapchainFramebuffers>,
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

    let framebuffers = RefCell::new(SwapchainFramebuffers::default());

    Warp {
        render_pass,
        pipeline,
        framebuffers,
        sampler,
        uniform_buffer,
    }
}

pub(crate) fn view(app: &App, model: &Model, inter_image: Arc<AttachmentImage>,frame: Frame) -> Frame {
    let Model {
        warp,
        controls,
        ..
    } = model;

    let n_corners = controls.corners.normalized();
    let inter_dims = inter_image.dimensions();
    /*
    let src_dims = [[0.0, 0.0],
    [inter_dims[0] as f32, 0.0],
    [inter_dims[0] as f32, inter_dims[1] as f32],
    [0.0, inter_dims[1] as f32]];


    let src_dims = [[-1.0, 1.0],
    [1.0, 1.0],
    [1.0, -1.0],
    [-1.0, -1.0]];
    let dst_dims = [[n_corners[0].x, n_corners[0].y],
    [n_corners[1].x, n_corners[1].y],
    [n_corners[3].x, n_corners[3].y],
    [n_corners[2].x, n_corners[2].y]];
    let w = inter_dims[0] as f32 / 2.0;
    let h = inter_dims[1] as f32 / 2.0;
    let src_dims = [
    [-w, h],
    [w, h],
    [w, -h],
    [-w, -h]];

    let tl = controls.corners.top_left.pos;
    let tr = controls.corners.top_right.pos;
    let br = controls.corners.bottom_right.pos;
    let bl = controls.corners.bottom_left.pos;
    let dst_dims = [[tl.x, tl.y],
    [tr.x, tr.y],
    [br.x, br.y],
    [bl.x, bl.y]];

    let src_dims = [[0.0, 0.0],
    [1.0, 0.0],
    [1.0, 1.0],
    [0.0, 1.0]];
    let src_dims = [[-1.0, 1.0],
    [1.0, 1.0],
    [1.0, -1.0],
    [-1.0, -1.0]];
    let min_x = n_corners.iter()
        .map(|c| c.x)
        .min_by(|a, b| a.partial_cmp(b).unwrap_or(Equal))
        .unwrap();
    let max_x = n_corners.iter()
        .map(|c| c.x)
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(Equal))
        .unwrap();
    let min_y = n_corners.iter()
        .map(|c| c.y)
        .min_by(|a, b| a.partial_cmp(b).unwrap_or(Equal))
        .unwrap();
    let max_y = n_corners.iter()
        .map(|c| c.y)
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(Equal))
        .unwrap();
    let src_dims = [[0.0, 0.0],
    [max_x, 0.0],
    [max_x, max_y],
    [0.0, max_y]];
    let dst_dims = [[n_corners[0].x, n_corners[0].y],
    [n_corners[1].x, n_corners[1].y],
    [n_corners[3].x, n_corners[3].y],
    [n_corners[2].x, n_corners[2].y]];
    let src_dims = [[0.0, 0.0],
    [inter_dims[0] as f32, 0.0],
    [inter_dims[0] as f32, inter_dims[1] as f32],
    [0.0, inter_dims[1] as f32]];
    */
    let inter_dims = inter_image.dimensions();
    let w = inter_dims[0] as f32 / 2.0;
    let h = inter_dims[1] as f32 / 2.0;
    let src_dims = [
    [-1.0, -1.0],
    [1.0, -1.0],
    [1.0, 1.0],
    [-1.0, 1.0]];
    let tl = n_corners[0];
    let tr = n_corners[1];
    let bl = n_corners[2];
    let br = n_corners[3];
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
    let tl = h_matrix * cgmath::Vector4::new(n_corners[0].x, n_corners[0].y, 0.0, 1.0);
    let tr = h_matrix * cgmath::Vector4::new(n_corners[1].x, n_corners[1].y, 0.0, 1.0);
    let bl = h_matrix * cgmath::Vector4::new(n_corners[2].x, n_corners[2].y, 0.0, 1.0);
    let br = h_matrix * cgmath::Vector4::new(n_corners[3].x, n_corners[3].y, 0.0, 1.0);
    let (vertex_buffer, buffer_future) = ImmutableBuffer::from_iter(
        [
        Vertex {
            //position: [tl.x, tl.y, tl.z],
            //position: [n_corners[0].x, n_corners[0].y, 0.0],
            position: [-1.0, -1.0, 0.0],
            v_tex_coords: [0.0, 0.0],
            
        },
        Vertex {
            //position: [bl.x, bl.y, bl.z],
            //position: [n_corners[2].x, n_corners[2].y, 0.0],
            position: [-1.0, 1.0, 0.0],
            v_tex_coords: [0.0, 1.0],
        },
        Vertex {
            //position: [tr.x, tr.y, tr.z],
            //position: [n_corners[1].x, n_corners[1].y, 0.0],
            position: [1.0, -1.0, 0.0],
            v_tex_coords: [1.0, 0.0],
        },
        Vertex {
            //position: [br.x, br.y, br.z],
            //position: [n_corners[3].x, n_corners[3].y, 0.0],
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

    // Update framebuffers so that count matches swapchain image count and dimensions match.
    warp.framebuffers.borrow_mut()
        .update(&frame, warp.render_pass.clone(), |builder, image| builder.add(image))
        .expect("framebuffer failed to create");

    let clear_values = vec![[0.0, 1.0, 0.0, 1.0].into()];

    let push_constants = fs::ty::PushConstantData {
        time: app.time * 30.0,
    };

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
            warp.framebuffers.borrow()[frame.swapchain_image_index()].clone(),
            false,
            clear_values,
        )
        .expect("Failed to start render pass")
        .draw(
            warp.pipeline.clone(),
            &dynamic_state,
            vec![vertex_buffer.clone()],
            desciptor_set.clone(),
            push_constants,
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
    vec4 flipped = uniforms.homography * vec4(position, 1.0);
    //flipped.y *= -1.0;
    gl_Position = flipped;
    //gl_Position = vec4(position, 1.0);
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
layout(set = 0, binding = 1) uniform Data {
    mat4 homography;
} uniforms;

layout(push_constant) uniform PushConstantData {
    float time;
} pc;

void main() {
    //vec4 c = vec4( abs(tex_coords.x + sin(pc.time)), tex_coords.x, tex_coords.y * abs(cos(pc.time)), 1.0);    
    //vec4 tx = uniforms.homography * vec4(tex_coords, 0.0, 1.0);
   f_color = texture(tex, tex_coords);
   //f_color = texture(tex, tx.xy);
}"
    }
}
