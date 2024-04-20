//! The same as the `wgpu_triangle` example, but demonstrates how to draw directly to the swap
//! chain texture (`RawFrame`) rather than to nannou's intermediary `Frame`.

use nannou::prelude::*;

struct Model {
    bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 2],
}

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
    nannou::app(model).run();
}

fn model(app: &App) -> Model {
    let w_id = app
        .new_window()
        .size(512, 512)
        .raw_view(view)
        .build()
        .unwrap();

    let window = app.window(w_id).unwrap();
    let device = window.device();
    // NOTE: We are drawing to the swap chain format, rather than the `Frame::TEXTURE_FORMAT`.
    let format = window.surface_configuration().format;

    let vs_desc = wgpu::include_wgsl!("shaders/vs.wgsl");
    let fs_desc = wgpu::include_wgsl!("shaders/fs.wgsl");
    let vs_mod = device.create_shader_module(vs_desc);
    let fs_mod = device.create_shader_module(fs_desc);

    let vertices_bytes = vertices_as_bytes(&VERTICES[..]);
    let usage = wgpu::BufferUsages::VERTEX;
    let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: vertices_bytes,
        usage,
    });

    let bind_group_layout = wgpu::BindGroupLayoutBuilder::new().build(device);
    let bind_group = wgpu::BindGroupBuilder::new().build(device, &bind_group_layout);
    let pipeline_layout = wgpu::create_pipeline_layout(device, None, &[&bind_group_layout], &[]);
    let render_pipeline = wgpu::RenderPipelineBuilder::from_layout(&pipeline_layout, &vs_mod)
        .fragment_shader(&fs_mod)
        .color_format(format)
        .add_vertex_buffer::<Vertex>(&wgpu::vertex_attr_array![0 => Float32x2])
        .build(device);

    Model {
        bind_group,
        vertex_buffer,
        render_pipeline,
    }
}

fn view(_app: &App, model: &Model, frame: RawFrame) {
    let mut encoder = frame.command_encoder();
    let mut render_pass = wgpu::RenderPassBuilder::new()
        // NOTE: We are drawing to the swap chain texture directly rather than the intermediary
        // texture as in `wgpu_triangle`.
        .color_attachment(frame.swap_chain_texture(), |color| color)
        .begin(&mut encoder);
    render_pass.set_pipeline(&model.render_pipeline);
    render_pass.set_bind_group(0, &model.bind_group, &[]);
    render_pass.set_vertex_buffer(0, model.vertex_buffer.slice(..));
    let vertex_range = 0..VERTICES.len() as u32;
    let instance_range = 0..1;
    render_pass.draw(vertex_range, instance_range);
}

// See the `nannou::wgpu::bytes` documentation for why this is necessary.
fn vertices_as_bytes(data: &[Vertex]) -> &[u8] {
    unsafe { wgpu::bytes::from_slice(data) }
}
