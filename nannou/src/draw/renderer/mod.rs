pub mod primitives;

pub use self::primitives::{PrimitiveRenderer, RenderContext, RenderPrimitive};
use crate::draw;
use crate::frame::Frame;
use crate::geom::{self, Rect};
use crate::glam::{Mat4, Vec2, Vec3};
use crate::math::map_range;
use crate::text;
use crate::wgpu;
use lyon::tessellation::{FillTessellator, StrokeTessellator};
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use wgpu::util::{BufferInitDescriptor, DeviceExt};

/// Information about the way in which a primitive was rendered.
pub struct PrimitiveRender {
    /// Whether or not a specific texture must be available when this primitive is drawn.
    ///
    /// If `Some` and the given texture is different than the currently set texture, a render
    /// command will be encoded that switches from the previous texture's bind group to the new
    /// one.
    pub texture_view: Option<wgpu::TextureView>,
    /// The way in which vertices should be coloured in the fragment shader.
    pub vertex_mode: VertexMode,
}

struct PrimitiveRendererImpl<'a> {
    mesh: &'a mut draw::Mesh,
    transform: &'a Mat4,
    intermediary_mesh: &'a draw::Mesh,
    theme: &'a draw::Theme,
    glyph_cache: &'a mut GlyphCache,
    fill_tessellator: &'a mut FillTessellator,
    stroke_tessellator: &'a mut StrokeTessellator,
    output_attachment_size: Vec2, // logical coords
    output_attachment_scale_factor: f32,
}

pub struct GlyphCache {
    /// Tracks glyphs and their location within the cache.
    pub cache: text::GlyphCache<'static>,
    /// The buffer used to store the pixels of the glyphs.
    pub pixel_buffer: Vec<u8>,
    /// Will be set to `true` after the cache has been updated if the texture requires re-uploading.
    pub requires_upload: bool,
}

/// A top-level indicator of whether or not
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[repr(u32)]
pub enum VertexMode {
    /// Use the color values and ignore the texture coordinates.
    Color = 0,
    /// Use the texture color and ignore the color values.
    Texture = 1,
    /// A special mode used by the text primitive.
    ///
    /// Uses the color values, but multiplies the alpha by the glyph cache texture's red value.
    Text = 2,
}

/// A helper type aimed at simplifying the rendering of conrod primitives via wgpu.
#[derive(Debug)]
pub struct Renderer {
    glyph_cache: GlyphCache,
    vs_mod: wgpu::ShaderModule,
    fs_mod: wgpu::ShaderModule,
    // One pipeline per unique Pipeline ID (combination of blend, topology and component type).
    pipelines: HashMap<PipelineId, wgpu::RenderPipeline>,
    glyph_cache_texture: wgpu::Texture,
    depth_texture: wgpu::Texture,
    depth_texture_view: wgpu::TextureView,
    default_texture: wgpu::Texture,
    default_texture_view: wgpu::TextureView,
    uniform_bind_group_layout: wgpu::BindGroupLayout,
    uniform_bind_group: wgpu::BindGroup,
    text_bind_group_layout: wgpu::BindGroupLayout,
    text_bind_group: wgpu::BindGroup,
    texture_samplers: HashMap<SamplerId, wgpu::Sampler>,
    texture_bind_group_layouts: HashMap<wgpu::TextureSampleType, wgpu::BindGroupLayout>,
    texture_bind_groups: HashMap<BindGroupId, wgpu::BindGroup>,
    output_color_format: wgpu::TextureFormat,
    sample_count: u32,
    scale_factor: f32,
    render_commands: Vec<RenderCommand>,
    mesh: draw::Mesh,
    vertex_mode_buffer: Vec<VertexMode>,
    uniform_buffer: wgpu::Buffer,
}

/// A type aimed at simplifying construction of a `draw::Renderer`.
#[derive(Clone, Debug)]
pub struct Builder {
    pub depth_format: wgpu::TextureFormat,
    pub glyph_cache_size: [u32; 2],
    pub glyph_cache_scale_tolerance: f32,
    pub glyph_cache_position_tolerance: f32,
}

/// Commands that map to wgpu encodable commands.
#[derive(Debug)]
enum RenderCommand {
    /// Change pipeline for the new blend mode and topology.
    SetPipeline(PipelineId),
    /// Change bind group for a new image.
    SetBindGroup(BindGroupId),
    /// Set the rectangular scissor.
    SetScissor(Scissor),
    /// Draw the given vertex range.
    DrawIndexed {
        start_vertex: i32,
        index_range: std::ops::Range<u32>,
    },
}

/// The position and dimensions of the scissor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scissor {
    left: u32,
    bottom: u32,
    width: u32,
    height: u32,
}

#[derive(Debug)]
pub struct DrawError;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Uniforms {
    /// Translates from "logical pixel coordinate space" (our "world space") to screen space.
    ///
    /// Specifically:
    ///
    /// - x is transformed from (-half_logical_win_w, half_logical_win_w) to (-1, 1).
    /// - y is transformed from (-half_logical_win_h, half_logical_win_h) to (1, -1).
    /// - z is transformed from (-max_logical_win_side, max_logical_win_side) to (0, 1).
    proj: Mat4,
}

