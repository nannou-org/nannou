//! A set of useful descriptors for blending colours!

pub const NORMAL: wgpu::BlendDescriptor = wgpu::BlendDescriptor {
    src_factor: wgpu::BlendFactor::SrcAlpha,
    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
    operation: wgpu::BlendOperation::Add,
};

pub const ADD: wgpu::BlendDescriptor = wgpu::BlendDescriptor {
    src_factor: wgpu::BlendFactor::SrcColor,
    dst_factor: wgpu::BlendFactor::DstColor,
    operation: wgpu::BlendOperation::Add,
};

pub const SUBTRACT: wgpu::BlendDescriptor = wgpu::BlendDescriptor {
    src_factor: wgpu::BlendFactor::SrcColor,
    dst_factor: wgpu::BlendFactor::DstColor,
    operation: wgpu::BlendOperation::Subtract,
};

pub const REVERSE_SUBTRACT: wgpu::BlendDescriptor = wgpu::BlendDescriptor {
    src_factor: wgpu::BlendFactor::SrcColor,
    dst_factor: wgpu::BlendFactor::DstColor,
    operation: wgpu::BlendOperation::ReverseSubtract,
};

pub const DARKEST: wgpu::BlendDescriptor = wgpu::BlendDescriptor {
    src_factor: wgpu::BlendFactor::SrcColor,
    dst_factor: wgpu::BlendFactor::DstColor,
    operation: wgpu::BlendOperation::Min,
};

pub const LIGHTEST: wgpu::BlendDescriptor = wgpu::BlendDescriptor {
    src_factor: wgpu::BlendFactor::SrcColor,
    dst_factor: wgpu::BlendFactor::DstColor,
    operation: wgpu::BlendOperation::Max,
};
