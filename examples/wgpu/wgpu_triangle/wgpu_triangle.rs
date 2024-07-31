//! A simple demonstration on how to create and draw with a custom wgpu render pipeline in nannou!
//!
//! The aim of this example is not to show the simplest way of drawing a triangle in nannou, but
//! rather provide a reference on how to get started creating your own rendering pipeline from
//! scratch. While nannou's provided graphics-y APIs can do a lot of things quite efficiently,
//! writing a custom pipeline that does only exactly what you need it to can sometimes result in
//! better performance.

use std::sync::Arc;
use nannou::prelude::*;

#[derive(Clone)]
struct Model {
    bind_group: Arc<wgpu::BindGroup>,
    render_pipeline: Arc<wgpu::RenderPipeline>,
    vertex_buffer: Arc<wgpu::Buffer>,
}

// The vertex type that we will use to represent a point on our triangle.
#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 2],
}

// The vertices that make up our triangle.
const VERTICES: [Vertex; 3] = [
    Vertex {
        position: [-0.5, -0.25],
    },
    Vertex {
        position: [0.0, 0.5],
    },
    Vertex {
        position: [0.25, -0.1],
    },
];

fn main() {
    nannou::app(model)
        .render(render)
        .run();
}

fn model(app: &App) -> Model {
    let w_id = app.new_window::<Model>().hdr(true).size(512, 512).build();

    // The gpu device associated with the window's swapchain
    let window = app.window(w_id);
    let device = window.device();
    let format = Frame::TEXTURE_FORMAT;
    let sample_count = window.msaa_samples();

    // Load shader modules.
    let vs_desc = wgpu::include_wgsl!("shaders/vs.wgsl");
    let fs_desc = wgpu::include_wgsl!("shaders/fs.wgsl");
    let vs_mod = device.create_shader_module(vs_desc);
    let fs_mod = device.create_shader_module(fs_desc);

    // Create the vertex buffer.
    let vertices_bytes = vertices_as_bytes(&VERTICES[..]);
    let usage = wgpu::BufferUsages::VERTEX;
    let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: vertices_bytes,
        usage,
    });

    // Create the render pipeline.
    let bind_group_layout = wgpu::BindGroupLayoutBuilder::new().build(&device);
    let bind_group = wgpu::BindGroupBuilder::new().build(&device, &bind_group_layout);
    let pipeline_layout = wgpu::create_pipeline_layout(&device, None, &[&bind_group_layout], &[]);
    let render_pipeline = wgpu::RenderPipelineBuilder::from_layout(&pipeline_layout, &vs_mod)
        .fragment_shader(&fs_mod)
        .color_format(format)
        .add_vertex_buffer::<Vertex>(&wgpu::vertex_attr_array![0 => Float32x2])
        .sample_count(sample_count)
        .build(&device);

    Model {
        bind_group: Arc::new(bind_group),
        vertex_buffer: Arc::new(vertex_buffer),
        render_pipeline: Arc::new(render_pipeline),
    }
}

// Draw the state of your `Model` into the given `Frame` here.
fn render(_app: &RenderApp, model: &Model, frame: Frame) {
    // Using this we will encode commands that will be submitted to the GPU.
    let mut encoder = frame.command_encoder();

    // The render pass can be thought of a single large command consisting of sub commands. Here we
    // begin a render pass that outputs to the frame's texture. Then we add sub-commands for
    // setting the bind group, render pipeline, vertex buffers and then finally drawing.
    let mut render_pass = wgpu::RenderPassBuilder::new()
        .color_attachment(frame.resolve_target_view().unwrap(), |color| color)
        .begin(&mut encoder);
    render_pass.set_bind_group(0, &model.bind_group, &[]);
    render_pass.set_pipeline(&model.render_pipeline);
    render_pass.set_vertex_buffer(0, model.vertex_buffer.slice(..));

    // We want to draw the whole range of vertices, and we're only drawing one instance of them.
    let vertex_range = 0..VERTICES.len() as u32;
    let instance_range = 0..1;
    render_pass.draw(vertex_range, instance_range);

    // Now we're done! The commands we added will be submitted after `view` completes.
}

// See the `nannou::wgpu::bytes` documentation for why this is necessary.
fn vertices_as_bytes(data: &[Vertex]) -> &[u8] {
    unsafe { wgpu::bytes::from_slice(data) }
}
