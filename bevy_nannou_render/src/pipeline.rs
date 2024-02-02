use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use bevy::core::cast_slice;
use bevy::ecs::query::QueryItem;
use bevy::prelude::*;
use bevy::render::render_graph::{NodeRunError, RenderGraphContext, ViewNode};
use bevy::render::render_resource as wgpu;
use bevy::render::render_resource::{BufferInitDescriptor, LoadOp, Operations, PipelineCache, RenderPassDescriptor, RenderPipelineDescriptor, SpecializedRenderPipeline};
use bevy::render::renderer::{RenderContext, RenderDevice};
use bevy::render::view::{ViewDepthTexture, ViewTarget, ViewUniform, ViewUniformOffset};
use bevy_nannou_draw::draw::mesh;
use bevy_nannou_draw::draw::mesh::vertex::Point;
use bevy_nannou_draw::draw::render::VertexMode;

use crate::ViewMesh;
use crate::{
    RenderCommand, Scissor, ViewRenderCommands, ViewUniformBindGroup,
    NANNOU_SHADER_HANDLE,
};

#[derive(Resource)]
pub struct NannouPipeline {
    glyph_cache_texture: wgpu::Texture,
    text_bind_group_layout: wgpu::BindGroupLayout,
    text_bind_group: wgpu::BindGroup,
    texture_samplers: HashMap<wgpu::SamplerId, wgpu::Sampler>,
    pub(crate) texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group: wgpu::BindGroup,
    output_color_format: wgpu::TextureFormat,
    pub(crate) view_bind_group_layout: wgpu::BindGroupLayout,
    view_bind_group: wgpu::BindGroup,
    view_buffer: wgpu::Buffer,
    pub(crate) vertex_mode_buffer: Vec<VertexMode>,
}

// This key is computed and used to cache the pipeline.
#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub struct NannouPipelineKey {
    pub sample_count: u32,
    pub depth_format: wgpu::TextureFormat,
    pub blend_state: wgpu::BlendState,
    pub topology: wgpu::PrimitiveTopology,
}

impl NannouPipeline {
    /// The default sample count
    pub const DEFAULT_SAMPLE_COUNT: u32 = 1;
    /// The default depth format
    pub const DEFAULT_DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    pub const DEFAULT_COLOR_BLEND: wgpu::BlendComponent = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::SrcAlpha,
        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
        operation: wgpu::BlendOperation::Add,
    };
    pub const DEFAULT_ALPHA_BLEND: wgpu::BlendComponent = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
        operation: wgpu::BlendOperation::Add,
    };
    /// The default color blend state
    pub const DEFAULT_BLEND_STATE: wgpu::BlendState = wgpu::BlendState {
        color: Self::DEFAULT_COLOR_BLEND,
        alpha: Self::DEFAULT_ALPHA_BLEND,
    };
    /// The default primitive topology
    pub const DEFAULT_PRIMITIVE_TOPOLOGY: wgpu::PrimitiveTopology =
        wgpu::PrimitiveTopology::TriangleList;
    pub const GLYPH_CACHE_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;

    fn render_pipeline(
        &self,
        color_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
        sample_count: u32,
        blend_state: wgpu::BlendState,
        topology: wgpu::PrimitiveTopology,
    ) -> RenderPipelineDescriptor {
        bevy_nannou_wgpu::RenderPipelineBuilder::from_layout(
            &[
                self.view_bind_group_layout.clone(),
                self.text_bind_group_layout.clone(),
                self.texture_bind_group_layout.clone(),
            ],
            NANNOU_SHADER_HANDLE,
        )
        .vertex_entry_point("vertex")
        .fragment_shader(NANNOU_SHADER_HANDLE)
        .fragment_entry_point("fragment")
        .color_format(color_format)
        .add_vertex_buffer::<Point>(&[wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x3,
            offset: 0,
            shader_location: 0,
        }])
        .add_vertex_buffer::<mesh::vertex::Color>(&[wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x4,
            offset: 0,
            shader_location: 1,
        }])
        .add_vertex_buffer::<mesh::vertex::TexCoords>(&[wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x2,
            offset: 0,
            shader_location: 2,
        }])
        .add_vertex_buffer::<VertexMode>(&[wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Uint32,
            offset: 0,
            shader_location: 3,
        }])
        // .depth_format(depth_format)
        .sample_count(sample_count)
        .color_blend(blend_state.color)
        .alpha_blend(blend_state.alpha)
        .primitive_topology(topology)
        .build()
    }

    fn create_texture_bind_group_layout(
        device: &RenderDevice,
        filtering: bool,
        texture_sample_type: wgpu::TextureSampleType,
    ) -> wgpu::BindGroupLayout {
        bevy_nannou_wgpu::BindGroupLayoutBuilder::new()
            .sampler(wgpu::ShaderStages::FRAGMENT, filtering)
            .texture(
                wgpu::ShaderStages::FRAGMENT,
                false,
                wgpu::TextureViewDimension::D2,
                texture_sample_type,
            )
            .build(device)
    }

    fn create_text_bind_group(
        device: &RenderDevice,
        layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        glyph_cache_texture_view: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        bevy_nannou_wgpu::BindGroupBuilder::new()
            .sampler(sampler)
            .texture_view(glyph_cache_texture_view)
            .build(device, layout)
    }

    pub fn create_texture_bind_group(
        device: &RenderDevice,
        layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        texture_view: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        bevy_nannou_wgpu::BindGroupBuilder::new()
            .sampler(sampler)
            .texture_view(texture_view)
            .build(device, layout)
    }

    fn sampler_descriptor_hash(desc: &wgpu::SamplerDescriptor) -> wgpu::SamplerId {
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
        // TODO: can we just use bevy's version?
        let id = s.finish() as u32;
        unsafe { std::mem::transmute(id) }
    }

    fn create_text_bind_group_layout(
        device: &RenderDevice,
        filtering: bool,
    ) -> wgpu::BindGroupLayout {
        bevy_nannou_wgpu::BindGroupLayoutBuilder::new()
            .sampler(wgpu::ShaderStages::FRAGMENT, filtering)
            .texture(
                wgpu::ShaderStages::FRAGMENT,
                false,
                wgpu::TextureViewDimension::D2,
                wgpu::TextureFormat::R8Unorm
                    .sample_type(None)
                    .expect("Expected format to have sample type"),
            )
            .build(device)
    }
}