type SamplerId = u64;
type BindGroupId = (SamplerId, wgpu::TextureViewId);
type BlendId = u64;
type ColorId = BlendId;
type AlphaId = BlendId;

/// Each of the properties that indicate a unique pipeline.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
struct PipelineId {
    color_id: ColorId,
    alpha_id: AlphaId,
    topology: wgpu::PrimitiveTopology,
    texture_sample_type: wgpu::TextureSampleType,
}

impl Default for PrimitiveRender {
    fn default() -> Self {
        Self::color()
    }
}

impl fmt::Debug for GlyphCache {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("GlyphCache")
            .field("cache", &self.cache.dimensions())
            .field("pixel_buffer", &self.pixel_buffer.len())
            .field("requires_upload", &self.requires_upload)
            .finish()
    }
}

impl PrimitiveRender {
    /// Specify a vertex mode for the primitive render.
    pub fn vertex_mode(vertex_mode: VertexMode) -> Self {
        PrimitiveRender {
            texture_view: None,
            vertex_mode,
        }
    }

    pub fn color() -> Self {
        Self::vertex_mode(VertexMode::Color)
    }

    pub fn texture(texture_view: wgpu::TextureView) -> Self {
        PrimitiveRender {
            vertex_mode: VertexMode::Texture,
            texture_view: Some(texture_view),
        }
    }

    pub fn text() -> Self {
        Self::vertex_mode(VertexMode::Text)
    }
}

impl Builder {
    /// The default depth format
    pub const DEFAULT_DEPTH_FORMAT: wgpu::TextureFormat = Renderer::DEFAULT_DEPTH_FORMAT;
    /// The default size for the inner glyph cache.
    pub const DEFAULT_GLYPH_CACHE_SIZE: [u32; 2] = Renderer::DEFAULT_GLYPH_CACHE_SIZE;
    /// The default scale tolerance for the glyph cache.
    pub const DEFAULT_GLYPH_CACHE_SCALE_TOLERANCE: f32 =
        Renderer::DEFAULT_GLYPH_CACHE_SCALE_TOLERANCE;
    /// The default position tolerance for the glyph cache.
    pub const DEFAULT_GLYPH_CACHE_POSITION_TOLERANCE: f32 =
        Renderer::DEFAULT_GLYPH_CACHE_POSITION_TOLERANCE;

    /// Begin building a new **draw::Renderer**.
    pub fn new() -> Self {
        Self {
            depth_format: Self::DEFAULT_DEPTH_FORMAT,
            glyph_cache_size: Self::DEFAULT_GLYPH_CACHE_SIZE,
            glyph_cache_scale_tolerance: Self::DEFAULT_GLYPH_CACHE_SCALE_TOLERANCE,
            glyph_cache_position_tolerance: Self::DEFAULT_GLYPH_CACHE_POSITION_TOLERANCE,
        }
    }

    /// Specify the texture format that should be used to represent depth data in the renderer's
    /// inner `depth_texture`.
    pub fn depth_format(mut self, format: wgpu::TextureFormat) -> Self {
        self.depth_format = format;
        self
    }

    /// The dimensions of the texture used to cache glyphs.
    ///
    /// Some text-heavy apps may require a text cache larger than the default size in order to run
    /// efficiently without text glitching. If the texture is insufficiently large for all text
    /// currently appearing within the output attachment, artifacts will appear in the text.
    pub fn glyph_cache_size(mut self, size: [u32; 2]) -> Self {
        self.glyph_cache_size = size;
        self
    }

    /// Specifies the tolerances (maximum allowed difference) for judging whether an existing glyph
    /// in the cache is close enough to the requested glyph in scale to be used in its place.
    ///
    /// Due to floating point inaccuracies a min value of 0.001 is enforced.
    pub fn glyph_cache_scale_tolerance(mut self, tolerance: f32) -> Self {
        self.glyph_cache_scale_tolerance = tolerance;
        self
    }

    /// Specifies the tolerances (maximum allowed difference) for judging whether an existing glyph
    /// in the cache is close enough to the requested glyph in subpixel offset to be used in its
    /// place.
    ///
    /// Due to floating point inaccuracies a min value of 0.001 is enforced.
    pub fn glyph_cache_position_tolerance(mut self, tolerance: f32) -> Self {
        self.glyph_cache_position_tolerance = tolerance;
        self
    }

    /// Build the **draw::Renderer** ready to target an output attachment of the given descriptor.
    pub fn build_from_texture_descriptor(
        self,
        device: &wgpu::Device,
        descriptor: &wgpu::TextureDescriptor,
    ) -> Renderer {
        let scale_factor = 1.0;
        self.build(
            device,
            [descriptor.size.width, descriptor.size.height],
            scale_factor,
            descriptor.sample_count,
            descriptor.format,
        )
    }

    /// Build the **draw::Renderer** ready to target an output attachment with the given size,
    /// sample count and format.
    pub fn build(
        self,
        device: &wgpu::Device,
        output_attachment_size: [u32; 2],
        output_scale_factor: f32,
        sample_count: u32,
        output_color_format: wgpu::TextureFormat,
    ) -> Renderer {
        Renderer::new(
            device,
            output_attachment_size,
            output_scale_factor,
            sample_count,
            output_color_format,
            self.depth_format,
            self.glyph_cache_size,
            self.glyph_cache_scale_tolerance,
            self.glyph_cache_position_tolerance,
        )
    }
}

