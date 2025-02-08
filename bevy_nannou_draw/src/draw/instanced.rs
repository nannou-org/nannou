//! A shader that renders a mesh multiple times in one draw call.

use crate::render::ShaderModelHandle;
use crate::{
    draw::{drawing::Drawing, primitive::Primitive, Draw, DrawCommand},
    render::{queue_shader_model, PreparedShaderModel, ShaderModel},
};
use bevy::{
    core_pipeline::core_3d::Transparent3d,
    ecs::system::{lifetimeless::*, SystemParamItem},
    pbr::{RenderMeshInstances, SetMeshBindGroup, SetMeshViewBindGroup},
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        extract_instances::ExtractedInstances,
        mesh::{
            allocator::MeshAllocator, MeshVertexBufferLayoutRef, RenderMesh, RenderMeshBufferInfo,
        },
        render_asset::{prepare_assets, RenderAssets},
        render_phase::{
            AddRenderCommand, BinnedRenderPhaseType, DrawFunctions, PhaseItem, RenderCommand,
            RenderCommandResult, SetItemPipeline, TrackedRenderPass, ViewBinnedRenderPhases,
        },
        render_resource::*,
        renderer::RenderDevice,
        storage::GpuShaderStorageBuffer,
        view,
        view::{ExtractedView, VisibilitySystems},
        Render, RenderApp, RenderSet,
    },
};
use rayon::prelude::*;
use std::{hash::Hash, marker::PhantomData, ops::Range};

pub struct Instanced<'a, SM>
where
    SM: ShaderModel + Default,
{
    draw: &'a Draw<SM>,
    primitive_index: Option<usize>,
    range: Option<Range<u32>>,
}

impl<'a, SM> Drop for Instanced<'a, SM>
where
    SM: ShaderModel + Default,
{
    fn drop(&mut self) {
        if let Some((index, data)) = self.primitive_index.take().zip(self.range.take()) {
            self.insert_instanced_draw_command(index, data);
        }
    }
}

pub fn new<SM>(draw: &Draw<SM>) -> Instanced<SM>
where
    SM: ShaderModel + Default,
{
    Instanced {
        draw,
        primitive_index: None,
        range: None,
    }
}

impl<'a, SM> Instanced<'a, SM>
where
    SM: ShaderModel + Default,
{
    pub fn primitive<T>(mut self, drawing: Drawing<T, SM>) -> Instanced<'a, SM>
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

    pub fn range(mut self, range: Range<u32>) -> Instanced<'a, SM> {
        self.range = Some(range);
        self
    }

    fn insert_instanced_draw_command(&self, index: usize, range: Range<u32>) {
        let mut state = self.draw.state.write().unwrap();
        let primitive = state.drawing.remove(&index).unwrap();
        state
            .draw_commands
            .push(Some(DrawCommand::Instanced(primitive, range)));
    }
}

#[derive(Component, ExtractComponent, Clone)]
pub struct InstancedMesh;

#[derive(Component, ExtractComponent, Clone)]
pub struct InstanceRange(pub Range<u32>);

pub struct InstancedShaderModelPlugin<SM>(PhantomData<SM>);

impl<SM> Default for InstancedShaderModelPlugin<SM>
where
    SM: Default,
{
    fn default() -> Self {
        InstancedShaderModelPlugin(PhantomData)
    }
}

impl<SM> Plugin for InstancedShaderModelPlugin<SM>
where
    SM: ShaderModel,
    SM::Data: PartialEq + Eq + Hash + Clone,
{
    fn build(&self, app: &mut App) {
        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawInstancedShaderModel<SM>>()
            .add_systems(
                Render,
                queue_shader_model::<SM, With<InstancedMesh>, DrawInstancedShaderModel<SM>>
                    .after(prepare_assets::<PreparedShaderModel<SM>>)
                    .in_set(RenderSet::QueueMeshes),
            );
    }
}

type DrawInstancedShaderModel<SM> = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    SetShaderModelBindGroup<SM, 2>,
    DrawMeshInstanced,
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
        let Some(shader_model) = models.get(&handle.0) else {
            return RenderCommandResult::Skip;
        };
        pass.set_bind_group(I, &shader_model.bind_group, &[]);
        RenderCommandResult::Success
    }
}

struct DrawMeshInstanced;
impl<P: PhaseItem> RenderCommand<P> for DrawMeshInstanced {
    type Param = (
        SRes<RenderAssets<RenderMesh>>,
        SRes<RenderMeshInstances>,
        SRes<MeshAllocator>,
        SRes<RenderAssets<GpuShaderStorageBuffer>>,
    );
    type ViewQuery = ();
    type ItemQuery = Read<InstanceRange>;

    #[inline]
    fn render<'w>(
        item: &P,
        _view: (),
        instance_range: Option<&'w InstanceRange>,
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
        let Some(instance_range) = instance_range else {
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
                pass.draw_indexed(
                    index_buffer_slice.range.clone(),
                    0,
                    instance_range.0.clone(),
                );
            }
            RenderMeshBufferInfo::NonIndexed => {
                pass.draw(vertex_buffer_slice.range.clone(), instance_range.0.clone());
            }
        }
        RenderCommandResult::Success
    }
}
