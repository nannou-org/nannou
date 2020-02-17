//! Items related to the **Frame** type, describing a single frame of graphics for a single window.

use crate::color::IntoLinSrgba;
use crate::wgpu;
use std::ops;

pub mod raw;

pub use self::raw::RawFrame;

/// A **Frame** to which the user can draw graphics before it is presented to the display.
///
/// **Frame**s are delivered to the user for drawing via the user's **view** function.
///
/// See the **RawFrame** docs for more details on how the implementation works under the hood. The
/// **Frame** type differs in that rather than drawing directly to the swapchain image the user may
/// draw to an intermediary linear sRGBA image. There are several advantages of drawing to an
/// intermediary image.
pub struct Frame<'swap_chain> {
    raw_frame: RawFrame<'swap_chain>,
    data: RenderData,
}

/// Data specific to the intermediary textures.
#[derive(Debug)]
pub struct RenderData {
    intermediary_lin_srgba: IntermediaryLinSrgba,
    msaa_samples: u32,
    // For writing the intermediary linear sRGBA texture to the swap chain texture.
    texture_format_converter: wgpu::TextureFormatConverter,
}

/// Intermediary textures used as a target before resolving multisampling and writing to the
/// swapchain texture.
#[derive(Debug)]
pub(crate) struct IntermediaryLinSrgba {
    msaa_texture: Option<wgpu::TextureView>,
    texture: wgpu::TextureView,
}

impl<'swap_chain> ops::Deref for Frame<'swap_chain> {
    type Target = RawFrame<'swap_chain>;
    fn deref(&self) -> &Self::Target {
        &self.raw_frame
    }
}

impl<'swap_chain> Frame<'swap_chain> {
    /// The default number of multisample anti-aliasing samples used if the window with which the
    /// `Frame` is associated supports it.
    pub const DEFAULT_MSAA_SAMPLES: u32 = 4;
    /// The texture format used by the intermediary linear sRGBA image.
    ///
    /// We use a high bit depth format in order to retain as much information as possible when
    /// converting from the linear representation to the swapchain format (normally a non-linear
    /// representation).
    pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Unorm;

    // Initialise a new empty frame ready for "drawing".
    pub(crate) fn new_empty(raw_frame: RawFrame<'swap_chain>, data: RenderData) -> Self {
        Frame { raw_frame, data }
    }

    // Called after the user's `view` function returns, this consumes the `Frame`, adds commands
    // for drawing the `lin_srgba_msaa` to the `swapchain_image` and returns the inner
    // `RenderData` and `RawFrame` so that the `RenderData` may be stored back within the `Window`
    // and the `RawFrame` may be `finish`ed.
    pub(crate) fn finish(self) -> (RenderData, RawFrame<'swap_chain>) {
        let Frame { data, raw_frame } = self;

        // Resolve the MSAA if necessary.
        if let Some(ref msaa_texture) = data.intermediary_lin_srgba.msaa_texture {
            let mut encoder = raw_frame.command_encoder();
            wgpu::resolve_texture(
                msaa_texture,
                &data.intermediary_lin_srgba.texture,
                &mut *encoder,
            );
        }

        // Convert the linear sRGBA image to the swapchain image.
        //
        // To do so, we sample the linear sRGBA image and draw it to the swapchain image using
        // two triangles and a fragment shader.
        {
            let mut encoder = raw_frame.command_encoder();
            data.texture_format_converter
                .encode_render_pass(raw_frame.swap_chain_texture(), &mut *encoder);
        }

        (data, raw_frame)
    }

    /// The texture to which all use graphics should be drawn this frame.
    ///
    /// This is **not** the swapchain texture, but rather an intermediary linear sRGBA image. This
    /// intermediary image is used in order to:
    ///
    /// - Ensure consistent MSAA resolve behaviour across platforms.
    /// - Avoid the need for multiple implicit conversions to and from linear sRGBA for each
    /// graphics pipeline render pass that is used.
    /// - Allow for the user's rendered image to persist between frames.
    ///
    /// The exact format of the texture is equal to `Frame::TEXTURE_FORMAT`.
    ///
    /// If the number of MSAA samples specified is greater than `1` (which it is by default if
    /// supported by the platform), this will be a multisampled texture. After the **view**
    /// function returns, this texture will be resolved to a non-multisampled linear sRGBA texture.
    /// After the texture has been resolved if necessary, it will then be used as a shader input
    /// within a graphics pipeline used to draw the swapchain texture.
    pub fn texture(&self) -> &wgpu::TextureView {
        self.data
            .intermediary_lin_srgba
            .msaa_texture
            .as_ref()
            .unwrap_or(&self.data.intermediary_lin_srgba.texture)
    }

