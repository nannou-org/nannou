//! Items related to the inter-operation of the `image` crate (images on disk and in RAM) and
//! textures from the wgpu crate (images in GPU memory).

use crate::wgpu;
use std::path::Path;

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
    pub fn load_array_from_image_buffers<'a, I, P, Container>(
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
        encode_load_texture_array_from_image_buffers(device, encoder, usage, buffers)
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
pub fn load_texture_array_from_image_buffers<'a, I, P, Container>(
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
        encode_load_texture_array_from_image_buffers(device, &mut encoder, usage, buffers);
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

    let buffer_image = wgpu::RowPaddedBuffer::from_image_buffer(device, buffer);
    buffer_image.encode_copy_into(encoder, &texture);

    texture
}

/// Encode the necessary commands to load a texture array directly from a sequence of image
/// buffers.
///
/// NOTE: The returned texture will remain empty u29ntil the given `encoder` has its command buffer
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
        .extent(wgpu::Extent3d {
            width,
            height,
            depth: array_layers,
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
