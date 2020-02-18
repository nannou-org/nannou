use crate::draw;
use crate::frame::Frame;
use crate::math::{BaseFloat, NumCast};

/// A helper type aimed at simplifying the rendering of conrod primitives via wgpu.
#[derive(Debug)]
pub struct Renderer {
    _vs_mod: wgpu::ShaderModule,
    _fs_mod: wgpu::ShaderModule,
    render_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

#[derive(Debug)]
pub struct DrawError;

/// The `Vertex` type passed to the vertex shader.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct Vertex {
    /// The position of the vertex within vector space.
    ///
    /// [-1.0, 1.0, 0.0] is the leftmost, bottom position of the display.
    /// [1.0, -1.0, 0.0] is the rightmost, top position of the display.
    pub position: [f32; 3],
    /// A color associated with the `Vertex`.
    ///
    /// These values should be in the linear sRGB format.
    ///
    /// The way that the color is used depends on the `mode`.
    pub color: [f32; 4],
    /// The coordinates of the texture used by this `Vertex`.
    ///
    /// [0.0, 0.0] is the leftmost, bottom position of the texture.
    /// [1.0, 1.0] is the rightmost, top position of the texture.
    pub tex_coords: [f32; 2],
    // /// The mode with which the `Vertex` will be drawn within the fragment shader.
    // ///
    // /// `0` for rendering text.
    // /// `1` for rendering an image.
    // /// `2` for rendering non-textured 2D geometry.
    // ///
    // /// If any other value is given, the fragment shader will not output any color.
    // pub mode: u32,
}

impl Vertex {
    /// Create a vertex from the given mesh vertex.
    pub fn from_mesh_vertex<S>(
        v: draw::mesh::Vertex<S>,
        framebuffer_width: f32,
        framebuffer_height: f32,
        dpi_factor: f32,
    ) -> Self
    where
        S: BaseFloat,
    {
        let point = v.point();
        let x_f: f32 = NumCast::from(point.x).unwrap();
        let y_f: f32 = NumCast::from(point.y).unwrap();
        let z_f: f32 = NumCast::from(point.z).unwrap();
        // Map coords from (-fb_dim, +fb_dim) to (-1.0, 1.0)
        // In vulkan, *y* increases in the downwards direction, so we negate it.
        let x = 2.0 * x_f * dpi_factor / framebuffer_width;
        let y = -(2.0 * y_f * dpi_factor / framebuffer_height);
        let z = 2.0 * z_f * dpi_factor / framebuffer_height;
        let tex_x = NumCast::from(v.tex_coords.x).unwrap();
        let tex_y = NumCast::from(v.tex_coords.y).unwrap();
        let position = [x, y, z];
        let (r, g, b, a) = v.color.into();
        let color = [r, g, b, a];
        let tex_coords = [tex_x, tex_y];
        Vertex {
            position,
            color,
            tex_coords,
        }
    }
}

impl Renderer {
    /// Construct a new `Renderer`.
    pub fn new(
        device: &wgpu::Device,
        msaa_samples: u32,
        swap_chain_format: wgpu::TextureFormat,
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

        // Create the render pipeline.
        let bind_group_layout = bind_group_layout(device);
        let bind_group = bind_group(device, &bind_group_layout);
        let pipeline_layout = pipeline_layout(device, &bind_group_layout);
        let render_pipeline = render_pipeline(
            device,
            &pipeline_layout,
            &vs_mod,
            &fs_mod,
            swap_chain_format,
            msaa_samples,
        );
        let vertices = vec![];
        let indices = vec![];

        Self {
            _vs_mod: vs_mod,
            _fs_mod: fs_mod,
            render_pipeline,
            bind_group_layout,
            bind_group,
            vertices,
            indices,
        }
    }

    pub fn render_to_frame<S>(
        &mut self,
        device: &wgpu::Device,
        draw: &draw::Draw<S>,
        scale_factor: f32,
        frame_dims: [u32; 2],
        depth_format: wgpu::TextureFormat,
        frame: &Frame,
    ) where
        S: BaseFloat,
    {
        let attachment = frame.texture();
        let resolve_target = frame.resolve_target();
        let mut command_encoder = frame.command_encoder();
        self.encode_render_pass(
            device,
            draw,
            scale_factor,
            frame_dims,
            depth_format,
            attachment,
            resolve_target,
            &mut *command_encoder,
        );
    }

