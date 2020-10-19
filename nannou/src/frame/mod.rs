//! Items related to the **Frame** type, describing a single frame of graphics for a single window.

use crate::color::IntoLinSrgba;
use crate::wgpu;
use std::ops;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

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
    render_data: &'swap_chain RenderData,
    capture_data: &'swap_chain CaptureData,
}

/// Data specific to the intermediary textures.
#[derive(Debug)]
pub struct RenderData {
    intermediary_lin_srgba: IntermediaryLinSrgba,
    msaa_samples: u32,
    size: [u32; 2],
    // For writing the intermediary linear sRGBA texture to the swap chain texture.
    texture_reshaper: wgpu::TextureReshaper,
}

/// Data related to the capturing of a frame.
#[derive(Debug)]
pub(crate) struct CaptureData {
    // If `Some`, indicates a path to which the current frame should be written.
    pub(crate) next_frame_path: Mutex<Option<PathBuf>>,
    // The `TextureCapturer` used to capture the frame.
    pub(crate) texture_capturer: wgpu::TextureCapturer,
}

/// Intermediary textures used as a target before resolving multisampling and writing to the
/// swapchain texture.
#[derive(Debug)]
pub(crate) struct IntermediaryLinSrgba {
    msaa_texture: Option<(wgpu::Texture, wgpu::TextureView)>,
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
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
    // TODO: Kvark recommends trying `Rgb10A2Unorm`.
    pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

    // Initialise a new empty frame ready for "drawing".
    pub(crate) fn new_empty(
        raw_frame: RawFrame<'swap_chain>,
        render_data: &'swap_chain RenderData,
        capture_data: &'swap_chain CaptureData,
    ) -> Self {
        Frame {
            raw_frame,
            render_data,
            capture_data,
        }
    }

    // The private implementation of `submit`, allowing it to be called during `drop` if submission
    // has not yet occurred.
    fn submit_inner(&mut self) {
        let Frame {
            ref capture_data,
            ref render_data,
            ref mut raw_frame,
        } = *self;

        // Resolve the MSAA if necessary.
        if let Some((_, ref msaa_texture_view)) = render_data.intermediary_lin_srgba.msaa_texture {
            let mut encoder = raw_frame.command_encoder();
            wgpu::resolve_texture(
                msaa_texture_view,
                &render_data.intermediary_lin_srgba.texture_view,
                &mut *encoder,
            );
        }

        // Check to see if the user specified capturing the frame.
        let mut snapshot_capture = None;
        if let Ok(mut guard) = capture_data.next_frame_path.lock() {
            if let Some(path) = guard.take() {
                let device = raw_frame.device_queue_pair().device();
                let mut encoder = raw_frame.command_encoder();
                let snapshot = capture_data.texture_capturer.capture(
                    device,
                    &mut *encoder,
                    &render_data.intermediary_lin_srgba.texture,
                );
                snapshot_capture = Some((path, snapshot));
            }
        }

        // Convert the linear sRGBA image to the swapchain image.
        //
        // To do so, we sample the linear sRGBA image and draw it to the swapchain image using
        // two triangles and a fragment shader.
        {
            let mut encoder = raw_frame.command_encoder();
            render_data
                .texture_reshaper
                .encode_render_pass(raw_frame.swap_chain_texture(), &mut *encoder);
        }

        // Submit all commands on the device queue.
        raw_frame.submit_inner();

        // If the user did specify capturing the frame, submit the asynchronous read.
        if let Some((path, snapshot)) = snapshot_capture {
            let result = snapshot.read(move |result| match result {
                // TODO: Log errors, don't print to stderr.
                Err(e) => eprintln!("failed to async read captured frame: {:?}", e),
                Ok(image) => {
                    let image = image.to_owned();
                    if let Err(e) = image.save(&path) {
                        // TODO: Log errors, don't print to stderr.
                        eprintln!(
                            "failed to save captured frame to \"{}\": {}",
                            path.display(),
                            e
                        );
                    }
                }
            });
            if let Err(wgpu::TextureCapturerAwaitWorkerTimeout(_)) = result {
                // TODO: Log errors, don't print to stderr.
                eprintln!("timed out while waiting for a worker thread to capture the frame");
            }
        }
    }

    /// The texture to which all graphics should be drawn this frame.
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
    pub fn texture(&self) -> &wgpu::Texture {
        self.render_data
            .intermediary_lin_srgba
            .msaa_texture
            .as_ref()
            .map(|(tex, _)| tex)
            .unwrap_or(&self.render_data.intermediary_lin_srgba.texture)
    }

    /// A full view into the frame's texture.
    ///
    /// See `texture` for details.
    pub fn texture_view(&self) -> &wgpu::TextureView {
        self.render_data
            .intermediary_lin_srgba
            .msaa_texture
            .as_ref()
            .map(|(_, view)| view)
            .unwrap_or(&self.render_data.intermediary_lin_srgba.texture_view)
    }

