//! A set of useful descriptors for blending colours!

use crate::wgpu;

pub const NORMAL: wgpu::BlendState = wgpu::BlendState {
    src_factor: wgpu::BlendFactor::SrcAlpha,
    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
    operation: wgpu::BlendOperation::Add,
};

pub const ADD: wgpu::BlendState = wgpu::BlendState {
    src_factor: wgpu::BlendFactor::SrcColor,
    dst_factor: wgpu::BlendFactor::DstColor,
    operation: wgpu::BlendOperation::Add,
};

pub const SUBTRACT: wgpu::BlendState = wgpu::BlendState {
    src_factor: wgpu::BlendFactor::SrcColor,
    dst_factor: wgpu::BlendFactor::DstColor,
    operation: wgpu::BlendOperation::Subtract,
};

pub const REVERSE_SUBTRACT: wgpu::BlendState = wgpu::BlendState {
    src_factor: wgpu::BlendFactor::SrcColor,
    dst_factor: wgpu::BlendFactor::DstColor,
    operation: wgpu::BlendOperation::ReverseSubtract,
};

pub const DARKEST: wgpu::BlendState = wgpu::BlendState {
    src_factor: wgpu::BlendFactor::SrcColor,
    dst_factor: wgpu::BlendFactor::DstColor,
    operation: wgpu::BlendOperation::Min,
};

pub const LIGHTEST: wgpu::BlendState = wgpu::BlendState {
    src_factor: wgpu::BlendFactor::SrcColor,
    dst_factor: wgpu::BlendFactor::DstColor,
    operation: wgpu::BlendOperation::Max,
};