    pub fn encode_render_pass<S>(
        &mut self,
        device: &wgpu::Device,
        draw: &draw::Draw<S>,
        scale_factor: f32,
        attachment_dims: [u32; 2],
        depth_format: wgpu::TextureFormat,
        output_attachment: &wgpu::TextureView,
        resolve_target: Option<&wgpu::TextureView>,
        encoder: &mut wgpu::CommandEncoder,
    ) where
        S: BaseFloat,
    {
        let Renderer {
            ref render_pipeline,
            ref mut vertices,
            ref mut indices,
            ref bind_group,
            ..
        } = *self;

        // Retrieve the clear values based on the bg color.
        let bg_color = draw.state.borrow().background_color;
        let (load_op, clear_color) = match bg_color {
            None => (wgpu::LoadOp::Load, wgpu::Color::TRANSPARENT),
            Some(color) => {
                let (r, g, b, a) = color.into();
                let (r, g, b, a) = (r as f64, g as f64, b as f64, a as f64);
                let clear_color = wgpu::Color { r, g, b, a };
                (wgpu::LoadOp::Clear, clear_color)
            }
        };

        let color_attachment_desc = wgpu::RenderPassColorAttachmentDescriptor {
            attachment: output_attachment,
            resolve_target,
            load_op,
            store_op: wgpu::StoreOp::Store,
            clear_color,
        };

        // let depth_stencil_attachment_desc = wgpu::RenderPassDepthStencilAttachmentDescriptor {
        // };

        let render_pass_desc = wgpu::RenderPassDescriptor {
            color_attachments: &[color_attachment_desc],
            depth_stencil_attachment: None,
        };


        // Create the vertex and index buffers.
        let [img_w, img_h] = attachment_dims;
        let map_vertex = |v| Vertex::from_mesh_vertex(v, img_w as _, img_h as _, scale_factor);
        vertices.clear();
        vertices.extend(draw.raw_vertices().map(map_vertex));
        let vertex_buffer = device
            .create_buffer_mapped(vertices.len(), wgpu::BufferUsage::VERTEX)
            .fill_from_slice(&vertices[..]);
        indices.clear();
        indices.extend(draw.inner_mesh().indices().iter().map(|&u| u as u32));
        let index_buffer = device
            .create_buffer_mapped(indices.len(), wgpu::BufferUsage::INDEX)
            .fill_from_slice(&indices[..]);


        let mut render_pass = encoder.begin_render_pass(&render_pass_desc);
        render_pass.set_pipeline(render_pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.set_index_buffer(&index_buffer, 0);
        render_pass.set_vertex_buffers(0, &[(&vertex_buffer, 0)]);
        let vertex_range = 0..vertices.len() as u32;
        let index_range = 0..indices.len() as u32;
        let instance_range = 0..1;
        render_pass.draw_indexed(index_range, 0, instance_range);
    }
}

fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    let bindings = &[];
    let desc = wgpu::BindGroupLayoutDescriptor { bindings };
    device.create_bind_group_layout(&desc)
}

fn bind_group(device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup {
    let bindings = &[];
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

fn vertex_attrs() -> [wgpu::VertexAttributeDescriptor; 3] {
    let position_offset = 0;
    let position_size = std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress;
    let rgba_offset = position_offset + position_size;
    let rgba_size = std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress;
    let tex_coords_offset = rgba_offset + rgba_size;
    [
        // position
        wgpu::VertexAttributeDescriptor {
            format: wgpu::VertexFormat::Float3,
            offset: position_offset,
            shader_location: 0,
        },
        // rgba
        wgpu::VertexAttributeDescriptor {
            format: wgpu::VertexFormat::Float4,
            offset: rgba_offset,
            shader_location: 1,
        },
        // tex_coords
        wgpu::VertexAttributeDescriptor {
            format: wgpu::VertexFormat::Float2,
            offset: tex_coords_offset,
            shader_location: 2,
        },
    ]
}

fn render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    vs_mod: &wgpu::ShaderModule,
    fs_mod: &wgpu::ShaderModule,
    dst_format: wgpu::TextureFormat,
    msaa_samples: u32,
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
        color_blend: wgpu::BlendDescriptor {
            src_factor: wgpu::BlendFactor::SrcAlpha,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
        },
        alpha_blend: wgpu::BlendDescriptor {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
        },
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
        index_format: wgpu::IndexFormat::Uint32,
        vertex_buffers: &[vertex_buffer_desc],
        sample_count: msaa_samples,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    };
    device.create_render_pipeline(&desc)
}
