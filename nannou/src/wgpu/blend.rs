//! A set of useful descriptors for blending colours!

use crate::wgpu;

pub const NORMAL: wgpu::BlendComponent = wgpu::BlendComponent {
    src_factor: wgpu::BlendFactor::SrcAlpha,
    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
    operation: wgpu::BlendOperation::Add,
};

pub const ADD: wgpu::BlendComponent = wgpu::BlendComponent {
    src_factor: wgpu::BlendFactor::Src,
    dst_factor: wgpu::BlendFactor::Dst,
    operation: wgpu::BlendOperation::Add,
};

pub const SUBTRACT: wgpu::BlendComponent = wgpu::BlendComponent {
    src_factor: wgpu::BlendFactor::Src,
    dst_factor: wgpu::BlendFactor::Dst,
    operation: wgpu::BlendOperation::Subtract,
};

pub const REVERSE_SUBTRACT: wgpu::BlendComponent = wgpu::BlendComponent {
    src_factor: wgpu::BlendFactor::Src,
    dst_factor: wgpu::BlendFactor::Dst,
    operation: wgpu::BlendOperation::ReverseSubtract,
};

pub const DARKEST: wgpu::BlendComponent = wgpu::BlendComponent {
    src_factor: wgpu::BlendFactor::One,
    dst_factor: wgpu::BlendFactor::One,
    operation: wgpu::BlendOperation::Min,
};

pub const LIGHTEST: wgpu::BlendComponent = wgpu::BlendComponent {
    src_factor: wgpu::BlendFactor::One,
    dst_factor: wgpu::BlendFactor::One,
    operation: wgpu::BlendOperation::Max,
};
