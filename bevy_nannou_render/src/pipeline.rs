use std::hash::Hash;
use std::ops::Range;

use bevy::core_pipeline::core_3d::{Transparent3d, CORE_3D_DEPTH_FORMAT};
use bevy::ecs::query::ROQueryItem;
use bevy::ecs::system::lifetimeless::{Read, SRes};
use bevy::ecs::system::SystemParamItem;
use bevy::pbr::{MeshPipeline, MeshPipelineKey, SetMeshViewBindGroup};
use bevy::prelude::*;
use bevy::render::extract_component::DynamicUniformIndex;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_phase::{
    DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult, RenderPhase, SetItemPipeline,
    TrackedRenderPass,
};
use bevy::render::render_resource as wgpu;
use bevy::render::render_resource::{
    PipelineCache, RenderPipelineDescriptor, SpecializedRenderPipeline, SpecializedRenderPipelines,
};
use bevy::render::renderer::RenderDevice;
use bevy::render::texture::BevyDefault;
use bevy::render::view::{ExtractedView, ViewDepthTexture, ViewTarget, ViewUniform};
use bevy_nannou_draw::draw::mesh;
use bevy_nannou_draw::draw::mesh::vertex::Point;
use bevy_nannou_draw::draw::render::VertexMode;

use crate::{DrawMesh, DrawMeshHandle, DrawMeshItem, DrawMeshUniform, DrawMeshUniformBindGroup, Scissor, TextureBindGroupCache, NANNOU_SHADER_HANDLE, DefaultTextureHandle};

#[derive(Resource)]
pub struct NannouPipeline {
    mesh_pipeline: MeshPipeline,
    glyph_cache_texture: wgpu::Texture,
    text_bind_group_layout: wgpu::BindGroupLayout,
    text_bind_group: wgpu::BindGroup,
    pub(crate) texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group: wgpu::BindGroup,
    pub(crate) mesh_bind_group_layout: wgpu::BindGroupLayout,
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
            glyph_cache_texture,
            text_bind_group_layout,
            text_bind_group,
            texture_bind_group_layout,
            texture_bind_group,
            mesh_bind_group_layout,
            mesh_pipeline,
        }
    }
}

pub fn queue_draw_mesh_items(
    draw_functions: Res<DrawFunctions<Transparent3d>>,
    pipeline: Res<NannouPipeline>,
    mut pipelines: ResMut<SpecializedRenderPipelines<NannouPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    msaa: Res<Msaa>,
    items: Query<(Entity, &DrawMeshItem)>,
    mut views: Query<(&ExtractedView, &mut RenderPhase<Transparent3d>)>,
) {
    let draw_function = draw_functions
        .read()
        .get_id::<DrawDrawMeshItem3d>()
        .unwrap();
    for (view, mut transparent_phase) in &mut views {
        for (entity, item) in items.iter() {
            let key = NannouPipelineKey {
                output_color_format: if view.hdr {
                    ViewTarget::TEXTURE_FORMAT_HDR
                } else {
                    wgpu::TextureFormat::bevy_default()
                },
                sample_count: msaa.samples(),
                topology: item.topology,
                blend_state: item.blend,
            };

            let pipeline = pipelines.specialize(&pipeline_cache, &pipeline, key);

            transparent_phase.add(Transparent3d {
                entity,
                draw_function,
                pipeline,
                distance: 0.,
                batch_range: 0..1,
                dynamic_offset: None,
            });
        }
    }
}

pub type DrawDrawMeshItem3d = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetDrawMeshUniformBindGroup<1>,
    SetDrawMeshTextBindGroup<2>,
    SetDrawMeshTextureBindGroup<3>,
    SetDrawMeshScissor,
    DrawDrawMeshItem,
);