impl GlyphCache {
    fn new(size: [u32; 2], scale_tolerance: f32, position_tolerance: f32) -> Self {
        let [w, h] = size;
        let cache = text::GlyphCache::builder()
            .dimensions(w, h)
            .scale_tolerance(scale_tolerance)
            .position_tolerance(position_tolerance)
            .build()
            .into();
        let pixel_buffer = vec![0u8; w as usize * h as usize];
        let requires_upload = false;
        GlyphCache {
            cache,
            pixel_buffer,
            requires_upload,
        }
    }
}

impl Renderer {
    /// The default depth format
    pub const DEFAULT_DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    /// The default size for the inner glyph cache.
    pub const DEFAULT_GLYPH_CACHE_SIZE: [u32; 2] = [1024; 2];
    /// The default scale tolerance for the glyph cache.
    pub const DEFAULT_GLYPH_CACHE_SCALE_TOLERANCE: f32 = 0.1;
    /// The default position tolerance for the glyph cache.
    pub const DEFAULT_GLYPH_CACHE_POSITION_TOLERANCE: f32 = 0.1;
    /// The texture format of the inner glyph cache.
    pub const GLYPH_CACHE_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;
    /// The index format used to index into vertices.
    pub const INDEX_FORMAT: wgpu::IndexFormat = wgpu::IndexFormat::Uint32;

    /// Create a new **Renderer**, ready to target an output attachment with the given size, sample
    /// count and color format.
    ///
    /// See the **RendererBuilder** type for a simplified approach to building a renderer that will
    /// fall back to a set of reasonable defaults.
    ///
    /// The `depth_format` will be used to construct a depth texture for depth testing.
    ///
    /// The `glyph_cache_size` will be used to create a texture on which glyphs will be stored for
    /// efficient look-up.
    pub fn new(
        device: &wgpu::Device,
        output_attachment_size: [u32; 2],
        output_scale_factor: f32,
        sample_count: u32,
        output_color_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
        glyph_cache_size: [u32; 2],
        glyph_cache_scale_tolerance: f32,
        glyph_cache_position_tolerance: f32,
    ) -> Self {
        // Construct the glyph cache.
        let glyph_cache = GlyphCache::new(
            glyph_cache_size,
            glyph_cache_scale_tolerance,
            glyph_cache_position_tolerance,
        );

        // Load shader modules.
        let vs_mod = wgpu::shader_from_spirv_bytes(device, include_bytes!("shaders/vert.spv"));
        let fs_mod = wgpu::shader_from_spirv_bytes(device, include_bytes!("shaders/frag.spv"));

        // Create the glyph cache texture.
        let text_sampler_desc = wgpu::SamplerBuilder::new().into_descriptor();
        let text_sampler_filtering = wgpu::sampler_filtering(&text_sampler_desc);
        let text_sampler = device.create_sampler(&text_sampler_desc);
        let glyph_cache_texture = wgpu::TextureBuilder::new()
            .size(glyph_cache_size)
            .usage(wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST)
            .format(Self::GLYPH_CACHE_TEXTURE_FORMAT)
            .build(device);
        let glyph_cache_texture_view =
            glyph_cache_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create the depth texture.
        let depth_texture =
            create_depth_texture(device, output_attachment_size, depth_format, sample_count);
        let depth_texture_view = depth_texture.view().build();

        // The default texture for the case where the user has not specified one.
        let default_texture = wgpu::TextureBuilder::new()
            .size([64; 2])
            .usage(wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST)
            .build(device);
        let default_texture_view = default_texture.view().build();

        // Initial uniform buffer values. These will be overridden on draw.
        let uniforms = create_uniforms(output_attachment_size, output_scale_factor);
        let contents = uniforms_as_bytes(&uniforms);
        let usage = wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST;
        let uniform_buffer = device.create_buffer_init(&wgpu::BufferInitDescriptor {
            label: Some("nannou Renderer uniform_buffer"),
            contents,
            usage,
        });

        // Bind group for uniforms.
        let uniform_bind_group_layout = create_uniform_bind_group_layout(device);
        let uniform_bind_group =
            create_uniform_bind_group(device, &uniform_bind_group_layout, &uniform_buffer);

        // Bind group for text.
        let text_bind_group_layout = create_text_bind_group_layout(device, text_sampler_filtering);
        let text_bind_group = create_text_bind_group(
            device,
            &text_bind_group_layout,
            &text_sampler,
            &glyph_cache_texture_view,
        );

        // Initialise the sampler set with the default sampler.
        let sampler_desc = wgpu::SamplerBuilder::new().into_descriptor();
        let sampler_id = sampler_descriptor_hash(&sampler_desc);
        let texture_sampler = device.create_sampler(&sampler_desc);

        // Bind group per user-uploaded texture.
        let texture_bind_group_layouts = Default::default();
        let texture_bind_groups = Default::default();

        // Pipeline per unique pipelin ID.
        let pipelines = HashMap::default();

        let texture_samplers = Some((sampler_id, texture_sampler)).into_iter().collect();
        let render_commands = vec![];
        let mesh = Default::default();
        let vertex_mode_buffer = vec![];

        Self {
            vs_mod,
            fs_mod,
            glyph_cache,
            glyph_cache_texture,
            depth_texture,
            depth_texture_view,
            default_texture,
            default_texture_view,
            uniform_bind_group_layout,
            uniform_bind_group,
            text_bind_group_layout,
            text_bind_group,
            texture_samplers,
            texture_bind_group_layouts,
            texture_bind_groups,
            pipelines,
            output_color_format,
            sample_count,
            scale_factor: output_scale_factor,
            render_commands,
            mesh,
            vertex_mode_buffer,
            uniform_buffer,
        }
    }

