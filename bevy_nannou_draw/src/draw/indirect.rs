//! A shader that renders a mesh multiple times in one draw call.

use crate::draw::drawing::Drawing;
use crate::draw::primitive::Primitive;
use crate::draw::{Draw, DrawCommand};
use crate::render::{PreparedShaderModel, ShaderModel};
use bevy::core_pipeline::core_3d::Opaque3dBinKey;
use bevy::pbr::{
    MaterialPipeline, MaterialPipelineKey, PreparedMaterial, RenderMaterialInstances,
    SetMaterialBindGroup,
};
use bevy::render::extract_component::ExtractComponentPlugin;
use bevy::render::extract_instances::ExtractedInstances;
use bevy::render::mesh::allocator::MeshAllocator;
use bevy::render::mesh::RenderMeshBufferInfo;
use bevy::render::render_asset::{prepare_assets, RenderAsset};
use bevy::render::render_phase::{BinnedRenderPhaseType, ViewBinnedRenderPhases};
use bevy::render::storage::{GpuShaderStorageBuffer, ShaderStorageBuffer};
use bevy::render::view;
use bevy::render::view::VisibilitySystems;
use bevy::{
    core_pipeline::core_3d::Opaque3d,
    ecs::system::{lifetimeless::*, SystemParamItem},
    pbr::{
        MeshPipeline, MeshPipelineKey, RenderMeshInstances, SetMeshBindGroup, SetMeshViewBindGroup,
    },
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        mesh::{MeshVertexBufferLayoutRef, RenderMesh},
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, PhaseItemExtraIndex, RenderCommand,
            RenderCommandResult, SetItemPipeline, TrackedRenderPass,
        },
        render_resource::*,
        renderer::RenderDevice,
        view::ExtractedView,
        Render, RenderApp, RenderSet,
    },
};
use rayon::prelude::*;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Range;

pub struct Indirect<'a, M>
where
    M: Material + Default,
{
    draw: &'a Draw<M>,
    primitive_index: Option<usize>,
    indirect_buffer: Option<Handle<ShaderStorageBuffer>>,
}

impl<'a, M> Drop for Indirect<'a, M>
where
    M: Material + Default,
{
    fn drop(&mut self) {
        if let Some((index, ssbo)) = self.primitive_index.take().zip(self.indirect_buffer.take()) {
            self.insert_indirect_draw_command(index, ssbo);
        }
    }
}

pub fn new<M>(draw: &Draw<M>) -> Indirect<M>
where
    M: Material + Default,
{
    Indirect {
        draw,
        primitive_index: None,
        indirect_buffer: None,
    }
}

impl<'a, M> Indirect<'a, M>
where
    M: Material + Default,
{
    pub fn primitive<T>(mut self, drawing: Drawing<T, M>) -> Indirect<'a, M>
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

    pub fn buffer(mut self, ssbo: Handle<ShaderStorageBuffer>) -> Indirect<'a, M> {
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

pub struct IndirectMaterialPlugin<M>(PhantomData<M>);

impl<M> Default for IndirectMaterialPlugin<M>
where
    M: Default,
{
    fn default() -> Self {
        IndirectMaterialPlugin(PhantomData)
    }
}

impl<SM> Plugin for IndirectMaterialPlugin<SM>
where
    SM: ShaderModel,
    SM::Data: PartialEq + Eq + Hash + Clone,
{
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<IndirectMesh>::default(),
            ExtractComponentPlugin::<Handle<ShaderStorageBuffer>>::default(),
        ))
        .add_systems(
            PostUpdate,
            view::check_visibility::<With<IndirectMesh>>.in_set(VisibilitySystems::CheckVisibility),
        );

        app.sub_app_mut(RenderApp)
            .add_render_command::<Opaque3d, DrawIndirectMaterial<SM>>()
            .init_resource::<SpecializedMeshPipelines<IndirectPipeline<SM>>>()
            .add_systems(
                Render,
                (queue_indirect::<SM>
                    .after(prepare_assets::<PreparedMaterial<SM>>)
                    .in_set(RenderSet::QueueMeshes),),
            );
    }

    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp)
            .init_resource::<IndirectPipeline<SM>>();
    }
}

