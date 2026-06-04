// FIXME: Remove deprecated `DirectionalLightBundle`.
#![allow(deprecated)]

use bevy::{
    camera::visibility::RenderLayers,
    light::DirectionalLight,
    prelude::{Color, Transform, Vec2},
};

use crate::prelude::{Entity, Vec3, default};

use crate::App;

/// A handle to an existing directional light [`Entity`], used to update its configuration.
///
/// Construct one via [`Light::new`] (or `app.light(entity)`) and use the [`SetLight`] methods to
/// mutate the light's transform and properties in place.
pub struct Light<'a, 'w> {
    entity: Entity,
    app: &'a App<'w>,
}

/// The set of components that make up a nannou directional light.
///
/// Used by the light [`Builder`] to accumulate configuration before spawning, and by [`Light`]
/// when reading back and updating an existing light entity.
#[derive(Default)]
pub struct LightComponents {
    pub transform: Transform,
    pub directional_light: DirectionalLight,
    pub render_layers: RenderLayers,
}

/// A context for building and spawning a new directional light.
///
/// Created via `app.new_light()`. Configure it with the [`SetLight`] methods, then call
/// [`Builder::build`] to spawn the light and obtain its [`Entity`].
pub struct Builder<'a, 'w> {
    app: &'a App<'w>,
    light: LightComponents,
}

/// Shared light configuration methods, implemented by both the light `Builder` and `Light`.
///
/// Implementors only need to provide [`map_light`](SetLight::map_light); all other methods are
/// expressed in terms of it, so the same fluent API configures a new light before it is spawned
/// or updates an existing one.
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