    /// Clear all pending render commands vertex data.
    pub fn clear(&mut self) {
        self.render_commands.clear();
        self.mesh.clear();
        self.vertex_mode_buffer.clear();
    }

    /// Generate a list of `RenderCommand`s from the given **Draw** instance and prepare any
    /// necessary vertex data.
    ///
    /// Note that the given **Draw** instance will be *drained* of its commands.
    pub fn fill(
        &mut self,
        device: &wgpu::Device,
        draw: &draw::Draw,
        scale_factor: f32,
        output_attachment_size: [u32; 2],
    ) {
        // Pushes a draw command and updates the `curr_start_index`.
        //
        // Returns `true` if the command was added, `false` if there was nothing to
        // draw.
        fn push_draw_cmd(
            curr_start_index: &mut u32,
            end_index: u32,
            render_commands: &mut Vec<RenderCommand>,
        ) -> bool {
            let index_range = *curr_start_index..end_index;
            if index_range.len() != 0 {
                let start_vertex = 0;
                *curr_start_index = index_range.end;
                let cmd = RenderCommand::DrawIndexed {
                    start_vertex,
                    index_range,
                };
                render_commands.push(cmd);
                true
            } else {
                false
            }
        }

        let [w_px, h_px] = output_attachment_size;

        // Converting between pixels and points.
        let px_to_pt = |s: u32| s as f32 / scale_factor;
        let pt_to_px = |s: f32| (s * scale_factor).round() as u32;
        let full_rect = Rect::from_w_h(px_to_pt(w_px), px_to_pt(h_px));

        let window_to_scissor = |v: Vec2| -> [u32; 2] {
            let x = map_range(v.x, full_rect.left(), full_rect.right(), 0u32, w_px);
            let y = map_range(v.y, full_rect.bottom(), full_rect.top(), 0u32, h_px);
            [x, y]
        };

        // TODO: Store these in `Renderer`.
        let mut fill_tessellator = FillTessellator::new();
        let mut stroke_tessellator = StrokeTessellator::new();

        // Keep track of context changes.
        let mut curr_ctxt = draw::Context::default();
        let mut new_pipeline_ids = HashMap::new();
        let mut curr_start_index = 0;
        let mut new_tex_views = HashMap::new();
        let mut new_tex_sampler_combos = HashMap::new();
        // Track whether new commands are required.
        let mut curr_pipeline_id = None;
        let mut curr_scissor = None;
        let mut curr_tex_sampler_id = None;

        // Collect all draw commands to avoid borrow errors.
        let draw_cmds: Vec<_> = draw.drain_commands().collect();
        let draw_state = draw.state.borrow_mut();
        let intermediary_state = draw_state.intermediary_state.borrow();
        for cmd in draw_cmds {
            match cmd {
                draw::DrawCommand::Context(ctxt) => curr_ctxt = ctxt,
                draw::DrawCommand::Primitive(prim) => {
                    // Track the prev index and vertex counts.
                    let prev_index_count = self.mesh.indices().len() as u32;
                    let prev_vert_count = self.mesh.vertex_count();

                    // Info required during rendering.
                    let ctxt = RenderContext {
                        path_event_buffer: &intermediary_state.path_event_buffer,
                        path_points_colored_buffer: &intermediary_state.path_points_colored_buffer,
                        path_points_textured_buffer: &intermediary_state
                            .path_points_textured_buffer,
                        text_buffer: &intermediary_state.text_buffer,
                        theme: &draw_state.theme,
                    };

                    let renderer = PrimitiveRendererImpl {
                        mesh: &mut self.mesh,
                        transform: &curr_ctxt.transform,
                        intermediary_mesh: &intermediary_state.intermediary_mesh,
                        theme: &draw_state.theme,
                        fill_tessellator: &mut fill_tessellator,
                        stroke_tessellator: &mut stroke_tessellator,
                        glyph_cache: &mut self.glyph_cache,
                        output_attachment_size: Vec2::new(px_to_pt(w_px), px_to_pt(h_px)),
                        output_attachment_scale_factor: scale_factor,
                    };

                    // Render the primitive.
                    let render = RenderPrimitive::render_primitive(prim, ctxt, renderer);

                    // If the mesh indices are unchanged, there's nothing to be drawn.
                    if prev_index_count == self.mesh.indices().len() as u32 {
                        assert_eq!(
                            prev_vert_count,
                            self.mesh.vertex_count(),
                            "vertices were submitted during `render` without submitting indices",
                        );
                        continue;
                    }

                    // Retrieve the current texture view and texture view ID. These are necessary
                    // for producing the current pipeline and bind group IDs. Also ensure we have
                    // an entry for them in our map.
                    let tex_view = match render.texture_view {
                        Some(tex_view) => tex_view,
                        None => self.default_texture_view.clone(),
                    };
                    let tex_view_id = tex_view.id();
                    let texture_sample_type = tex_view.sample_type();
                    new_tex_views.insert(tex_view_id, tex_view);

                    // Determine the new current bind group layout ID, pipeline ID, bind group ID
                    // and scissor required for drawing this primitive.
                    let new_pipeline_id = {
                        let color_id = blend_component_hash(&curr_ctxt.blend.color);
                        let alpha_id = blend_component_hash(&curr_ctxt.blend.alpha);
                        let topology = curr_ctxt.topology;
                        PipelineId {
                            color_id,
                            alpha_id,
                            topology,
                            texture_sample_type,
                        }
                    };
                    let new_bind_group_id = {
                        let sampler_id = sampler_descriptor_hash(&curr_ctxt.sampler);
                        (sampler_id, tex_view_id)
                    };
                    let new_scissor = curr_ctxt.scissor;

                    // Determine which have changed and in turn which require submitting new
                    // commands.
                    let pipeline_changed = Some(new_pipeline_id) != curr_pipeline_id;
                    let bind_group_changed = Some(new_bind_group_id) != curr_tex_sampler_id;
                    let scissor_changed = Some(new_scissor) != curr_scissor;

                    // If we require submitting a scissor, pipeline or bind group command, first
                    // draw whatever pending vertices we have collected so far. If there have been
                    // no graphics yet, this will do nothing.
                    if scissor_changed || pipeline_changed || bind_group_changed {
                        push_draw_cmd(
                            &mut curr_start_index,
                            prev_index_count,
                            &mut self.render_commands,
                        );
                    }

                    // If necessary, push a new pipeline command.
                    if pipeline_changed {
                        curr_pipeline_id = Some(new_pipeline_id);
                        let color_blend = curr_ctxt.blend.color.clone();
                        let alpha_blend = curr_ctxt.blend.alpha.clone();
                        let sampler_filtering = wgpu::sampler_filtering(&curr_ctxt.sampler);
                        new_pipeline_ids.insert(
                            new_pipeline_id,
                            (color_blend, alpha_blend, sampler_filtering),
                        );
                        let cmd = RenderCommand::SetPipeline(new_pipeline_id);
                        self.render_commands.push(cmd);
                    }

                    // If necessary, push a new bind group command.
                    if bind_group_changed {
                        curr_tex_sampler_id = Some(new_bind_group_id);
                        new_tex_sampler_combos.insert(new_bind_group_id, new_pipeline_id);
                        let cmd = RenderCommand::SetBindGroup(new_bind_group_id);
                        self.render_commands.push(cmd);
                    }

                    // If necessary, push a new scissor command.
                    if scissor_changed {
                        curr_scissor = Some(new_scissor);
                        let rect = match curr_ctxt.scissor {
                            draw::Scissor::Full => full_rect,
                            draw::Scissor::Rect(rect) => full_rect
                                .overlap(rect)
                                .unwrap_or(geom::Rect::from_w_h(0.0, 0.0)),
                            draw::Scissor::NoOverlap => geom::Rect::from_w_h(0.0, 0.0),
                        };
                        let [left, bottom] = window_to_scissor(rect.bottom_left().into());
                        let (width, height) = rect.w_h();
                        let (width, height) = (pt_to_px(width), pt_to_px(height));
                        let scissor = Scissor {
                            left,
                            bottom,
                            width,
                            height,
                        };
                        let cmd = RenderCommand::SetScissor(scissor);
                        self.render_commands.push(cmd);
                    }

                    // Extend the vertex mode channel.
                    let mode = render.vertex_mode;
                    let new_vs = self.mesh.points().len() - self.vertex_mode_buffer.len();
                    self.vertex_mode_buffer.extend((0..new_vs).map(|_| mode));
                }
            }
        }

        // Insert the final draw command if there is still some drawing to be done.
        push_draw_cmd(
            &mut curr_start_index,
            self.mesh.indices().len() as u32,
            &mut self.render_commands,
        );

        // Clear out unnecessary pipelines.
        self.pipelines
            .retain(|id, _| new_pipeline_ids.contains_key(id));
        // Clear new combos that we already have.
        new_pipeline_ids.retain(|id, _| !self.pipelines.contains_key(id));
        // Create new render pipelines as necessary.
        for (new_id, (color_blend, alpha_blend, sampler_filtering)) in new_pipeline_ids {
            let bind_group_layout = self
                .texture_bind_group_layouts
                .entry(new_id.texture_sample_type)
                .or_insert_with(|| {
                    create_texture_bind_group_layout(
                        device,
                        sampler_filtering,
                        new_id.texture_sample_type,
                    )
                });
            let new_pipeline = create_render_pipeline(
                device,
                &self.uniform_bind_group_layout,
                &self.text_bind_group_layout,
                &bind_group_layout,
                &self.vs_mod,
                &self.fs_mod,
                self.output_color_format,
                self.depth_texture.format(),
                self.sample_count,
                color_blend,
                alpha_blend,
                new_id.topology,
            );
            self.pipelines.insert(new_id, new_pipeline);
        }

        // Clear out unnecessary bind groups.
        self.texture_bind_groups
            .retain(|id, _| new_tex_sampler_combos.contains_key(id));
        // Clear new combos that we already have.
        new_tex_sampler_combos.retain(|id, _| !self.texture_bind_groups.contains_key(id));
        // Only keep the samplers around that we need.
        self.texture_samplers
            .retain(|id, _| new_tex_sampler_combos.keys().any(|(s_id, _)| id == s_id));
        // Ensure we have a bind group for each of the texture views, but no more.
        for (new_id, pipeline_id) in new_tex_sampler_combos {
            let (new_sampler_id, new_tex_view_id) = new_id;
            // Retrieve the sampler or create it if necessary.
            let sampler = self
                .texture_samplers
                .entry(new_sampler_id)
                .or_insert_with(|| device.create_sampler(&curr_ctxt.sampler));
            // Retrieve the texture view.
            let texture_view = &new_tex_views[&new_tex_view_id];
            // Retrieve the associated bind group layout.
            let bind_group_layout =
                &self.texture_bind_group_layouts[&pipeline_id.texture_sample_type];
            // Create the bind group.
            let bind_group =
                create_texture_bind_group(device, bind_group_layout, sampler, texture_view);
            self.texture_bind_groups.insert(new_id, bind_group);
        }
    }

