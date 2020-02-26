use nannou::prelude::*;

struct Model {
    bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
}

// The vertex type that we will use to represent a point on our triangle.
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

fn main() {
    nannou::app(model).run();
}

fn model(app: &App) -> Model {
    let w_id = app.new_window().size(512, 512).view(view).build().unwrap();

    // The gpu device associated with the window's swapchain
    let window = app.window(w_id).unwrap();
    let device = window.swap_chain_device();
    let format = Frame::TEXTURE_FORMAT;
    let msaa_samples = window.msaa_samples();

    // Load shader modules.
    let vs = include_bytes!("shaders/vert.spv");
    let vs_spirv =
        wgpu::read_spirv(std::io::Cursor::new(&vs[..])).expect("failed to read hard-coded SPIRV");
    let vs_mod = device.create_shader_module(&vs_spirv);
    let fs = include_bytes!("shaders/frag.spv");
    let fs_spirv =
        wgpu::read_spirv(std::io::Cursor::new(&fs[..])).expect("failed to read hard-coded SPIRV");
    let fs_mod = device.create_shader_module(&fs_spirv);

    // Load an image from disk, then load it to the GPU as a `Texture`.
    let logo_path = app.assets_path().unwrap().join("images").join("Nannou.png");
    let image = image::open(logo_path).unwrap().to_rgba();
    let texture = {
        let mut queue = window.swap_chain_queue().lock().unwrap();
        create_texture_from_rgba_image(device, &mut *queue, image)
    };
    let texture_view = texture.create_default_view();

    // Create the sampler for sampling from the source texture.
    let sampler = wgpu::SamplerBuilder::new().build(device);

    // Create the render pipeline.
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

// Draw the state of your `Model` into the given `Frame` here.
fn view(_app: &App, model: &Model, frame: &Frame) {
    // Using this we will encode commands that will be submitted to the GPU.
    let mut encoder = frame.command_encoder();

    // A render pass describes how to draw to an output "attachment".
    let render_pass_desc = wgpu::RenderPassDescriptor {
        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
            attachment: frame.texture(),
            resolve_target: None,
            load_op: wgpu::LoadOp::Clear,
            store_op: wgpu::StoreOp::Store,
            clear_color: wgpu::Color::TRANSPARENT,
        }],
        depth_stencil_attachment: None,
    };

    // The render pass can be thought of a single large command consisting of sub commands.
    // Here we begin the render pass and add sub-commands for setting the bind group, render
    // pipeline, vertex buffers and then finally drawing.
    let mut render_pass = encoder.begin_render_pass(&render_pass_desc);
    render_pass.set_bind_group(0, &model.bind_group, &[]);
    render_pass.set_pipeline(&model.render_pipeline);
    render_pass.set_vertex_buffers(0, &[(&model.vertex_buffer, 0)]);

    // We want to draw the whole range of vertices, and we're only drawing one instance of them.
    let vertex_range = 0..VERTICES.len() as u32;
    let instance_range = 0..1;
    render_pass.draw(vertex_range, instance_range);

    // Now we're done! The commands we added will be submitted after `view` completes.
}

// TODO: Generalise this into a nice API exposed by nannou (e.g. `Texture::from_image`).
fn create_texture_from_rgba_image(
    device: &wgpu::Device,
    queue: &mut wgpu::Queue,
    image: image::RgbaImage,
) -> wgpu::Texture {
    const LOGO_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

    // Initialise the texture.
    let (width, height) = image.dimensions();
    let logo_tex = wgpu::TextureBuilder::new()
        .size([width, height])
        .format(LOGO_TEXTURE_FORMAT)
        .usage(wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST)
        .build(device);

    // Upload the pixel data.
    let data = &image.into_raw()[..];
    let buffer = device
        .create_buffer_mapped(data.len(), wgpu::BufferUsage::COPY_SRC)
        .fill_from_slice(data);

    // Submit command for copying pixel data to the texture.
    let pixel_size_bytes = 4; // Rgba8, as above.
    let buffer_copy_view = wgpu::BufferCopyView {
        buffer: &buffer,
        offset: 0,
        row_pitch: width * pixel_size_bytes,
        image_height: height,
    };
    let texture_copy_view = wgpu::TextureCopyView {
        texture: &logo_tex,
        mip_level: 0,
        array_layer: 0,
        origin: wgpu::Origin3d::ZERO,
    };
    let extent = wgpu::Extent3d {
        width: width,
        height: height,
        depth: 1,
    };
    let cmd_encoder_desc = wgpu::CommandEncoderDescriptor { todo: 0 };
    let mut encoder = device.create_command_encoder(&cmd_encoder_desc);
    encoder.copy_buffer_to_texture(buffer_copy_view, texture_copy_view, extent);
    queue.submit(&[encoder.finish()]);

    logo_tex
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
    sample_count: u32,
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
        primitive_topology: wgpu::PrimitiveTopology::TriangleStrip,
        color_states: &[color_state_desc],
        depth_stencil_state: None,
        index_format: wgpu::IndexFormat::Uint16,
        vertex_buffers: &[vertex_buffer_desc],
        sample_count,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    };
    device.create_render_pipeline(&desc)
}
