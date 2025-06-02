//! Items related to the inter-operation of the `image` crate (images on disk and in RAM) and
//! textures from the wgpu crate (images in GPU memory).
//!
//! This module can be enabled via the `image` feature.

use crate as wgpu;
use image::PixelWithColorType;
use std::{ops::Deref, path::Path};

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

/// A wrapper around a slice of bytes representing an image.
///
/// An `ImageReadMapping` may only be created via `RowPaddedBuffer::read()`.
pub struct ImageReadMapping<'buffer> {
    buffer: &'buffer wgpu::RowPaddedBuffer,
    view: wgpu::BufferView<'buffer>,
}

/// Workaround for the fact that `image::SubImage` requires a `Deref` impl on the wrapped image.
pub struct ImageHolder<'b, P: Pixel>(image::ImageBuffer<P, &'b [P::Subpixel]>);
impl<'b, P: Pixel> Deref for ImageHolder<'b, P> {
    type Target = image::ImageBuffer<P, &'b [P::Subpixel]>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl wgpu::TextureBuilder {
    /// The minimum required texture usage when loading from an image.
    pub const REQUIRED_IMAGE_TEXTURE_USAGE: wgpu::TextureUsages = wgpu::TextureUsages::COPY_DST;

    /// Produce a texture descriptor from an image.
    ///
    /// Specifically, this supports any image type implementing `image::GenericImageView` whose
    /// `Pixel` type implements `Pixel`.
    ///
    /// By default, the produced builder will have the `wgpu::TextureUsages` returned by
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
    pub fn default_image_texture_usage() -> wgpu::TextureUsages {
        wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::RENDER_ATTACHMENT
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
    /// `RENDER_ATTACHMENT` usages enabled. If you wish to specify the usage yourself, see the
    /// `load_from_path` constructor.
    ///
    /// If the `&App` is passed as the `src`, the window returned via `app.main_window()` will be
    /// used as the source of the device and queue.
    pub fn from_path<T, P>(src: T, path: P) -> image::ImageResult<Self>
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
    /// `RENDER_ATTACHMENT` usages enabled. If you wish to specify the usage yourself, see the
    /// `load_from_path` constructor.
    ///
    /// If the `&App` is passed as the `src`, the window returned via `app.main_window()` will be
    /// used as the source of the device and queue.
    ///
    /// The `DeviceQueuePairSource` can be either the `App`, a `Window`, a `DeviceQueuePair` or a
    /// tuple `(&Device, &Queue)`.
    pub fn from_image<T>(src: T, image: &image::DynamicImage) -> Self
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
        usage: wgpu::TextureUsages,
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
        usage: wgpu::TextureUsages,
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
        usage: wgpu::TextureUsages,
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
    pub fn load_array_from_image_buffers<'a, I, P, Container>(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        usage: wgpu::TextureUsages,
        buffers: I,
    ) -> Option<Self>
    where
        I: IntoIterator<Item = &'a image::ImageBuffer<P, Container>>,
        I::IntoIter: ExactSizeIterator,
        P: 'static + Pixel,
        Container: 'a + std::ops::Deref<Target = [P::Subpixel]>,
    {
        load_texture_array_from_image_buffers(device, queue, usage, buffers)
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
        usage: wgpu::TextureUsages,
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
        usage: wgpu::TextureUsages,
        buffer: &image::ImageBuffer<P, Container>,
    ) -> Self
    where
        P: 'static + Pixel + PixelWithColorType,
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
        usage: wgpu::TextureUsages,
        buffers: I,
    ) -> Option<Self>
    where
        I: IntoIterator<Item = &'a image::ImageBuffer<P, Container>>,
        I::IntoIter: ExactSizeIterator,
        P: 'static + Pixel + PixelWithColorType,
        Container: 'a + std::ops::Deref<Target = [P::Subpixel]>,
    {
        encode_load_texture_array_from_image_buffers(device, encoder, usage, buffers)
    }
}

