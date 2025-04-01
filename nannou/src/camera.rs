// FIXME: Remove deprecatd `Camera3dBundle`.
#![allow(deprecated)]

use crate::{prelude::bevy_render::camera::RenderTarget, App};
use bevy::{
    core_pipeline::{
        bloom::{Bloom, BloomPrefilter},
        tonemapping::Tonemapping,
    },
    math::UVec2,
    prelude::{Projection, Transform, Vec2},
    render::{camera, view::RenderLayers},
    window::WindowRef,
};
use bevy_nannou::prelude::{
    default, render::NannouCamera, ClearColorConfig, Entity, OrthographicProjection, Vec3,
};

pub struct Camera<'a, 'w> {
    entity: Entity,
    app: &'a App<'w>,
}

#[derive(Default)]
pub struct CameraComponents {
    pub transform: Transform,
    pub camera: camera::Camera,
    pub projection: Projection,
    pub tonemapping: Tonemapping,
    pub bloom_settings: Option<Bloom>,
    pub render_layers: RenderLayers,
}

pub struct Builder<'a, 'w> {
    app: &'a App<'w>,
    camera: CameraComponents,
}

pub trait SetCamera: Sized {
    fn layer(self, layer: RenderLayers) -> Self {
        self.map_camera(|mut camera| {
            camera.render_layers = layer;
            camera
        })
    }

    fn order(self, order: isize) -> Self {
        self.map_camera(|mut camera| {
            camera.camera.order = order;
            camera
        })
    }

    fn x_y(self, x: f32, y: f32) -> Self {
        self.map_camera(|mut camera| {
            camera.transform.translation =
                bevy::math::Vec3::new(x, y, camera.transform.translation.z);
            camera
        })
    }

    fn xy(self, p: Vec2) -> Self {
        self.map_camera(|mut camera| {
            camera.transform.translation = p.extend(camera.transform.translation.z);
            camera
        })
    }

    fn x_y_z(self, x: f32, y: f32, z: f32) -> Self {
        self.map_camera(|mut camera| {
            camera.transform.translation = bevy::math::Vec3::new(x, y, z);
            camera
        })
    }

    fn xyz(self, p: Vec3) -> Self {
        self.map_camera(|mut camera| {
            camera.transform.translation = p;
            camera
        })
    }

    fn hdr(self, hdr: bool) -> Self {
        self.map_camera(|mut camera| {
            camera.camera.hdr = hdr;
            camera
        })
    }

    fn viewport(self, position: UVec2, size: UVec2) -> Self {
        self.map_camera(|mut camera| {
            camera.camera.viewport = Some(camera::Viewport {
                physical_position: position,
                physical_size: size,
                depth: Default::default(),
            });
            camera
        })
    }

    fn window(self, window: Entity) -> Self {
        self.map_camera(|mut camera| {
            camera.camera.target = RenderTarget::Window(WindowRef::Entity(window));
            camera
        })
    }

    fn tonemapping(self, tonemapping: Tonemapping) -> Self {
        self.map_camera(|mut camera| {
            camera.tonemapping = tonemapping;
            camera
        })
    }

    fn clear_color(self, color: ClearColorConfig) -> Self {
        self.map_camera(|mut camera| {
            camera.camera.clear_color = color;
            camera
        })
    }

    fn bloom_settings(self, settings: Bloom) -> Self {
        self.map_camera(|mut camera| {
            camera.bloom_settings = Some(settings);
            camera
        })
    }

    fn bloom_intensity(self, intensity: f32) -> Self {
        self.map_camera(|mut camera| {
            let settings = camera.bloom_settings.get_or_insert_with(Bloom::default);
            settings.intensity = intensity;
            camera
        })
    }

    fn bloom_low_frequency_boost(self, low_frequency_boost: f32) -> Self {
        self.map_camera(|mut camera| {
            let settings = camera.bloom_settings.get_or_insert_with(Bloom::default);
            settings.low_frequency_boost = low_frequency_boost;
            camera
        })
    }

