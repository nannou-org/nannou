use crate::asset::{Isf, IsfAssetPlugin};
use crate::inputs::{IsfInputValue, IsfInputs};
use crate::render::IsfRenderPlugin;
use bevy::asset::embedded_asset;
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponentPlugin;
use bevy::render::view;
use bevy::render::view::{NoFrustumCulling, VisibilitySystems};

mod asset;
mod inputs;
mod render;

pub mod prelude {
    pub use crate::asset::Isf;
    pub use crate::inputs::{IsfInputValue, IsfInputs};
    pub use crate::IsfBundle;
}

pub struct NannouIsfPlugin;

type WithIsfInputs = With<IsfInputs>;

impl Plugin for NannouIsfPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(First, asset_event_handler)
            .add_systems(
                PostUpdate,
                view::check_visibility::<WithIsfInputs>.in_set(VisibilitySystems::CheckVisibility),
            )
            .add_plugins((
                IsfRenderPlugin,
                IsfAssetPlugin,
                ExtractComponentPlugin::<Handle<Isf>>::default(),
                ExtractComponentPlugin::<IsfInputs>::default(),
            ))
            .register_type::<IsfInputs>()
            .register_type::<IsfInputValue>();
    }
}

fn asset_event_handler(
    mut commands: Commands,
    mut ev_asset: EventReader<AssetEvent<Isf>>,
    mut event_writer: EventWriter<AssetEvent<Shader>>,
    shaders: Res<Assets<Shader>>,
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
                let isf_inputs = IsfInputs::from_isf(&isf.isf);
                for camera in cameras.iter() {
                    commands
                        .entity(camera)
                        .insert((handle.clone(), isf_inputs.clone()));
                }
            }
            _ => {}
        }
    }
}

#[derive(Bundle, Default)]
pub struct IsfBundle {
    /// The ISF asset.
    pub isf: Handle<Isf>,
    /// The inputs of the entity.
    pub inputs: IsfInputs,
    /// The visibility of the entity.
    pub visibility: Visibility,
    /// The inherited visibility of the entity.
    pub inherited_visibility: InheritedVisibility,
    /// The view visibility of the entity.
    pub view_visibility: ViewVisibility,
    /// The transform of the entity.
    pub transform: Transform,
    /// The global transform of the entity.
    pub global_transform: GlobalTransform,
    /// No frustum culling.
    pub no_frustum_culling: NoFrustumCulling,
}
