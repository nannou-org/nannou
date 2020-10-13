//! Items aimed at easing the contruction of a render pipeline.
//!
//! Creating a `RenderPipeline` tends to involve a lot of boilerplate that we don't always want to
//! have to consider when writing graphics code. Here we define a set of helpers that allow us to
//! simplify the process and fall back to a set of reasonable defaults.

use crate::wgpu;

#[derive(Debug)]
enum Layout<'a> {
    Descriptor(wgpu::PipelineLayoutDescriptor<'a>),
    Created(&'a wgpu::PipelineLayout),
}

/// Types that may be directly converted into a pipeline layout descriptor.
pub trait IntoPipelineLayoutDescriptor<'a> {
    /// Convert the type into a pipeline layout descriptor.
    fn into_pipeline_layout_descriptor(self) -> wgpu::PipelineLayoutDescriptor<'a>;
}

/// A builder type to help simplify the construction of a **RenderPipeline**.
///
/// We've attempted to provide a suite of reasonable defaults in the case that none are provided.
#[derive(Debug)]
pub struct RenderPipelineBuilder<'a> {
    layout: Layout<'a>,
    vs_mod: &'a wgpu::ShaderModule,
    fs_mod: Option<&'a wgpu::ShaderModule>,
    vs_entry_point: &'a str,
    fs_entry_point: &'a str,
    rasterization_state: Option<wgpu::RasterizationStateDescriptor>,
    primitive_topology: wgpu::PrimitiveTopology,
    color_state: Option<wgpu::ColorStateDescriptor>,
    color_states: &'a [wgpu::ColorStateDescriptor],
    depth_stencil_state: Option<wgpu::DepthStencilStateDescriptor>,
    index_format: wgpu::IndexFormat,
    vertex_buffers: Vec<wgpu::VertexBufferDescriptor<'static>>,
    sample_count: u32,
    sample_mask: u32,
    alpha_to_coverage_enabled: bool,
}

impl<'a> RenderPipelineBuilder<'a> {
    // The default entry point used for shaders when unspecified.
    pub const DEFAULT_SHADER_ENTRY_POINT: &'static str = "main";

    // Rasterization state defaults for the case where the user has submitted a fragment shader.
    pub const DEFAULT_FRONT_FACE: wgpu::FrontFace = wgpu::FrontFace::Ccw;
    pub const DEFAULT_CULL_MODE: wgpu::CullMode = wgpu::CullMode::None;
    pub const DEFAULT_DEPTH_BIAS: i32 = 0;
    pub const DEFAULT_DEPTH_BIAS_SLOPE_SCALE: f32 = 0.0;
    pub const DEFAULT_DEPTH_BIAS_CLAMP: f32 = 0.0;
    pub const DEFAULT_RASTERIZATION_STATE: wgpu::RasterizationStateDescriptor =
        wgpu::RasterizationStateDescriptor {
            front_face: Self::DEFAULT_FRONT_FACE,
            cull_mode: Self::DEFAULT_CULL_MODE,
            depth_bias: Self::DEFAULT_DEPTH_BIAS,
            depth_bias_slope_scale: Self::DEFAULT_DEPTH_BIAS_SLOPE_SCALE,
            depth_bias_clamp: Self::DEFAULT_DEPTH_BIAS_CLAMP,
            clamp_depth: false,
        };

    // Primitive topology.
    pub const DEFAULT_PRIMITIVE_TOPOLOGY: wgpu::PrimitiveTopology =
        wgpu::PrimitiveTopology::TriangleList;

