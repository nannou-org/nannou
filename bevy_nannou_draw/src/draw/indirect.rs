//! A shader that renders a mesh multiple times in one draw call.

use crate::render::{ShaderModelHandle, ShaderStorageBufferHandle};
use crate::{
    draw::{drawing::Drawing, primitive::Primitive, Draw, DrawCommand},
    render::{queue_shader_model, PreparedShaderModel, ShaderModel},
};
use bevy::pbr::RenderMaterialInstances;
use bevy::render::extract_instances::ExtractedInstances;
use bevy::{
    core_pipeline::core_3d::Transparent3d,
    ecs::system::{lifetimeless::*, SystemParamItem},
    pbr::{RenderMeshInstances, SetMeshBindGroup, SetMeshViewBindGroup},
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        mesh::{allocator::MeshAllocator, RenderMesh, RenderMeshBufferInfo},
        render_asset::{prepare_assets, RenderAsset, RenderAssets},
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult,
            SetItemPipeline, TrackedRenderPass,
        },
        render_resource::*,
        storage::{GpuShaderStorageBuffer, ShaderStorageBuffer},
        Render, RenderApp, RenderSet,
    },
};
use rayon::prelude::*;
use std::{hash::Hash, marker::PhantomData};

pub struct Indirect<'a, SM>
where
    SM: ShaderModel + Default,
{
    draw: &'a Draw<SM>,
    primitive_index: Option<usize>,
    indirect_buffer: Option<Handle<ShaderStorageBuffer>>,
}

impl<'a, SM> Drop for Indirect<'a, SM>
where
    SM: ShaderModel + Default,
{
    fn drop(&mut self) {
        if let Some((index, ssbo)) = self.primitive_index.take().zip(self.indirect_buffer.take()) {
            self.insert_indirect_draw_command(index, ssbo);
        }
    }
}

pub fn new<SM>(draw: &Draw<SM>) -> Indirect<SM>
where
    SM: ShaderModel + Default,
{
    Indirect {
        draw,
        primitive_index: None,
        indirect_buffer: None,
    }
}

impl<'a, SM> Indirect<'a, SM>
where
    SM: ShaderModel + Default,
{
    pub fn primitive<T>(mut self, drawing: Drawing<T, SM>) -> Indirect<'a, SM>
    where
        T: Into<Primitive>,
    {
        self.draw
            .state
            .write()
            .unwrap()
            .ignored_drawings
            .insert(drawing.index);
        self.primitive_index = Some(drawing.index);
        self
    }

    pub fn buffer(mut self, ssbo: Handle<ShaderStorageBuffer>) -> Indirect<'a, SM> {
        self.indirect_buffer = Some(ssbo);
        self
    }

    fn insert_indirect_draw_command(
        &self,
        index: usize,
        indirect_buffer: Handle<ShaderStorageBuffer>,
    ) {
        let mut state = self.draw.state.write().unwrap();
        let primitive = state.drawing.remove(&index).unwrap();
        state
            .draw_commands
            .push(Some(DrawCommand::Indirect(primitive, indirect_buffer)));
    }
}

#[derive(Component, ExtractComponent, Clone)]
pub struct IndirectMesh;

pub struct IndirectShaderModelPlugin<SM>(PhantomData<SM>);

impl<SM> Default for IndirectShaderModelPlugin<SM>
where
    SM: Default,
{
    fn default() -> Self {
        IndirectShaderModelPlugin(PhantomData)
    }
}

impl<SM> Plugin for IndirectShaderModelPlugin<SM>
where
    SM: ShaderModel,
    SM::Data: PartialEq + Eq + Hash + Clone,
{
    fn build(&self, app: &mut App) {
        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawIndirectShaderModel<SM>>()
            .add_systems(
                Render,
                queue_shader_model::<SM, With<IndirectMesh>, DrawIndirectShaderModel<SM>>
                    .after(prepare_assets::<PreparedShaderModel<SM>>)
                    .in_set(RenderSet::QueueMeshes),
            );
    }
}

type DrawIndirectShaderModel<SM> = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    SetShaderModelBindGroup<SM, 2>,
    DrawMeshIndirect,
);

struct SetShaderModelBindGroup<SM: ShaderModel, const I: usize>(PhantomData<SM>);
impl<P: PhaseItem, SM: ShaderModel, const I: usize> RenderCommand<P>
    for SetShaderModelBindGroup<SM, I>
{
    type Param = (
        SRes<RenderAssets<PreparedShaderModel<SM>>>,
        SRes<ExtractedInstances<ShaderModelHandle<SM>>>,
    );
    type ViewQuery = ();
    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        item: &P,
        _view: (),
        _item_query: Option<()>,
        (models, instances): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let models = models.into_inner();
        let instances = instances.into_inner();

        let Some(handle) = instances.get(&item.main_entity()) else {
            return RenderCommandResult::Skip;
        };
        let Some(model) = models.get(handle.0.id()) else {
            return RenderCommandResult::Skip;
        };
        pass.set_bind_group(I, &model.bind_group, &[]);
        RenderCommandResult::Success
    }
}

struct DrawMeshIndirect;
impl<P: PhaseItem> RenderCommand<P> for DrawMeshIndirect {
    type Param = (
        SRes<RenderAssets<RenderMesh>>,
        SRes<RenderMeshInstances>,
        SRes<MeshAllocator>,
        SRes<RenderAssets<GpuShaderStorageBuffer>>,
    );
    type ViewQuery = ();
    type ItemQuery = Read<ShaderStorageBufferHandle>;

    #[inline]
    fn render<'w>(
        item: &P,
        _view: (),
        indirect_buffer: Option<&'w ShaderStorageBufferHandle>,
        (meshes, render_mesh_instances, mesh_allocator, ssbos): SystemParamItem<
            'w,
            '_,
            Self::Param,
        >,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let mesh_allocator = mesh_allocator.into_inner();

        let Some(mesh_instance) = render_mesh_instances.render_mesh_queue_data(item.main_entity())
        else {
            return RenderCommandResult::Skip;
        };
        let Some(gpu_mesh) = meshes.into_inner().get(mesh_instance.mesh_asset_id) else {
            return RenderCommandResult::Skip;
        };
        let Some(indirect_buffer) = indirect_buffer else {
            return RenderCommandResult::Skip;
        };
        let Some(indirect_buffer) = ssbos.into_inner().get(&indirect_buffer.0) else {
            return RenderCommandResult::Skip;
        };
        let Some(vertex_buffer_slice) =
            mesh_allocator.mesh_vertex_slice(&mesh_instance.mesh_asset_id)
        else {
            return RenderCommandResult::Skip;
        };

        pass.set_vertex_buffer(0, vertex_buffer_slice.buffer.slice(..));

        match &gpu_mesh.buffer_info {
            RenderMeshBufferInfo::Indexed { index_format, .. } => {
                let Some(index_buffer_slice) =
                    mesh_allocator.mesh_index_slice(&mesh_instance.mesh_asset_id)
                else {
                    return RenderCommandResult::Skip;
                };

                pass.set_index_buffer(index_buffer_slice.buffer.slice(..), 0, *index_format);
                pass.draw_indexed_indirect(&indirect_buffer.buffer, 0);
            }
            RenderMeshBufferInfo::NonIndexed => {
                pass.draw_indirect(&indirect_buffer.buffer, 0);
            }
        }
        RenderCommandResult::Success
    }
}