pub struct SetDrawMeshUniformBindGroup<const I: usize>;
impl<const I: usize, P: PhaseItem> RenderCommand<P> for SetDrawMeshUniformBindGroup<I> {
    type Param = SRes<DrawMeshUniformBindGroup>;
    type ViewWorldQuery = ();
    type ItemWorldQuery = Read<DynamicUniformIndex<DrawMeshUniform>>;

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: ROQueryItem<'w, Self::ViewWorldQuery>,
        uniform_index: ROQueryItem<'w, Self::ItemWorldQuery>,
        bind_group: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_bind_group(
            I,
            &bind_group.into_inner().bind_group,
            &[uniform_index.index()],
        );
        RenderCommandResult::Success
    }
}

pub struct SetDrawMeshTextureBindGroup<const I: usize>;
impl<const I: usize, P: PhaseItem> RenderCommand<P> for SetDrawMeshTextureBindGroup<I> {
    type Param = (
        SRes<DefaultTextureHandle>,
        SRes<TextureBindGroupCache>
    );
    type ViewWorldQuery = ();
    type ItemWorldQuery = Read<DrawMeshItem>;

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: ROQueryItem<'w, Self::ViewWorldQuery>,
        draw_mesh_item: ROQueryItem<'w, Self::ItemWorldQuery>,
        (default_texture, bind_groups): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let texture = match &draw_mesh_item.texture {
            None => &default_texture.0,
            Some(texture) => texture,
        };

        let Some(bind_group) = bind_groups.into_inner().get(texture) else {
            return RenderCommandResult::Failure;
        };

        pass.set_bind_group(I, &bind_group, &[]);
        RenderCommandResult::Success
    }
}

pub struct SetDrawMeshTextBindGroup<const I: usize>;
impl<const I: usize, P: PhaseItem> RenderCommand<P> for SetDrawMeshTextBindGroup<I> {
    type Param = SRes<NannouPipeline>;
    type ViewWorldQuery = ();
    type ItemWorldQuery = ();

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: ROQueryItem<'w, Self::ViewWorldQuery>,
        _entity: ROQueryItem<'w, Self::ItemWorldQuery>,
        pipeline: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_bind_group(I, &pipeline.into_inner().text_bind_group, &[]);
        RenderCommandResult::Success
    }
}

pub struct SetDrawMeshScissor;
impl<P: PhaseItem> RenderCommand<P> for SetDrawMeshScissor {
    type Param = ();
    type ViewWorldQuery = ();
    type ItemWorldQuery = Read<DrawMeshItem>;

    fn render<'w>(
        _item: &P,
        _view: ROQueryItem<'w, Self::ViewWorldQuery>,
        entity: ROQueryItem<'w, Self::ItemWorldQuery>,
        _param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Some(scissor) = entity.scissor {
            pass.set_scissor_rect(scissor.left, scissor.bottom, scissor.width, scissor.height);
        }

        RenderCommandResult::Success
    }
}


pub struct DrawDrawMeshItem;
impl<P: PhaseItem> RenderCommand<P> for DrawDrawMeshItem {
    type Param = SRes<RenderAssets<DrawMesh>>;
    type ViewWorldQuery = Read<DrawMeshHandle>;
    type ItemWorldQuery = Read<DrawMeshItem>;

    #[inline]
    fn render<'w>(
        _item: &P,
        handle: ROQueryItem<'w, Self::ViewWorldQuery>,
        draw_mesh_item: ROQueryItem<'w, Self::ItemWorldQuery>,
        draw_meshes: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(mesh) = draw_meshes.into_inner().get(&handle.0) else {
            return RenderCommandResult::Failure;
        };

        // Set the buffers.
        pass.set_index_buffer(mesh.index_buffer.slice(..), 0, wgpu::IndexFormat::Uint32);
        pass.set_vertex_buffer(0, mesh.point_buffer.slice(..));
        pass.set_vertex_buffer(1, mesh.color_buffer.slice(..));
        pass.set_vertex_buffer(2, mesh.tex_coords_buffer.slice(..));

        pass.draw_indexed(draw_mesh_item.index_range.clone(), 0, 0..1);
        RenderCommandResult::Success
    }
}
