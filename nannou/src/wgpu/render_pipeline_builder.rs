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
    primitive: wgpu::PrimitiveState,
    color_state: Option<wgpu::ColorTargetState>,
    color_states: &'a [wgpu::ColorTargetState],
    depth_stencil: Option<wgpu::DepthStencilState>,
    vertex_buffers: Vec<wgpu::VertexBufferLayout<'static>>,
    multisample: wgpu::MultisampleState,
}

impl<'a> RenderPipelineBuilder<'a> {
    // The default entry point used for shaders when unspecified.
    pub const DEFAULT_SHADER_ENTRY_POINT: &'static str = "main";

    // Primitive state.
    pub const DEFAULT_FRONT_FACE: wgpu::FrontFace = wgpu::FrontFace::Ccw;
    pub const DEFAULT_CULL_MODE: wgpu::CullMode = wgpu::CullMode::None;
    pub const DEFAULT_POLYGON_MODE: wgpu::PolygonMode = wgpu::PolygonMode::Fill;
    pub const DEFAULT_PRIMITIVE_TOPOLOGY: wgpu::PrimitiveTopology =
        wgpu::PrimitiveTopology::TriangleList;
    pub const DEFAULT_PRIMITIVE: wgpu::PrimitiveState = wgpu::PrimitiveState {
        topology: Self::DEFAULT_PRIMITIVE_TOPOLOGY,
        strip_index_format: None,
        front_face: Self::DEFAULT_FRONT_FACE,
        cull_mode: Self::DEFAULT_CULL_MODE,
        polygon_mode: Self::DEFAULT_POLYGON_MODE,
    };

    // Color state defaults.
    pub const DEFAULT_COLOR_FORMAT: wgpu::TextureFormat = crate::frame::Frame::TEXTURE_FORMAT;
    pub const DEFAULT_COLOR_BLEND: wgpu::BlendState = wgpu::BlendState {
        src_factor: wgpu::BlendFactor::SrcAlpha,
        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
        operation: wgpu::BlendOperation::Add,
    };
    pub const DEFAULT_ALPHA_BLEND: wgpu::BlendState = wgpu::BlendState {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
        operation: wgpu::BlendOperation::Add,
    };
    pub const DEFAULT_COLOR_WRITE: wgpu::ColorWrite = wgpu::ColorWrite::ALL;
    pub const DEFAULT_COLOR_STATE: wgpu::ColorTargetState = wgpu::ColorTargetState {
        format: Self::DEFAULT_COLOR_FORMAT,
        color_blend: Self::DEFAULT_COLOR_BLEND,
        alpha_blend: Self::DEFAULT_ALPHA_BLEND,
        write_mask: Self::DEFAULT_COLOR_WRITE,
    };

    // Depth state defaults.
    pub const DEFAULT_DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    pub const DEFAULT_DEPTH_WRITE_ENABLED: bool = true;
    pub const DEFAULT_DEPTH_COMPARE: wgpu::CompareFunction = wgpu::CompareFunction::LessEqual;
    pub const DEFAULT_STENCIL_FRONT: wgpu::StencilFaceState = wgpu::StencilFaceState::IGNORE;
    pub const DEFAULT_STENCIL_BACK: wgpu::StencilFaceState = wgpu::StencilFaceState::IGNORE;
    pub const DEFAULT_STENCIL_READ_MASK: u32 = 0;
    pub const DEFAULT_STENCIL_WRITE_MASK: u32 = 0;
    pub const DEFAULT_STENCIL: wgpu::StencilState = wgpu::StencilState {
        front: Self::DEFAULT_STENCIL_FRONT,
        back: Self::DEFAULT_STENCIL_BACK,
        read_mask: Self::DEFAULT_STENCIL_READ_MASK,
        write_mask: Self::DEFAULT_STENCIL_WRITE_MASK,
    };
    pub const DEFAULT_DEPTH_BIAS_CONSTANT: i32 = 0;
    pub const DEFAULT_DEPTH_BIAS_SLOPE_SCALE: f32 = 0.0;
    pub const DEFAULT_DEPTH_BIAS_CLAMP: f32 = 0.0;
    pub const DEFAULT_DEPTH_BIAS: wgpu::DepthBiasState = wgpu::DepthBiasState {
        constant: Self::DEFAULT_DEPTH_BIAS_CONSTANT,
        slope_scale: Self::DEFAULT_DEPTH_BIAS_SLOPE_SCALE,
        clamp: Self::DEFAULT_DEPTH_BIAS_CLAMP,
    };
    pub const DEFAULT_CLAMP_DEPTH: bool = false;
    pub const DEFAULT_DEPTH_STENCIL: wgpu::DepthStencilState = wgpu::DepthStencilState {
        format: Self::DEFAULT_DEPTH_FORMAT,
        depth_write_enabled: Self::DEFAULT_DEPTH_WRITE_ENABLED,
        depth_compare: Self::DEFAULT_DEPTH_COMPARE,
        stencil: Self::DEFAULT_STENCIL,
        bias: Self::DEFAULT_DEPTH_BIAS,
        clamp_depth: Self::DEFAULT_CLAMP_DEPTH,
    };

