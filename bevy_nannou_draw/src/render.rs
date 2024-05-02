use bevy::pbr::{
    ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline,
};
use std::ops::{Deref, DerefMut};

use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::mesh::MeshVertexBufferLayoutRef;
use bevy::render::render_resource as wgpu;
use bevy::render::render_resource::{AsBindGroup, BlendComponent, BlendState, PolygonMode, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError};
use bevy::window::WindowRef;
use lyon::lyon_tessellation::{FillTessellator, StrokeTessellator};

use crate::draw::mesh::MeshExt;
use crate::draw::primitive::Primitive;
use crate::draw::render::{GlyphCache, RenderContext, RenderPrimitive};
use crate::draw::Context;
use nannou_core::math::map_range;

pub struct NannouRenderPlugin;

impl Plugin for NannouRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_default_texture)
            .add_plugins((
                ExtractComponentPlugin::<NannouTextureHandle>::default(),
                MaterialPlugin::<DefaultNannouMaterial>::default(),
            ))
            .add_plugins(ExtractResourcePlugin::<DefaultTextureHandle>::default())
            .insert_resource(GlyphCache::new([1024; 2], 0.1, 0.1))
            .add_systems(Update, texture_event_handler)
            .add_systems(Last, (update_background_color,));
    }
}

// ----------------------------------------------------------------------------
// Components and Resources
// ----------------------------------------------------------------------------

pub type DefaultNannouMaterial = ExtendedMaterial<StandardMaterial, NannouMaterial<"", "">>;

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone, Default)]
#[bind_group_data(NannouMaterialKey)]
pub struct NannouMaterial<const VS: &'static str, const FS: &'static str> {
    pub polygon_mode: Option<PolygonMode>,
    pub blend: Option<BlendComponent>,
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct NannouMaterialKey {
    polygon_mode: Option<PolygonMode>,
    blend_component: Option<BlendComponent>,
}

impl<const VS: &'static str, const FS: &'static str> From<&NannouMaterial<VS, FS>>
    for NannouMaterialKey
{
    fn from(material: &NannouMaterial<VS, FS>) -> Self {
        Self {
            polygon_mode: material.polygon_mode,
            blend_component: material.blend,
        }
    }
}

impl<const VS: &'static str, const FS: &'static str> MaterialExtension for NannouMaterial<VS, FS> {
    fn vertex_shader() -> ShaderRef {
        if !VS.is_empty() {
            VS.into()
        } else {
            ShaderRef::Default
        }
    }

    fn fragment_shader() -> ShaderRef {
        if !FS.is_empty() {
            FS.into()
        } else {
            ShaderRef::Default
        }
    }

    fn specialize(
        _pipeline: &MaterialExtensionPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        key: MaterialExtensionKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(blend_component) = key.bind_group_data.blend_component {
            let fragment = descriptor.fragment.as_mut().unwrap();
            fragment.targets.iter_mut().for_each(|target| {
                if let Some(target) = target {
                    target.blend = Some(BlendState {
                        color: blend_component.clone(),
                        alpha: BlendComponent::OVER,
                    });
                }
            });
        }

        if let Some(polygon_mode) = key.bind_group_data.polygon_mode {
            descriptor.primitive.polygon_mode = polygon_mode;
        }

        Ok(())
    }
}

#[derive(Resource, Deref, DerefMut, ExtractResource, Clone)]
pub struct DefaultTextureHandle(Handle<Image>);

#[derive(Component, Deref, DerefMut, ExtractComponent, Clone)]
pub struct NannouTextureHandle(Handle<Image>);

fn texture_event_handler(
    mut commands: Commands,
    mut ev_asset: EventReader<AssetEvent<Image>>,
    assets: Res<Assets<Image>>,
) {
    for ev in ev_asset.read() {
        match ev {
            AssetEvent::Added { .. } | AssetEvent::Modified { .. } | AssetEvent::Removed { .. } => {
                // TODO: handle these events
            }
            AssetEvent::LoadedWithDependencies { id } => {
                let handle = Handle::Weak(*id);
                let image = assets.get(&handle).unwrap();
                // TODO hack to only handle 2D textures for now
                // We should maybe require users to spawn a NannouTextureHandle themselves
                if image.texture_descriptor.dimension == wgpu::TextureDimension::D2 {
                    commands.spawn(NannouTextureHandle(handle));
                }
            }
            _ => {}
        }
    }
}

fn setup_default_texture(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let texture = images.add(Image::default());
    commands.insert_resource(DefaultTextureHandle(texture));
}

fn update_background_color(
    mut cameras_q: Query<(&mut Camera)>,
    draw_q: Query<(Entity, &crate::draw::BackgroundColor)>,
) {
    for (entity, bg_color) in draw_q.iter() {
        for (mut camera) in cameras_q.iter_mut() {
            if let RenderTarget::Window(WindowRef::Entity(window_target)) = camera.target {
                if window_target == entity {
                    camera.clear_color = ClearColorConfig::Custom(bg_color.0);
                }
            }
        }
    }
}

#[derive(Component)]
pub struct NannouMesh;

#[derive(Component)]
pub struct NannouPersistantMesh;

// BLEND
pub mod blend {
    use bevy::render::render_resource as wgpu;

    pub const BLEND_NORMAL: wgpu::BlendComponent = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::SrcAlpha,
        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
        operation: wgpu::BlendOperation::Add,
    };

    pub const BLEND_ADD: wgpu::BlendComponent = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::Src,
        dst_factor: wgpu::BlendFactor::Dst,
        operation: wgpu::BlendOperation::Add,
    };

    pub const BLEND_SUBTRACT: wgpu::BlendComponent = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::Src,
        dst_factor: wgpu::BlendFactor::Dst,
        operation: wgpu::BlendOperation::Subtract,
    };

    pub const BLEND_REVERSE_SUBTRACT: wgpu::BlendComponent = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::Src,
        dst_factor: wgpu::BlendFactor::Dst,
        operation: wgpu::BlendOperation::ReverseSubtract,
    };

    pub const BLEND_DARKEST: wgpu::BlendComponent = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::One,
        operation: wgpu::BlendOperation::Min,
    };

    pub const BLEND_LIGHTEST: wgpu::BlendComponent = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::One,
        operation: wgpu::BlendOperation::Max,
    };
}