    // Color state defaults.
    pub const DEFAULT_COLOR_FORMAT: wgpu::TextureFormat = crate::frame::Frame::TEXTURE_FORMAT;
    pub const DEFAULT_COLOR_BLEND: wgpu::BlendDescriptor = wgpu::BlendDescriptor {
        src_factor: wgpu::BlendFactor::SrcAlpha,
        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
        operation: wgpu::BlendOperation::Add,
    };
    pub const DEFAULT_ALPHA_BLEND: wgpu::BlendDescriptor = wgpu::BlendDescriptor {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
        operation: wgpu::BlendOperation::Add,
    };
    pub const DEFAULT_COLOR_WRITE: wgpu::ColorWrite = wgpu::ColorWrite::ALL;
    pub const DEFAULT_COLOR_STATE: wgpu::ColorStateDescriptor = wgpu::ColorStateDescriptor {
        format: Self::DEFAULT_COLOR_FORMAT,
        color_blend: Self::DEFAULT_COLOR_BLEND,
        alpha_blend: Self::DEFAULT_ALPHA_BLEND,
        write_mask: Self::DEFAULT_COLOR_WRITE,
    };

    // Depth state defaults.
    pub const DEFAULT_DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    pub const DEFAULT_DEPTH_WRITE_ENABLED: bool = true;
    pub const DEFAULT_DEPTH_COMPARE: wgpu::CompareFunction = wgpu::CompareFunction::LessEqual;
    pub const DEFAULT_STENCIL_FRONT: wgpu::StencilStateFaceDescriptor =
        wgpu::StencilStateFaceDescriptor::IGNORE;
    pub const DEFAULT_STENCIL_BACK: wgpu::StencilStateFaceDescriptor =
        wgpu::StencilStateFaceDescriptor::IGNORE;
    pub const DEFAULT_STENCIL_READ_MASK: u32 = 0;
    pub const DEFAULT_STENCIL_WRITE_MASK: u32 = 0;
    pub const DEFAULT_STENCIL: wgpu::StencilStateDescriptor = wgpu::StencilStateDescriptor {
        front: Self::DEFAULT_STENCIL_FRONT,
        back: Self::DEFAULT_STENCIL_BACK,
        read_mask: Self::DEFAULT_STENCIL_READ_MASK,
        write_mask: Self::DEFAULT_STENCIL_WRITE_MASK,
    };
    pub const DEFAULT_DEPTH_STENCIL_STATE: wgpu::DepthStencilStateDescriptor =
        wgpu::DepthStencilStateDescriptor {
            format: Self::DEFAULT_DEPTH_FORMAT,
            depth_write_enabled: Self::DEFAULT_DEPTH_WRITE_ENABLED,
            depth_compare: Self::DEFAULT_DEPTH_COMPARE,
            stencil: Self::DEFAULT_STENCIL,
        };

    // Vertex buffer defaults.
    pub const DEFAULT_INDEX_FORMAT: wgpu::IndexFormat = wgpu::IndexFormat::Uint32;
    // No MSAA by default.
    pub const DEFAULT_SAMPLE_COUNT: u32 = 1;
    pub const DEFAULT_SAMPLE_MASK: u32 = !0;
    pub const DEFAULT_ALPHA_TO_COVERAGE_ENABLED: bool = false;

    // Constructors

    /// Begin building the render pipeline for the given pipeline layout and the vertex shader
    /// module.
    pub fn from_layout(layout: &'a wgpu::PipelineLayout, vs_mod: &'a wgpu::ShaderModule) -> Self {
        let layout = Layout::Created(layout);
        Self::new_inner(layout, vs_mod)
    }

