//! A shader that renders a mesh multiple times in one draw call.

use crate::draw::drawing::Drawing;
use crate::draw::primitive::Primitive;
use crate::draw::{Draw, DrawCommand};
use bevy::core_pipeline::core_3d::Opaque3dBinKey;
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey, PreparedMaterial, SetMaterialBindGroup};
use bevy::render::mesh::allocator::MeshAllocator;
use bevy::render::mesh::RenderMeshBufferInfo;
use bevy::render::render_asset::prepare_assets;
use bevy::render::render_phase::{BinnedRenderPhaseType, ViewBinnedRenderPhases};
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

pub struct Instanced<'a, M>
where
    M: Material + Default,
{
    draw: &'a Draw<M>,
    data: Option<(usize, Range<u32>)>,
}

impl<'a, M> Drop for Instanced<'a, M>
where
    M: Material + Default,
{
    fn drop(&mut self) {
        if let Some((index, data)) = self.data.take() {
            self.insert_instanced_draw_command(index, data);
        }
    }
}

pub fn new<M>(draw: &Draw<M>) -> Instanced<M>
where
    M: Material + Default,
{
    Instanced { draw, data: None }
}

impl<'a, M> Instanced<'a, M>
where
    M: Material + Default,
{
    pub fn with<T>(mut self, drawing: Drawing<T, M>, instance_range: Range<u32>) -> Instanced<'a, M>
    where
        T: Into<Primitive>,
    {
        self.draw
            .state
            .write()
            .unwrap()
            .instanced
            .insert(drawing.index);
        self.data = Some((drawing.index, instance_range));
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

#[derive(Component)]
pub struct InstancedEntity;

pub struct InstancedMaterialPlugin<M>(PhantomData<M>);

impl<M> Default for InstancedMaterialPlugin<M>
where
    M: Default,
{
    fn default() -> Self {
        InstancedMaterialPlugin(PhantomData)
    }
}

impl<M> Plugin for InstancedMaterialPlugin<M>
where
    M: Material + Default,
    M::Data: PartialEq + Eq + Hash + Clone,
{
    fn build(&self, app: &mut App) {
        app.sub_app_mut(RenderApp)
            .add_render_command::<Opaque3d, DrawInstancedMaterial<M>>()
            .init_resource::<SpecializedMeshPipelines<InstancedDataPipeline<M>>>()
            .add_systems(
                Render,
                (queue_instanced::<M>
                    .after(prepare_assets::<PreparedMaterial<M>>)
                    .in_set(RenderSet::QueueMeshes)),
            );
    }

    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp)
            .init_resource::<InstancedDataPipeline<M>>();
    }
}