    /// Encode a render pass with the given **Draw**ing to the given `output_attachment`.
    ///
    /// If the **Draw**ing has been scaled for handling DPI, specify the necessary `scale_factor`
    /// for scaling back to the `output_attachment_size` (physical dimensions).
    ///
    /// If the `output_attachment` is multisampled and should be resolved to another texture,
    /// include the `resolve_target`.
    pub fn encode_render_pass(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        draw: &draw::Draw,
        scale_factor: f32,
        output_attachment_size: [u32; 2],
        output_attachment: &wgpu::TextureView,
        resolve_target: Option<&wgpu::TextureView>,
    ) {
        self.clear();
        self.fill(device, draw, scale_factor, output_attachment_size);

        let Renderer {
            ref pipelines,
            ref glyph_cache,
            ref glyph_cache_texture,
            ref mut depth_texture,
            ref mut depth_texture_view,
            ref uniform_bind_group,
            ref text_bind_group,
            ref texture_bind_groups,
            ref mesh,
            ref vertex_mode_buffer,
            ref mut render_commands,
            ref uniform_buffer,
            scale_factor: ref mut old_scale_factor,
            ..
        } = *self;

        // Update glyph cache texture if necessary.
        if glyph_cache.requires_upload {
            glyph_cache_texture.upload_data(device, encoder, &glyph_cache.pixel_buffer);
        }

        // Resize the depth texture if the output attachment size has changed.
        let depth_size = depth_texture.size();
        if output_attachment_size != depth_size {
            let depth_format = depth_texture.format();
            let sample_count = depth_texture.sample_count();
            *depth_texture =
                create_depth_texture(device, output_attachment_size, depth_format, sample_count);
            *depth_texture_view = depth_texture.view().build();
        }

        // Retrieve the clear values based on the bg color.
        let bg_color = draw.state.borrow().background_color;
        let load_op = match bg_color {
            None => wgpu::LoadOp::Load,
            Some(color) => {
                let (r, g, b, a) = color.into();
                let (r, g, b, a) = (r as f64, g as f64, b as f64, a as f64);
                let clear_color = wgpu::Color { r, g, b, a };
                wgpu::LoadOp::Clear(clear_color)
            }
        };

        // Create render pass builder.
        let render_pass_builder = wgpu::RenderPassBuilder::new()
            .color_attachment(output_attachment, |color| {
                color.resolve_target(resolve_target).load_op(load_op)
            })
            .depth_stencil_attachment(&*depth_texture_view, |depth| depth);

        // Guard for empty mesh.
        if mesh.points().is_empty() {
            // Encode the render pass. Only clears the frame.
            render_pass_builder.begin(encoder);
            return;
        }

        // Create the vertex and index buffers.
        let vertex_usage = wgpu::BufferUsage::VERTEX;
        let points_bytes = points_as_bytes(mesh.points());
        let colors_bytes = colors_as_bytes(mesh.colors());
        let tex_coords_bytes = tex_coords_as_bytes(mesh.tex_coords());
        let modes_bytes = vertex_modes_as_bytes(vertex_mode_buffer);
        let indices_bytes = indices_as_bytes(mesh.indices());
        let point_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("nannou Renderer point_buffer"),
            contents: points_bytes,
            usage: vertex_usage,
        });
        let color_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("nannou Renderer color_buffer"),
            contents: colors_bytes,
            usage: vertex_usage,
        });
        let tex_coords_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("nannou Renderer tex_coords_buffer"),
            contents: tex_coords_bytes,
            usage: vertex_usage,
        });
        let mode_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("nannou Renderer mode_buffer"),
            contents: modes_bytes,
            usage: vertex_usage,
        });
        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("nannou Renderer index_buffer"),
            contents: indices_bytes,
            usage: wgpu::BufferUsage::INDEX,
        });

        // If the scale factor or window size has changed, update the uniforms for vertex scaling.
        if *old_scale_factor != scale_factor || output_attachment_size != depth_size {
            *old_scale_factor = scale_factor;
            // Upload uniform data for vertex scaling.
            let uniforms = create_uniforms(output_attachment_size, scale_factor);
            let uniforms_size = std::mem::size_of::<Uniforms>() as wgpu::BufferAddress;
            let uniforms_bytes = uniforms_as_bytes(&uniforms);
            let usage = wgpu::BufferUsage::COPY_SRC;
            let new_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("nannou Renderer uniform_buffer"),
                contents: uniforms_bytes,
                usage,
            });
            // Copy new uniform buffer state.
            encoder.copy_buffer_to_buffer(&new_uniform_buffer, 0, uniform_buffer, 0, uniforms_size);
        }

        // Encode the render pass.
        let mut render_pass = render_pass_builder.begin(encoder);

        // Set the buffers.
        render_pass.set_index_buffer(index_buffer.slice(..), Self::INDEX_FORMAT);
        render_pass.set_vertex_buffer(0, point_buffer.slice(..));
        render_pass.set_vertex_buffer(1, color_buffer.slice(..));
        render_pass.set_vertex_buffer(2, tex_coords_buffer.slice(..));
        render_pass.set_vertex_buffer(3, mode_buffer.slice(..));

        // Set the uniform and text bind groups here.
        render_pass.set_bind_group(0, uniform_bind_group, &[]);
        render_pass.set_bind_group(1, text_bind_group, &[]);

        // Follow the render commands.
        for cmd in render_commands.drain(..) {
            match cmd {
                RenderCommand::SetPipeline(id) => {
                    let pipeline = &pipelines[&id];
                    render_pass.set_pipeline(pipeline);
                }

                RenderCommand::SetBindGroup(tex_view_id) => {
                    let bind_group = &texture_bind_groups[&tex_view_id];
                    render_pass.set_bind_group(2, bind_group, &[]);
                }

                RenderCommand::SetScissor(Scissor {
                    left,
                    bottom,
                    width,
                    height,
                }) => {
                    render_pass.set_scissor_rect(left, bottom, width, height);
                }

                RenderCommand::DrawIndexed {
                    start_vertex,
                    index_range,
                } => {
                    let instance_range = 0..1u32;
                    render_pass.draw_indexed(index_range, start_vertex, instance_range);
                }
            }
        }
    }

    /// Encode the necessary commands to render the contents of the given **Draw**ing to the given
    /// **Texture**.
    pub fn render_to_texture(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        draw: &draw::Draw,
        texture: &wgpu::Texture,
    ) {
        let size = texture.size();
        let view = texture.view().build();
        // TODO: Should we expose this for rendering to textures?
        let scale_factor = 1.0;
        let resolve_target = None;
        self.encode_render_pass(
            device,
            encoder,
            draw,
            scale_factor,
            size,
            &view,
            resolve_target,
        );
    }

    /// Encode the necessary commands to render the contents of the given **Draw**ing to the given
    /// **Frame**.
    pub fn render_to_frame(
        &mut self,
        device: &wgpu::Device,
        draw: &draw::Draw,
        scale_factor: f32,
        frame: &Frame,
    ) {
        let size = frame.texture().size();
        let attachment = frame.texture_view();
        let resolve_target = None;
        let mut command_encoder = frame.command_encoder();
        self.encode_render_pass(
            device,
            &mut *command_encoder,
            draw,
            scale_factor,
            size,
            attachment,
            resolve_target,
        );
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for GlyphCache {
    type Target = text::GlyphCache<'static>;
    fn deref(&self) -> &Self::Target {
        &self.cache
    }
}

impl DerefMut for GlyphCache {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cache
    }
}

