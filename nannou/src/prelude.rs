//! A collection of commonly used items that we recommend importing for ease of use.

pub use bevy::asset::{self as bevy_asset, Asset};
pub use bevy::camera;
pub use bevy::color::palettes::css::*;
pub use bevy::color::prelude::*;
pub use bevy::ecs::{self as bevy_ecs, prelude::*};
pub use bevy::image as bevy_image;
pub use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor, prelude::*};
pub use bevy::input::mouse::MouseWheel;
pub use bevy::post_process::bloom::*;
pub use bevy::prelude::{
    ClearColorConfig, Entity, Handle, Image, KeyCode, MonitorSelection, MouseButton,
    OrthographicProjection, TouchInput, Vec3, Window, WindowResizeConstraints, debug, default,
    error, info, light_consts, trace, warn,
};
pub use bevy::reflect::{self as bevy_reflect, Reflect, TypePath};
pub use bevy::render as bevy_render;
pub use bevy::render::render_asset::*;
pub use bevy::render::render_resource::*;
pub use bevy::shader::*;
pub use bevy::tasks::prelude::{AsyncComputeTaskPool, IoTaskPool, block_on};
pub use bevy::tasks::{Task, futures_lite::future};
pub use bevy::window::CursorOptions;
pub use bevy::winit::UpdateMode;

#[cfg(feature = "egui")]
pub use bevy_egui::egui;

pub use nannou_derive::shader_model;

pub use nannou_draw::color::*;
pub use nannou_draw::draw::*;
pub use nannou_draw::render::NannouShaderModelPlugin;
pub use nannou_draw::render::blend::*;
pub use nannou_draw::text::*;
pub use nannou_draw::*;

#[cfg(feature = "video")]
pub use nannou_video::prelude::*;

pub use crate::frame::*;
pub use crate::render::RenderApp;
pub use crate::render::compute::*;
pub use crate::wgpu;
pub use crate::wgpu::util::{BufferInitDescriptor, DeviceExt};
pub use nannou_core::prelude::*;

pub use crate::app::{self, RunMode, UpdateModeExt};
pub use crate::camera::SetCamera;
pub use crate::context::{self, App};
#[cfg(feature = "serde")]
pub use crate::io::{load_from_json, load_from_toml, safe_file_save, save_to_json, save_to_toml};
pub use crate::light::SetLight;
pub use crate::time::DurationF64;