    /// Begin building the render pipeline for a pipeline with the given layout descriptor and the
    /// vertex shader module.
    pub fn from_layout_descriptor<T>(layout_desc: T, vs_mod: &'a wgpu::ShaderModule) -> Self
    where
        T: IntoPipelineLayoutDescriptor<'a>,
    {
        let desc = layout_desc.into_pipeline_layout_descriptor();
        let layout = Layout::Descriptor(desc);
        Self::new_inner(layout, vs_mod)
    }

    // Shared between constructors.
    fn new_inner(layout: Layout<'a>, vs_mod: &'a wgpu::ShaderModule) -> Self {
        RenderPipelineBuilder {
            layout,
            vs_mod,
            fs_mod: None,
            vs_entry_point: Self::DEFAULT_SHADER_ENTRY_POINT,
            fs_entry_point: Self::DEFAULT_SHADER_ENTRY_POINT,
            rasterization_state: None,
            color_state: None,
            color_states: &[],
            primitive_topology: Self::DEFAULT_PRIMITIVE_TOPOLOGY,
            depth_stencil_state: None,
            index_format: Self::DEFAULT_INDEX_FORMAT,
            vertex_buffers: vec![],
            sample_count: Self::DEFAULT_SAMPLE_COUNT,
            sample_mask: Self::DEFAULT_SAMPLE_MASK,
            alpha_to_coverage_enabled: Self::DEFAULT_ALPHA_TO_COVERAGE_ENABLED,
        }
    }

    // Builders

    pub fn vertex_entry_point(mut self, entry_point: &'a str) -> Self {
        self.vs_entry_point = entry_point;
        self
    }

    pub fn fragment_entry_point(mut self, entry_point: &'a str) -> Self {
        self.fs_entry_point = entry_point;
        self
    }

    /// Specify a fragment shader for the render pipeline.
    pub fn fragment_shader(mut self, fs_mod: &'a wgpu::ShaderModule) -> Self {
        self.fs_mod = Some(fs_mod);
        self
    }

    // Rasterization state.

    /// Specify the full rasterization state.
    pub fn rasterization_state(mut self, state: wgpu::RasterizationStateDescriptor) -> Self {
        self.rasterization_state = Some(state);
        self
    }

    pub fn front_face(mut self, front_face: wgpu::FrontFace) -> Self {
        let state = self
            .rasterization_state
            .get_or_insert(Self::DEFAULT_RASTERIZATION_STATE);
        state.front_face = front_face;
        self
    }

    pub fn cull_mode(mut self, cull_mode: wgpu::CullMode) -> Self {
        let state = self
            .rasterization_state
            .get_or_insert(Self::DEFAULT_RASTERIZATION_STATE);
        state.cull_mode = cull_mode;
        self
    }

    pub fn depth_bias(mut self, bias: i32) -> Self {
        let state = self
            .rasterization_state
            .get_or_insert(Self::DEFAULT_RASTERIZATION_STATE);
        state.depth_bias = bias;
        self
    }

    pub fn depth_bias_slope_scale(mut self, scale: f32) -> Self {
        let state = self
            .rasterization_state
            .get_or_insert(Self::DEFAULT_RASTERIZATION_STATE);
        state.depth_bias_slope_scale = scale;
        self
    }

    pub fn depth_bias_clamp(mut self, clamp: f32) -> Self {
        let state = self
            .rasterization_state
            .get_or_insert(Self::DEFAULT_RASTERIZATION_STATE);
        state.depth_bias_clamp = clamp;
        self
    }

    // Primitive topology.

    /// Specify the primitive topology.
    ///
    /// This represents the way vertices will be read from the **VertexBuffer**.
    pub fn primitive_topology(mut self, topology: wgpu::PrimitiveTopology) -> Self {
        self.primitive_topology = topology;
        self
    }

    // Color state.

    /// Specify the full color state for drawing to the output attachment.
    ///
    /// If you have multiple output attachments, see the `color_states` method.
    pub fn color_state(mut self, state: wgpu::ColorStateDescriptor) -> Self {
        self.color_state = Some(state);
        self
    }

    pub fn color_format(mut self, format: wgpu::TextureFormat) -> Self {
        let state = self.color_state.get_or_insert(Self::DEFAULT_COLOR_STATE);
        state.format = format;
        self
    }

    pub fn color_blend(mut self, blend: wgpu::BlendDescriptor) -> Self {
        let state = self.color_state.get_or_insert(Self::DEFAULT_COLOR_STATE);
        state.color_blend = blend;
        self
    }

    pub fn alpha_blend(mut self, blend: wgpu::BlendDescriptor) -> Self {
        let state = self.color_state.get_or_insert(Self::DEFAULT_COLOR_STATE);
        state.alpha_blend = blend;
        self
    }

    pub fn write_mask(mut self, mask: wgpu::ColorWrite) -> Self {
        let state = self.color_state.get_or_insert(Self::DEFAULT_COLOR_STATE);
        state.write_mask = mask;
        self
    }

    // Depth / Stencil state

    pub fn depth_stencil_state(mut self, state: wgpu::DepthStencilStateDescriptor) -> Self {
        self.depth_stencil_state = Some(state);
        self
    }

    pub fn depth_format(mut self, format: wgpu::TextureFormat) -> Self {
        let state = self
            .depth_stencil_state
            .get_or_insert(Self::DEFAULT_DEPTH_STENCIL_STATE);
        state.format = format;
        self
    }

    pub fn depth_write_enabled(mut self, enabled: bool) -> Self {
        let state = self
            .depth_stencil_state
            .get_or_insert(Self::DEFAULT_DEPTH_STENCIL_STATE);
        state.depth_write_enabled = enabled;
        self
    }

    pub fn depth_compare(mut self, compare: wgpu::CompareFunction) -> Self {
        let state = self
            .depth_stencil_state
            .get_or_insert(Self::DEFAULT_DEPTH_STENCIL_STATE);
        state.depth_compare = compare;
        self
    }

    pub fn stencil_front(mut self, stencil: wgpu::StencilStateFaceDescriptor) -> Self {
        let state = self
            .depth_stencil_state
            .get_or_insert(Self::DEFAULT_DEPTH_STENCIL_STATE);
        state.stencil.front = stencil;
        self
    }

    pub fn stencil_back(mut self, stencil: wgpu::StencilStateFaceDescriptor) -> Self {
        let state = self
            .depth_stencil_state
            .get_or_insert(Self::DEFAULT_DEPTH_STENCIL_STATE);
        state.stencil.back = stencil;
        self
    }

    pub fn stencil_read_mask(mut self, mask: u32) -> Self {
        let state = self
            .depth_stencil_state
            .get_or_insert(Self::DEFAULT_DEPTH_STENCIL_STATE);
        state.stencil.read_mask = mask;
        self
    }

    pub fn stencil_write_mask(mut self, mask: u32) -> Self {
        let state = self
            .depth_stencil_state
            .get_or_insert(Self::DEFAULT_DEPTH_STENCIL_STATE);
        state.stencil.write_mask = mask;
        self
    }

    // Vertex buffer methods.

    /// The format of the type used within the index buffer.
    pub fn index_format(mut self, format: wgpu::IndexFormat) -> Self {
        self.index_format = format;
        self
    }

    /// Add a new vertex buffer descriptor to the render pipeline.
    pub fn add_vertex_buffer_descriptor(
        mut self,
        d: wgpu::VertexBufferDescriptor<'static>,
    ) -> Self {
        self.vertex_buffers.push(d);
        self
    }

    /// Short-hand for adding a descriptor to the render pipeline describing a buffer of vertices
    /// of the given vertex type.
    ///
    /// The vertex stride is assumed to be equal to `size_of::<V>()`. If this is not the case,
    /// consider using `add_vertex_buffer_descriptor` instead.
    pub fn add_vertex_buffer<V>(self, attrs: &'static [wgpu::VertexAttributeDescriptor]) -> Self {
        let stride = std::mem::size_of::<V>() as wgpu::BufferAddress;
        let step_mode = wgpu::InputStepMode::Vertex;
        let descriptor = wgpu::VertexBufferDescriptor {
            stride,
            step_mode,
            attributes: attrs,
        };
        self.add_vertex_buffer_descriptor(descriptor)
    }

    /// Short-hand for adding a descriptor to the render pipeline describing a buffer of instances
    /// of the given vertex type.
    pub fn add_instance_buffer<I>(self, attrs: &'static [wgpu::VertexAttributeDescriptor]) -> Self {
        let stride = std::mem::size_of::<I>() as wgpu::BufferAddress;
        let step_mode = wgpu::InputStepMode::Instance;
        let descriptor = wgpu::VertexBufferDescriptor {
            stride,
            step_mode,
            attributes: attrs,
        };
        self.add_vertex_buffer_descriptor(descriptor)
    }

    /// The sample count of the output attachment.
    pub fn sample_count(mut self, sample_count: u32) -> Self {
        self.sample_count = sample_count;
        self
    }

    // Finalising methods.

    /// Build the render pipeline layout, its descriptor and ultimately the pipeline itself with
    /// the specified parameters.
    ///
    /// **Panic!**s in the following occur:
    ///
    /// - A rasterization state field was specified but no fragment shader was given.
    /// - A color state field was specified but no fragment shader was given.
    pub fn build(self, device: &wgpu::Device) -> wgpu::RenderPipeline {
        match self.layout {
            Layout::Descriptor(ref desc) => {
                let layout = device.create_pipeline_layout(desc);
                build(self, &layout, device)
            }
            Layout::Created(layout) => build(self, layout, device),
        }
    }
}

impl<'a> IntoPipelineLayoutDescriptor<'a> for wgpu::PipelineLayoutDescriptor<'a> {
    fn into_pipeline_layout_descriptor(self) -> wgpu::PipelineLayoutDescriptor<'a> {
        self
    }
}

