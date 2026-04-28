use bevy::prelude::*;

use crate::asset::Video;

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct VideoPlayer {
    pub video: Handle<Video>,
    pub mode: PlaybackMode,
    pub speed: f32,
    pub paused: bool,
    pub hw_accel: HwAccelPolicy,
    pub resize: VideoResize,
}

impl VideoPlayer {
    pub fn new(video: Handle<Video>) -> Self {
        Self {
            video,
            mode: PlaybackMode::default(),
            speed: 1.0,
            paused: false,
            hw_accel: HwAccelPolicy::default(),
            resize: VideoResize::default(),
        }
    }

    pub fn with_mode(mut self, mode: PlaybackMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    pub fn paused(mut self) -> Self {
        self.paused = true;
        self
    }

    pub fn with_hw_accel(mut self, policy: HwAccelPolicy) -> Self {
        self.hw_accel = policy;
        self
    }

    pub fn with_resize(mut self, resize: VideoResize) -> Self {
        self.resize = resize;
        self
    }
}

#[derive(Reflect, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PlaybackMode {
    #[default]
    Once,
    Loop,
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct VideoOutput {
    pub image: Handle<Image>,
    pub size: UVec2,
    pub position_seconds: f64,
}

#[derive(Component, Reflect, Debug, Clone, Copy)]
#[reflect(Component)]
pub struct SeekTo(pub f64);

#[derive(Reflect, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum VideoResize {
    #[default]
    None,
    Exact(UVec2),
    Fit(UVec2),
    FitEven(UVec2),
}

#[cfg(not(target_arch = "wasm32"))]
impl VideoResize {
    pub(crate) fn resolve(self) -> Option<video_rs::resize::Resize> {
        use video_rs::resize::Resize;
        match self {
            VideoResize::None => None,
            VideoResize::Exact(d) => Some(Resize::Exact(d.x, d.y)),
            VideoResize::Fit(d) => Some(Resize::Fit(d.x, d.y)),
            VideoResize::FitEven(d) => Some(Resize::FitEven(d.x, d.y)),
        }
    }
}

#[derive(Reflect, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum HwAccelPolicy {
    #[default]
    Disabled,
    Auto,
    VideoToolbox,
    Cuda,
    VaApi,
    Qsv,
    D3D11Va,
    D3D12Va,
    VdPau,
    Dxva2,
    Drm,
    OpenCl,
    MediaCodec,
}

#[cfg(not(target_arch = "wasm32"))]
impl HwAccelPolicy {
    pub(crate) fn resolve(self) -> Option<video_rs::hwaccel::HardwareAccelerationDeviceType> {
        use video_rs::hwaccel::HardwareAccelerationDeviceType as D;
        let candidates: &[D] = match self {
            HwAccelPolicy::Disabled => return None,
            HwAccelPolicy::Auto => auto_preference(),
            HwAccelPolicy::VideoToolbox => &[D::VideoToolbox],
            HwAccelPolicy::Cuda => &[D::Cuda],
            HwAccelPolicy::VaApi => &[D::VaApi],
            HwAccelPolicy::Qsv => &[D::Qsv],
            HwAccelPolicy::D3D11Va => &[D::D3D11Va],
            HwAccelPolicy::D3D12Va => &[D::D3D12Va],
            HwAccelPolicy::VdPau => &[D::Vdpau],
            HwAccelPolicy::Dxva2 => &[D::Dxva2],
            HwAccelPolicy::Drm => &[D::Drm],
            HwAccelPolicy::OpenCl => &[D::OpenCl],
            HwAccelPolicy::MediaCodec => &[D::MediaCodec],
        };
        candidates.iter().copied().find(|d| d.is_available())
    }
}

#[cfg(not(target_arch = "wasm32"))]
const fn auto_preference() -> &'static [video_rs::hwaccel::HardwareAccelerationDeviceType] {
    use video_rs::hwaccel::HardwareAccelerationDeviceType as D;
    #[cfg(target_os = "macos")]
    {
        &[D::VideoToolbox]
    }
    #[cfg(target_os = "linux")]
    {
        &[D::VaApi, D::Cuda, D::Vdpau, D::Drm]
    }
    #[cfg(target_os = "windows")]
    {
        &[D::D3D12Va, D::D3D11Va, D::Qsv, D::Dxva2]
    }
    #[cfg(target_os = "android")]
    {
        &[D::MediaCodec]
    }
    #[cfg(not(any(
        target_os = "macos",
        target_os = "linux",
        target_os = "windows",
        target_os = "android"
    )))]
    {
        &[]
    }
}
