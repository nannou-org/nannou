use bevy::prelude::*;

#[derive(Clone, Debug, Reflect)]
pub struct WebcamSupportedFormat {
    pub resolution: UVec2,
    pub framerate: u32,
}

#[derive(Component, Clone, Debug, Reflect)]
pub struct WebcamDevice {
    pub description: String,
    pub formats: Vec<WebcamSupportedFormat>,
}

#[derive(Component, Clone, Debug, Reflect)]
pub struct Webcam {
    pub device: Option<Entity>,
    pub format: WebcamFormat,
    pub srgb: bool,
}

impl Default for Webcam {
    fn default() -> Self {
        Self {
            device: None,
            format: WebcamFormat::default(),
            srgb: true,
        }
    }
}

#[derive(Component, Clone, Debug, Reflect)]
pub struct WebcamStream {
    pub image: Handle<Image>,
    pub resolution: UVec2,
    pub framerate: u32,
}

#[derive(Component, Clone, Debug, Reflect)]
pub struct WebcamError {
    pub message: String,
}

#[derive(Clone, Debug, Reflect, Default)]
pub enum WebcamFormat {
    #[default]
    HighestFrameRate,
    HighestResolution,
    Resolution(UVec2),
    FrameRate(u32),
    Exact {
        resolution: UVec2,
        framerate: u32,
    },
}
