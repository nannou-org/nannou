//! Items related to WGPU and the Rust API used by nannou to access the GPU.
//!
//! This module re-exports the entire `wgpu` crate along with all of its documentation while also
//! adding some additional helper items.

mod texture_format_converter;

// Re-export all of `wgpu` along with its documentation.
#[doc(inline)]
pub use wgpu::*;

pub use self::texture_format_converter::TextureFormatConverter;

/// The default set of options used to request a `wgpu::Adapter` when creating windows.
pub const DEFAULT_ADAPTER_REQUEST_OPTIONS: RequestAdapterOptions = RequestAdapterOptions {
    power_preference: PowerPreference::Default,
    backends: BackendBit::PRIMARY,
};

/// The default set of `Extensions` used within the `default_device_descriptor()` function.
pub const DEFAULT_EXTENSIONS: Extensions = Extensions {
    anisotropic_filtering: true,
};

/// The default device descriptor used to instantiate a logical device when creating windows.
pub fn default_device_descriptor() -> DeviceDescriptor {
    let extensions = DEFAULT_EXTENSIONS;
    let limits = Limits::default();
    DeviceDescriptor { extensions, limits }
}

/// Adds a simple render pass command to the given encoder that resolves the given multisampled
/// `src_texture` to the given non-multisampled `dst_texture`.
///
/// Both the `src_texture` and `dst_texture` must have:
///
/// - `TextureUsage::OUTPUT_ATTACHMENT` enabled.
/// - The same dimensions.
/// - The same `TextureFormat`.
pub fn resolve_texture(
    src_texture: &wgpu::TextureView,
    dst_texture: &wgpu::TextureView,
    encoder: &mut wgpu::CommandEncoder,
) {
    let color_attachment = wgpu::RenderPassColorAttachmentDescriptor {
        attachment: src_texture,
        resolve_target: Some(dst_texture),
        load_op: wgpu::LoadOp::Load,
        store_op: wgpu::StoreOp::Store,
        clear_color: wgpu::Color::TRANSPARENT,
    };
    let render_pass_desc = wgpu::RenderPassDescriptor {
        color_attachments: &[color_attachment],
        depth_stencil_attachment: None,
    };
    let _render_pass = encoder.begin_render_pass(&render_pass_desc);
}

/// Adds a simple render pass command to the given encoder that simply clears the given texture
/// with the given colour.
///
/// The given `texture` must have `TextureUsage::OUTPUT_ATTACHMENT` enabled.
pub fn clear_texture(
    texture: &wgpu::TextureView,
    clear_color: wgpu::Color,
    encoder: &mut wgpu::CommandEncoder,
) {
    let color_attachment = wgpu::RenderPassColorAttachmentDescriptor {
        attachment: texture,
        resolve_target: None,
        load_op: wgpu::LoadOp::Clear,
        store_op: wgpu::StoreOp::Store,
        clear_color,
    };
    let render_pass_desc = wgpu::RenderPassDescriptor {
        color_attachments: &[color_attachment],
        depth_stencil_attachment: None,
    };
    let _render_pass = encoder.begin_render_pass(&render_pass_desc);
}