fn create_depth_texture(
    device: &wgpu::Device,
    size: [u32; 2],
    depth_format: wgpu::TextureFormat,
    sample_count: u32,
) -> wgpu::Texture {
    wgpu::TextureBuilder::new()
        .size(size)
        .format(depth_format)
        .usage(wgpu::TextureUsage::RENDER_ATTACHMENT)
        .sample_count(sample_count)
        .build(device)
}

fn create_uniforms([img_w, img_h]: [u32; 2], scale_factor: f32) -> Uniforms {
    let right = img_w as f32 * 0.5 / scale_factor;
    let left = -right;
    let top = img_h as f32 * 0.5 / scale_factor;
    let bottom = -top;
    let far = std::cmp::max(img_w, img_h) as f32 / scale_factor;
    let near = -far;
    let proj = Mat4::orthographic_rh_gl(left, right, bottom, top, near, far);
    // By default, ortho scales z values to the range -1.0 to 1.0. We want to scale and translate
    // the z axis so that it is in the range of 0.0 to 1.0.
    // TODO: Can possibly solve this more easily by using `Mat4::orthographic_rh` above instead.
    let trans = Mat4::from_translation(Vec3::Z);
    let scale = Mat4::from_scale([1.0, 1.0, 0.5].into());
    let proj = scale * trans * proj;
    let proj = proj.into();
    Uniforms { proj }
}

