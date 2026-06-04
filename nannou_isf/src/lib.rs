use crate::asset::{Isf, IsfAssetPlugin, IsfHandle};
use crate::inputs::{IsfInputValue, IsfInputs};
use crate::render::{IsfRenderPlugin, IsfRenderTargets};
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponentPlugin;
use bevy::render::extract_resource::ExtractResourcePlugin;
// TODO: waiting on bevy 0.19 support for bevy-inspector-egui
//use bevy_inspector_egui::inspector_egui_impls::InspectorEguiImpl;
//use bevy_inspector_egui::quick::ResourceInspectorPlugin;

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
                // TODO: waiting on bevy 0.19 support for bevy-inspector-egui
                //ResourceInspectorPlugin::<IsfInputs>::default(),
                ExtractComponentPlugin::<IsfHandle>::default(),
                ExtractResourcePlugin::<IsfInputs>::default(),
                ExtractResourcePlugin::<IsfRenderTargets>::default(),
            ))
            .init_resource::<IsfRenderTargets>()
            .init_resource::<IsfInputs>()
            .register_type::<IsfInputs>()
            .register_type::<IsfInputValue>()
            .register_asset_reflect::<Image>();

        // TODO: waiting on bevy 0.19 support for bevy-inspector-egui
        //let type_registry = app.world().resource::<AppTypeRegistry>();
        //let mut type_registry = type_registry.write();
        //type_registry.register_type_data::<Handle<Image>, InspectorEguiImpl>();
    }
}

fn asset_event_handler(
    mut commands: Commands,
    mut ev_asset: MessageReader<AssetEvent<Isf>>,
    mut isf_inputs: ResMut<IsfInputs>,
    cameras: Query<Entity, With<Camera>>,
    mut assets: ResMut<Assets<Isf>>,
) {
    for ev in ev_asset.read() {
        match ev {
            AssetEvent::Added { .. } | AssetEvent::Modified { .. } | AssetEvent::Removed { .. } => {
                // TODO: handle these events
            }
            AssetEvent::LoadedWithDependencies { id } => {
                let isf = assets.get(*id).unwrap();
                *isf_inputs = IsfInputs::from_isf(&isf.isf);
                let handle = assets.get_strong_handle(*id).unwrap();
                for camera in cameras.iter() {
                    commands.entity(camera).insert(IsfHandle(handle.clone()));
                }
            }
            _ => {}
        }
    }
}
