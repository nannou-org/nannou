//! A collection of commonly used items that we recommend importing for ease of use.

pub use bevy::asset as bevy_asset;
pub use bevy::ecs as bevy_ecs;
pub use bevy::reflect as bevy_reflect;
pub use bevy::render as bevy_render;
pub use bevy::tasks::*;
#[cfg(feature = "egui")]
pub use bevy_egui::egui;

pub use crate::frame::*;
pub use crate::render::*;
pub use crate::wgpu;
pub use crate::wgpu::util::{BufferInitDescriptor, DeviceExt};
pub use bevy_nannou::prelude::*;
pub use bevy_nannou_derive::shader_model;
pub use nannou_core::prelude::*;

pub use crate::app::{self, App, RunMode, UpdateModeExt};
pub use crate::camera::SetCamera;
#[cfg(feature = "serde")]
pub use crate::io::{load_from_json, load_from_toml, safe_file_save, save_to_json, save_to_toml};
pub use crate::light::SetLight;
pub use crate::time::DurationF64;