impl wgpu::RowPaddedBuffer {
    /// Initialize from an image buffer (i.e. an image on CPU).
    pub fn from_image_buffer<P, Container>(
        device: &wgpu::Device,
        image_buffer: &image::ImageBuffer<P, Container>,
    ) -> Self
    where
        P: 'static + Pixel + PixelWithColorType,
        Container: std::ops::Deref<Target = [P::Subpixel]>,
    {
        let result = Self::new(
            device,
            image_buffer.width() * P::COLOR_TYPE.bits_per_pixel() as u32 * 8,
            image_buffer.height(),
            wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC,
        );
        // TODO:
        // This can theoretically be exploited by implementing `image::Primitive` for some type
        // that has padding. Instead, should make some `Subpixel` trait that we can control and is
        // only guaranteed to be implemented for safe types.
        result.write(unsafe { wgpu::bytes::from_slice(&*image_buffer) });
        result
    }

    /// Asynchronously maps the buffer of bytes from GPU to host memory.
    ///
    /// Note: The returned future will not be ready until the memory is mapped and the device is
    /// polled. You should *not* rely on the being ready immediately.
    pub async fn read<'b>(&'b self) -> Result<ImageReadMapping<'b>, wgpu::BufferAsyncError> {
        let slice = self.buffer.slice(..);
        let (tx, rx) = futures::channel::oneshot::channel();

        slice.map_async(wgpu::MapMode::Read, |res| {
            tx.send(res).expect("Failed to send map_async result");
        });

        rx.await.expect("Failed to receive map_async result")?;

        Ok(wgpu::ImageReadMapping {
            buffer: self,
            // fun exercise:
            // read the signature of wgpu::BufferSlice::get_mapped_range()
            // and try to figure out why we don't need another lifetime in ImageReadMapping :)
            view: slice.get_mapped_range(),
        })
    }
}

impl<'buffer> ImageReadMapping<'buffer> {
    /// View as an image::SubImage.
    ///
    /// Unsafe: `P::TEXTURE_FORMAT` MUST match the texture format / image type used to create the
    /// wrapped RowPaddedBuffer! If this is not the case, may result in undefined behavior!
    pub unsafe fn as_image<P>(&self) -> image::SubImage<ImageHolder<P>>
    where
        P: Pixel + PixelWithColorType + 'static,
    {
        let subpixel_size = std::mem::size_of::<P::Subpixel>() as u32;
        let pixel_size = subpixel_size * P::CHANNEL_COUNT as u32;
        assert_eq!(pixel_size, P::COLOR_TYPE.bits_per_pixel() as u32 * 8);

        assert_eq!(
            self.buffer.padded_width() % pixel_size,
            0,
            "buffer padded width not an even multiple of primitive size"
        );
        assert_eq!(
            self.buffer.width() % pixel_size,
            0,
            "buffer row width not an even multiple of primitive size"
        );

        let width_pixels = self.buffer.width() / pixel_size;
        let padded_width_pixels = self.buffer.padded_width() / pixel_size;

        // ways this cast could go wrong:
        // - buffer is the wrong size: checked in to_slice, panics
        // - buffer is the wrong alignment: checked in to_slice, panics
        // - buffer rows are the wrong size: checked above, panics
        // - buffer has not been initialized / has invalid data for primitive type:
        //   very possible. That's why this function is `unsafe`.
        let container = unsafe {
            wgpu::bytes::to_slice::<P::Subpixel>(&self.view[..])
        };

        let full_image =
            image::ImageBuffer::from_raw(padded_width_pixels, self.buffer.height(), container)
                .expect("nannou internal error: incorrect buffer size");
        image::SubImage::new(
            ImageHolder(full_image),
            0,
            0,
            width_pixels,
            self.buffer.height(),
        )
    }
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

impl<'a, 'b, T> wgpu::WithDeviceQueuePair for &'a std::cell::Ref<'b, T>
where
    &'a T: wgpu::WithDeviceQueuePair,
{
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
        _ => return None,
    };
    Some(format)
}

