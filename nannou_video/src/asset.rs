use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(not(target_arch = "wasm32"))]
use {
    bevy::asset::io::Reader,
    bevy::asset::io::file::FileAssetReader,
    bevy::asset::{AssetLoader, LoadContext},
    std::io::Write,
    std::path::PathBuf,
    std::sync::Arc,
    tempfile::NamedTempFile,
    thiserror::Error,
    video_rs::location::{Location, Url},
    video_rs::options::Options,
    video_rs::{Decoder, DecoderBuilder},
};

#[cfg(target_arch = "wasm32")]
use std::sync::Arc;

#[derive(Asset, TypePath, Debug, Clone)]
pub struct Video {
    pub source: VideoSource,
    pub size: UVec2,
    pub frame_rate: f32,
    pub duration_seconds: Option<f64>,
    pub frame_count: Option<u64>,
    /// FFmpeg demuxer options applied when the asset was probed. Re-applied by each
    /// [`VideoPlayer`](crate::VideoPlayer) worker when it opens its own decoder. Native only.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) options: HashMap<String, String>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub enum VideoSource {
    File(PathBuf),
    Url(Url),
    /// Bytes delivered through the asset `Reader` (http, embedded, custom source) and spilled
    /// to a temp file so ffmpeg can open them. The `Arc` keeps the file alive for every
    /// [`Video`] clone and worker decoder that references it.
    TempFile(Arc<NamedTempFile>),
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone)]
pub enum VideoSource {
    /// URL or path used as the `src` attribute on the underlying `<video>` element —
    /// `http://`, `https://`, `blob:`, or a page-relative path.
    Url(String),
    /// Raw bytes. A blob URL is created per player when the element is attached.
    Bytes(Arc<Vec<u8>>),
}

#[cfg(not(target_arch = "wasm32"))]
impl VideoSource {
    pub(crate) fn to_location(&self) -> Location {
        match self {
            VideoSource::File(p) => Location::File(p.clone()),
            VideoSource::Url(u) => Location::Network(u.clone()),
            VideoSource::TempFile(tmp) => Location::File(tmp.path().to_path_buf()),
        }
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct VideoLoaderSettings {
    /// Well-known preset for the ffmpeg demuxer. Native only; ignored on wasm.
    pub preset: NetworkPreset,
    /// Additional raw ffmpeg demuxer options, merged on top of [`Self::preset`]. Native only.
    pub extra_options: HashMap<String, String>,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkPreset {
    #[default]
    None,
    /// Prefer TCP over UDP when reading an RTSP stream.
    RtspOverTcp,
    /// Like [`Self::RtspOverTcp`] but with reduced socket/I/O timeouts (16 s).
    RtspOverTcpWithSaneTimeouts,
    /// Fragmented MP4 output compatible with Media Source Extensions.
    FragmentedMov,
}

#[cfg(not(target_arch = "wasm32"))]
impl NetworkPreset {
    fn options(self) -> Options {
        match self {
            NetworkPreset::None => Options::default(),
            NetworkPreset::RtspOverTcp => Options::preset_rtsp_transport_tcp(),
            NetworkPreset::RtspOverTcpWithSaneTimeouts => {
                Options::preset_rtsp_transport_tcp_and_sane_timeouts()
            }
            NetworkPreset::FragmentedMov => Options::preset_fragmented_mov(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn merge_options(
    preset: NetworkPreset,
    extra: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut combined: HashMap<String, String> = preset.options().into();
    for (k, v) in extra {
        combined.insert(k.clone(), v.clone());
    }
    combined
}

#[cfg(not(target_arch = "wasm32"))]
impl Video {
    /// Probe the given source to build a [`Video`] asset without going through the asset loader.
    ///
    /// Use this for live sources the asset loader can't handle — Bevy's `WebAssetReader`
    /// buffers the full response before calling the loader, which hangs on infinite streams
    /// (RTSP, HLS live, MSE). Pass those straight to ffmpeg via [`VideoSource::Url`] instead.
    pub fn probe(
        source: VideoSource,
        settings: &VideoLoaderSettings,
    ) -> Result<Self, VideoAssetLoaderError> {
        let options_map = merge_options(settings.preset, &settings.extra_options);
        let options: Options = options_map.clone().into();
        let decoder = DecoderBuilder::new(source.to_location())
            .with_options(&options)
            .build()
            .map_err(VideoAssetLoaderError::Decoder)?;
        Ok(Self::from_probe(source, options_map, &decoder))
    }

    fn from_probe(
        source: VideoSource,
        options: HashMap<String, String>,
        decoder: &Decoder,
    ) -> Self {
        let (width, height) = decoder.size_out();
        let frame_rate = decoder.frame_rate();
        let duration_seconds = decoder
            .duration()
            .ok()
            .filter(|t| t.has_value() && !t.has_no_pts())
            .map(|t| t.as_secs_f64())
            .filter(|&s| s > 0.0);
        let frame_count = decoder.frames().ok().filter(|&n| n > 0);
        Video {
            source,
            size: UVec2::new(width, height),
            frame_rate,
            duration_seconds,
            frame_count,
            options,
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl Video {
    /// Build a [`Video`] asset from a source without going through the asset loader. On wasm
    /// the browser performs the actual probing asynchronously; `size`/`frame_rate`/`duration`
    /// stay zero/`None` until `loadedmetadata` fires on the underlying `<video>` element.
    pub fn probe(
        source: VideoSource,
        _settings: &VideoLoaderSettings,
    ) -> Result<Self, VideoAssetLoaderError> {
        Ok(Video {
            source,
            size: UVec2::ZERO,
            frame_rate: 0.0,
            duration_seconds: None,
            frame_count: None,
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Default, TypePath)]
pub(crate) struct VideoLoader;

#[cfg(not(target_arch = "wasm32"))]
impl AssetLoader for VideoLoader {
    type Asset = Video;
    type Settings = VideoLoaderSettings;
    type Error = VideoAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        // ffmpeg needs a path or URL, not a byte stream. Take the fast path when the asset
        // lives on the local filesystem; otherwise spill the Reader (http, embedded, custom)
        // to a temp file the decoder can open.
        let rel_path = load_context.path().path();
        let file_path = FileAssetReader::get_base_path().join("assets").join(rel_path);
        let source = if file_path.is_file() {
            VideoSource::File(file_path)
        } else {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let ext = rel_path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| format!(".{e}"))
                .unwrap_or_default();
            let mut tmp = tempfile::Builder::new()
                .prefix("nannou_video_")
                .suffix(&ext)
                .tempfile()?;
            tmp.write_all(&bytes)?;
            tmp.flush()?;
            VideoSource::TempFile(Arc::new(tmp))
        };
        Video::probe(source, settings)
    }

    fn extensions(&self) -> &[&str] {
        &["mp4", "mov", "mkv", "webm", "avi", "m4v", "ts"]
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Error)]
pub enum VideoAssetLoaderError {
    #[error("Failed to construct video decoder {0}")]
    Decoder(#[from] video_rs::Error),
    #[error("Failed to load video file")]
    Io(#[from] std::io::Error),
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, thiserror::Error)]
pub enum VideoAssetLoaderError {
    #[error("Failed to read video asset bytes")]
    Io(#[from] std::io::Error),
    #[error("Browser error: {0}")]
    Browser(String),
}
