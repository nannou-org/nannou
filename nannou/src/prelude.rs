//! A collection of commonly used items that we recommend importing for ease of use.

pub use bevy::asset::{self as bevy_asset, Asset};
pub use bevy::ecs as bevy_ecs;
pub use bevy::image as bevy_image;
pub use bevy::reflect::{self as bevy_reflect, TypePath};
pub use bevy::render as bevy_render;
pub use bevy::tasks::prelude::{block_on, AsyncComputeTaskPool, IoTaskPool};
pub use bevy::tasks::{futures_lite::future, Task};

#[cfg(feature = "egui")]
pub use bevy_egui::egui;

pub use crate::frame::*;
pub use crate::render::compute::*;
pub use crate::render::RenderApp;
pub use crate::wgpu;
pub use crate::wgpu::util::{BufferInitDescriptor, DeviceExt};
pub use bevy_nannou::prelude::*;
pub use nannou_core::prelude::*;

pub use crate::app::{self, App, RunMode, UpdateModeExt};
pub use crate::camera::SetCamera;
#[cfg(feature = "serde")]
pub use crate::io::{load_from_json, load_from_toml, safe_file_save, save_to_json, save_to_toml};
pub use crate::light::SetLight;
pub use crate::time::DurationF64;