impl<'a> IntoPipelineLayoutDescriptor<'a> for &'a [&'a wgpu::BindGroupLayout] {
    fn into_pipeline_layout_descriptor(self) -> wgpu::PipelineLayoutDescriptor<'a> {
        wgpu::PipelineLayoutDescriptor {
            label: Some("nannou render pipeline layout"),
            bind_group_layouts: self,
            // wgpu 0.5-06 TODO: maybe constants are needed to be specified here
            push_constant_ranges: &[],
        }
    }
}

fn build(
    builder: RenderPipelineBuilder,
    layout: &wgpu::PipelineLayout,
    device: &wgpu::Device,
) -> wgpu::RenderPipeline {
    let RenderPipelineBuilder {
        layout: _layout,
        vs_mod,
        fs_mod,
        vs_entry_point,
        fs_entry_point,
        rasterization_state,
        primitive_topology,
        color_state,
        color_states,
        depth_stencil_state,
        index_format,
        vertex_buffers,
        sample_count,
        sample_mask,
        alpha_to_coverage_enabled,
    } = builder;

    let vertex_stage = wgpu::ProgrammableStageDescriptor {
        module: &vs_mod,
        entry_point: vs_entry_point,
    };

    let fragment_stage = fs_mod.map(|fs_mod| wgpu::ProgrammableStageDescriptor {
        module: fs_mod,
        entry_point: fs_entry_point,
    });

    let rasterization_state = match fragment_stage.is_some() {
        true => {
            Some(rasterization_state.unwrap_or(RenderPipelineBuilder::DEFAULT_RASTERIZATION_STATE))
        }
        false => {
            if rasterization_state.is_some() {
                panic!("specified rasterization state fields but no fragment shader");
            } else {
                None
            }
        }
    };

    let mut single_color_state = [RenderPipelineBuilder::DEFAULT_COLOR_STATE];
    let color_states = match (fragment_stage.is_some(), color_states.is_empty()) {
        (true, true) => {
            if let Some(cs) = color_state {
                single_color_state[0] = cs;
            }
            &single_color_state[..]
        }
        (true, false) => color_states,
        (false, true) => panic!("specified color states but no fragment shader"),
        (false, false) => match color_state.is_some() {
            true => panic!("specified color state fields but no fragment shader"),
            false => &[],
        },
    };

    let vertex_state = wgpu::VertexStateDescriptor {
        index_format,
        vertex_buffers: &vertex_buffers[..],
    };

    let pipeline_desc = wgpu::RenderPipelineDescriptor {
        label: Some("nannou render pipeline"),
        layout: Some(layout),
        vertex_stage,
        fragment_stage,
        rasterization_state,
        primitive_topology,
        color_states,
        depth_stencil_state,
        vertex_state,
        sample_count,
        sample_mask,
        alpha_to_coverage_enabled,
    };

    device.create_render_pipeline(&pipeline_desc)
}
