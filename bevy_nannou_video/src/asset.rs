use bevy::asset::io::file::FileAssetReader;
use bevy::asset::io::{AssetReader, AssetReaderError, AssetSource, PathStream, Reader};
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureFormat};
use bevy::utils::HashMap;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};
use std::path::Path;
use thiserror::Error;
use video_rs::hwaccel::HardwareAccelerationDeviceType;
use video_rs::{Decoder, DecoderBuilder};

pub struct VideoAssetPlugin;

impl Plugin for VideoAssetPlugin {
    fn build(&self, app: &mut App) {
        info!("Adding video asset plugin");
        app.init_asset::<Video>()
            .init_asset_loader::<VideoLoader>()
            .add_systems(Update, load_next_frame);
    }
}

fn load_next_frame(
    mut videos: ResMut<Assets<Video>>,
    mut images: ResMut<Assets<Image>>,
    time: Res<Time>,
) {
    for (_, video) in videos.iter_mut() {
        let current_time = time.elapsed_seconds_f64();
        if current_time - video.last_update < video.frame_duration {
            continue; // Not time for next frame yet
        }
        video.last_update = current_time;
        if let Some(next_texture) = video.next_texture() {
            video.texture = images.add(next_texture);
        }
    }
}

#[derive(Asset, TypePath)]
pub struct Video {
    last_update: f64,
    frame_duration: f64,
    pub decoder: Decoder,
    pub texture: Handle<Image>,
}

impl Video {
    fn next_texture(&mut self) -> Option<Image> {
        let (width, height) = self.decoder.size();
        if let Some(Ok(frame)) = self.decoder.decode_raw_iter().next() {
            // TODO: Handle other formats
            let rgba = frame
                .data(0)
                .par_iter()
                .chunks(3)
                .flat_map(|chunk| {
                    let r = chunk[0];
                    let g = chunk[1];
                    let b = chunk[2];
                    [*r, *g, *b, 255]
                })
                .collect::<Vec<u8>>();

            let mut image = Image::default();
            image.texture_descriptor.format = TextureFormat::Rgba8UnormSrgb;
            image.resize(Extent3d {
                width,
                height,
                ..default()
            });
            image.data = rgba;
            image.asset_usage = RenderAssetUsages::RENDER_WORLD;
            return Some(image);
        }
        None
    }
}

impl Deref for Video {
    type Target = Decoder;

    fn deref(&self) -> &Self::Target {
        &self.decoder
    }
}

impl DerefMut for Video {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.decoder
    }
}

#[derive(Default)]
struct VideoLoader;

#[derive(Default, Serialize, Deserialize)]
pub struct VideoLoaderSettings {
    pub options: std::collections::HashMap<String, String>,
}

impl AssetLoader for VideoLoader {
    type Asset = Video;
    type Settings = VideoLoaderSettings;
    type Error = VideoAssetLoaderError;

    async fn load<'a>(
        &'a self,
        _reader: &'a mut Reader<'_>,
        settings: &'a Self::Settings,
        load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let path = load_context.asset_path().path();
        // TODO: support web loading
        let base_path = FileAssetReader::get_base_path().join("assets");
        let path = base_path.join(path);
        let options = settings.options.clone();
        let options = options.into();
        let decoder = DecoderBuilder::new(path)
            .with_options(&options)
            .build()
            .map_err(|e| VideoAssetLoaderError::Decoder(e))?;
        let (width, height) = decoder.size();
        let mut image = Image::default();
        image.texture_descriptor.format = TextureFormat::Rgba8UnormSrgb;
        image.resize(Extent3d {
            width,
            height,
            ..default()
        });
        image.asset_usage = RenderAssetUsages::RENDER_WORLD;
        let texture = load_context.add_labeled_asset(String::from("video_texture"), image);
        let fps = decoder.frame_rate() as f64;
        Ok(Video {
            last_update: 0.0,
            frame_duration: 1.0 / fps,
            decoder,
            texture,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["mp4"]
    }
}

#[derive(Debug, Error)]
pub enum VideoAssetLoaderError {
    #[error("Failed to construct video decoder {0}")]
    Decoder(#[from] video_rs::Error),
    #[error("Failed to load video file")]
    Io(#[from] std::io::Error),
}
