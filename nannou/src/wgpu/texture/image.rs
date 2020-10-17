//! Items related to the inter-operation of the `image` crate (images on disk and in RAM) and
//! textures from the wgpu crate (images in GPU memory).

use crate::wgpu;
use std::path::Path;

use wgpu::util::{DeviceExt, BufferInitDescriptor};
use wgpu::Extent3d;

/// The set of pixel types from the image crate that can be loaded directly into a texture.
///
/// The `Rgba8` and `Bgra8` color types are assumed to be non-linear sRGB.
///
/// Note that wgpu only supports texture formats whose size are a power of 2. If you notice a
/// `image::Pixel` type that does not implement `Pixel`, this is likely why.
pub trait Pixel: image::Pixel {
    /// The wgpu texture format of the pixel type.
    const TEXTURE_FORMAT: wgpu::TextureFormat;
}

/// A wrapper around a wgpu buffer that contains an image of a known size and `image::ColorType`.
/// Not to be confused with an `image::BufferImage`, which represents an image stored on CPU.
///
/// Note: as of `wgpu` 0.6, texture-to-buffer and buffer-to-texture copies require that image rows
/// are padded to a multiple `wgpu::COPY_BYTES::PER_ROW_ALIGNMENT` bytes. Note that this is a
/// requirement on the *buffers*, not on the textures! You can have textures of whatever size you
/// like, but when you copy them to/from a buffer, the *buffer rows* need padding. This is referred
/// to as "pitch alignment".
///
/// This wrapper handles these details transparently.
///
/// The image is stored at (0,0) in the buffer, and the remainder of the buffer is left uninitialized.
#[derive(Debug)]
pub struct BufferImage {
    /// The width of the image in pixels, without padding.
    width: u32,
    /// The height of the image in pixels, without padding.
    height: u32,
    /// The color type of the image.
    color_type: image::ColorType,
    /// The number of extra *pixels* (not bytes!) per row of the buffer
    row_padding_pixels: u32,
    /// The buffer storing the image data.
    buffer: wgpu::Buffer,
    /// The descriptor used to create the buffer.
    buffer_descriptor: wgpu::BufferDescriptor<'static>
}
impl BufferImage {
    /// Initialize from an image buffer (i.e. an image on CPU).
    /// Resulting BufferImage will be ready to read immediately.
    pub fn from_image_buffer<P, Container>(
        device: &wgpu::Device,
        image_buffer: &image::ImageBuffer<P, Container>
    ) -> BufferImage
        where
            P: 'static + Pixel,
            Container: std::ops::Deref<Target=[P::Subpixel]>,
    {
        BufferImage::from_image_buffer_internal(device, image_buffer.width(), image_buffer.height(), P::COLOR_TYPE, unsafe { wgpu::bytes::from_slice(&*image_buffer) })
    }
    /// Initialize from an image buffer.
    #[inline(never)]
    fn from_image_buffer_internal(device: &wgpu::Device, width: u32, height: u32, color_type: image::ColorType, raw_data: &[u8]) -> BufferImage {
        let row_padding_pixels = BufferImage::compute_row_padding_pixels(width, color_type);
        let bytes_per_pixel = color_type.bytes_per_pixel() as u32;
        let buffer_size = (padded_width_bytes) * height;
        let buffer_descriptor = wgpu::BufferDescriptor {
            label: Some("nannou::BufferImage"),
            size: buffer_size as u64,
            usage: wgpu::BufferUsage::COPY_SRC,
            mapped_at_creation: true,
        };
        let buffer = device.create_buffer(&buffer_descriptor);

        let mapped = buffer.slice(..).get_mapped_range_mut();
        let mapped = &mut mapped[..];

        let width_bytes = (width * bytes_per_pixel) as usize;
        let padded_width_bytes = width_bytes + (row_padding_pixels * bytes_per_pixel) as usize;
        for row in 0..height as usize {
            let in_start = row * width_bytes;
            let out_start = row * padded_width_bytes;

            // note: leaves mapped[out_start + width_bytes..out_start + padded_width_bytes] uninitialized!
            mapped[out_start..out_start + width_bytes].copy_from_slice(&raw_data[in_start..in_start + width_bytes]);
        }
        BufferImage {
            width,
            height,
            color_type,
            row_padding_pixels,
            buffer,
            buffer_descriptor
        }
    }

