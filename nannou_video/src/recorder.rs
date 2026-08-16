//! Offline video recording. Frame timestamps come from the frame index and
//! configured fps, never wall-clock time.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Once;
use std::thread;

use flume::{Receiver, Sender};
use video_rs::encode::{Encoder, Settings};
use video_rs::frame::{PixelFormat, RawFrame};
use video_rs::time::Time;

/// Pixel layout of the frames passed to [`VideoRecorder::record_frame`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecorderPixelFormat {
    /// 4 bytes per pixel, `[r, g, b, a]`. Alpha is discarded.
    Rgba8,
    /// 4 bytes per pixel, `[b, g, r, a]`. Alpha is discarded.
    Bgra8,
    /// 3 bytes per pixel, `[r, g, b]`.
    Rgb8,
}

impl RecorderPixelFormat {
    pub fn bytes_per_pixel(self) -> usize {
        match self {
            RecorderPixelFormat::Rgba8 | RecorderPixelFormat::Bgra8 => 4,
            RecorderPixelFormat::Rgb8 => 3,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VideoRecorderConfig {
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub pixel_format: RecorderPixelFormat,
    /// Use the encoder's realtime preset (faster encode, larger output).
    pub realtime: bool,
    /// Keyframe interval in frames. `None` uses the encoder default.
    pub keyframe_interval: Option<u64>,
    /// Constant Rate Factor, 0 (lossless) to 51 (worst quality). `None` uses
    /// x264's default of 23; ~18 is visually lossless.
    pub crf: Option<u8>,
    /// x264 speed/compression preset (`ultrafast` .. `veryslow`). Slower
    /// presets compress better at the same quality. `None` uses `medium`.
    pub preset: Option<String>,
}

impl VideoRecorderConfig {
    pub fn new(width: u32, height: u32, fps: f64) -> Self {
        Self {
            width,
            height,
            fps,
            pixel_format: RecorderPixelFormat::Rgba8,
            realtime: false,
            keyframe_interval: None,
            crf: None,
            preset: None,
        }
    }

    pub fn with_pixel_format(mut self, format: RecorderPixelFormat) -> Self {
        self.pixel_format = format;
        self
    }

    pub fn with_realtime(mut self, realtime: bool) -> Self {
        self.realtime = realtime;
        self
    }

    pub fn with_keyframe_interval(mut self, interval: u64) -> Self {
        self.keyframe_interval = Some(interval);
        self
    }

    pub fn with_crf(mut self, crf: u8) -> Self {
        self.crf = Some(crf);
        self
    }

    pub fn with_preset(mut self, preset: impl Into<String>) -> Self {
        self.preset = Some(preset.into());
        self
    }
}

#[derive(Debug, thiserror::Error)]
pub enum VideoRecordError {
    #[error("invalid recorder config: {0}")]
    InvalidConfig(String),
    #[error("failed to create encoder: {0}")]
    Create(video_rs::Error),
    #[error("failed to encode frame {frame}: {source}")]
    Encode { frame: u64, source: video_rs::Error },
    #[error("failed to finalize video: {0}")]
    Finish(video_rs::Error),
    #[error("frame has {got} bytes but {expected} were expected ({width}x{height} {format:?})")]
    FrameSize {
        expected: usize,
        got: usize,
        width: u32,
        height: u32,
        format: RecorderPixelFormat,
    },
    #[error("no frames were recorded; the empty output file was removed")]
    NoFrames,
    #[error("encoder thread terminated unexpectedly")]
    WorkerDied,
}

/// Encodes pixel frames into a video file (H.264; container chosen by the path
/// extension). Dropping without [`finish`](Self::finish) finalizes best-effort
/// and swallows errors.
pub struct VideoRecorder {
    frame_tx: Option<Sender<Vec<u8>>>,
    handle: Option<thread::JoinHandle<Result<u64, VideoRecordError>>>,
    expected_len: usize,
    config: VideoRecorderConfig,
    frames_sent: u64,
}

impl VideoRecorder {
    pub fn new(
        path: impl AsRef<Path>,
        config: VideoRecorderConfig,
    ) -> Result<Self, VideoRecordError> {
        if config.width == 0 || config.height == 0 {
            return Err(VideoRecordError::InvalidConfig(
                "width and height must be non-zero".to_string(),
            ));
        }
        // 1e6 = the encoder's microsecond timebase; higher fps collapses
        // consecutive frames onto duplicate timestamps.
        if !(config.fps.is_finite() && config.fps > 0.0 && config.fps <= 1_000_000.0) {
            return Err(VideoRecordError::InvalidConfig(format!(
                "fps must be a positive number no greater than 1000000, got {}",
                config.fps
            )));
        }

        if let Some(crf) = config.crf
            && crf > 51
        {
            return Err(VideoRecordError::InvalidConfig(format!(
                "crf must be between 0 (lossless) and 51 (worst), got {crf}"
            )));
        }

        static FFMPEG_INIT: Once = Once::new();
        FFMPEG_INIT.call_once(|| {
            let _ = video_rs::init();
        });

        let mut options = HashMap::new();
        options.insert(
            "preset".to_string(),
            config
                .preset
                .clone()
                .unwrap_or_else(|| "medium".to_string()),
        );
        if config.realtime {
            options.insert("tune".to_string(), "zerolatency".to_string());
        }
        if let Some(crf) = config.crf {
            options.insert("crf".to_string(), crf.to_string());
        }
        let mut settings = Settings::preset_h264_custom(
            config.width as usize,
            config.height as usize,
            PixelFormat::YUV420P,
            options.into(),
        );
        if let Some(interval) = config.keyframe_interval {
            settings.set_keyframe_interval(interval);
        }

        let encoder = Encoder::new(path.as_ref(), settings).map_err(VideoRecordError::Create)?;

        let (frame_tx, frame_rx) = flume::bounded(3);
        let worker_config = config.clone();
        let dest = path.as_ref().to_path_buf();
        let handle = thread::Builder::new()
            .name("nannou_video_encode".to_string())
            .spawn(move || encode_main(encoder, worker_config, frame_rx, dest))
            .expect("failed to spawn video encoder thread");

        let expected_len =
            config.width as usize * config.height as usize * config.pixel_format.bytes_per_pixel();
        Ok(Self {
            frame_tx: Some(frame_tx),
            handle: Some(handle),
            expected_len,
            config,
            frames_sent: 0,
        })
    }

    /// Queue one frame: tightly packed rows in the configured pixel format,
    /// top row first. Blocks while the encoder is a few frames behind.
    pub fn record_frame(&mut self, pixels: Vec<u8>) -> Result<(), VideoRecordError> {
        if pixels.len() != self.expected_len {
            return Err(VideoRecordError::FrameSize {
                expected: self.expected_len,
                got: pixels.len(),
                width: self.config.width,
                height: self.config.height,
                format: self.config.pixel_format,
            });
        }
        let tx = self.frame_tx.as_ref().ok_or(VideoRecordError::WorkerDied)?;
        if tx.send(pixels).is_err() {
            // The worker only exits early on an error; join to surface it.
            return Err(self
                .join_worker()
                .err()
                .unwrap_or(VideoRecordError::WorkerDied));
        }
        self.frames_sent += 1;
        Ok(())
    }

    pub fn frames_recorded(&self) -> u64 {
        self.frames_sent
    }

    pub fn width(&self) -> u32 {
        self.config.width
    }

    pub fn height(&self) -> u32 {
        self.config.height
    }

    pub fn fps(&self) -> f64 {
        self.config.fps
    }

    /// Flush, finalize the container, and return the number of frames encoded.
    pub fn finish(mut self) -> Result<u64, VideoRecordError> {
        self.join_worker()
    }

    fn join_worker(&mut self) -> Result<u64, VideoRecordError> {
        // Closing the channel signals the worker to flush and finish.
        drop(self.frame_tx.take());
        match self.handle.take() {
            Some(handle) => handle.join().map_err(|_| VideoRecordError::WorkerDied)?,
            None => Err(VideoRecordError::WorkerDied),
        }
    }
}

impl Drop for VideoRecorder {
    fn drop(&mut self) {
        if self.handle.is_some() {
            let _ = self.join_worker();
        }
    }
}

fn encode_main(
    mut encoder: Encoder,
    config: VideoRecorderConfig,
    frame_rx: Receiver<Vec<u8>>,
    dest: PathBuf,
) -> Result<u64, VideoRecordError> {
    let time_base = encoder.time_base();
    let mut frame_index: u64 = 0;

    while let Ok(pixels) = frame_rx.recv() {
        let mut frame = RawFrame::new(PixelFormat::RGB24, config.width, config.height);
        fill_rgb24(&mut frame, &pixels, &config);

        let pts = Time::from_secs_f64(frame_index as f64 / config.fps)
            .with_time_base(time_base)
            .into_value();
        frame.set_pts(pts);

        encoder
            .encode_raw(frame)
            .map_err(|source| VideoRecordError::Encode {
                frame: frame_index,
                source,
            })?;
        frame_index += 1;
    }

    if frame_index == 0 {
        drop(encoder);
        let _ = std::fs::remove_file(&dest);
        return Err(VideoRecordError::NoFrames);
    }

    encoder.finish().map_err(VideoRecordError::Finish)?;
    Ok(frame_index)
}

fn fill_rgb24(frame: &mut RawFrame, pixels: &[u8], config: &VideoRecorderConfig) {
    let width = config.width as usize;
    let height = config.height as usize;
    let bpp = config.pixel_format.bytes_per_pixel();
    let src_stride = width * bpp;
    let dst_stride = frame.stride(0);
    let data = frame.data_mut(0);

    for y in 0..height {
        let src_row = &pixels[y * src_stride..(y + 1) * src_stride];
        let dst_row = &mut data[y * dst_stride..y * dst_stride + width * 3];
        match config.pixel_format {
            RecorderPixelFormat::Rgb8 => dst_row.copy_from_slice(src_row),
            RecorderPixelFormat::Rgba8 => {
                for (dst, src) in dst_row.chunks_exact_mut(3).zip(src_row.chunks_exact(4)) {
                    dst.copy_from_slice(&src[..3]);
                }
            }
            RecorderPixelFormat::Bgra8 => {
                for (dst, src) in dst_row.chunks_exact_mut(3).zip(src_row.chunks_exact(4)) {
                    dst[0] = src[2];
                    dst[1] = src[1];
                    dst[2] = src[0];
                }
            }
        }
    }
}
