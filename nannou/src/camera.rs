use crate::prelude::bevy_render::camera::RenderTarget;
use crate::prelude::Camera3dBundle;
use crate::App;
use bevy::core_pipeline::bloom::BloomSettings;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::math::UVec2;
use bevy::prelude::{Camera3d, PerspectiveProjection, Projection, Transform, Vec2};
use bevy::render::camera;
use bevy::render::view::RenderLayers;
use bevy::window::WindowRef;
use bevy_nannou::prelude::render::NannouCamera;
use bevy_nannou::prelude::{default, ClearColorConfig, Entity, OrthographicProjection, Vec3};

pub struct Camera<'a, 'w> {
    entity: Entity,
    app: &'a App<'w>,
}

pub struct Builder<'a, 'w> {
    app: &'a App<'w>,
    camera: Camera3dBundle,
    bloom_settings: Option<BloomSettings>,
    layer: Option<RenderLayers>,
}

pub trait SetCamera: Sized {
    fn layer(mut self, layer: RenderLayers) -> Self {
        self.map_layer(|_| layer)
    }

    fn order(mut self, order: isize) -> Self {
        self.map_camera(|mut camera| {
            camera.camera.order = order;
            camera
        })
    }

    fn x_y(mut self, x: f32, y: f32) -> Self {
        self.map_camera(|mut camera| {
            camera.transform.translation =
                bevy::math::Vec3::new(x, y, camera.transform.translation.z);
            camera
        })
    }

    fn xy(mut self, p: Vec2) -> Self {
        self.map_camera(|mut camera| {
            camera.transform.translation = p.extend(camera.transform.translation.z);
            camera
        })
    }

    fn x_y_z(mut self, x: f32, y: f32, z: f32) -> Self {
        self.map_camera(|mut camera| {
            camera.transform.translation = bevy::math::Vec3::new(x, y, z);
            camera
        })
    }

    fn xyz(mut self, p: Vec3) -> Self {
        self.map_camera(|mut camera| {
            camera.transform.translation = p;
            camera
        })
    }

    fn hdr(mut self, hdr: bool) -> Self {
        self.map_camera(|mut camera| {
            camera.camera.hdr = hdr;
            camera
        })
    }

    fn viewport(mut self, position: UVec2, size: UVec2) -> Self {
        self.map_camera(|mut camera| {
            camera.camera.viewport = Some(camera::Viewport {
                physical_position: position,
                physical_size: size,
                depth: Default::default(),
            });
            camera
        })
    }

    fn window(mut self, window: Entity) -> Self {
        self.map_camera(|mut camera| {
            camera.camera.target = RenderTarget::Window(WindowRef::Entity(window));
            camera
        })
    }

    fn tonemapping(mut self, tonemapping: Tonemapping) -> Self {
        self.map_camera(|mut camera| {
            camera.tonemapping = tonemapping;
            camera
        })
    }

    fn clear_color(mut self, color: ClearColorConfig) -> Self {
        self.map_camera(|mut camera| {
            camera.camera.clear_color = color;
            camera
        })
    }

    fn bloom_settings(mut self, settings: BloomSettings) -> Self {
        self.map_bloom_settings(|_| settings)
    }

    fn bloom_intensity(mut self, intensity: f32) -> Self {
        self.map_bloom_settings(|mut settings| {
            settings.intensity = intensity;
            settings
        })
    }

    fn bloom_low_frequency_boost(mut self, low_frequency_boost: f32) -> Self {
        self.map_bloom_settings(|mut settings| {
            settings.low_frequency_boost = low_frequency_boost;
            settings
        })
    }

    fn bloom_low_frequency_boost_curvature(mut self, low_frequency_boost_curvature: f32) -> Self {
        self.map_bloom_settings(|mut settings| {
            settings.low_frequency_boost_curvature = low_frequency_boost_curvature;
            settings
        })
    }

    fn bloom_high_pass_frequency(mut self, high_pass_frequency: f32) -> Self {
        self.map_bloom_settings(|mut settings| {
            settings.high_pass_frequency = high_pass_frequency;
            settings
        })
    }

    fn bloom_prefilter_settings(mut self, threshold: f32, threshold_softness: f32) -> Self {
        self.map_bloom_settings(|mut settings| {
            settings.prefilter_settings = bevy::core_pipeline::bloom::BloomPrefilterSettings {
                threshold,
                threshold_softness,
            };
            settings
        })
    }

