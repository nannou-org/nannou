// FIXME: Remove deprecated `DirectionalLightBundle`.
#![allow(deprecated)]

use bevy::{
    pbr::DirectionalLight,
    prelude::{Color, Transform, Vec2},
    render::view::RenderLayers,
};

use bevy_nannou::prelude::{Entity, Vec3, default};

use crate::App;

pub struct Light<'a, 'w> {
    entity: Entity,
    app: &'a App<'w>,
}

#[derive(Default)]
pub struct LightComponents {
    pub transform: Transform,
    pub directional_light: DirectionalLight,
    pub render_layers: RenderLayers,
}

pub struct Builder<'a, 'w> {
    app: &'a App<'w>,
    light: LightComponents,
}

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

impl<'a, 'w> Builder<'a, 'w> {
    pub fn new(app: &'a App<'w>) -> Self {
        Self {
            app,
            light: LightComponents {
                transform: Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            },
        }
    }

    pub fn build(self) -> Entity {
        let entity = self
            .app
            .component_world_mut()
            .spawn((
                self.light.transform,
                self.light.directional_light,
                self.light.render_layers,
            ))
            .id();
        entity
    }
}

impl<'a, 'w> SetLight for Builder<'a, 'w> {
    fn map_light<F>(self, f: F) -> Self
    where
        F: FnOnce(LightComponents) -> LightComponents,
    {
        Self {
            light: f(self.light),
            ..self
        }
    }
}

impl<'a, 'w> Light<'a, 'w> {
    pub fn new(app: &'a App<'w>, entity: Entity) -> Self {
        Self { entity, app }
    }
}

impl<'a, 'w> SetLight for Light<'a, 'w> {
    fn map_light<F>(self, f: F) -> Self
    where
        F: FnOnce(LightComponents) -> LightComponents,
    {
        let mut world = self.app.component_world_mut();
        let mut camera_q = world.query::<(&Transform, &DirectionalLight, &RenderLayers)>();
        let (transform, light, render_layers) = camera_q.get_mut(&mut world, self.entity).unwrap();
        let bundle = LightComponents {
            transform: transform.clone(),
            directional_light: light.clone(),
            render_layers: render_layers.clone(),
        };

        let bundle = f(bundle);
        world.entity_mut(self.entity).insert({
            let LightComponents {
                transform,
                directional_light,
                render_layers,
            } = bundle;
            (transform, directional_light, render_layers)
        });
        self
    }
}
