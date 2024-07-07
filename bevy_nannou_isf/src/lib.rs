use std::any::TypeId;
use crate::asset::{Isf, IsfAssetPlugin};
use crate::inputs::{IsfInputValue, IsfInputs};
use crate::render::{IsfRenderPlugin, IsfRenderTargets};
use bevy::asset::embedded_asset;
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponentPlugin;
use bevy::render::extract_resource::ExtractResourcePlugin;
use bevy::render::view;
use bevy::render::view::{NoFrustumCulling, VisibilitySystems};
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContext};
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use bevy_inspector_egui::inspector_egui_impls::InspectorEguiImpl;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;

mod asset;
mod inputs;
mod render;

pub mod prelude {
    pub use crate::asset::Isf;
    pub use crate::inputs::{IsfInputValue, IsfInputs};
}

pub struct NannouIsfPlugin;
impl Plugin for NannouIsfPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(First, asset_event_handler)
            .add_plugins((
                IsfRenderPlugin,
                IsfAssetPlugin,
                ResourceInspectorPlugin::<IsfInputs>::default(),
                ExtractComponentPlugin::<Handle<Isf>>::default(),
                ExtractResourcePlugin::<IsfInputs>::default(),
                ExtractResourcePlugin::<IsfRenderTargets>::default(),
            ))
            .init_resource::<IsfRenderTargets>()
            .init_resource::<IsfInputs>()
            .register_type::<IsfInputs>()
            .register_type::<IsfInputValue>()
            .register_asset_reflect::<Image>();

        let type_registry = app.world().resource::<AppTypeRegistry>();
        let mut type_registry = type_registry.write();
        type_registry.register_type_data::<Handle<Image>, InspectorEguiImpl>();
    }
}

fn asset_event_handler(
    mut commands: Commands,
    mut ev_asset: EventReader<AssetEvent<Isf>>,
    mut isf_inputs: ResMut<IsfInputs>,
    mut cameras: Query<Entity, With<Camera>>,
    assets: Res<Assets<Isf>>,
) {
    for ev in ev_asset.read() {
        match ev {
            AssetEvent::Added { .. } | AssetEvent::Modified { .. } | AssetEvent::Removed { .. } => {
                // TODO: handle these events
            }
            AssetEvent::LoadedWithDependencies { id } => {
                let handle = Handle::Weak(*id);
                let isf = assets.get(&handle).unwrap();
                *isf_inputs = IsfInputs::from_isf(&isf.isf);
                for camera in cameras.iter() {
                    commands
                        .entity(camera)
                        .insert(handle.clone());
                }
            }
            _ => {}
        }
    }
}