    /// Returns the resolve target texture in the case that MSAA is enabled.
    pub fn resolve_target(&self) -> Option<&wgpu::TextureView> {
        if self.render_data.msaa_samples <= 1 {
            None
        } else {
            Some(&self.render_data.intermediary_lin_srgba.texture_view)
        }
    }

    /// The color format of the `Frame`'s intermediary linear sRGBA texture (equal to
    /// `Frame::TEXTURE_FORMAT`).
    pub fn texture_format(&self) -> wgpu::TextureFormat {
        Self::TEXTURE_FORMAT
    }

    /// The number of MSAA samples of the `Frame`'s intermediary linear sRGBA texture.
    pub fn texture_msaa_samples(&self) -> u32 {
        self.render_data.msaa_samples
    }

    /// The size of the frame's texture in pixels.
    pub fn texture_size(&self) -> [u32; 2] {
        self.render_data.size
    }

    /// Short-hand for constructing a `wgpu::RenderPassColorAttachmentDescriptor` for use within a
    /// render pass that targets this frame's texture. The returned descriptor's `attachment` will
    /// the same `wgpu::TextureView` returned by the `Frame::texture` method.
    ///
    /// Note that this method will not perform any resolving. In the case that `msaa_samples` is
    /// greater than `1`, a render pass will be automatically added after the `view` completes and
    /// before the texture is drawn to the swapchain.
    pub fn color_attachment_descriptor(&self) -> wgpu::RenderPassColorAttachmentDescriptor {
        let load = wgpu::LoadOp::Load;
        let store = true;
        let attachment = match self.render_data.intermediary_lin_srgba.msaa_texture {
            None => &self.render_data.intermediary_lin_srgba.texture_view,
            Some((_, ref msaa_texture_view)) => msaa_texture_view,
        };
        let resolve_target = None;
        wgpu::RenderPassColorAttachmentDescriptor {
            attachment,
            resolve_target,
            ops: wgpu::Operations { load, store },
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
        wgpu::clear_texture(self.texture_view(), color, &mut *self.command_encoder())
    }

    /// Submit the frame to the GPU!
    ///
    /// Note that you do not need to call this manually as submission will occur automatically when
    /// the **Frame** is dropped.
    ///
    /// Before submission, the frame does the following:
    ///
    /// - If the frame's intermediary linear sRGBA texture is multisampled, resolve it.
    /// - Write the intermediary linear sRGBA image to the swap chain texture.
    ///
    /// It can sometimes be useful to submit the **Frame** before `view` completes in order to read
    /// the frame's texture back to the CPU (e.g. for screen shots, recordings, etc).
    pub fn submit(mut self) {
        self.submit_inner();
    }
}

impl CaptureData {
    pub(crate) fn new(max_jobs: u32, timeout: Option<Duration>) -> Self {
        CaptureData {
            next_frame_path: Default::default(),
            texture_capturer: wgpu::TextureCapturer::new(Some(max_jobs), timeout),
        }
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
        let src_sample_count = 1;
        let swap_chain_sample_count = 1;
        let texture_reshaper = wgpu::TextureReshaper::new(
            device,
            &intermediary_lin_srgba.texture_view,
            src_sample_count,
            intermediary_lin_srgba
                .texture_view
                .component_type()
                .expect("Texture view has no component type?"),
            swap_chain_sample_count,
            swap_chain_format,
        );
        RenderData {
            intermediary_lin_srgba,
            texture_reshaper,
            size: swap_chain_dims,
            msaa_samples,
        }
    }
}

impl<'swap_chain> Drop for Frame<'swap_chain> {
    fn drop(&mut self) {
        if !self.raw_frame.is_submitted() {
            self.submit_inner();
        }
    }
}

fn create_lin_srgba_msaa_texture(
    device: &wgpu::Device,
    swap_chain_dims: [u32; 2],
    msaa_samples: u32,
) -> wgpu::Texture {
    wgpu::TextureBuilder::new()
        .size(swap_chain_dims)
        .sample_count(msaa_samples)
        .usage(wgpu::TextureUsage::OUTPUT_ATTACHMENT)
        .format(Frame::TEXTURE_FORMAT)
        .build(device)
}

fn create_lin_srgba_texture(device: &wgpu::Device, swap_chain_dims: [u32; 2]) -> wgpu::Texture {
    wgpu::TextureBuilder::new()
        .size(swap_chain_dims)
        .format(Frame::TEXTURE_FORMAT)
        .usage(wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED)
        .build(device)
}

fn create_intermediary_lin_srgba(
    device: &wgpu::Device,
    swap_chain_dims: [u32; 2],
    msaa_samples: u32,
) -> IntermediaryLinSrgba {
    let msaa_texture = match msaa_samples {
        0 | 1 => None,
        _ => {
            let texture = create_lin_srgba_msaa_texture(device, swap_chain_dims, msaa_samples);
            let texture_view = texture.view().build();
            Some((texture, texture_view))
        }
    };
    let texture = create_lin_srgba_texture(device, swap_chain_dims);
    let texture_view = texture.view().build();
    IntermediaryLinSrgba {
        msaa_texture,
        texture,
        texture_view,
    }
}