    // Returns row_padding
    fn compute_row_padding_pixels(width: u32, color_type: image::ColorType) -> u32 {
        let row_bytes = width * color_type.bytes_per_pixel();
        let padding_bytes = if row_bytes % wgpu::COPY_BYTES_PER_ROW_ALIGNMENT == 0 {
            row_bytes
        } else {
            row_bytes + row_bytes % wgpu::COPY_BYTES_PER_ROW_ALIGNMENT
        };
        padding_bytes / color_type.bytes_per_pixel()
    }

    /// The width of the image stored in the buffer.
    ///
    /// (Does not include padding pixels, you don't need to worry about them.)
    pub fn width(&self) -> u32 {
        self.width
    }

    /// The height of the image stored in the buffer.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// The color type of the image stored within the buffer.
    pub fn color_type(&self) -> image::ColorType {
        self.color_type
    }

    /// Asynchronously maps the buffer of bytes from GPU to host memory and, once mapped, calls the
    /// given user callback with the data represented as an `ImageReadMapping`.
    ///
    /// Note: The given callback will not be called until the memory is mapped and the device is
    /// polled. You should not rely on the callback being called immediately.
    pub async fn read<'b>(&'b self) -> Result<ImageReadMapping<'b>, wgpu::BufferAsyncError> {
        let slice = self.buffer.slice(..);
        slice.map_async(wgpu::MapMode::Read).await?;
        Ok(ImageReadMapping {
            buffer_image: self,
            // fun exercise:
            // read the signature of wgpu::BufferSlice::get_mapped_range()
            // and try to figure out why we don't need another lifetime in ImageReadMapping :)
            view: slice.get_mapped_range()
        })
    }
}

/// A wrapper around a slice of bytes representing an image.
///
/// An `ImageReadMapping` may only be created by reading from a `BufferImage` returned by a
/// `Texture::to_image` call.
pub struct ImageReadMapping<'buffer> {
    buffer_image: &'buffer BufferImage,
    view: wgpu::BufferView<'buffer>
}

/*
impl <'buffer> ImageReadMapping<'buffer> {
    pub fn as_image<'s, P>(&'s self)
        where
            P: Pixel + 'static
    -> image::SubImage<image::ImageBuffer<P, &'s [P::Subpixel]>> {

    }
}
 */

impl wgpu::TextureBuilder {
    /// The minimum required texture usage when loading from an image.
    pub const REQUIRED_IMAGE_TEXTURE_USAGE: wgpu::TextureUsage = wgpu::TextureUsage::COPY_DST;

    /// Produce a texture descriptor from an image.
    ///
    /// Specifically, this supports any image type implementing `image::GenericImageView` whose
    /// `Pixel` type implements `Pixel`.
    ///
    /// By default, the produced builder will have the `wgpu::TextureUsage` returned by
    /// `wgpu::TextureBuilder::default_image_texture_usage()`. This is a general-purpose usage that
    /// should allow for copying to and from the texture, sampling the texture and rendering to the
    /// texture. Specifying only the texture usage required may result in better performance. It
    /// may be necessary to manually specify the the usage if `STORAGE` is required.
    pub fn from_image_view<T>(image_view: &T) -> Self
    where
        T: image::GenericImageView,
        T::Pixel: Pixel,
    {
        builder_from_image_view(image_view)
    }

    /// The default texture usage for the case where a user has loaded a texture from an image.
    pub fn default_image_texture_usage() -> wgpu::TextureUsage {
        wgpu::TextureUsage::COPY_SRC
            | wgpu::TextureUsage::COPY_DST
            | wgpu::TextureUsage::SAMPLED
            | wgpu::TextureUsage::OUTPUT_ATTACHMENT
    }
}

