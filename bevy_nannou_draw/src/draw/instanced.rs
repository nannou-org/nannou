//! A shader that renders a mesh multiple times in one draw call.

use crate::draw::drawing::Drawing;
use crate::draw::primitive::Primitive;
use crate::draw::{Draw, DrawCommand};
use crate::render::{PreparedShaderModel, ShaderModel};
use bevy::core_pipeline::core_3d::Opaque3dBinKey;
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey, PreparedMaterial};
use bevy::render::extract_component::ExtractComponentPlugin;
use bevy::render::extract_instances::ExtractedInstances;
use bevy::render::mesh::allocator::MeshAllocator;
use bevy::render::mesh::RenderMeshBufferInfo;
use bevy::render::render_asset::prepare_assets;
use bevy::render::render_phase::{BinnedRenderPhaseType, ViewBinnedRenderPhases};
use bevy::render::storage::GpuShaderStorageBuffer;
use bevy::render::view;
use bevy::render::view::VisibilitySystems;
use bevy::{
    core_pipeline::core_3d::Opaque3d,
    ecs::system::{lifetimeless::*, SystemParamItem},
    pbr::{MeshPipeline, MeshPipelineKey, RenderMeshInstances, SetMeshViewBindGroup},
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        mesh::{MeshVertexBufferLayoutRef, RenderMesh},
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult,
            SetItemPipeline, TrackedRenderPass,
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

pub struct Instanced<'a, M>
where
    M: Material + Default,
{
    draw: &'a Draw<M>,
    primitive_index: Option<usize>,
    range: Option<Range<u32>>,
}

impl<'a, M> Drop for Instanced<'a, M>
where
    M: Material + Default,
{
    fn drop(&mut self) {
        if let Some((index, data)) = self.primitive_index.take().zip(self.range.take()) {
            self.insert_instanced_draw_command(index, data);
        }
    }
}

pub fn new<M>(draw: &Draw<M>) -> Instanced<M>
where
    M: Material + Default,
{
    Instanced {
        draw,
        primitive_index: None,
        range: None,
    }
}

impl<'a, M> Instanced<'a, M>
where
    M: Material + Default,
{
    pub fn primitive<T>(mut self, drawing: Drawing<T, M>) -> Instanced<'a, M>
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

    pub fn range(mut self, range: Range<u32>) -> Instanced<'a, M> {
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

pub struct InstancedMaterialPlugin<M>(PhantomData<M>);

impl<M> Default for InstancedMaterialPlugin<M>
where
    M: Default,
{
    fn default() -> Self {
        InstancedMaterialPlugin(PhantomData)
    }
}

impl<SM> Plugin for InstancedMaterialPlugin<SM>
where
    SM: ShaderModel,
    SM::Data: PartialEq + Eq + Hash + Clone,
{
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<InstancedMesh>::default(),
            ExtractComponentPlugin::<InstanceRange>::default(),
        ))
        .add_systems(
            PostUpdate,
            view::check_visibility::<With<InstancedMesh>>
                .in_set(VisibilitySystems::CheckVisibility),
        );

        app.sub_app_mut(RenderApp)
            .add_render_command::<Opaque3d, DrawInstancedMaterial<SM>>()
            .init_resource::<SpecializedMeshPipelines<InstancedPipeline<SM>>>()
            .add_systems(
                Render,
                (queue_instanced::<SM>
                    .after(prepare_assets::<PreparedMaterial<SM>>)
                    .in_set(RenderSet::QueueMeshes),),
            );
    }

    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp)
            .init_resource::<InstancedPipeline<SM>>();
    }
}

#[allow(clippy::too_many_arguments)]
fn queue_instanced<SM>(
    draw_functions: Res<DrawFunctions<Opaque3d>>,
    custom_pipeline: Res<InstancedPipeline<SM>>,
    mut pipelines: ResMut<SpecializedMeshPipelines<InstancedPipeline<SM>>>,
    pipeline_cache: Res<PipelineCache>,
    meshes: Res<RenderAssets<RenderMesh>>,
    (
        render_mesh_instances,
        instanced_meshes,
        mut phases,
        mut views,
        shader_models,
        extracted_instances,
    ): (
        Res<RenderMeshInstances>,
        Query<Entity, With<InstancedMesh>>,
        ResMut<ViewBinnedRenderPhases<Opaque3d>>,
        Query<(Entity, &ExtractedView, &Msaa)>,
        Res<RenderAssets<PreparedShaderModel<SM>>>,
        Res<ExtractedInstances<AssetId<SM>>>,
    ),
) where
    SM: ShaderModel,
    SM::Data: PartialEq + Eq + Hash + Clone,
{
    let drawn_function = draw_functions.read().id::<DrawInstancedMaterial<SM>>();

    for (view_entity, view, msaa) in &mut views {
        let msaa_key = MeshPipelineKey::from_msaa_samples(msaa.samples());
        let Some(phase) = phases.get_mut(&view_entity) else {
            continue;
        };

        let view_key = msaa_key | MeshPipelineKey::from_hdr(view.hdr);
        for (entity) in &instanced_meshes {
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
struct InstancedPipeline<M> {
    mesh_pipeline: MeshPipeline,
    shader_model_layout: BindGroupLayout,
    vertex_shader: Option<Handle<Shader>>,
    fragment_shader: Option<Handle<Shader>>,
    marker: PhantomData<M>,
}

impl<SM: ShaderModel> FromWorld for InstancedPipeline<SM> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let render_device = world.resource::<RenderDevice>();

        InstancedPipeline {
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

impl<SM: ShaderModel> SpecializedMeshPipeline for InstancedPipeline<SM>
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

type DrawInstancedMaterial<SM> = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetShaderModelBindGroup<SM, 2>,
    DrawMeshInstanced,
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

        let Some(mesh_instance) = render_mesh_instances.render_mesh_queue_data(item.entity())
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
