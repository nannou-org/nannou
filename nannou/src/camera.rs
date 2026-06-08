use bevy::camera::RenderTarget;
use bevy::{
    camera::Hdr,
    camera::{self, visibility::RenderLayers},
    core_pipeline::tonemapping::Tonemapping,
    math::UVec2,
    post_process::bloom::{Bloom, BloomPrefilter},
    prelude::{Projection, Transform, Vec2},
    window::WindowRef,
};

use crate::prelude::{ClearColorConfig, Entity, Vec3};

/// The set of components that make up a nannou camera.
///
/// Used by the camera builder to accumulate configuration before spawning.
#[derive(Default)]
pub struct CameraComponents {
    pub transform: Transform,
    pub camera: camera::Camera,
    pub hdr: Option<Hdr>,
    pub projection: Projection,
    pub tonemapping: Tonemapping,
    pub bloom_settings: Option<Bloom>,
    pub render_layers: RenderLayers,
    pub render_target: Option<RenderTarget>,
}

/// Shared camera configuration methods, implemented by the camera builder.
///
/// Implementors only need to provide [`map_camera`](SetCamera::map_camera); all other methods are
/// expressed in terms of it.
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
            camera.hdr = if hdr { Some(Hdr) } else { None };
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
            camera.render_target = Some(RenderTarget::Window(WindowRef::Entity(window)));
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
        composite_mode: bevy::post_process::bloom::BloomCompositeMode,
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