/// Produce a texture descriptor from any type implementing `image::GenericImageView` whose `Pixel`
/// type implements `Pixel`.
///
/// By default, the produced builder will have the `wgpu::TextureUsages` returned by
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
/// This uses the `Queue::write_texture` method, meaning that the texture is not immediately
/// written. Rather, the write is enqueued internally and scheduled to happen at the start of the
/// next call to `Queue::submit`.
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
    usage: wgpu::TextureUsages,
    image: &image::DynamicImage,
) -> wgpu::Texture {
    use image::DynamicImage::*;
    match image {
        ImageLuma8(img) => load_texture_from_image_buffer(device, queue, usage, img),
        ImageLumaA8(img) => load_texture_from_image_buffer(device, queue, usage, img),
        ImageRgba8(img) => load_texture_from_image_buffer(device, queue, usage, img),
        ImageLuma16(img) => load_texture_from_image_buffer(device, queue, usage, img),
        ImageLumaA16(img) => load_texture_from_image_buffer(device, queue, usage, img),
        ImageRgba16(img) => load_texture_from_image_buffer(device, queue, usage, img),
        ImageRgb8(_img) => {
            let img = image.to_rgba8();
            load_texture_from_image_buffer(device, queue, usage, &img)
        }
        ImageRgb16(_img) => {
            let img = image.to_rgba16();
            load_texture_from_image_buffer(device, queue, usage, &img)
        }
        _ => panic!("Unsupported image format: {:?}", image.color()),
    }
}