#[allow(clippy::too_many_arguments)]
fn queue_indirect<SM>(
    draw_functions: Res<DrawFunctions<Opaque3d>>,
    custom_pipeline: Res<IndirectPipeline<SM>>,
    mut pipelines: ResMut<SpecializedMeshPipelines<IndirectPipeline<SM>>>,
    pipeline_cache: Res<PipelineCache>,
    meshes: Res<RenderAssets<RenderMesh>>,
    (
        render_mesh_instances,
        indirect_meshes,
        mut phases,
        mut views,
        shader_models,
        extracted_instances,
    ): (
        Res<RenderMeshInstances>,
        Query<Entity, With<IndirectMesh>>,
        ResMut<ViewBinnedRenderPhases<Opaque3d>>,
        Query<(Entity, &ExtractedView, &Msaa)>,
        Res<RenderAssets<PreparedShaderModel<SM>>>,
        Res<ExtractedInstances<AssetId<SM>>>,
    ),
) where
    SM: ShaderModel,
    SM::Data: PartialEq + Eq + Hash + Clone,
{
    let drawn_function = draw_functions.read().id::<DrawIndirectMaterial<SM>>();

    for (view_entity, view, msaa) in &mut views {
        let msaa_key = MeshPipelineKey::from_msaa_samples(msaa.samples());
        let Some(phase) = phases.get_mut(&view_entity) else {
            continue;
        };

        let view_key = msaa_key | MeshPipelineKey::from_hdr(view.hdr);
        for (entity) in &indirect_meshes {
            let Some(shader_model) = extracted_instances.get(&entity) else {
                continue;
            };
            let shader_model = shader_models.get(*shader_model).unwrap();
            let Some(mesh_instance) = render_mesh_instances.render_mesh_queue_data(entity) else {
                continue;
            };
            let Some(mesh) = meshes.get(mesh_instance.mesh_asset_id) else {
                continue;
            };
            let mesh_key =
                view_key | MeshPipelineKey::from_primitive_topology(mesh.primitive_topology());
            let key = MaterialPipelineKey {
                mesh_key,
                bind_group_data: shader_model.key.clone(),
            };
            let pipeline = pipelines
                .specialize(&pipeline_cache, &custom_pipeline, key, &mesh.layout)
                .unwrap();
            phase.add(
                Opaque3dBinKey {
                    draw_function: drawn_function,
                    pipeline,
                    asset_id: AssetId::<Mesh>::invalid().untyped(),
                    material_bind_group_id: None,
                    lightmap_image: None,
                },
                entity,
                BinnedRenderPhaseType::NonMesh,
            );
        }
    }
}

#[derive(Resource)]
struct IndirectPipeline<M> {
    mesh_pipeline: MeshPipeline,
    shader_model_layout: BindGroupLayout,
    vertex_shader: Option<Handle<Shader>>,
    fragment_shader: Option<Handle<Shader>>,
    marker: PhantomData<M>,
}

impl<SM: ShaderModel> FromWorld for IndirectPipeline<SM> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let render_device = world.resource::<RenderDevice>();

        IndirectPipeline {
            mesh_pipeline: world.resource::<MeshPipeline>().clone(),
            shader_model_layout: SM::bind_group_layout(render_device),
            vertex_shader: match <SM as ShaderModel>::vertex_shader() {
                ShaderRef::Default => None,
                ShaderRef::Handle(handle) => Some(handle),
                ShaderRef::Path(path) => Some(asset_server.load(path)),
            },
            fragment_shader: match <SM as ShaderModel>::fragment_shader() {
                ShaderRef::Default => None,
                ShaderRef::Handle(handle) => Some(handle),
                ShaderRef::Path(path) => Some(asset_server.load(path)),
            },
            marker: PhantomData,
        }
    }
}

impl<SM: ShaderModel> SpecializedMeshPipeline for IndirectPipeline<SM>
where
    SM::Data: PartialEq + Eq + Hash + Clone,
{
    type Key = MaterialPipelineKey<SM>;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayoutRef,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key.mesh_key, layout)?;
        if let Some(vertex_shader) = &self.vertex_shader {
            descriptor.vertex.shader = vertex_shader.clone();
        }

        if let Some(fragment_shader) = &self.fragment_shader {
            descriptor.fragment.as_mut().unwrap().shader = fragment_shader.clone();
        }

        descriptor
            .layout
            .insert(2, self.shader_model_layout.clone());

        let pipeline = MaterialPipeline {
            mesh_pipeline: self.mesh_pipeline.clone(),
            material_layout: self.shader_model_layout.clone(),
            vertex_shader: self.vertex_shader.clone(),
            fragment_shader: self.fragment_shader.clone(),
            marker: Default::default(),
        };
        SM::specialize(&pipeline, &mut descriptor, layout, key)?;
        Ok(descriptor)
    }
}

type DrawIndirectMaterial<SM> = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetShaderModelBindGroup<SM, 2>,
    DrawMeshIndirect,
);

struct SetShaderModelBindGroup<M: ShaderModel, const I: usize>(PhantomData<M>);
impl<P: PhaseItem, SM: ShaderModel, const I: usize> RenderCommand<P>
    for SetShaderModelBindGroup<SM, I>
{
    type Param = (
        SRes<RenderAssets<PreparedShaderModel<SM>>>,
        SRes<ExtractedInstances<AssetId<SM>>>,
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

        let Some(asset_id) = instances.get(&item.entity()) else {
            return RenderCommandResult::Skip;
        };
        let Some(material) = models.get(*asset_id) else {
            return RenderCommandResult::Skip;
        };
        pass.set_bind_group(I, &material.bind_group, &[]);
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
    type ItemQuery = Read<Handle<ShaderStorageBuffer>>;

    #[inline]
    fn render<'w>(
        item: &P,
        _view: (),
        indirect_buffer: Option<&'w Handle<ShaderStorageBuffer>>,
        (meshes, render_mesh_instances, mesh_allocator, ssbos): SystemParamItem<
            'w,
            '_,
            Self::Param,
        >,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let mesh_allocator = mesh_allocator.into_inner();

        let Some(mesh_instance) = render_mesh_instances.render_mesh_queue_data(item.entity())
        else {
            return RenderCommandResult::Skip;
        };
        let Some(gpu_mesh) = meshes.into_inner().get(mesh_instance.mesh_asset_id) else {
            return RenderCommandResult::Skip;
        };
        let Some(indirect_buffer) = indirect_buffer else {
            return RenderCommandResult::Skip;
        };
        let Some(indirect_buffer) = ssbos.into_inner().get(indirect_buffer) else {
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
