use bevy::ecs::query::ROQueryItem;
use bevy::ecs::system::lifetimeless::{Read, SRes};
use bevy::ecs::system::SystemParamItem;
use bevy::pbr::SetMeshViewBindGroup;
use bevy::render::extract_component::DynamicUniformIndex;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_phase::{
    PhaseItem, RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass,
};
use bevy::render::render_resource as wgpu;

use crate::pipeline::NannouPipeline;
use crate::{
    DefaultTextureHandle, DrawMesh, DrawMeshHandle, DrawMeshItem, DrawMeshUniform,
    DrawMeshUniformBindGroup, TextureBindGroupCache,
};

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
    type Param = (SRes<DefaultTextureHandle>, SRes<TextureBindGroupCache>);
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
        // Note: this is a no-op if the buffers have already been set on this pass
        pass.set_index_buffer(mesh.index_buffer.slice(..), 0, wgpu::IndexFormat::Uint32);
        pass.set_vertex_buffer(0, mesh.point_buffer.slice(..));
        pass.set_vertex_buffer(1, mesh.color_buffer.slice(..));
        pass.set_vertex_buffer(2, mesh.tex_coords_buffer.slice(..));

        pass.draw_indexed(draw_mesh_item.index_range.clone(), 0, 0..1);
        RenderCommandResult::Success
    }
}