impl SpecializedRenderPipeline for NannouPipeline {
    type Key = NannouPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        self.render_pipeline(
            self.output_color_format,
            key.depth_format,
            key.sample_count,
            key.blend_state,
            key.topology,
        )
    }
}

impl FromWorld for NannouPipeline {
    fn from_world(render_world: &mut World) -> Self {
        let device = render_world.get_resource::<RenderDevice>().unwrap();

        // Create the glyph cache texture.
        let text_sampler_desc = bevy_nannou_wgpu::SamplerBuilder::new().into_descriptor();
        let text_sampler_filtering = bevy_nannou_wgpu::sampler_filtering(&text_sampler_desc);
        let text_sampler = device.create_sampler(&text_sampler_desc);
        let glyph_cache_texture = bevy_nannou_wgpu::TextureBuilder::new()
            .size([1024; 2])
            .usage(wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST)
            .format(Self::GLYPH_CACHE_TEXTURE_FORMAT)
            .build(device);


        let glyph_cache_texture_view =
            glyph_cache_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // The default texture for the case where the user has not specified one.
        let default_texture = bevy_nannou_wgpu::TextureBuilder::new()
            .size([64; 2])
            .usage(wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST)
            .build(device);

        // Bind group for text.
        let text_bind_group_layout =
            Self::create_text_bind_group_layout(device, text_sampler_filtering);
        let text_bind_group = Self::create_text_bind_group(
            device,
            &text_bind_group_layout,
            &text_sampler,
            &glyph_cache_texture_view,
        );

        // Initialise the sampler set with the default sampler.
        let sampler_desc = bevy_nannou_wgpu::SamplerBuilder::new().into_descriptor();
        let sampler_id = Self::sampler_descriptor_hash(&sampler_desc);
        let texture_sampler = device.create_sampler(&sampler_desc);
        let texture_samplers = Some((sampler_id, texture_sampler.clone()))
            .into_iter()
            .collect();

        let texture_bind_group_layout = Self::create_texture_bind_group_layout(
            device,
            bevy_nannou_wgpu::sampler_filtering(&sampler_desc),
            wgpu::TextureSampleType::Float { filterable: true },
        );
        let texture_bind_group = Self::create_texture_bind_group(
            device,
            &texture_bind_group_layout,
            &texture_sampler,
            &default_texture.create_view(&wgpu::TextureViewDescriptor::default()),
        );

        // Create the view bind group.
        let usage = wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST;
        let view_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("nannou Renderer uniform_buffer"),
            usage,
            size: std::mem::size_of::<ViewUniform>() as u64,
            mapped_at_creation: false,
        });
        let view_bind_group_layout = bevy_nannou_wgpu::BindGroupLayoutBuilder::new()
            .uniform_buffer(wgpu::ShaderStages::VERTEX, true)
            .build(device);
        let view_bind_group = bevy_nannou_wgpu::BindGroupBuilder::new()
            .buffer::<ViewUniform>(&view_buffer, 0..1)
            .build(device, &view_bind_group_layout);

        let vertex_mode_buffer = Vec::new();

        NannouPipeline {
            glyph_cache_texture,
            text_bind_group_layout,
            text_bind_group,
            texture_samplers,
            texture_bind_group_layout,
            texture_bind_group,
            // TODO: make configurable.
            output_color_format: wgpu::TextureFormat::Rgba8UnormSrgb,
            view_bind_group_layout,
            view_bind_group,
            view_buffer,
            vertex_mode_buffer,
        }
    }
}