    // Multisample state.
    pub const DEFAULT_SAMPLE_COUNT: u32 = 1;
    pub const DEFAULT_SAMPLE_MASK: u64 = !0;
    pub const DEFAULT_ALPHA_TO_COVERAGE_ENABLED: bool = false;
    pub const DEFAULT_MULTISAMPLE: wgpu::MultisampleState = wgpu::MultisampleState {
        count: Self::DEFAULT_SAMPLE_COUNT,
        mask: Self::DEFAULT_SAMPLE_MASK,
        alpha_to_coverage_enabled: Self::DEFAULT_ALPHA_TO_COVERAGE_ENABLED,
    };

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
            color_state: None,
            color_states: &[],
            primitive: Self::DEFAULT_PRIMITIVE,
            depth_stencil: None,
            vertex_buffers: vec![],
            multisample: Self::DEFAULT_MULTISAMPLE,
        }
    }

    // Builders

    /// The name of the entry point in the compiled shader.
    ///
    /// There must be a function that returns void with this name in the shader.
    pub fn vertex_entry_point(mut self, entry_point: &'a str) -> Self {
        self.vs_entry_point = entry_point;
        self
    }

    /// The name of the entry point in the compiled shader.
    ///
    /// There must be a function that returns void with this name in the shader.
    pub fn fragment_entry_point(mut self, entry_point: &'a str) -> Self {
        self.fs_entry_point = entry_point;
        self
    }

    /// Specify a compiled fragment shader for the render pipeline.
    pub fn fragment_shader(mut self, fs_mod: &'a wgpu::ShaderModule) -> Self {
        self.fs_mod = Some(fs_mod);
        self
    }

    // Primitive state.

    /// Specify the full primitive state.
    ///
    /// Describes the state of primitive assembly and rasterization in a render pipeline.
    pub fn primitive(mut self, p: wgpu::PrimitiveState) -> Self {
        self.primitive = p;
        self
    }

    /// The face to consider the front for the purpose of culling and stencil operations.
    pub fn front_face(mut self, front_face: wgpu::FrontFace) -> Self {
        self.primitive.front_face = front_face;
        self
    }

    /// The face culling mode.
    pub fn cull_mode(mut self, cull_mode: wgpu::CullMode) -> Self {
        self.primitive.cull_mode = cull_mode;
        self
    }

    /// Specify the primitive topology.
    ///
    /// This represents the way vertices will be read from the **VertexBuffer**.
    pub fn primitive_topology(mut self, topology: wgpu::PrimitiveTopology) -> Self {
        self.primitive.topology = topology;
        self
    }

    /// Controls the way each polygon is rasterized. Can be either `Fill` (default), `Line` or
    /// `Point`.
    ///
    /// Setting this to something other than `Fill` requires `Features::NON_FILL_POLYGON_MODE` to
    /// be enabled.
    pub fn polygon_mode(mut self, mode: wgpu::PolygonMode) -> Self {
        self.primitive.polygon_mode = mode;
        self
    }

    // Color state.

    /// Specify the full color state for drawing to the output attachment.
    ///
    /// If you have multiple output attachments, see the `color_states` method.
    pub fn color_state(mut self, state: wgpu::ColorTargetState) -> Self {
        self.color_state = Some(state);
        self
    }

    /// The texture formrat of the image that this pipelinew ill render to.
    ///
    /// Must match the format of the corresponding color attachment.
    pub fn color_format(mut self, format: wgpu::TextureFormat) -> Self {
        let state = self.color_state.get_or_insert(Self::DEFAULT_COLOR_STATE);
        state.format = format;
        self
    }

    /// The color blending used for this pipeline.
    pub fn color_blend(mut self, blend: wgpu::BlendState) -> Self {
        let state = self.color_state.get_or_insert(Self::DEFAULT_COLOR_STATE);
        state.color_blend = blend;
        self
    }

    /// The alpha blending used for this pipeline.
    pub fn alpha_blend(mut self, blend: wgpu::BlendState) -> Self {
        let state = self.color_state.get_or_insert(Self::DEFAULT_COLOR_STATE);
        state.alpha_blend = blend;
        self
    }

    /// Mask which enables/disables writes to different color/alpha channel.
    pub fn write_mask(mut self, mask: wgpu::ColorWrite) -> Self {
        let state = self.color_state.get_or_insert(Self::DEFAULT_COLOR_STATE);
        state.write_mask = mask;
        self
    }

    // Depth / Stencil state

    /// Specify the full depth stencil state.
    pub fn depth_stencil(mut self, state: wgpu::DepthStencilState) -> Self {
        self.depth_stencil = Some(state);
        self
    }

    /// Format of the depth/stencil buffer. Must be one of the depth formats. Must match the format
    /// of the depth/stencil attachment.
    pub fn depth_format(mut self, format: wgpu::TextureFormat) -> Self {
        let state = self
            .depth_stencil
            .get_or_insert(Self::DEFAULT_DEPTH_STENCIL);
        state.format = format;
        self
    }

    pub fn depth_write_enabled(mut self, enabled: bool) -> Self {
        let state = self
            .depth_stencil
            .get_or_insert(Self::DEFAULT_DEPTH_STENCIL);
        state.depth_write_enabled = enabled;
        self
    }

    /// Comparison function used to compare depth values in the depth test.
    pub fn depth_compare(mut self, compare: wgpu::CompareFunction) -> Self {
        let state = self
            .depth_stencil
            .get_or_insert(Self::DEFAULT_DEPTH_STENCIL);
        state.depth_compare = compare;
        self
    }

    /// Specify the full set of stencil parameters.
    pub fn stencil(mut self, stencil: wgpu::StencilState) -> Self {
        let state = self
            .depth_stencil
            .get_or_insert(Self::DEFAULT_DEPTH_STENCIL);
        state.stencil = stencil;
        self
    }

    /// Front face mode.
    pub fn stencil_front(mut self, stencil: wgpu::StencilFaceState) -> Self {
        let state = self
            .depth_stencil
            .get_or_insert(Self::DEFAULT_DEPTH_STENCIL);
        state.stencil.front = stencil;
        self
    }

    /// Back face mode.
    pub fn stencil_back(mut self, stencil: wgpu::StencilFaceState) -> Self {
        let state = self
            .depth_stencil
            .get_or_insert(Self::DEFAULT_DEPTH_STENCIL);
        state.stencil.back = stencil;
        self
    }

    /// Stencil values are AND'd with this mask when reading and writing from the stencil buffer.
    /// Only low 8 bits are used.
    pub fn stencil_read_mask(mut self, mask: u32) -> Self {
        let state = self
            .depth_stencil
            .get_or_insert(Self::DEFAULT_DEPTH_STENCIL);
        state.stencil.read_mask = mask;
        self
    }

    /// Stencil values are AND'd with this mask when writing to the stencil buffer.
    /// Only low 8 bits are used.
    pub fn stencil_write_mask(mut self, mask: u32) -> Self {
        let state = self
            .depth_stencil
            .get_or_insert(Self::DEFAULT_DEPTH_STENCIL);
        state.stencil.write_mask = mask;
        self
    }

    /// Specify the full set of depth bias parameters.
    ///
    /// Describes the biasing setting for the depth target.
    pub fn depth_bias(mut self, bias: wgpu::DepthBiasState) -> Self {
        let state = self
            .depth_stencil
            .get_or_insert(Self::DEFAULT_DEPTH_STENCIL);
        state.bias = bias;
        self
    }

    /// Constant depth biasing factor, in basic units of the depth format.
    pub fn depth_bias_constant(mut self, constant: i32) -> Self {
        let state = self
            .depth_stencil
            .get_or_insert(Self::DEFAULT_DEPTH_STENCIL);
        state.bias.constant = constant;
        self
    }

    /// Slope depth biasing factor.
    pub fn depth_bias_slope_scale(mut self, scale: f32) -> Self {
        let state = self
            .depth_stencil
            .get_or_insert(Self::DEFAULT_DEPTH_STENCIL);
        state.bias.slope_scale = scale;
        self
    }

    /// Depth bias clamp value (absolute).
    pub fn depth_bias_clamp(mut self, clamp: f32) -> Self {
        let state = self
            .depth_stencil
            .get_or_insert(Self::DEFAULT_DEPTH_STENCIL);
        state.bias.clamp = clamp;
        self
    }

    /// If enabled polygon depth is clamped to 0-1 range instead of being clipped.
    ///
    /// Requires `Features::DEPTH_CLAMPING` enabled.
    pub fn clamp_depth(mut self, b: bool) -> Self {
        let state = self
            .depth_stencil
            .get_or_insert(Self::DEFAULT_DEPTH_STENCIL);
        state.clamp_depth = b;
        self
    }

    // Vertex buffer methods.

    /// Add a new vertex buffer descriptor to the render pipeline.
    pub fn add_vertex_buffer_layout(mut self, d: wgpu::VertexBufferLayout<'static>) -> Self {
        self.vertex_buffers.push(d);
        self
    }

    /// Short-hand for adding a descriptor to the render pipeline describing a buffer of vertices
    /// of the given vertex type.
    ///
    /// The vertex stride is assumed to be equal to `size_of::<V>()`. If this is not the case,
    /// consider using `add_vertex_buffer_layout` instead.
    pub fn add_vertex_buffer<V>(self, attrs: &'static [wgpu::VertexAttribute]) -> Self {
        let array_stride = std::mem::size_of::<V>() as wgpu::BufferAddress;
        let step_mode = wgpu::InputStepMode::Vertex;
        let descriptor = wgpu::VertexBufferLayout {
            array_stride,
            step_mode,
            attributes: attrs,
        };
        self.add_vertex_buffer_layout(descriptor)
    }

    /// Short-hand for adding a descriptor to the render pipeline describing a buffer of instances
    /// of the given vertex type.
    pub fn add_instance_buffer<I>(self, attrs: &'static [wgpu::VertexAttribute]) -> Self {
        let array_stride = std::mem::size_of::<I>() as wgpu::BufferAddress;
        let step_mode = wgpu::InputStepMode::Instance;
        let descriptor = wgpu::VertexBufferLayout {
            array_stride,
            step_mode,
            attributes: attrs,
        };
        self.add_vertex_buffer_layout(descriptor)
    }

    // Multisample state.

    /// Specify the full multisample state.
    pub fn multisample(mut self, multisample: wgpu::MultisampleState) -> Self {
        self.multisample = multisample;
        self
    }

    /// The number of samples calculated per pixel (for MSAA).
    ///
    /// For non-multisampled textures, this should be 1 (the default).
    pub fn sample_count(mut self, sample_count: u32) -> Self {
        self.multisample.count = sample_count;
        self
    }

    /// Bitmask that restricts the samples of a pixel modified by this pipeline. All samples can be
    /// enabled using the value !0 (the default).
    pub fn sample_mask(mut self, sample_mask: u64) -> Self {
        self.multisample.mask = sample_mask;
        self
    }

    /// When enabled, produces another sample mask per pixel based on the alpha output value, that
    /// is ANDed with the sample_mask and the primitive coverage to restrict the set of samples
    /// affected by a primitive.
    ///
    /// The implicit mask produced for alpha of zero is guaranteed to be zero, and for alpha of one
    /// is guaranteed to be all 1-s.
    ///
    /// Disabled by default.
    pub fn alpha_to_coverage_enabled(mut self, b: bool) -> Self {
        self.multisample.alpha_to_coverage_enabled = b;
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
        primitive,
        color_state,
        color_states,
        depth_stencil,
        multisample,
        vertex_buffers,
    } = builder;

    let vertex = wgpu::VertexState {
        module: &vs_mod,
        entry_point: vs_entry_point,
        buffers: &vertex_buffers[..],
    };

    let mut single_color_state = [RenderPipelineBuilder::DEFAULT_COLOR_STATE];
    let color_states = match (fs_mod.is_some(), color_states.is_empty()) {
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
    let fragment = match (fs_mod, color_states.is_empty()) {
        (Some(fs_mod), false) => Some(wgpu::FragmentState {
            module: &fs_mod,
            entry_point: fs_entry_point,
            targets: color_states,
        }),
        _ => None,
    };

    let pipeline_desc = wgpu::RenderPipelineDescriptor {
        label: Some("nannou render pipeline"),
        layout: Some(layout),
        vertex,
        primitive,
        depth_stencil,
        multisample,
        fragment,
    };

    device.create_render_pipeline(&pipeline_desc)
}
