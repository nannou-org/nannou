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