    /// The color format of the `Frame`'s intermediary linear sRGBA texture (equal to
    /// `Frame::TEXTURE_FORMAT`).
    pub fn texture_format(&self) -> wgpu::TextureFormat {
        Self::TEXTURE_FORMAT
    }

    /// The number of MSAA samples of the `Frame`'s intermediary linear sRGBA texture.
    pub fn texture_msaa_samples(&self) -> u32 {
        self.data.msaa_samples
    }

    /// Short-hand for constructing a `wgpu::RenderPassColorAttachmentDescriptor` for use within a
    /// render pass that targets this frame's texture. The returned descriptor's `attachment` will
    /// the same `wgpu::TextureView` returned by the `Frame::texture` method.
    ///
    /// Note that this method will not perform any resolving. In the case that `msaa_samples` is
    /// greater than `1`, a render pass will be automatically added after the `view` completes and
    /// before the texture is drawn to the swapchain.
    pub fn color_attachment_descriptor(&self) -> wgpu::RenderPassColorAttachmentDescriptor {
        let load_op = wgpu::LoadOp::Load;
        let store_op = wgpu::StoreOp::Store;
        let attachment = match self.data.intermediary_lin_srgba.msaa_texture {
            None => &self.data.intermediary_lin_srgba.texture,
            Some(ref msaa_texture) => msaa_texture,
        };
        let resolve_target = None;
        let clear_color = wgpu::Color::TRANSPARENT;
        wgpu::RenderPassColorAttachmentDescriptor {
            attachment,
            resolve_target,
            load_op,
            store_op,
            clear_color,
        }
    }

    /// Clear the texture with the given color.
    pub fn clear<C>(&self, color: C)
    where
        C: IntoLinSrgba<f32>,
    {
        let lin_srgba = color.into_lin_srgba();
        let (r, g, b, a) = lin_srgba.into_components();
        let (r, g, b, a) = (r as f64, g as f64, b as f64, a as f64);
        let color = wgpu::Color { r, g, b, a };
        wgpu::clear_texture(self.texture(), color, &mut *self.command_encoder())
    }
}

impl RenderData {
    /// Initialise the render data.
    ///
    /// Creates an `wgpu::TextureView` with the given parameters.
    ///
    /// If `msaa_samples` is greater than 1 a `multisampled` texture will also be created. Otherwise the
    /// a regular non-multisampled image will be created.
    pub(crate) fn new(
        device: &wgpu::Device,
        swap_chain_dims: [u32; 2],
        swap_chain_format: wgpu::TextureFormat,
        msaa_samples: u32,
    ) -> Self {
        let intermediary_lin_srgba =
            create_intermediary_lin_srgba(device, swap_chain_dims, msaa_samples);
        let texture_format_converter = wgpu::TextureFormatConverter::new(
            device,
            &intermediary_lin_srgba.texture,
            swap_chain_format,
        );
        RenderData {
            intermediary_lin_srgba,
            texture_format_converter,
            msaa_samples,
        }
    }
}

fn create_lin_srgba_msaa_texture(
    device: &wgpu::Device,
    swap_chain_dims: [u32; 2],
    msaa_samples: u32,
) -> wgpu::TextureView {
    let [width, height] = swap_chain_dims;
    let msaa_texture_extent = wgpu::Extent3d {
        width,
        height,
        depth: 1,
    };
    let msaa_texture_descriptor = wgpu::TextureDescriptor {
        size: msaa_texture_extent,
        array_layer_count: 1,
        mip_level_count: 1,
        sample_count: msaa_samples,
        dimension: wgpu::TextureDimension::D2,
        format: Frame::TEXTURE_FORMAT,
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
    };
    device
        .create_texture(&msaa_texture_descriptor)
        .create_default_view()
}

fn create_lin_srgba_texture(device: &wgpu::Device, swap_chain_dims: [u32; 2]) -> wgpu::TextureView {
    let [width, height] = swap_chain_dims;
    let texture_extent = wgpu::Extent3d {
        width,
        height,
        depth: 1,
    };
    let texture_descriptor = wgpu::TextureDescriptor {
        size: texture_extent,
        array_layer_count: 1,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: Frame::TEXTURE_FORMAT,
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED,
    };
    device
        .create_texture(&texture_descriptor)
        .create_default_view()
}

fn create_intermediary_lin_srgba(
    device: &wgpu::Device,
    swap_chain_dims: [u32; 2],
    msaa_samples: u32,
) -> IntermediaryLinSrgba {
    let msaa_texture = match msaa_samples {
        0 | 1 => None,
        _ => Some(create_lin_srgba_msaa_texture(
            device,
            swap_chain_dims,
            msaa_samples,
        )),
    };
    let texture = create_lin_srgba_texture(device, swap_chain_dims);
    IntermediaryLinSrgba {
        msaa_texture,
        texture,
    }
}