#[derive(Resource, Deref, DerefMut, Default)]
pub struct TextureBindGroupCache(HashMap<Handle<Image>, wgpu::BindGroup>);

pub struct NannouViewNode;

impl NannouViewNode {
    pub const NAME: &'static str = "nannou";
}

impl FromWorld for NannouViewNode {
    fn from_world(_world: &mut World) -> Self {
        NannouViewNode
    }
}

impl ViewNode for NannouViewNode {
    type ViewQuery = (
        Entity,
        &'static ViewTarget,
        &'static ViewUniformOffset,
        &'static ViewMesh,
        &'static ViewRenderCommands,
        &'static ViewDepthTexture,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (entity, target, uniform_offset, mesh, render_commands, depth_texture): QueryItem<
            Self::ViewQuery,
        >,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let nannou_pipeline = world.resource::<NannouPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let bind_group_cache = world.resource::<TextureBindGroupCache>();

        let render_device = render_context.render_device();

        // TODO: we should just be able to cast the color slice
        let colors = mesh.colors().iter().map(|c| Vec4::new(c.red, c.green, c.blue, c.alpha)).collect::<Vec<Vec4>>();
        let modes = nannou_pipeline.vertex_mode_buffer.iter().map(|vm| *vm as u32).collect::<Vec<u32>>();
        let vertex_usage = wgpu::BufferUsages::VERTEX;
        let points_bytes = cast_slice(&mesh.points()[..]);
        let colors_bytes = cast_slice(&colors);
        let tex_coords_bytes = cast_slice(mesh.tex_coords());
        let modes_bytes = cast_slice(&modes);
        let indices_bytes = cast_slice(mesh.indices());
        let point_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("nannou Renderer point_buffer"),
            contents: points_bytes,
            usage: vertex_usage,
        });
        let color_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("nannou Renderer color_buffer"),
            contents: colors_bytes,
            usage: vertex_usage,
        });
        let tex_coords_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("nannou Renderer tex_coords_buffer"),
            contents: tex_coords_bytes,
            usage: vertex_usage,
        });
        let mode_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("nannou Renderer mode_buffer"),
            contents: modes_bytes,
            usage: vertex_usage,
        });
        let index_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("nannou Renderer index_buffer"),
            contents: indices_bytes,
            usage: wgpu::BufferUsages::INDEX,
        });

        // Create render pass builder.
        let render_pass_descriptor = RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(target.get_color_attachment(Operations {
                load: LoadOp::Load,
                store: true,
            }))],
            depth_stencil_attachment: None,
        };
        let mut render_pass = render_context.begin_tracked_render_pass(render_pass_descriptor);

        // Set the buffers.
        render_pass.set_index_buffer(index_buffer.slice(..), 0, wgpu::IndexFormat::Uint32);
        render_pass.set_vertex_buffer(0, point_buffer.slice(..));
        render_pass.set_vertex_buffer(1, color_buffer.slice(..));
        render_pass.set_vertex_buffer(2, tex_coords_buffer.slice(..));
        render_pass.set_vertex_buffer(3, mode_buffer.slice(..));

        // Set the uniform and text bind groups here.
        let uniform_bind_group = world.resource::<ViewUniformBindGroup>();
        render_pass.set_bind_group(0, &uniform_bind_group.bind_group, &[uniform_offset.offset]);
        render_pass.set_bind_group(1, &nannou_pipeline.text_bind_group, &[]);

        // Follow the render commands.
        // todo: can we get mutable access in `view`?
        for cmd in render_commands.deref().clone() {
            match cmd {
                RenderCommand::SetPipeline(id) => {
                    let pipeline = pipeline_cache
                        .get_render_pipeline(id)
                        .expect("Expected pipeline to exist");
                    render_pass.set_render_pipeline(pipeline);
                }

                RenderCommand::SetBindGroup(texture) => {
                    let bind_group = bind_group_cache
                        .get(&texture)
                        .expect("Expected texture bind group to exist");
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

        Ok(())
    }
}
