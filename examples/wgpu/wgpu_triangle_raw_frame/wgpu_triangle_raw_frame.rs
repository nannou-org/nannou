// The same as the `wgpu_triangle` example, but demonstrates how to draw directly to the swap chain
// texture (`RawFrame`) rather than to nannou's intermediary `Frame`.

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
    let device = window.swap_chain_device();
    // NOTE: We are drawing to the swap chain format, rather than the `Frame::TEXTURE_FORMAT`.
    let format = window.swap_chain_descriptor().format;

    let vs = include_bytes!("shaders/vert.spv");
    let vs_spirv =
        wgpu::read_spirv(std::io::Cursor::new(&vs[..])).expect("failed to read hard-coded SPIRV");
    let vs_mod = device.create_shader_module(&vs_spirv);
    let fs = include_bytes!("shaders/frag.spv");
    let fs_spirv =
        wgpu::read_spirv(std::io::Cursor::new(&fs[..])).expect("failed to read hard-coded SPIRV");
    let fs_mod = device.create_shader_module(&fs_spirv);

    let vertex_buffer = device
        .create_buffer_mapped(VERTICES.len(), wgpu::BufferUsage::VERTEX)
        .fill_from_slice(&VERTICES[..]);

    let bind_group_layout = create_bind_group_layout(device);
    let bind_group = create_bind_group(device, &bind_group_layout);
    let pipeline_layout = create_pipeline_layout(device, &bind_group_layout);
    let render_pipeline =
        create_render_pipeline(device, &pipeline_layout, &vs_mod, &fs_mod, format);

    Model {
        bind_group,
        vertex_buffer,
        render_pipeline,
    }
}

fn view(_app: &App, model: &Model, frame: RawFrame) {
    let vertex_range = 0..VERTICES.len() as u32;
    let instance_range = 0..1;
    let render_pass_desc = wgpu::RenderPassDescriptor {
        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
            // NOTE: We are drawing to the swap chain texture directly rather than the intermediary
            // texture as in `wgpu_triangle`.
            attachment: frame.swap_chain_texture(),
            resolve_target: None,
            load_op: wgpu::LoadOp::Clear,
            store_op: wgpu::StoreOp::Store,
            clear_color: wgpu::Color::TRANSPARENT,
        }],
        depth_stencil_attachment: None,
    };
    let mut encoder = frame.command_encoder();
    let mut render_pass = encoder.begin_render_pass(&render_pass_desc);
    render_pass.set_pipeline(&model.render_pipeline);
    render_pass.set_bind_group(0, &model.bind_group, &[]);
    render_pass.set_vertex_buffers(0, &[(&model.vertex_buffer, 0)]);
    render_pass.draw(vertex_range, instance_range);
}

fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    let bindings = &[];
    let desc = wgpu::BindGroupLayoutDescriptor { bindings };
    device.create_bind_group_layout(&desc)
}

fn create_bind_group(device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup {
    let bindings = &[];
    let desc = wgpu::BindGroupDescriptor { layout, bindings };
    device.create_bind_group(&desc)
}

fn create_pipeline_layout(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::PipelineLayout {
    let desc = wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[&bind_group_layout],
    };
    device.create_pipeline_layout(&desc)
}

fn vertex_attrs() -> [wgpu::VertexAttributeDescriptor; 1] {
    [wgpu::VertexAttributeDescriptor {
        format: wgpu::VertexFormat::Float2,
        offset: 0,
        shader_location: 0,
    }]
}

fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    vs_mod: &wgpu::ShaderModule,
    fs_mod: &wgpu::ShaderModule,
    dst_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let vs_desc = wgpu::ProgrammableStageDescriptor {
        module: &vs_mod,
        entry_point: "main",
    };
    let fs_desc = wgpu::ProgrammableStageDescriptor {
        module: &fs_mod,
        entry_point: "main",
    };
    let raster_desc = wgpu::RasterizationStateDescriptor {
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: wgpu::CullMode::None,
        depth_bias: 0,
        depth_bias_slope_scale: 0.0,
        depth_bias_clamp: 0.0,
    };
    let color_state_desc = wgpu::ColorStateDescriptor {
        format: dst_format,
        color_blend: wgpu::BlendDescriptor::REPLACE,
        alpha_blend: wgpu::BlendDescriptor::REPLACE,
        write_mask: wgpu::ColorWrite::ALL,
    };
    let vertex_attrs = vertex_attrs();
    let vertex_buffer_desc = wgpu::VertexBufferDescriptor {
        stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::InputStepMode::Vertex,
        attributes: &vertex_attrs[..],
    };
    let desc = wgpu::RenderPipelineDescriptor {
        layout,
        vertex_stage: vs_desc,
        fragment_stage: Some(fs_desc),
        rasterization_state: Some(raster_desc),
        primitive_topology: wgpu::PrimitiveTopology::TriangleList,
        color_states: &[color_state_desc],
        depth_stencil_state: None,
        index_format: wgpu::IndexFormat::Uint16,
        vertex_buffers: &[vertex_buffer_desc],
        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    };
    device.create_render_pipeline(&desc)
}
