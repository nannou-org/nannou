use bevy::{
    camera::visibility::RenderLayers,
    light::DirectionalLight,
    prelude::{Color, Transform, Vec2},
};

use crate::prelude::Vec3;

/// The set of components that make up a nannou directional light.
///
/// Used by the light builder to accumulate configuration before spawning.
#[derive(Default)]
pub struct LightComponents {
    pub transform: Transform,
    pub directional_light: DirectionalLight,
    pub render_layers: RenderLayers,
}

/// Shared light configuration methods, implemented by the light builder.
///
/// Implementors only need to provide [`map_light`](SetLight::map_light); all other methods are
/// expressed in terms of it.
pub trait SetLight: Sized {
    fn x_y(self, x: f32, y: f32) -> Self {
        self.map_light(|mut light| {
            light.transform.translation = Vec3::new(x, y, light.transform.translation.z);
            light
        })
    }

    fn xy(self, p: Vec2) -> Self {
        self.map_light(|mut light| {
            light.transform.translation = p.extend(light.transform.translation.z);
            light
        })
    }

    fn x_y_z(self, x: f32, y: f32, z: f32) -> Self {
        self.map_light(|mut light| {
            light.transform.translation = Vec3::new(x, y, z);
            light
        })
    }

    fn xyz(self, p: Vec3) -> Self {
        self.map_light(|mut light| {
            light.transform.translation = p;
            light
        })
    }

    fn look_at(self, target: Vec2) -> Self {
        self.map_light(|mut light| {
            light.transform = Transform::from_translation(light.transform.translation)
                .looking_at(target.extend(0.0), Vec3::Y);
            light
        })
    }

    fn color<C: Into<Color>>(self, color: C) -> Self {
        self.map_light(|mut light| {
            light.directional_light.color = color.into();
            light
        })
    }

    fn illuminance(self, illuminance: f32) -> Self {
        self.map_light(|mut light| {
            light.directional_light.illuminance = illuminance;
            light
        })
    }

    fn layer(self, layer: RenderLayers) -> Self {
        self.map_light(|mut light| {
            light.render_layers = layer;
            light
        })
    }

    fn map_light<F>(self, f: F) -> Self
    where
        F: FnOnce(LightComponents) -> LightComponents;
}
