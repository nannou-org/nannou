use nannou::image::GenericImageView;
use nannou::prelude::*;

struct Model {
    bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
}

// The vertex type that we will use to represent a point on our triangle.
#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 2],
}

// The vertices that make up the rectangle to which the image will be drawn.
const VERTICES: [Vertex; 4] = [
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
];

impl wgpu::VertexDescriptor for Vertex {
    const STRIDE: wgpu::BufferAddress = std::mem::size_of::<Vertex>() as _;
    const ATTRIBUTES: &'static [wgpu::VertexAttributeDescriptor] =
        &[wgpu::VertexAttributeDescriptor {
            format: wgpu::VertexFormat::Float2,
            offset: 0,
            shader_location: 0,
        }];
}

fn main() {
    nannou::app(model).run();
}

fn model(app: &App) -> Model {
    // Load the image.
    let logo_path = app.assets_path().unwrap().join("images").join("nannou.png");
    let image = image::open(logo_path).unwrap();
    let (img_w, img_h) = image.dimensions();

    let w_id = app
        .new_window()
        .size(img_w, img_h)
        .view(view)
        .build()
        .unwrap();
    let window = app.window(w_id).unwrap();
    let device = window.swap_chain_device();
    let format = Frame::TEXTURE_FORMAT;
    let msaa_samples = window.msaa_samples();

    let vs = include_bytes!("shaders/vert.spv");
    let vs_spirv =
        wgpu::read_spirv(std::io::Cursor::new(&vs[..])).expect("failed to read hard-coded SPIRV");
    let vs_mod = device.create_shader_module(&vs_spirv);
    let fs = include_bytes!("shaders/frag.spv");
    let fs_spirv =
        wgpu::read_spirv(std::io::Cursor::new(&fs[..])).expect("failed to read hard-coded SPIRV");
    let fs_mod = device.create_shader_module(&fs_spirv);

    // Ensure the image.
    let image_rgba = image.into_rgba();
    let texture = {
        // The wgpu device queue used to load the image data.
        let mut queue = window.swap_chain_queue().lock().unwrap();
        // Describe how we will use the texture so that the GPU may handle it efficiently.
        let usage = wgpu::TextureUsage::SAMPLED;
        wgpu::Texture::load_from_image_buffer(device, &mut *queue, usage, &image_rgba)
    };
    let texture_view = texture.create_default_view();

    // Create the sampler for sampling from the source texture.
    let sampler = wgpu::SamplerBuilder::new().build(device);

    let bind_group_layout = create_bind_group_layout(device);
    let bind_group = create_bind_group(device, &bind_group_layout, &texture_view, &sampler);
    let pipeline_layout = create_pipeline_layout(device, &bind_group_layout);
    let render_pipeline = create_render_pipeline(
        device,
        &pipeline_layout,
        &vs_mod,
        &fs_mod,
        format,
        msaa_samples,
    );

    // Create the vertex buffer.
    let vertex_buffer = device
        .create_buffer_mapped(VERTICES.len(), wgpu::BufferUsage::VERTEX)
        .fill_from_slice(&VERTICES[..]);

    Model {
        bind_group,
        vertex_buffer,
        render_pipeline,
    }
}

fn view(_app: &App, model: &Model, frame: Frame) {
    let mut encoder = frame.command_encoder();

    let render_pass_desc = wgpu::RenderPassDescriptor {
        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
            attachment: frame.texture_view(),
            resolve_target: None,
            load_op: wgpu::LoadOp::Clear,
            store_op: wgpu::StoreOp::Store,
            clear_color: wgpu::Color::TRANSPARENT,
        }],
        depth_stencil_attachment: None,
    };

    let mut render_pass = encoder.begin_render_pass(&render_pass_desc);
    render_pass.set_bind_group(0, &model.bind_group, &[]);
    render_pass.set_pipeline(&model.render_pipeline);
    render_pass.set_vertex_buffers(0, &[(&model.vertex_buffer, 0)]);

    let vertex_range = 0..VERTICES.len() as u32;
    let instance_range = 0..1;
    render_pass.draw(vertex_range, instance_range);
}

fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    let texture_binding = wgpu::BindGroupLayoutBinding {
        binding: 0,
        visibility: wgpu::ShaderStage::FRAGMENT,
        ty: wgpu::BindingType::SampledTexture {
            multisampled: false,
            dimension: wgpu::TextureViewDimension::D2,
        },
    };
    let sampler_binding = wgpu::BindGroupLayoutBinding {
        binding: 1,
        visibility: wgpu::ShaderStage::FRAGMENT,
        ty: wgpu::BindingType::Sampler,
    };
    let bindings = &[texture_binding, sampler_binding];
    let desc = wgpu::BindGroupLayoutDescriptor { bindings };
    device.create_bind_group_layout(&desc)
}

fn create_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    texture: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    let texture_binding = wgpu::Binding {
        binding: 0,
        resource: wgpu::BindingResource::TextureView(&texture),
    };
    let sampler_binding = wgpu::Binding {
        binding: 1,
        resource: wgpu::BindingResource::Sampler(&sampler),
    };
    let bindings = &[texture_binding, sampler_binding];
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

fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    vs_mod: &wgpu::ShaderModule,
    fs_mod: &wgpu::ShaderModule,
    dst_format: wgpu::TextureFormat,
    sample_count: u32,
) -> wgpu::RenderPipeline {
    wgpu::RenderPipelineBuilder::from_layout(layout, vs_mod)
        .fragment_shader(fs_mod)
        .color_format(dst_format)
        .add_vertex_buffer::<Vertex>()
        .sample_count(sample_count)
        .primitive_topology(wgpu::PrimitiveTopology::TriangleStrip)
        .build(device)
}
