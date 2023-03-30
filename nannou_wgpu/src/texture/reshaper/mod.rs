use crate::{self as wgpu, util::DeviceExt, BufferInitDescriptor};

/// Reshapes a texture from its original size, sample_count and format to the destination size,
/// sample_count and format.
///
/// The `src_texture` must have the `TextureUsages::SAMPLED` enabled.
///
/// The `dst_texture` must have the `TextureUsages::RENDER_ATTACHMENT` enabled.
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
        src_sample_type: wgpu::TextureSampleType,
        dst_sample_count: u32,
        dst_format: wgpu::TextureFormat,
    ) -> Self {
        // Load shader modules.
        let vs_desc = wgpu::include_wgsl!("shaders/vs.wgsl");
        let fs_desc = match src_sample_count {
            1 => wgpu::include_wgsl!("shaders/fs.wgsl"),
            2 => wgpu::include_wgsl!("shaders/fs_msaa2.wgsl"),
            4 => wgpu::include_wgsl!("shaders/fs_msaa4.wgsl"),
            8 => wgpu::include_wgsl!("shaders/fs_msaa8.wgsl"),
            16 => wgpu::include_wgsl!("shaders/fs_msaa16.wgsl"),
            _ => wgpu::include_wgsl!("shaders/fs_msaa.wgsl"),
        };
        let vs_mod = device.create_shader_module(vs_desc);
        let fs_mod = device.create_shader_module(fs_desc);

        // Create the sampler for sampling from the source texture.
        let sampler_desc = wgpu::SamplerBuilder::new().into_descriptor();
        let sampler_filtering = wgpu::sampler_filtering(&sampler_desc);
        let sampler = device.create_sampler(&sampler_desc);

        // Create the render pipeline.
        let bind_group_layout =
            bind_group_layout(device, src_sample_count, src_sample_type, sampler_filtering);
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
                let uniforms_bytes = uniforms_as_bytes(&uniforms);
                let usage = wgpu::BufferUsages::UNIFORM;
                let buffer = device.create_buffer_init(&BufferInitDescriptor {
                    label: None,
                    contents: &uniforms_bytes,
                    usage,
                });
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
        let vertices_bytes = vertices_as_bytes(&VERTICES[..]);
        let vertex_usage = wgpu::BufferUsages::VERTEX;
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: vertices_bytes,
            usage: vertex_usage,
        });

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
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        let vertex_range = 0..VERTICES.len() as u32;
        let instance_range = 0..1;
        render_pass.draw(vertex_range, instance_range);
    }
}

const VERTICES: [Vertex; 4] = [
    Vertex {
        position: [-1.0, 1.0],
    },
    Vertex {
        position: [-1.0, -1.0],
    },
    Vertex {
        position: [1.0, 1.0],
    },
    Vertex {
        position: [1.0, -1.0],
    },
];

// We provide pre-prepared fragment shaders with unrolled resolves for common sample counts.
fn unrolled_sample_count(sample_count: u32) -> bool {
    match sample_count {
        1 | 2 | 4 | 8 | 16 => true,
        _ => false,
    }
}

fn bind_group_layout(
    device: &wgpu::Device,
    src_sample_count: u32,
    src_sample_type: wgpu::TextureSampleType,
    sampler_filtering: bool,
) -> wgpu::BindGroupLayout {
    let mut builder = wgpu::BindGroupLayoutBuilder::new()
        .texture(
            wgpu::ShaderStages::FRAGMENT,
            src_sample_count > 1,
            wgpu::TextureViewDimension::D2,
            src_sample_type,
        )
        .sampler(wgpu::ShaderStages::FRAGMENT, sampler_filtering);
    if !unrolled_sample_count(src_sample_count) {
        builder = builder.uniform_buffer(wgpu::ShaderStages::FRAGMENT, false);
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
        label: Some("nannou_reshaper"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
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
        .color_blend(wgpu::BlendComponent::REPLACE)
        .alpha_blend(wgpu::BlendComponent::REPLACE)
        .add_vertex_buffer::<Vertex>(&wgpu::vertex_attr_array![0 => Float32x2])
        .primitive_topology(wgpu::PrimitiveTopology::TriangleStrip)
        .sample_count(dst_sample_count)
        .build(device)
}

fn uniforms_as_bytes(uniforms: &Uniforms) -> &[u8] {
    unsafe { wgpu::bytes::from(uniforms) }
}

fn vertices_as_bytes(data: &[Vertex]) -> &[u8] {
    unsafe { wgpu::bytes::from_slice(data) }
}