/// Types that may provide access to a `wgpu::Device` and an associated `wgpu::Queue` for loading
/// a texture from an image.
///
/// Notably, implementations exist for `&App`, `&Window`, `&wgpu::DeviceQueuePair` and `(&Device,
/// &Queue)`.
pub trait WithDeviceQueuePair {
    fn with_device_queue_pair<F, O>(self, f: F) -> O
    where
        F: FnOnce(&wgpu::Device, &wgpu::Queue) -> O;
}

impl wgpu::Texture {
    /// Load an image from the given path and upload it as a texture.
    ///
    /// The device and queue `src` can be either the `App`, a `Window`, a `wgpu::DeviceQueuePair`
    /// or a tuple `(&wgpu::Device, &mut wgpu::Queue)`. Access to a `Device` is necessary in order
    /// to create the texture and buffer GPU resources, and access to a `Queue` is necessary for
    /// submitting the commands responsible for copying the buffer contents to the texture. Note
    /// that a texture may only be used with the device with which it was created. This is worth
    /// keeping in mind if you have more than one window and they do not share the same device.
    ///
    /// By default, the texture will have the `COPY_SRC`, `COPY_DST`, `SAMPLED` and
    /// `OUTPUT_ATTACHMENT` usages enabled. If you wish to specify the usage yourself, see the
    /// `load_from_path` constructor.
    ///
    /// If the `&App` is passed as the `src`, the window returned via `app.main_window()` will be
    /// used as the source of the device and queue.
    pub fn from_path<'a, T, P>(src: T, path: P) -> image::ImageResult<Self>
    where
        T: WithDeviceQueuePair,
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let usage = wgpu::TextureBuilder::default_image_texture_usage();
        src.with_device_queue_pair(|device, queue| {
            wgpu::Texture::load_from_path(device, queue, usage, path)
        })
    }

    /// Load a texture from the given image.
    ///
    /// The device and queue `src` can be either the `App`, a `Window`, a `wgpu::DeviceQueuePair`
    /// or a tuple `(&wgpu::Device, &mut wgpu::Queue)`. Access to a `Device` is necessary in order
    /// to create the texture and buffer GPU resources, and access to a `Queue` is necessary for
    /// submitting the commands responsible for copying the buffer contents to the texture. Note
    /// that a texture may only be used with the device with which it was created. This is worth
    /// keeping in mind if you have more than one window and they do not share the same device.
    ///
    /// By default, the texture will have the `COPY_SRC`, `COPY_DST`, `SAMPLED` and
    /// `OUTPUT_ATTACHMENT` usages enabled. If you wish to specify the usage yourself, see the
    /// `load_from_path` constructor.
    ///
    /// If the `&App` is passed as the `src`, the window returned via `app.main_window()` will be
    /// used as the source of the device and queue.
    ///
    /// The `DeviceQueuePairSource` can be either the `App`, a `Window`, a `DeviceQueuePair` or a
    /// tuple `(&Device, &Queue)`.
    pub fn from_image<'a, T>(src: T, image: &image::DynamicImage) -> Self
    where
        T: WithDeviceQueuePair,
    {
        let usage = wgpu::TextureBuilder::default_image_texture_usage();
        src.with_device_queue_pair(|device, queue| {
            wgpu::Texture::load_from_image(device, queue, usage, image)
        })
    }

    /// Read an image file from the given path and load it directly into a texture.
    ///
    /// This is short-hand for calling `image::open` and then `Texture::load_from_image`.
    pub fn load_from_path<P>(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        usage: wgpu::TextureUsage,
        path: P,
    ) -> image::ImageResult<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let image = image::open(path)?;
        Ok(Self::load_from_image(device, queue, usage, &image))
    }

    /// Load a texture directly from a dynamic image.
    ///
    /// If the image is already in a format supported by wgpu, no conversions are performed and the
    /// image is loaded directly as-is with a texture format that matches the original image color
    /// type.
    ///
    /// If the image is of an unsupported format, it will be converted to the closest supported format
    /// before being uploaded.
    pub fn load_from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        usage: wgpu::TextureUsage,
        image: &image::DynamicImage,
    ) -> Self {
        load_texture_from_image(device, queue, usage, image)
    }

    /// Load a texture directly from an image buffer using the given device queue.
    ///
    /// No format or size conversions are performed - the given buffer is loaded directly into GPU
    /// memory.
    ///
    /// Pixel type compatibility is ensured via the `Pixel` trait.
    pub fn load_from_image_buffer<P, Container>(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        usage: wgpu::TextureUsage,
        buffer: &image::ImageBuffer<P, Container>,
    ) -> Self
    where
        P: 'static + Pixel,
        Container: std::ops::Deref<Target = [P::Subpixel]>,
    {
        load_texture_from_image_buffer(device, queue, usage, buffer)
    }

    /// Load a texture array directly from a sequence of image buffers.
    ///
    /// No format or size conversions are performed - the given buffer is loaded directly into GPU
    /// memory.
    ///
    /// Pixel type compatibility is ensured via the `Pixel` trait.
    ///
    /// Returns `None` if there are no images in the given sequence.
    pub fn load_3d_from_image_buffers<'a, I, P, Container>(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        usage: wgpu::TextureUsage,
        buffers: I,
    ) -> Option<Self>
    where
        I: IntoIterator<Item = &'a image::ImageBuffer<P, Container>>,
        I::IntoIter: ExactSizeIterator,
        P: 'static + Pixel,
        Container: 'a + std::ops::Deref<Target = [P::Subpixel]>,
    {
        load_3d_texture_from_image_buffers(device, queue, usage, buffers)
    }

    /// Encode the necessary commands to load a texture directly from a dynamic image.
    ///
    /// If the image is already in a format supported by wgpu, no conversions are performed and the
    /// image is loaded directly as-is with a texture format that matches the original image color
    /// type.
    ///
    /// If the image is of an unsupported format, it will be converted to the closest supported format
    /// before being uploaded.
    ///
    /// NOTE: The returned texture will remain empty until the given `encoder` has its command buffer
    /// submitted to the given `device`'s queue.
    pub fn encode_load_from_image(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        usage: wgpu::TextureUsage,
        image: &image::DynamicImage,
    ) -> Self {
        encode_load_texture_from_image(device, encoder, usage, image)
    }

    /// Encode the necessary commands to load a texture from the given image buffer.
    ///
    /// NOTE: The returned texture will remain empty until the given `encoder` has its command
    /// buffer submitted to the given `device`'s queue.
    ///
    /// No format or size conversions are performed - the given buffer is loaded directly into GPU
    /// memory.
    ///
    /// Pixel type compatibility is ensured via the `Pixel` trait.
    pub fn encode_load_from_image_buffer<P, Container>(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        usage: wgpu::TextureUsage,
        buffer: &image::ImageBuffer<P, Container>,
    ) -> Self
    where
        P: 'static + Pixel,
        Container: std::ops::Deref<Target = [P::Subpixel]>,
    {
        encode_load_texture_from_image_buffer(device, encoder, usage, buffer)
    }

    /// Encode the necessary commands to load a 3d texture directly from a sequence of image
    /// buffers.
    ///
    /// NOTE: The returned texture will remain empty until the given `encoder` has its command buffer
    /// submitted to the given `device`'s queue.
    ///
    /// NOTE: The returned texture will be 3d; you must create
    ///
    /// No format or size conversions are performed - the given buffer is loaded directly into GPU
    /// memory.
    ///
    /// Pixel type compatibility is ensured via the `Pixel` trait.
    ///
    /// Returns `None` if there are no images in the given sequence.
    pub fn encode_load_3d_from_image_buffers<'a, I, P, Container>(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        usage: wgpu::TextureUsage,
        buffers: I,
    ) -> Option<Self>
    where
        I: IntoIterator<Item = &'a image::ImageBuffer<P, Container>>,
        I::IntoIter: ExactSizeIterator,
        P: 'static + Pixel,
        Container: 'a + std::ops::Deref<Target = [P::Subpixel]>,
    {
        encode_load_3d_texture_from_image_buffers(device, encoder, usage, buffers)
    }

    /// Write the contents of the texture into a new image buffer.
    ///
    /// Commands will be added to the given encoder to copy the entire contents of the texture into
    /// the buffer.
    ///
    /// Returns a buffer from which the image can be read asynchronously via `read`.
    ///
    /// Returns `None` if there is no directly compatible `image::ColorType` for the texture's format.
    ///
    /// NOTE: `read` should not be called on the returned buffer until the encoded commands have
    /// been submitted to the device queue.
    pub fn to_image(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Option<BufferImage> {
        let color_type = image_color_type_from_format(self.format())?;
        let size = self.size();
        let buffer = self.to_buffer(device, encoder);
        Some(BufferImage {
            color_type,
            size,
            buffer,
        })
    }
}