#[allow(clippy::too_many_arguments)]
fn queue_instanced<M>(
    draw_functions: Res<DrawFunctions<Opaque3d>>,
    custom_pipeline: Res<InstancedDataPipeline<M>>,
    mut pipelines: ResMut<SpecializedMeshPipelines<InstancedDataPipeline<M>>>,
    pipeline_cache: Res<PipelineCache>,
    meshes: Res<RenderAssets<RenderMesh>>,
    (render_mesh_instances, material_meshes, mut phases, mut views, materials): (
        Res<RenderMeshInstances>,
        Query<(Entity, &Handle<M>), With<InstancedEntity>>,
        ResMut<ViewBinnedRenderPhases<Opaque3d>>,
        Query<(Entity, &ExtractedView, &Msaa)>,
        Res<RenderAssets<PreparedMaterial<M>>>,
    ),
) where
    M: Material,
    M::Data: PartialEq + Eq + Hash + Clone,
{
    let drawn_function = draw_functions.read().id::<DrawInstancedMaterial<M>>();

    for (view_entity, view, msaa) in &mut views {
        let msaa_key = MeshPipelineKey::from_msaa_samples(msaa.samples());
        let Some(phase) = phases.get_mut(&view_entity) else {
            continue;
        };

        let view_key = msaa_key | MeshPipelineKey::from_hdr(view.hdr);
        for (entity, material) in &material_meshes {
            let material = materials.get(material).unwrap();
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
                bind_group_data: material.key.clone(),
            };
            let pipeline = pipelines
                .specialize(&pipeline_cache, &custom_pipeline, key, &mesh.layout)
                .unwrap();
            info!("Queueing instanced mesh {:?}", entity);
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

#[derive(Component)]
pub(crate) struct InstanceRange(pub Range<u32>);

#[derive(Resource)]
struct InstancedDataPipeline<M> {
    mesh_pipeline: MeshPipeline,
    material_layout: BindGroupLayout,
    vertex_shader: Option<Handle<Shader>>,
    fragment_shader: Option<Handle<Shader>>,
    marker: PhantomData<M>,
}

impl<M: Material> FromWorld for InstancedDataPipeline<M> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let render_device = world.resource::<RenderDevice>();

        InstancedDataPipeline {
            mesh_pipeline: world.resource::<MeshPipeline>().clone(),
            material_layout: M::bind_group_layout(render_device),
            vertex_shader: match M::vertex_shader() {
                ShaderRef::Default => None,
                ShaderRef::Handle(handle) => Some(handle),
                ShaderRef::Path(path) => Some(asset_server.load(path)),
            },
            fragment_shader: match M::fragment_shader() {
                ShaderRef::Default => None,
                ShaderRef::Handle(handle) => Some(handle),
                ShaderRef::Path(path) => Some(asset_server.load(path)),
            },
            marker: PhantomData,
        }
    }
}

impl<M: Material> SpecializedMeshPipeline for InstancedDataPipeline<M>
where
    M::Data: PartialEq + Eq + Hash + Clone,
{
    type Key = MaterialPipelineKey<M>;

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

        descriptor.layout.insert(2, self.material_layout.clone());

        let pipeline = MaterialPipeline {
            mesh_pipeline: self.mesh_pipeline.clone(),
            material_layout: self.material_layout.clone(),
            vertex_shader: self.vertex_shader.clone(),
            fragment_shader: self.fragment_shader.clone(),
            marker: Default::default(),
        };
        M::specialize(&pipeline, &mut descriptor, layout, key)?;
        Ok(descriptor)
    }
}

type DrawInstancedMaterial<M> = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    SetMaterialBindGroup<M, 2>,
    DrawMeshInstanced,
);

struct DrawMeshInstanced;

impl<P: PhaseItem> RenderCommand<P> for DrawMeshInstanced {
    type Param = (
        SRes<RenderAssets<RenderMesh>>,
        SRes<RenderMeshInstances>,
        SRes<MeshAllocator>,
    );
    type ViewQuery = ();
    type ItemQuery = Read<InstanceRange>;

    #[inline]
    fn render<'w>(
        item: &P,
        _view: (),
        instance_range: Option<&'w InstanceRange>,
        (meshes, render_mesh_instances, mesh_allocator): SystemParamItem<'w, '_, Self::Param>,
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
            RenderMeshBufferInfo::Indexed {
                index_format,
                count,
            } => {
                let Some(index_buffer_slice) =
                    mesh_allocator.mesh_index_slice(&mesh_instance.mesh_asset_id)
                else {
                    return RenderCommandResult::Skip;
                };

                pass.set_index_buffer(index_buffer_slice.buffer.slice(..), 0, *index_format);
                pass.draw_indexed(
                    index_buffer_slice.range.start..(index_buffer_slice.range.start + count),
                    vertex_buffer_slice.range.start as i32,
                    instance_range.0.clone(),
                );
            }
            RenderMeshBufferInfo::NonIndexed => {
                pass.draw(0..gpu_mesh.vertex_count, instance_range.0.clone());
            }
        }
        RenderCommandResult::Success
    }
}