fn create_uniform_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    wgpu::BindGroupLayoutBuilder::new()
        .uniform_buffer(wgpu::ShaderStage::VERTEX, false)
        .build(device)
}

fn create_text_bind_group_layout(device: &wgpu::Device, filtering: bool) -> wgpu::BindGroupLayout {
    wgpu::BindGroupLayoutBuilder::new()
        .sampler(wgpu::ShaderStage::FRAGMENT, filtering)
        .texture(
            wgpu::ShaderStage::FRAGMENT,
            false,
            wgpu::TextureViewDimension::D2,
            Renderer::GLYPH_CACHE_TEXTURE_FORMAT.describe().sample_type,
        )
        .build(device)
}

fn create_texture_bind_group_layout(
    device: &wgpu::Device,
    filtering: bool,
    texture_sample_type: wgpu::TextureSampleType,
) -> wgpu::BindGroupLayout {
    wgpu::BindGroupLayoutBuilder::new()
        .sampler(wgpu::ShaderStage::FRAGMENT, filtering)
        .texture(
            wgpu::ShaderStage::FRAGMENT,
            false,
            wgpu::TextureViewDimension::D2,
            texture_sample_type,
        )
        .build(device)
}

fn create_uniform_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    uniform_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    wgpu::BindGroupBuilder::new()
        .buffer::<Uniforms>(uniform_buffer, 0..1)
        .build(device, layout)
}