    fn bloom_composite_mode(
        mut self,
        composite_mode: bevy::core_pipeline::bloom::BloomCompositeMode,
    ) -> Self {
        self.map_bloom_settings(|mut settings| {
            settings.composite_mode = composite_mode;
            settings
        })
    }

    fn projection(self, projection: impl Into<Projection>) -> Self {
        self.map_camera(|mut camera| {
            camera.projection = projection.into();
            camera
        })
    }

    fn map_layer<F>(self, f: F) -> Self
    where
        F: FnOnce(RenderLayers) -> RenderLayers;

    fn map_bloom_settings<F>(self, f: F) -> Self
    where
        F: FnOnce(BloomSettings) -> BloomSettings;

    fn map_camera<F>(self, f: F) -> Self
    where
        F: FnOnce(Camera3dBundle) -> Camera3dBundle;
}

impl<'a, 'w> Builder<'a, 'w> {
    pub fn new(app: &'a App<'w>) -> Self {
        Self {
            app,
            camera: Camera3dBundle {
                transform: Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
                projection: OrthographicProjection::default().into(),
                ..default()
            },
            bloom_settings: None,
            layer: None,
        }
    }

    pub fn build(self) -> Entity {
        let entity = self.app.world_mut().spawn((self.camera, NannouCamera)).id();
        if let Some(layer) = self.layer {
            self.app.world_mut().entity_mut(entity).insert(layer);
        } else {
            self.app
                .world_mut()
                .entity_mut(entity)
                .insert(RenderLayers::default());
        }
        if let Some(bloom_settings) = self.bloom_settings {
            self.app
                .world_mut()
                .entity_mut(entity)
                .insert(bloom_settings);
        }
        entity
    }
}

impl<'a, 'w> SetCamera for Builder<'a, 'w> {
    fn map_layer<F>(self, f: F) -> Self
    where
        F: FnOnce(RenderLayers) -> RenderLayers,
    {
        Self {
            layer: Some(f(self.layer.unwrap_or(RenderLayers::default()))),
            ..self
        }
    }

    fn map_bloom_settings<F>(self, f: F) -> Self
    where
        F: FnOnce(BloomSettings) -> BloomSettings,
    {
        Self {
            bloom_settings: Some(f(self.bloom_settings.unwrap_or(BloomSettings::default()))),
            ..self
        }
    }

    fn map_camera<F>(self, f: F) -> Self
    where
        F: FnOnce(Camera3dBundle) -> Camera3dBundle,
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
    fn map_layer<F>(self, f: F) -> Self
    where
        F: FnOnce(RenderLayers) -> RenderLayers,
    {
        let mut world = self.app.world_mut();
        let mut layer_q = world.query::<Option<&mut RenderLayers>>();
        if let Ok(mut layer) = layer_q.get_mut(&mut world, self.entity) {
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

    fn map_bloom_settings<F>(self, f: F) -> Self
    where
        F: FnOnce(BloomSettings) -> BloomSettings,
    {
        let mut world = self.app.world_mut();
        let mut bloom_q = world.query::<Option<&mut BloomSettings>>();
        if let Ok(mut bloom) = bloom_q.get_mut(&mut world, self.entity) {
            if let Some(mut bloom) = bloom {
                *bloom = f(bloom.clone());
            } else {
                world
                    .entity_mut(self.entity)
                    .insert(f(BloomSettings::default()));
            }
        }
        self
    }

    fn map_camera<F>(self, f: F) -> Self
    where
        F: FnOnce(Camera3dBundle) -> Camera3dBundle,
    {
        let mut world = self.app.world_mut();
        let mut camera_q = world.query::<(
            &mut Transform,
            &mut camera::Camera,
            &mut Camera3d,
            &mut Projection,
        )>();
        let (transform, camera, camera_3d, projection) =
            camera_q.get_mut(&mut world, self.entity).unwrap();
        let bundle = Camera3dBundle {
            transform: transform.clone(),
            camera: camera.clone(),
            camera_3d: camera_3d.clone(),
            projection: projection.clone(),
            ..default()
        };

        let bundle = f(bundle);
        world.entity_mut(self.entity).insert(bundle);
        self
    }
}