impl Pixel for image::Bgra<u8> {
    const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;
}

impl Pixel for image::Luma<u8> {
    const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;
}

impl Pixel for image::Luma<i8> {
    const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Snorm;
}

impl Pixel for image::Luma<u16> {
    const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R16Uint;
}

impl Pixel for image::Luma<i16> {
    const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R16Sint;
}

impl Pixel for image::LumaA<u8> {
    const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rg8Unorm;
}

impl Pixel for image::LumaA<i8> {
    const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rg8Snorm;
}

impl Pixel for image::LumaA<u16> {
    const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rg16Uint;
}

impl Pixel for image::LumaA<i16> {
    const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rg16Sint;
}

impl Pixel for image::Rgba<u8> {
    const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
}

impl Pixel for image::Rgba<i8> {
    const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Snorm;
}

impl Pixel for image::Rgba<u16> {
    const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Uint;
}

impl Pixel for image::Rgba<i16> {
    const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Sint;
}

impl<'a> WithDeviceQueuePair for (&'a wgpu::Device, &'a wgpu::Queue) {
    fn with_device_queue_pair<F, O>(self, f: F) -> O
    where
        F: FnOnce(&wgpu::Device, &wgpu::Queue) -> O,
    {
        let (device, queue) = self;
        f(device, queue)
    }
}

