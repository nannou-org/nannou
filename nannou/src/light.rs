use bevy::pbr::{DirectionalLight, DirectionalLightBundle};
use bevy::prelude::{Color, Transform, Vec2};
use bevy::render::view::RenderLayers;

use bevy_nannou::prelude::{default, Entity, Vec3};

use crate::App;

pub struct Light<'a, 'w> {
    entity: Entity,
    app: &'a App<'w>,
}

pub struct Builder<'a, 'w> {
    app: &'a App<'w>,
    light: DirectionalLightBundle,
    layer: Option<RenderLayers>,
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
        self.map_layer(|_| layer)
    }

    fn map_layer<F>(self, f: F) -> Self
    where
        F: FnOnce(RenderLayers) -> RenderLayers;

    fn map_light<F>(self, f: F) -> Self
    where
        F: FnOnce(DirectionalLightBundle) -> DirectionalLightBundle;
}

impl<'a, 'w> Builder<'a, 'w> {
    pub fn new(app: &'a App<'w>) -> Self {
        Self {
            app,
            light: DirectionalLightBundle {
                transform: Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            },
            layer: None,
        }
    }

    pub fn build(self) -> Entity {
        let entity = self.app.component_world_mut().spawn(self.light).id();
        if let Some(layer) = self.layer {
            self.app
                .component_world_mut()
                .entity_mut(entity)
                .insert(layer);
        } else {
            self.app
                .component_world_mut()
                .entity_mut(entity)
                .insert(RenderLayers::default());
        }
        entity
    }
}

impl<'a, 'w> SetLight for Builder<'a, 'w> {
    fn map_layer<F>(self, f: F) -> Self
    where
        F: FnOnce(RenderLayers) -> RenderLayers,
    {
        Self {
            layer: Some(f(self.layer.unwrap_or(RenderLayers::default()))),
            ..self
        }
    }

    fn map_light<F>(self, f: F) -> Self
    where
        F: FnOnce(DirectionalLightBundle) -> DirectionalLightBundle,
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
    fn map_layer<F>(self, f: F) -> Self
    where
        F: FnOnce(RenderLayers) -> RenderLayers,
    {
        let mut world = self.app.component_world_mut();
        let mut layer_q = world.query::<Option<&mut RenderLayers>>();
        if let Ok(layer) = layer_q.get_mut(&mut world, self.entity) {
            if let Some(mut layer) = layer {
                *layer = f(layer.clone());
            } else {
                world
                    .entity_mut(self.entity)
                    .insert(f(RenderLayers::default()));
            }
        }
        self
    }

    fn map_light<F>(self, f: F) -> Self
    where
        F: FnOnce(DirectionalLightBundle) -> DirectionalLightBundle,
    {
        let mut world = self.app.component_world_mut();
        let mut camera_q = world.query::<(&mut Transform, &mut DirectionalLight)>();
        let (transform, light) = camera_q.get_mut(&mut world, self.entity).unwrap();
        let bundle = DirectionalLightBundle {
            transform: transform.clone(),
            directional_light: light.clone(),
            ..default()
        };

        let bundle = f(bundle);
        world.entity_mut(self.entity).insert(bundle);
        self
    }
}
