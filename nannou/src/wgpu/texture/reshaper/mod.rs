use crate::wgpu;

/// Reshapes a texture from its original size, sample_count and format to the destination size,
/// sample_count and format.
///
/// The `src_texture` must have the `TextureUsage::SAMPLED` enabled.
///
/// The `dst_texture` must have the `TextureUsage::OUTPUT_ATTACHMENT` enabled.
#[derive(Debug)]
pub struct Reshaper {
    _vs_mod: wgpu::ShaderModule,
    _fs_mod: wgpu::ShaderModule,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    sampler: wgpu::Sampler,
    uniform_buffer: Option<wgpu::Buffer>,
    vertex_buffer: wgpu::Buffer,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
struct Vertex {
    pub position: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone)]
struct Uniforms {
    sample_count: u32,
}

impl Reshaper {
    /// Construct a new `Reshaper`.
    pub fn new(
        device: &wgpu::Device,
        src_texture: &wgpu::TextureViewHandle,
        src_sample_count: u32,
        dst_sample_count: u32,
        dst_format: wgpu::TextureFormat,
    ) -> Self {
        // Load shader modules.
        let vs_mod = wgpu::shader_from_spirv_bytes(device, include_bytes!("shaders/vert.spv"));
        let fs_mod = match src_sample_count {
            1 => wgpu::shader_from_spirv_bytes(device, include_bytes!("shaders/frag.spv")),
            2 => wgpu::shader_from_spirv_bytes(device, include_bytes!("shaders/frag_msaa2.spv")),
            4 => wgpu::shader_from_spirv_bytes(device, include_bytes!("shaders/frag_msaa4.spv")),
            8 => wgpu::shader_from_spirv_bytes(device, include_bytes!("shaders/frag_msaa8.spv")),
            16 => wgpu::shader_from_spirv_bytes(device, include_bytes!("shaders/frag_msaa16.spv")),
            _ => wgpu::shader_from_spirv_bytes(device, include_bytes!("shaders/frag_msaa.spv")),
        };

        // Create the sampler for sampling from the source texture.
        let sampler = wgpu::SamplerBuilder::new().build(device);

        // Create the render pipeline.
        let bind_group_layout = bind_group_layout(device, src_sample_count);
        let pipeline_layout = pipeline_layout(device, &bind_group_layout);
        let render_pipeline = render_pipeline(
            device,
            &pipeline_layout,
            &vs_mod,
            &fs_mod,
            dst_sample_count,
            dst_format,
        );

        // Create the uniform buffer to pass the sample count if we don't have an unrolled resolve
        // fragment shader for it.
        let uniform_buffer = match unrolled_sample_count(src_sample_count) {
            true => None,
            false => {
                let uniforms = Uniforms {
                    sample_count: src_sample_count,
                };
                let buffer = device
                    .create_buffer_mapped(1, wgpu::BufferUsage::UNIFORM)
                    .fill_from_slice(&[uniforms]);
                Some(buffer)
            }
        };

        // Create the bind group.
        let bind_group = bind_group(
            device,
            &bind_group_layout,
            src_texture,
            &sampler,
            uniform_buffer.as_ref(),
        );

        // Create the vertex buffer.
        let vertex_buffer = device
            .create_buffer_mapped(VERTICES.len(), wgpu::BufferUsage::VERTEX)
            .fill_from_slice(&VERTICES[..]);

        Reshaper {
            _vs_mod: vs_mod,
            _fs_mod: fs_mod,
            bind_group_layout,
            bind_group,
            render_pipeline,
            sampler,
            uniform_buffer,
            vertex_buffer,
        }
    }

    /// Given an encoder, submits a render pass command for writing the source texture to the
    /// destination texture.
    pub fn encode_render_pass(
        &self,
        dst_texture: &wgpu::TextureViewHandle,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let mut render_pass = wgpu::RenderPassBuilder::new()
            .color_attachment(dst_texture, |color| color)
            .begin(encoder);
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffers(0, &[(&self.vertex_buffer, 0)]);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        let vertex_range = 0..VERTICES.len() as u32;
        let instance_range = 0..1;
        render_pass.draw(vertex_range, instance_range);
    }
}

impl wgpu::VertexDescriptor for Vertex {
    const STRIDE: wgpu::BufferAddress = std::mem::size_of::<Vertex>() as _;
    const ATTRIBUTES: &'static [wgpu::VertexAttributeDescriptor] =
        &[wgpu::VertexAttributeDescriptor {
            format: wgpu::VertexFormat::Float2,
            offset: 0,
            shader_location: 0,
        }];
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

// We provide pre-prepared fragment shaders with unrolled resolves for common sample counts.
fn unrolled_sample_count(sample_count: u32) -> bool {
    match sample_count {
        1 | 2 | 4 | 8 | 16 => true,
        _ => false,
    }
}

fn bind_group_layout(device: &wgpu::Device, src_sample_count: u32) -> wgpu::BindGroupLayout {
    let mut builder = wgpu::BindGroupLayoutBuilder::new()
        .sampled_texture(
            wgpu::ShaderStage::FRAGMENT,
            src_sample_count > 1,
            wgpu::TextureViewDimension::D2,
        )
        .sampler(wgpu::ShaderStage::FRAGMENT);
    if !unrolled_sample_count(src_sample_count) {
        builder = builder.uniform_buffer(wgpu::ShaderStage::FRAGMENT, false);
    }
    builder.build(device)
}
fn bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    texture: &wgpu::TextureViewHandle,
    sampler: &wgpu::Sampler,
    uniform_buffer: Option<&wgpu::Buffer>,
) -> wgpu::BindGroup {
    let mut builder = wgpu::BindGroupBuilder::new()
        .texture_view(texture)
        .sampler(sampler);
    if let Some(buffer) = uniform_buffer {
        builder = builder.buffer::<Uniforms>(buffer, 0..1);
    }
    builder.build(device, layout)
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
    dst_sample_count: u32,
    dst_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    wgpu::RenderPipelineBuilder::from_layout(layout, vs_mod)
        .fragment_shader(fs_mod)
        .color_format(dst_format)
        .color_blend(wgpu::BlendDescriptor::REPLACE)
        .alpha_blend(wgpu::BlendDescriptor::REPLACE)
        .add_vertex_buffer::<Vertex>()
        .primitive_topology(wgpu::PrimitiveTopology::TriangleStrip)
        .index_format(wgpu::IndexFormat::Uint16)
        .sample_count(dst_sample_count)
        .build(device)
}
