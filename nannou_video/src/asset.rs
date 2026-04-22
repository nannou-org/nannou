use bevy::asset::io::Reader;
use bevy::asset::io::file::FileAssetReader;
use bevy::asset::{AssetLoader, LoadContext, RenderAssetUsages};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};
use thiserror::Error;
use video_rs::{Decoder, DecoderBuilder};

pub struct VideoAssetPlugin;

impl Plugin for VideoAssetPlugin {
    fn build(&self, app: &mut App) {
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
        if video.finished {
            continue;
        }
        let current_time = time.elapsed_secs_f64();
        if current_time - video.last_update < video.frame_duration {
            continue;
        }
        let Some(mut image) = images.get_mut(&video.texture) else {
            continue;
        };
        match video.write_next_frame(&mut image) {
            FrameStatus::Decoded => video.last_update = current_time,
            FrameStatus::Eof => video.finished = true,
            FrameStatus::TransientError => {}
        }
    }
}

enum FrameStatus {
    Decoded,
    Eof,
    TransientError,
}

#[derive(Asset, TypePath)]
pub struct Video {
    last_update: f64,
    frame_duration: f64,
    finished: bool,
    pub decoder: Decoder,
    pub texture: Handle<Image>,
}

impl Video {
    fn write_next_frame(&mut self, image: &mut Image) -> FrameStatus {
        // Raw frames are swscaled to RGB24 by video-rs; rows may be stride-padded.
        let frame = match self.decoder.decode_raw_iter().next() {
            Some(Ok(frame)) => frame,
            Some(Err(video_rs::Error::DecodeExhausted)) => return FrameStatus::Eof,
            Some(Err(err)) => {
                warn!("video decode error: {}", err);
                return FrameStatus::TransientError;
            }
            None => return FrameStatus::TransientError,
        };

        let width = frame.width();
        let height = frame.height();
        let stride = frame.stride(0);
        let src = frame.data(0);
        let row_rgb = (width as usize) * 3;
        let required = (width as usize) * (height as usize) * 4;

        let buffer = image
            .data
            .get_or_insert_with(|| Vec::with_capacity(required));
        buffer.clear();
        buffer.reserve(required);
        for y in 0..height as usize {
            let row_start = y * stride;
            let row = &src[row_start..row_start + row_rgb];
            for rgb in row.chunks_exact(3) {
                buffer.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
            }
        }
        FrameStatus::Decoded
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

#[derive(Default, TypePath)]
struct VideoLoader;

#[derive(Default, Serialize, Deserialize)]
pub struct VideoLoaderSettings {
    pub options: std::collections::HashMap<String, String>,
}

impl AssetLoader for VideoLoader {
    type Asset = Video;
    type Settings = VideoLoaderSettings;
    type Error = VideoAssetLoaderError;

    async fn load(
        &self,
        _reader: &mut dyn Reader,
        settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let path = load_context.path().path();
        // TODO: support web loading
        let base_path = FileAssetReader::get_base_path().join("assets");
        let path = base_path.join(path);
        let options = settings.options.clone();
        let options = options.into();
        let decoder = DecoderBuilder::new(path)
            .with_options(&options)
            .build()
            .map_err(VideoAssetLoaderError::Decoder)?;
        let (width, height) = decoder.size_out();
        let image = Image::new_fill(
            Extent3d {
                width,
                height,
                ..default()
            },
            TextureDimension::D2,
            &[0, 0, 0, 255],
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        let texture = load_context.add_labeled_asset(String::from("video_texture"), image);
        let fps = decoder.frame_rate() as f64;
        let frame_duration = if fps > 0.0 { 1.0 / fps } else { 1.0 / 30.0 };
        Ok(Video {
            last_update: 0.0,
            frame_duration,
            finished: false,
            decoder,
            texture,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["mp4", "mov", "mkv", "webm", "avi", "m4v", "ts"]
    }
}

#[derive(Debug, Error)]
pub enum VideoAssetLoaderError {
    #[error("Failed to construct video decoder {0}")]
    Decoder(#[from] video_rs::Error),
    #[error("Failed to load video file")]
    Io(#[from] std::io::Error),
}