impl<'a> WithDeviceQueuePair for &'a wgpu::DeviceQueuePair {
    fn with_device_queue_pair<F, O>(self, f: F) -> O
    where
        F: FnOnce(&wgpu::Device, &wgpu::Queue) -> O,
    {
        let device = self.device();
        let queue = self.queue();
        f(&*device, &*queue)
    }
}

impl<'a> WithDeviceQueuePair for &'a crate::window::Window {
    fn with_device_queue_pair<F, O>(self, f: F) -> O
    where
        F: FnOnce(&wgpu::Device, &wgpu::Queue) -> O,
    {
        self.swap_chain_device_queue_pair()
            .with_device_queue_pair(f)
    }
}

impl<'a> WithDeviceQueuePair for &'a crate::app::App {
    fn with_device_queue_pair<F, O>(self, f: F) -> O
    where
        F: FnOnce(&wgpu::Device, &wgpu::Queue) -> O,
    {
        self.main_window().with_device_queue_pair(f)
    }
}

impl<'a, 'b> WithDeviceQueuePair for &'a std::cell::Ref<'b, crate::window::Window> {
    fn with_device_queue_pair<F, O>(self, f: F) -> O
    where
        F: FnOnce(&wgpu::Device, &wgpu::Queue) -> O,
    {
        (**self).with_device_queue_pair(f)
    }
}

/// Convert the given color type from the `image` crate to the corresponding wgpu texture format.
///
/// Returns `None` if there is no directly compatible texture format - this is normally the case if
/// the `ColorType` would have a bits_per_pixel that is not equal to a power of 2.
///
/// The `Rgba8` and `Bgra8` color types are assumed to be non-linear sRGB.
pub fn format_from_image_color_type(color_type: image::ColorType) -> Option<wgpu::TextureFormat> {
    let format = match color_type {
        image::ColorType::L8 => wgpu::TextureFormat::R8Unorm,
        image::ColorType::La8 => wgpu::TextureFormat::Rg8Unorm,
        image::ColorType::Rgba8 => wgpu::TextureFormat::Rgba8UnormSrgb,
        image::ColorType::L16 => wgpu::TextureFormat::R16Uint,
        image::ColorType::La16 => wgpu::TextureFormat::Rg16Uint,
        image::ColorType::Rgba16 => wgpu::TextureFormat::Rgba16Uint,
        image::ColorType::Bgra8 => wgpu::TextureFormat::Bgra8UnormSrgb,
        _ => return None,
    };
    Some(format)
}