fn create_text_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    glyph_cache_texture_view: &wgpu::TextureViewHandle,
) -> wgpu::BindGroup {
    wgpu::BindGroupBuilder::new()
        .sampler(sampler)
        .texture_view(glyph_cache_texture_view)
        .build(device, layout)
}

fn create_texture_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    texture_view: &wgpu::TextureViewHandle,
) -> wgpu::BindGroup {
    wgpu::BindGroupBuilder::new()
        .sampler(sampler)
        .texture_view(texture_view)
        .build(device, layout)
}

fn create_render_pipeline(
    device: &wgpu::Device,
    uniform_layout: &wgpu::BindGroupLayout,
    text_layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
    vs_mod: &wgpu::ShaderModule,
    fs_mod: &wgpu::ShaderModule,
    dst_format: wgpu::TextureFormat,
    depth_format: wgpu::TextureFormat,
    sample_count: u32,
    color_blend: wgpu::BlendComponent,
    alpha_blend: wgpu::BlendComponent,
    topology: wgpu::PrimitiveTopology,
) -> wgpu::RenderPipeline {
    let bind_group_layouts = &[uniform_layout, text_layout, texture_layout];
    wgpu::RenderPipelineBuilder::from_layout_descriptor(&bind_group_layouts[..], vs_mod)
        .fragment_shader(fs_mod)
        .color_format(dst_format)
        .add_vertex_buffer::<draw::mesh::vertex::Point>(&wgpu::vertex_attr_array![0 => Float32x3])
        .add_vertex_buffer::<draw::mesh::vertex::Color>(&wgpu::vertex_attr_array![1 => Float32x4])
        .add_vertex_buffer::<draw::mesh::vertex::TexCoords>(
            &wgpu::vertex_attr_array![2 => Float32x2],
        )
        .add_vertex_buffer::<VertexMode>(&wgpu::vertex_attr_array![3 => Uint32])
        .depth_format(depth_format)
        .sample_count(sample_count)
        .color_blend(color_blend)
        .alpha_blend(alpha_blend)
        .primitive_topology(topology)
        .build(device)
}

fn sampler_descriptor_hash(desc: &wgpu::SamplerDescriptor) -> SamplerId {
    let mut s = std::collections::hash_map::DefaultHasher::new();
    desc.address_mode_u.hash(&mut s);
    desc.address_mode_v.hash(&mut s);
    desc.address_mode_w.hash(&mut s);
    desc.mag_filter.hash(&mut s);
    desc.min_filter.hash(&mut s);
    desc.mipmap_filter.hash(&mut s);
    desc.lod_min_clamp.to_bits().hash(&mut s);
    desc.lod_max_clamp.to_bits().hash(&mut s);
    desc.compare.hash(&mut s);
    desc.anisotropy_clamp.hash(&mut s);
    desc.border_color.hash(&mut s);
    s.finish()
}

fn blend_component_hash(desc: &wgpu::BlendComponent) -> BlendId {
    let mut s = std::collections::hash_map::DefaultHasher::new();
    desc.src_factor.hash(&mut s);
    desc.dst_factor.hash(&mut s);
    desc.operation.hash(&mut s);
    s.finish()
}

// See `nannou::wgpu::bytes` docs for why these are necessary.

fn uniforms_as_bytes(uniforms: &Uniforms) -> &[u8] {
    unsafe { wgpu::bytes::from(uniforms) }
}

fn points_as_bytes(data: &[draw::mesh::vertex::Point]) -> &[u8] {
    unsafe { wgpu::bytes::from_slice(data) }
}

fn colors_as_bytes(data: &[draw::mesh::vertex::Color]) -> &[u8] {
    unsafe { wgpu::bytes::from_slice(data) }
}

fn tex_coords_as_bytes(data: &[draw::mesh::vertex::TexCoords]) -> &[u8] {
    unsafe { wgpu::bytes::from_slice(data) }
}

fn vertex_modes_as_bytes(data: &[VertexMode]) -> &[u8] {
    unsafe { wgpu::bytes::from_slice(data) }
}

fn indices_as_bytes(data: &[u32]) -> &[u8] {
    unsafe { wgpu::bytes::from_slice(data) }
}