/// Load a texture directly from an image buffer using the given device queue.
///
/// This uses the `Queue::write_texture` method, meaning that the texture is not immediately
/// written. Rather, the write is enqueued internally and scheduled to happen at the start of the
/// next call to `Queue::submit`.
///
/// No format or size conversions are performed - the given buffer is loaded directly into GPU
/// memory.
///
/// Pixel type compatibility is ensured via the `Pixel` trait.
pub fn load_texture_from_image_buffer<P, Container>(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    usage: wgpu::TextureUsages,
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

    // Describe the layout of the data.
    let extent = texture.extent();
    let format = texture.format();
    let block_size = format
        .block_copy_size(None)
        .expect("Expected the format to have a block size");
    let bytes_per_row = extent.width * block_size as u32;
    let image_data_layout = wgpu::TexelCopyBufferLayout {
        offset: 0,
        bytes_per_row: Some(bytes_per_row),
        rows_per_image: None,
    };

    // Copy into the entire texture.
    let image_copy_texture = texture.as_image_copy();

    // TODO:
    // This can theoretically be exploited by implementing our `image::Pixel` trait for some type
    // that has padding. Perhaps it should be an unsafe trait? Should investigate how to achieve
    // this in a safer manner.
    let data = unsafe { wgpu::bytes::from_slice(&*buffer) };

    queue.write_texture(image_copy_texture, data, image_data_layout, extent);
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
pub fn load_texture_array_from_image_buffers<'a, I, P, Container>(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    usage: wgpu::TextureUsages,
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

    // Build the texture ready to receive the data.
    let (width, height) = first_buffer.dimensions();
    let extent = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: array_layers,
    };
    let texture = wgpu::TextureBuilder::from_image_view(first_buffer)
        .extent(extent)
        .dimension(wgpu::TextureDimension::D2) // force an array
        .usage(wgpu::TextureBuilder::REQUIRED_IMAGE_TEXTURE_USAGE | usage)
        .build(device);

    // Describe the layout of the data.
    let format = texture.format();
    let block_size = format
        .block_copy_size(None)
        .expect("Expected the format to have a block size");
    let bytes_per_row = extent.width * block_size as u32;
    let image_data_layout = wgpu::TexelCopyBufferLayout {
        offset: 0,
        bytes_per_row: Some(bytes_per_row),
        rows_per_image: Some(height),
    };

    // Collect the data into a single slice.
    //
    // NOTE: Previously we used `encode_load_texture_array_from_image_buffers` which avoids
    // collecting the image data into a single slice. However, the `wgpu::Texture::from_*`
    // constructors have been changed to avoid submitting an extra command buffer in favour
    // of using `Queue::write_texture` which schedules the write for the next call to
    // `Queue::submit`. This is to avoid an Intel driver bug where submitting more than one command
    // buffer per frame appears to be causing issues:
    // https://github.com/gfx-rs/wgpu/issues/1672#issuecomment-917510810
    //
    // While this likely means consuming more RAM, it also likely results in slightly better
    // performance due to reducing the number of command buffers submitted.
    //
    // Users can still use `encode_load_texture_array_from_image_buffers` directly if they wish.
    let capacity = bytes_per_row as usize * height as usize * array_layers as usize;
    let mut data: Vec<u8> = Vec::with_capacity(capacity);
    for buffer in Some(first_buffer).into_iter().chain(buffers) {
        let layer_data = unsafe { wgpu::bytes::from_slice(&*buffer) };
        data.extend_from_slice(layer_data);
    }

    // Copy into the entire texture.
    let image_copy_texture = texture.as_image_copy();

    queue.write_texture(image_copy_texture, &data, image_data_layout, extent);

    Some(texture)
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
    usage: wgpu::TextureUsages,
    image: &image::DynamicImage,
) -> wgpu::Texture {
    use image::DynamicImage::*;
    match image {
        ImageLuma8(img) => encode_load_texture_from_image_buffer(device, encoder, usage, img),
        ImageLumaA8(img) => encode_load_texture_from_image_buffer(device, encoder, usage, img),
        ImageRgba8(img) => encode_load_texture_from_image_buffer(device, encoder, usage, img),
        ImageLuma16(img) => encode_load_texture_from_image_buffer(device, encoder, usage, img),
        ImageLumaA16(img) => encode_load_texture_from_image_buffer(device, encoder, usage, img),
        ImageRgba16(img) => encode_load_texture_from_image_buffer(device, encoder, usage, img),
        ImageRgb8(_img) => {
            let img = image.to_rgba8();
            encode_load_texture_from_image_buffer(device, encoder, usage, &img)
        }
        ImageRgb16(_img) => {
            let img = image.to_rgba16();
            encode_load_texture_from_image_buffer(device, encoder, usage, &img)
        }
        _ => panic!("Unsupported image format: {:?}", image.color()),
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
    usage: wgpu::TextureUsages,
    buffer: &image::ImageBuffer<P, Container>,
) -> wgpu::Texture
where
    P: 'static + Pixel + PixelWithColorType,
    Container: std::ops::Deref<Target = [P::Subpixel]>,
{
    // Create the texture.
    let texture = wgpu::TextureBuilder::from_image_view(buffer)
        .usage(wgpu::TextureBuilder::REQUIRED_IMAGE_TEXTURE_USAGE | usage)
        .build(device);

    let buffer_image = wgpu::RowPaddedBuffer::from_image_buffer(device, buffer);
    buffer_image.encode_copy_into(encoder, &texture);

    texture
}

/// Encode the necessary commands to load a texture array directly from a sequence of image
/// buffers.
///
/// NOTE: The returned texture will remain empty until the given `encoder` has its command buffer
/// submitted to the given `device`'s queue.
///
/// No format or size conversions are performed - the given buffer is loaded directly into GPU
/// memory.
///
/// Pixel type compatibility is ensured via the `Pixel` trait.
///
/// Returns `None` if there are no images in the given sequence.
pub fn encode_load_texture_array_from_image_buffers<'a, I, P, Container>(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    usage: wgpu::TextureUsages,
    buffers: I,
) -> Option<wgpu::Texture>
where
    I: IntoIterator<Item = &'a image::ImageBuffer<P, Container>>,
    I::IntoIter: ExactSizeIterator,
    P: 'static + Pixel + PixelWithColorType,
    Container: 'a + std::ops::Deref<Target = [P::Subpixel]>,
{
    let mut buffers = buffers.into_iter();
    let array_layers = buffers.len() as u32;
    let first_buffer = buffers.next()?;

    let (width, height) = first_buffer.dimensions();

    // Build the texture ready to receive the data.
    let texture = wgpu::TextureBuilder::from_image_view(first_buffer)
        .extent(wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: array_layers,
        })
        .dimension(wgpu::TextureDimension::D2) // force an array
        .usage(wgpu::TextureBuilder::REQUIRED_IMAGE_TEXTURE_USAGE | usage)
        .build(device);

    // Copy each buffer to the texture, one layer at a time.
    for (layer, buffer) in Some(first_buffer).into_iter().chain(buffers).enumerate() {
        // Upload the pixel data.
        let buffer = wgpu::RowPaddedBuffer::from_image_buffer(device, &buffer);
        buffer.encode_copy_into_at(encoder, &texture, layer as u32);
    }

    Some(texture)
}