/// Convert the given texture format to the corresponding color type from the `image` crate.
///
/// Returns `None` if there is no directly compatible color type.
///
/// The `Rgba8` and `Bgra8` color types are assumed to be non-linear sRGB.
pub fn image_color_type_from_format(format: wgpu::TextureFormat) -> Option<image::ColorType> {
    let color_type = match format {
        // TODO: Should we add branches for other same-size formats? e.g. R8Snorm, R8Uint, etc?
        wgpu::TextureFormat::R8Unorm => image::ColorType::L8,
        wgpu::TextureFormat::Rg8Unorm => image::ColorType::La8,
        wgpu::TextureFormat::Rgba8UnormSrgb => image::ColorType::Rgba8,
        wgpu::TextureFormat::R16Uint => image::ColorType::L16,
        wgpu::TextureFormat::Rg16Uint => image::ColorType::La16,
        wgpu::TextureFormat::Rgba16Uint => image::ColorType::Rgba16,
        wgpu::TextureFormat::Bgra8UnormSrgb => image::ColorType::Bgra8,
        _ => return None,
    };
    Some(color_type)
}

/// Produce a texture descriptor from any type implementing `image::GenericImageView` whose `Pixel`
/// type implements `Pixel`.
///
/// By default, the produced builder will have the `wgpu::TextureUsage` returned by
/// `wgpu::TextureBuilder::default_image_texture_usage()`. This is a general-purpose usage that
/// should allow for copying to and from the texture, sampling the texture and rendering to the
/// texture. Specifying only the texture usage required may result in better performance. It
/// may be necessary to manually specify the the usage if `STORAGE` is required.
pub fn builder_from_image_view<T>(image: &T) -> wgpu::TextureBuilder
where
    T: image::GenericImageView,
    T::Pixel: Pixel,
{
    let (width, height) = image.dimensions();
    let format = <T::Pixel as Pixel>::TEXTURE_FORMAT;
    wgpu::TextureBuilder::new()
        .size([width, height])
        .format(format)
        .usage(wgpu::TextureBuilder::default_image_texture_usage())
}

/// Load a texture directly from a dynamic image.
///
/// If the image is already in a format supported by wgpu, no conversions are performed and the
/// image is loaded directly as-is with a texture format that matches the original image color
/// type.
///
/// If the image is of an unsupported format, it will be converted to the closest supported format
/// before being uploaded.
pub fn load_texture_from_image(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    usage: wgpu::TextureUsage,
    image: &image::DynamicImage,
) -> wgpu::Texture {
    let cmd_encoder_desc = wgpu::CommandEncoderDescriptor {
        label: Some("nannou_texture_from_image"),
    };
    let mut encoder = device.create_command_encoder(&cmd_encoder_desc);
    let texture = encode_load_texture_from_image(device, &mut encoder, usage, image);
    queue.submit(std::iter::once(encoder.finish()));
    texture
}

/// Load a texture directly from an image buffer using the given device queue.
///
/// No format or size conversions are performed - the given buffer is loaded directly into GPU
/// memory.
///
/// Pixel type compatibility is ensured via the `Pixel` trait.
pub fn load_texture_from_image_buffer<P, Container>(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    usage: wgpu::TextureUsage,
    buffer: &image::ImageBuffer<P, Container>,
) -> wgpu::Texture
where
    P: 'static + Pixel,
    Container: std::ops::Deref<Target = [P::Subpixel]>,
{
    let cmd_encoder_desc = wgpu::CommandEncoderDescriptor {
        label: Some("nannou_texture_from_image_buffer"),
    };
    let mut encoder = device.create_command_encoder(&cmd_encoder_desc);
    let texture = encode_load_texture_from_image_buffer(device, &mut encoder, usage, buffer);
    queue.submit(std::iter::once(encoder.finish()));
    texture
}