    fn bloom_low_frequency_boost_curvature(self, low_frequency_boost_curvature: f32) -> Self {
        self.map_camera(|mut camera| {
            let settings = camera.bloom_settings.get_or_insert_with(Bloom::default);
            settings.low_frequency_boost_curvature = low_frequency_boost_curvature;
            camera
        })
    }

    fn bloom_high_pass_frequency(self, high_pass_frequency: f32) -> Self {
        self.map_camera(|mut camera| {
            let settings = camera.bloom_settings.get_or_insert_with(Bloom::default);
            settings.high_pass_frequency = high_pass_frequency;
            camera
        })
    }

    fn bloom_prefilter(self, threshold: f32, threshold_softness: f32) -> Self {
        self.map_camera(|mut camera| {
            let settings = camera.bloom_settings.get_or_insert_with(Bloom::default);
            settings.prefilter = BloomPrefilter {
                threshold,
                threshold_softness,
            };
            camera
        })
    }

    fn bloom_composite_mode(
        self,
        composite_mode: bevy::core_pipeline::bloom::BloomCompositeMode,
    ) -> Self {
        self.map_camera(|mut camera| {
            let settings = camera.bloom_settings.get_or_insert_with(Bloom::default);
            settings.composite_mode = composite_mode;
            camera
        })
    }

    fn projection(self, projection: impl Into<Projection>) -> Self {
        self.map_camera(|mut camera| {
            camera.projection = projection.into();
            camera
        })
    }

    fn map_camera<F>(self, f: F) -> Self
    where
        F: FnOnce(CameraComponents) -> CameraComponents;
}

impl<'a, 'w> Builder<'a, 'w> {
    pub fn new(app: &'a App<'w>) -> Self {
        Self {
            app,
            camera: CameraComponents {
                transform: Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
                projection: OrthographicProjection::default_3d().into(),
                ..default()
            },
        }
    }

    pub fn build(self) -> Entity {
        let entity = self
            .app
            .component_world_mut()
            .spawn((
                self.camera.transform,
                self.camera.camera,
                self.camera.projection,
                self.camera.tonemapping,
                self.camera.render_layers,
                NannouCamera,
            ))
            .id();
        if let Some(bloom_settings) = self.camera.bloom_settings {
            self.app
                .component_world_mut()
                .entity_mut(entity)
                .insert(bloom_settings);
        }
        entity
    }
}

impl<'a, 'w> SetCamera for Builder<'a, 'w> {
    fn map_camera<F>(self, f: F) -> Self
    where
        F: FnOnce(CameraComponents) -> CameraComponents,
    {
        Self {
            camera: f(self.camera),
            ..self
        }
    }
}

impl<'a, 'w> Camera<'a, 'w> {
    pub fn new(app: &'a App<'w>, entity: Entity) -> Self {
        Self { entity, app }
    }
}

impl<'a, 'w> SetCamera for Camera<'a, 'w> {
    fn map_camera<F>(self, f: F) -> Self
    where
        F: FnOnce(CameraComponents) -> CameraComponents,
    {
        let mut world = self.app.component_world_mut();
        let mut camera_q = world.query::<(
            &Transform,
            &camera::Camera,
            &Projection,
            &Tonemapping,
            &RenderLayers,
            Option<&Bloom>,
        )>();
        let (transform, camera, projection, tonemapping, render_layers, bloom_settings) =
            camera_q.get(&mut world, self.entity).unwrap();
        let camera = CameraComponents {
            transform: transform.clone(),
            camera: camera.clone(),
            projection: projection.clone(),
            tonemapping: tonemapping.clone(),
            render_layers: render_layers.clone(),
            bloom_settings: bloom_settings.cloned(),
        };
        let mut camera = f(camera);
        if let Some(bloom_settings) = camera.bloom_settings.take() {
            world.entity_mut(self.entity).insert(bloom_settings);
        }

        world.entity_mut(self.entity).insert({
            let CameraComponents {
                transform,
                camera,
                projection,
                tonemapping,
                render_layers,
                ..
            } = camera;
            (transform, camera, projection, tonemapping, render_layers)
        });
        self
    }
}
