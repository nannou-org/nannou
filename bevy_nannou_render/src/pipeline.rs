use std::hash::Hash;

use bevy::core_pipeline::core_3d::CORE_3D_DEPTH_FORMAT;
use bevy::pbr::{MeshPipeline, MeshPipelineKey};
use bevy::prelude::*;
use bevy::render::render_phase::RenderCommand;
use bevy::render::render_resource as wgpu;
use bevy::render::render_resource::{RenderPipelineDescriptor, SpecializedRenderPipeline};
use bevy::render::renderer::RenderDevice;
use bevy::render::view::ViewTarget;

use bevy_nannou_draw::draw::mesh;
use bevy_nannou_draw::draw::mesh::vertex::Point;

use crate::{DrawMeshUniform, NANNOU_SHADER_HANDLE};

#[derive(Resource)]
pub struct NannouPipeline {
    pub mesh_pipeline: MeshPipeline,
    pub glyph_cache_texture: wgpu::Texture,
    pub text_bind_group_layout: wgpu::BindGroupLayout,
    pub text_bind_group: wgpu::BindGroup,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    pub texture_bind_group: wgpu::BindGroup,
    pub mesh_bind_group_layout: wgpu::BindGroupLayout,
}

// This key is computed and used to cache the pipeline.
#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub struct NannouPipelineKey {
    pub output_color_format: wgpu::TextureFormat,
    pub sample_count: u32,
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
        sample_count: u32,
        blend_state: wgpu::BlendState,
        topology: wgpu::PrimitiveTopology,
    ) -> RenderPipelineDescriptor {
        let view_key = MeshPipelineKey::from_msaa_samples(sample_count)
            | MeshPipelineKey::from_hdr(color_format == ViewTarget::TEXTURE_FORMAT_HDR)
            | MeshPipelineKey::from_primitive_topology(topology);
        let view_layout = self.mesh_pipeline.get_view_layout(view_key.into());

        bevy_nannou_wgpu::RenderPipelineBuilder::from_layout(
            &[
                view_layout.clone(),
                self.mesh_bind_group_layout.clone(),
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
        .depth_format(CORE_3D_DEPTH_FORMAT)
        .sample_count(sample_count)
        .color_blend(blend_state.color)
        .alpha_blend(blend_state.alpha)
        .primitive_topology(topology)
        .build()
    }

    pub(crate) fn create_texture_bind_group_layout(
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
}

impl SpecializedRenderPipeline for NannouPipeline {
    type Key = NannouPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        self.render_pipeline(
            key.output_color_format,
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
        let texture_sampler = device.create_sampler(&sampler_desc);

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

        let mesh_bind_group_layout = bevy_nannou_wgpu::BindGroupLayoutBuilder::new()
            .uniform_buffer::<DrawMeshUniform>(wgpu::ShaderStages::VERTEX, true)
            .build(device);
        let mesh_pipeline = render_world.resource::<MeshPipeline>().clone();

        NannouPipeline {
            mesh_pipeline,
            mesh_bind_group_layout,
            glyph_cache_texture,
            text_bind_group_layout,
            text_bind_group,
            texture_bind_group_layout,
            texture_bind_group,
        }
    }
}