/// Load a 3d texture directly from a sequence of image buffers.
///
/// No format or size conversions are performed - the given buffer is loaded directly into GPU
/// memory.
///
/// Pixel type compatibility is ensured via the `Pixel` trait.
///
/// Returns `None` if there are no images in the given sequence.
pub fn load_3d_texture_from_image_buffers<'a, I, P, Container>(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    usage: wgpu::TextureUsage,
    buffers: I,
) -> Option<wgpu::Texture>
where
    I: IntoIterator<Item = &'a image::ImageBuffer<P, Container>>,
    I::IntoIter: ExactSizeIterator,
    P: 'static + Pixel,
    Container: 'a + std::ops::Deref<Target = [P::Subpixel]>,
{
    let cmd_encoder_desc = wgpu::CommandEncoderDescriptor {
        label: Some("nannou_load_3d_texture_from_image_buffers"),
    };
    let mut encoder = device.create_command_encoder(&cmd_encoder_desc);
    let texture =
        encode_load_3d_texture_from_image_buffers(device, &mut encoder, usage, buffers);
    queue.submit(std::iter::once(encoder.finish()));
    texture
}

/// Encode the necessary commands to load a texture directly from a dynamic image.
///
/// If the image is already in a format supported by wgpu, no conversions are performed and the
/// image is loaded directly as-is with a texture format that matches the original image color
/// type.
///
/// If the image is of an unsupported format, it will be converted to the closest supported format
/// before being uploaded.
///
/// NOTE: The returned texture will remain empty until the given `encoder` has its command buffer
/// submitted to the given `device`'s queue.
pub fn encode_load_texture_from_image(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    usage: wgpu::TextureUsage,
    image: &image::DynamicImage,
) -> wgpu::Texture {
    use image::DynamicImage::*;
    match image {
        ImageLuma8(img) => encode_load_texture_from_image_buffer(device, encoder, usage, img),
        ImageLumaA8(img) => encode_load_texture_from_image_buffer(device, encoder, usage, img),
        ImageRgba8(img) => encode_load_texture_from_image_buffer(device, encoder, usage, img),
        ImageBgra8(img) => encode_load_texture_from_image_buffer(device, encoder, usage, img),
        ImageLuma16(img) => encode_load_texture_from_image_buffer(device, encoder, usage, img),
        ImageLumaA16(img) => encode_load_texture_from_image_buffer(device, encoder, usage, img),
        ImageRgba16(img) => encode_load_texture_from_image_buffer(device, encoder, usage, img),
        ImageRgb8(_img) => {
            let img = image.to_rgba();
            encode_load_texture_from_image_buffer(device, encoder, usage, &img)
        }
        ImageBgr8(_img) => {
            let img = image.to_bgra();
            encode_load_texture_from_image_buffer(device, encoder, usage, &img)
        }
        ImageRgb16(_img) => {
            // TODO: I think we lose some quality here - e.g. 16-bit channels down to 8-bit??.
            let img = image.to_rgba();
            encode_load_texture_from_image_buffer(device, encoder, usage, &img)
        }
    }
}

