/// Writes a texture to another texture of the same dimensions but with a different format.
///
/// The `src_texture` must have the `TextureUsage::SAMPLED` enabled.
///
/// The `dst_texture` must have the `TextureUsage::OUTPUT_ATTACHMENT` enabled.
///
/// Both `src_texture` and `dst_texture` must have the same dimensions.
///
/// Both textures should **not** be multisampled. *Note: Please open an issue if you would like
/// support for multisampled source textures as it should be quite trivial to add.*
#[derive(Debug)]
pub struct FormatConverter {
    _vs_mod: wgpu::ShaderModule,
    _fs_mod: wgpu::ShaderModule,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    sampler: wgpu::Sampler,
    vertex_buffer: wgpu::Buffer,
}

impl FormatConverter {
    /// Construct a new `FormatConverter`.
    pub fn new(
        device: &wgpu::Device,
        src_texture: &wgpu::TextureView,
        dst_format: wgpu::TextureFormat,
    ) -> Self {
        // Load shader modules.
        let vs = include_bytes!("shaders/vert.spv");
        let vs_spirv = wgpu::read_spirv(std::io::Cursor::new(&vs[..]))
            .expect("failed to read hard-coded SPIRV");
        let vs_mod = device.create_shader_module(&vs_spirv);
        let fs = include_bytes!("shaders/frag.spv");
        let fs_spirv = wgpu::read_spirv(std::io::Cursor::new(&fs[..]))
            .expect("failed to read hard-coded SPIRV");
        let fs_mod = device.create_shader_module(&fs_spirv);

        // Create the sampler for sampling from the source texture.
        let sampler_desc = sampler_desc();
        let sampler = device.create_sampler(&sampler_desc);

        // Create the render pipeline.
        let bind_group_layout = bind_group_layout(device);
        let pipeline_layout = pipeline_layout(device, &bind_group_layout);
        let render_pipeline =
            render_pipeline(device, &pipeline_layout, &vs_mod, &fs_mod, dst_format);

        // Create the bind group.
        let bind_group = bind_group(device, &bind_group_layout, src_texture, &sampler);

        // Create the vertex buffer.
        let vertex_buffer = device
            .create_buffer_mapped(VERTICES.len(), wgpu::BufferUsage::VERTEX)
            .fill_from_slice(&VERTICES[..]);

        FormatConverter {
            _vs_mod: vs_mod,
            _fs_mod: fs_mod,
            bind_group_layout,
            bind_group,
            render_pipeline,
            sampler,
            vertex_buffer,
        }
    }

    /// Given an encoder, submits a render pass command for writing the source texture to the
    /// destination texture.
    pub fn encode_render_pass(
        &self,
        dst_texture: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let vertex_range = 0..VERTICES.len() as u32;
        let instance_range = 0..1;
        let render_pass_desc = wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: dst_texture,
                resolve_target: None,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: wgpu::Color::TRANSPARENT,
            }],
            depth_stencil_attachment: None,
        };
        let mut render_pass = encoder.begin_render_pass(&render_pass_desc);
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffers(0, &[(&self.vertex_buffer, 0)]);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(vertex_range, instance_range);
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
struct Vertex {
    pub position: [f32; 2],
}

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

fn vertex_attrs() -> [wgpu::VertexAttributeDescriptor; 1] {
    [wgpu::VertexAttributeDescriptor {
        format: wgpu::VertexFormat::Float2,
        offset: 0,
        shader_location: 0,
    }]
}

fn sampler_desc() -> wgpu::SamplerDescriptor {
    wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Linear,
        lod_min_clamp: -100.0,
        lod_max_clamp: 100.0,
        compare_function: wgpu::CompareFunction::Always,
    }
}

fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
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

fn bind_group(
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

fn pipeline_layout(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::PipelineLayout {
    let desc = wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[&bind_group_layout],
    };
    device.create_pipeline_layout(&desc)
}

fn render_pipeline(
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
        primitive_topology: wgpu::PrimitiveTopology::TriangleStrip,
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