/// Encode the necessary commands to load a texture directly from an image buffer.
///
/// NOTE: The returned texture will remain empty until the given `encoder` has its command buffer
/// submitted to the given `device`'s queue.
///
/// No format or size conversions are performed - the given buffer is loaded directly into GPU
/// memory.
///
/// Pixel type compatibility is ensured via the `Pixel` trait.
pub fn encode_load_texture_from_image_buffer<P, Container>(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    usage: wgpu::TextureUsage,
    buffer: &image::ImageBuffer<P, Container>,
) -> wgpu::Texture
where
    P: 'static + Pixel,
    Container: std::ops::Deref<Target = [P::Subpixel]>,
{
    // Create the texture.
    let texture = wgpu::TextureBuilder::from_image_view(buffer)
        .usage(wgpu::TextureBuilder::REQUIRED_IMAGE_TEXTURE_USAGE | usage)
        .build(device);

    // Upload the pixel data.
    let subpixel_data: &[P::Subpixel] = std::ops::Deref::deref(buffer);
    // TODO:
    // This can theoretically be exploited by implementing `image::Primitive` for some type that
    // has padding. Instead, should make some `Subpixel` trait that we can control and is only
    // guaranteed to be implemented for safe types.
    let subpixel_bytes = unsafe { wgpu::bytes::from_slice(subpixel_data) };
    let buffer = device.create_buffer_init(&BufferInitDescriptor{ label: None, contents: subpixel_bytes, usage: wgpu::BufferUsage::COPY_SRC });

    // Submit command for copying pixel data to the texture.
    let buffer_copy_view = texture.default_buffer_copy_view(&buffer);
    let texture_copy_view = texture.default_copy_view();
    let extent = texture.extent();
    encoder.copy_buffer_to_texture(buffer_copy_view, texture_copy_view, extent);

    texture
}

/// Encode the necessary commands to load a texture array directly from a sequence of image
/// buffers.
///
/// NOTE: The returned texture will remain empty u29ntil the given `encoder` has its command buffer
/// submitted to the given `device`'s queue.
///
/// NOTE: The returned texture will not be an array! It will be a 3d texture with a depth equal
/// to the number of textures in the iterator. To get an array
///
/// No format or size conversions are performed - the given buffer is loaded directly into GPU
/// memory.
///
/// Pixel type compatibility is ensured via the `Pixel` trait.
///
/// Returns `None` if there are no images in the given sequence.
pub fn encode_load_3d_texture_from_image_buffers<'a, I, P, Container>(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    usage: wgpu::TextureUsage,
    buffers: I,
) -> Option<wgpu::Texture>
where
    I: IntoIterator<Item = &'a image::ImageBuffer<P, Container>>,
    I::IntoIter: ExactSizeIterator,
    P: 'static + Pixel,
    Container: 'a + std::ops::Deref<Target = [P::Subpixel]>,
{
    let mut buffers = buffers.into_iter();
    let array_layers = buffers.len() as u32;
    let first_buffer = buffers.next()?;

    let (width, height) = first_buffer.dimensions();

    // Build the texture ready to receive the data.
    let texture = wgpu::TextureBuilder::from_image_view(first_buffer)
        .extent(Extent3d { width, height, depth: array_layers })
        .usage(wgpu::TextureBuilder::REQUIRED_IMAGE_TEXTURE_USAGE | usage)
        .build(device);

    // Copy each buffer to the texture, one layer at a time.
    for (layer, buffer) in Some(first_buffer).into_iter().chain(buffers).enumerate() {
        // Upload the pixel data.
        let subpixel_data: &[P::Subpixel] = std::ops::Deref::deref(buffer);
        // TODO:
        // This can theoretically be exploited by implementing `image::Primitive` for some type
        // that has padding. Instead, should make some `Subpixel` trait that we can control and is
        // only guaranteed to be implemented for safe types.
        let subpixel_bytes = unsafe { wgpu::bytes::from_slice(subpixel_data) };
        let buffer = device.create_buffer_init(&BufferInitDescriptor { label: None, contents: subpixel_bytes, usage: wgpu::BufferUsage::COPY_SRC });

        // Submit command for copying pixel data to the texture.
        let buffer_copy_view = texture.default_buffer_copy_view(&buffer);
        let mut texture_copy_view = texture.default_copy_view();

        // TODO(jhg): verify this works correctly
        texture_copy_view.origin = wgpu::Origin3d { x: 0, y: 0, z: layer as u32 };
        let mut extent = texture.extent();
        extent.depth = 1;
        encoder.copy_buffer_to_texture(buffer_copy_view, texture_copy_view, extent);
    }

    Some(texture)
}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn row_alignment_subsumes_buffer_alignment() {
        assert_eq!(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT % wgpu::COPY_BUFFER_ALIGNMENT, 0, "BufferImage allocation logic is broken!");
    }
}