use crate::wgpu::{self, TextureHandle};
use std::ops::Deref;

pub mod capturer;
pub mod image;
pub mod reshaper;

/// A convenient wrapper around a handle to a texture on the GPU along with its descriptor.
///
/// A texture can be thought of as an image that resides in GPU memory (as opposed to CPU memory).
///
/// This type is a thin wrapper around the `wgpu` crate's `Texture` type, but provides access to
/// useful information like size, format, usage, etc.
#[derive(Debug)]
pub struct Texture {
    texture: TextureHandle,
    descriptor: wgpu::TextureDescriptor,
}

/// A type aimed at simplifying the construction of a **Texture**.
///
/// The builder assumes a set of defaults describing a 128x128, non-multisampled, single-layer,
/// non-linear sRGBA-8 texture. A suite of builder methods may be used to specify the exact
/// properties desired.
#[derive(Debug)]
pub struct Builder {
    descriptor: wgpu::TextureDescriptor,
}

/// A wrapper around a `wgpu::Buffer` containing bytes of a known length.
#[derive(Debug)]
pub struct BufferBytes {
    buffer: wgpu::Buffer,
    len_bytes: wgpu::BufferAddress,
}

impl Texture {
    // `wgpu::TextureDescriptor` accessor methods.

    /// The inner descriptor from which this **Texture** was constructed.
    pub fn descriptor(&self) -> &wgpu::TextureDescriptor {
        &self.descriptor
    }

    /// The inner descriptor from which this **Texture** was constructed.
    pub fn descriptor_cloned(&self) -> wgpu::TextureDescriptor {
        wgpu::TextureDescriptor {
            size: self.extent(),
            array_layer_count: self.array_layer_count(),
            mip_level_count: self.mip_level_count(),
            sample_count: self.sample_count(),
            dimension: self.dimension(),
            format: self.format(),
            usage: self.usage(),
        }
    }

    /// Consume the **Texture** and produce the inner **TextureHandle**.
    pub fn into_inner(self) -> TextureHandle {
        self.into()
    }

    /// A reference to the inner **TextureHandle**.
    pub fn inner(&self) -> &TextureHandle {
        &self.texture
    }

    /// The width and height of the texture.
    ///
    /// See the `extent` method for producing the full width, height and *depth* of the texture.
    pub fn size(&self) -> [u32; 2] {
        [self.descriptor.size.width, self.descriptor.size.height]
    }

    /// The width, height and depth of the texture.
    pub fn extent(&self) -> wgpu::Extent3d {
        self.descriptor.size
    }

    pub fn array_layer_count(&self) -> u32 {
        self.descriptor.array_layer_count
    }

    pub fn mip_level_count(&self) -> u32 {
        self.descriptor.mip_level_count
    }

    pub fn sample_count(&self) -> u32 {
        self.descriptor.sample_count
    }

    /// Describes whether the texture is of 1, 2 or 3 dimensions.
    pub fn dimension(&self) -> wgpu::TextureDimension {
        self.descriptor.dimension
    }

    /// A `TextureViewDimension` for a full view of the entire texture.
    ///
    /// NOTE: This will never produce the `Cube` or `CubeArray` variants. You may have to construct
    /// your own `wgpu::TextureViewDimension` if these are desired.
    pub fn view_dimension(&self) -> wgpu::TextureViewDimension {
        match self.dimension() {
            wgpu::TextureDimension::D1 => wgpu::TextureViewDimension::D1,
            wgpu::TextureDimension::D2 => match self.array_layer_count() {
                1 => wgpu::TextureViewDimension::D2,
                _ => wgpu::TextureViewDimension::D2Array,
            },
            wgpu::TextureDimension::D3 => wgpu::TextureViewDimension::D3,
        }
    }

    /// The format of the underlying texture data.
    pub fn format(&self) -> wgpu::TextureFormat {
        self.descriptor.format
    }

    /// The set of usage bits describing the ways in which the **Texture** may be used.
    pub fn usage(&self) -> wgpu::TextureUsage {
        self.descriptor.usage
    }

    // Custom constructors.

    /// Create a **Texture** from the inner wgpu texture handle and the descriptor used to create
    /// it.
    ///
    /// This constructor should only be used in the case that you already have a texture handle and
    /// a descriptor but need a **Texture**. The preferred construction approach is to use the
    /// [**TextureBuilder**](./struct.Builder.html).
    ///
    /// The `descriptor` must be the same used to create the texture.
    pub fn from_handle_and_descriptor(
        handle: TextureHandle,
        descriptor: wgpu::TextureDescriptor,
    ) -> Self {
        Texture {
            texture: handle,
            descriptor,
        }
    }

    // Custom common use methods.

    /// Write the contents of the texture into a new buffer.
    ///
    /// Commands will be added to the given encoder to copy the entire contents of the texture into
    /// the buffer.
    ///
    /// The buffer is returned alongside its size in bytes.
    ///
    /// If the texture has a sample count greater than one, it will first be resolved to a
    /// non-multisampled texture before being copied to the buffer.
    /// `copy_texture_to_buffer` command has been performed by the GPU.
    ///
    /// NOTE: `map_read_async` should not be called on the returned buffer until the encoded commands have
    /// been submitted to the device queue.
    pub fn to_buffer(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
    ) -> (wgpu::Buffer, wgpu::BufferAddress) {
        // Create the buffer and encode the copy.
        fn texture_to_buffer(
            texture: &wgpu::Texture,
            device: &wgpu::Device,
            encoder: &mut wgpu::CommandEncoder,
        ) -> (wgpu::Buffer, wgpu::BufferAddress) {
            // Create buffer that will be mapped for reading.
            let size = texture.extent();
            let format = texture.format();
            let format_size_bytes = format_size_bytes(format) as u64;
            let layer_len_pixels = size.width as u64 * size.height as u64 * size.depth as u64;
            let layer_size_bytes = layer_len_pixels * format_size_bytes;
            let data_size_bytes = layer_size_bytes * texture.array_layer_count() as u64;
            let buffer_descriptor = wgpu::BufferDescriptor {
                size: data_size_bytes,
                usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::MAP_READ,
            };
            let buffer = device.create_buffer(&buffer_descriptor);

            // Copy the full contents of the texture to the buffer.
            let texture_copy_view = texture.create_default_copy_view();
            let buffer_copy_view = texture.create_default_buffer_copy_view(&buffer);
            encoder.copy_texture_to_buffer(texture_copy_view, buffer_copy_view, size);

            (buffer, data_size_bytes)
        }

        // If this texture is multi-sampled, resolve it first.
        if self.sample_count() > 1 {
            let view = self.create_default_view();
            let descriptor = self.descriptor_cloned();
            let resolved_texture = wgpu::TextureBuilder::from(descriptor)
                .sample_count(1)
                .usage(wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::COPY_SRC)
                .build(device);
            let resolved_view = resolved_texture.create_default_view();
            wgpu::resolve_texture(&view, &resolved_view, encoder);
            texture_to_buffer(&resolved_texture, device, encoder)
        } else {
            texture_to_buffer(self, device, encoder)
        }
    }

    /// Encode the necessary commands to read the contents of the texture into memory.
    ///
    /// The entire contents of the texture will be made available as a single slice of bytes.
    ///
    /// This method uses `to_buffer` internally, exposing a simplified API for reading the produced
    /// buffer as a slice of bytes.
    ///
    /// If the texture has a sample count greater than one, it will first be resolved to a
    /// non-multisampled texture before being copied to the buffer.
    ///
    /// NOTE: `read` should not be called on the returned buffer until the encoded commands have
    /// been submitted to the device queue.
    pub fn to_buffer_bytes(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
    ) -> BufferBytes {
        let (buffer, len_bytes) = self.to_buffer(device, encoder);
        BufferBytes { buffer, len_bytes }
    }

    /// The view descriptor describing a full view of the texture.
    pub fn create_default_view_descriptor(&self) -> wgpu::TextureViewDescriptor {
        let dimension = self.view_dimension();
        // TODO: Is this correct? Should we check the format?
        let aspect = wgpu::TextureAspect::All;
        wgpu::TextureViewDescriptor {
            format: self.format(),
            dimension,
            aspect,
            base_mip_level: 0,
            level_count: self.mip_level_count(),
            base_array_layer: 0,
            array_layer_count: self.array_layer_count(),
        }
    }

    /// The view descriptor for a single layer of the texture.
    pub fn create_layer_view_descriptor(&self, layer: u32) -> wgpu::TextureViewDescriptor {
        let mut desc = self.create_default_view_descriptor();
        desc.dimension = wgpu::TextureViewDimension::D2;
        desc.base_array_layer = layer;
        desc.array_layer_count = 1;
        desc
    }

    /// Creates a `TextureCopyView` ready for copying to or from the entire texture.
    pub fn create_default_copy_view(&self) -> wgpu::TextureCopyView {
        wgpu::TextureCopyView {
            texture: &self.texture,
            mip_level: 0,
            array_layer: 0,
            origin: wgpu::Origin3d::ZERO,
        }
    }

    /// Creates a `BufferCopyView` ready for copying to or from the given buffer where the given
    /// buffer is assumed to have the same size as the entirety of this texture.
    pub fn create_default_buffer_copy_view<'a>(
        &self,
        buffer: &'a wgpu::Buffer,
    ) -> wgpu::BufferCopyView<'a> {
        let format_size_bytes = format_size_bytes(self.format());
        let [width, height] = self.size();
        wgpu::BufferCopyView {
            buffer,
            offset: 0,
            row_pitch: width * format_size_bytes,
            image_height: height,
        }
    }
}

impl Builder {
    pub const DEFAULT_SIDE: u32 = 128;
    pub const DEFAULT_DEPTH: u32 = 1;
    pub const DEFAULT_SIZE: wgpu::Extent3d = wgpu::Extent3d {
        width: Self::DEFAULT_SIDE,
        height: Self::DEFAULT_SIDE,
        depth: Self::DEFAULT_DEPTH,
    };
    pub const DEFAULT_ARRAY_LAYER_COUNT: u32 = 1;
    pub const DEFAULT_MIP_LEVEL_COUNT: u32 = 1;
    pub const DEFAULT_SAMPLE_COUNT: u32 = 1;
    pub const DEFAULT_DIMENSION: wgpu::TextureDimension = wgpu::TextureDimension::D2;
    pub const DEFAULT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
    pub const DEFAULT_USAGE: wgpu::TextureUsage = wgpu::TextureUsage::ORDERED;
    pub const DEFAULT_DESCRIPTOR: wgpu::TextureDescriptor = wgpu::TextureDescriptor {
        size: Self::DEFAULT_SIZE,
        array_layer_count: Self::DEFAULT_ARRAY_LAYER_COUNT,
        mip_level_count: Self::DEFAULT_MIP_LEVEL_COUNT,
        sample_count: Self::DEFAULT_SAMPLE_COUNT,
        dimension: Self::DEFAULT_DIMENSION,
        format: Self::DEFAULT_FORMAT,
        usage: Self::DEFAULT_USAGE,
    };

    /// Creates a new `Default` builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Specify the width and height of the texture.
    ///
    /// Note: On calls to `size`, `depth` and `extent` the `Builder` will attempt to infer the
    /// `wgpu::TextureDimension` of its inner `wgpu::TextureDescriptor` by examining its `size`
    /// field.
    pub fn size(mut self, [width, height]: [u32; 2]) -> Self {
        self.descriptor.size.width = width;
        self.descriptor.size.height = height;
        self.infer_dimension_from_size();
        self
    }

    /// Specify the depth of the texture.
    ///
    /// Note: On calls to `size`, `depth` and `extent` the `Builder` will attempt to infer the
    /// `wgpu::TextureDimension` of its inner `wgpu::TextureDescriptor` by examining its `size`
    /// field.
    pub fn depth(mut self, depth: u32) -> Self {
        self.descriptor.size.depth = depth;
        self.infer_dimension_from_size();
        self
    }

    /// Specify the width, height and depth of the texture.
    ///
    /// Note: On calls to `size`, `depth` and `extent` the `Builder` will attempt to infer the
    /// `wgpu::TextureDimension` of its inner `wgpu::TextureDescriptor` by examining its `size`
    /// field.
    pub fn extent(mut self, extent: wgpu::Extent3d) -> Self {
        self.descriptor.size = extent;
        self.infer_dimension_from_size();
        self
    }

    pub fn array_layer_count(mut self, count: u32) -> Self {
        self.descriptor.array_layer_count = count;
        self
    }

    pub fn mip_level_count(mut self, count: u32) -> Self {
        self.descriptor.mip_level_count = count;
        self
    }

    /// Specify the number of samples per pixel in the case that the texture is multisampled.
    pub fn sample_count(mut self, count: u32) -> Self {
        self.descriptor.sample_count = count;
        self
    }

    /// Specify the texture format.
    pub fn format(mut self, format: wgpu::TextureFormat) -> Self {
        self.descriptor.format = format;
        self
    }

    /// Describes to the implementation how the texture is to be used.
    ///
    /// It is important that the set of usage bits reflects the
    pub fn usage(mut self, usage: wgpu::TextureUsage) -> Self {
        self.descriptor.usage = usage;
        self
    }

    // If `depth` is greater than `1` then `D3` is assumed, otherwise if `height` is greater than
    // `1` then `D2` is assumed, otherwise `D1` is assumed.
    fn infer_dimension_from_size(&mut self) {
        if self.descriptor.size.depth > 1 {
            self.descriptor.dimension = wgpu::TextureDimension::D3;
        } else if self.descriptor.size.height > 1 {
            self.descriptor.dimension = wgpu::TextureDimension::D2;
        } else {
            self.descriptor.dimension = wgpu::TextureDimension::D1;
        }
    }

    /// Build the texture resulting from the specified parameters with the given device.
    pub fn build(self, device: &wgpu::Device) -> Texture {
        let texture = device.create_texture(&self.descriptor);
        let descriptor = self.into();
        Texture {
            texture,
            descriptor,
        }
    }

    /// Consumes the builder and returns the resulting `wgpu::TextureDescriptor`.
    pub fn into_descriptor(self) -> wgpu::TextureDescriptor {
        self.into()
    }
}

impl BufferBytes {
    /// Asynchronously maps the buffer of bytes to host memory and, once mapped, calls the given
    /// user callback with the data as a slice of bytes.
    ///
    /// Note: The given callback will not be called until the memory is mapped and the device is
    /// polled. You should not rely on the callback being called immediately.
    pub fn read<F>(&self, callback: F)
    where
        F: 'static + FnOnce(wgpu::BufferMapAsyncResult<&[u8]>),
    {
        self.buffer.map_read_async(0, self.len_bytes, callback)
    }

    /// The length of the `wgpu::Buffer` in bytes.
    pub fn len_bytes(&self) -> wgpu::BufferAddress {
        self.len_bytes
    }

    /// A reference to the inner `wgpu::Buffer`.
    pub fn inner(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    /// Consumes `self` and returns the inner `wgpu::Buffer`.
    pub fn into_inner(self) -> wgpu::Buffer {
        self.buffer
    }
}

impl Deref for Texture {
    type Target = TextureHandle;
    fn deref(&self) -> &Self::Target {
        &self.texture
    }
}

impl Into<TextureHandle> for Texture {
    fn into(self) -> TextureHandle {
        self.texture
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            descriptor: Self::DEFAULT_DESCRIPTOR,
        }
    }
}

impl From<wgpu::TextureDescriptor> for Builder {
    fn from(descriptor: wgpu::TextureDescriptor) -> Self {
        Self { descriptor }
    }
}

impl Into<wgpu::TextureDescriptor> for Builder {
    fn into(self) -> wgpu::TextureDescriptor {
        self.descriptor
    }
}

/// Return the size of the given texture format in bytes.
pub fn format_size_bytes(format: wgpu::TextureFormat) -> u32 {
    use crate::wgpu::TextureFormat::*;
    match format {
        R8Unorm | R8Snorm | R8Uint | R8Sint => 1,
        R16Unorm | R16Snorm | R16Uint | R16Sint | R16Float | Rg8Unorm | Rg8Snorm | Rg8Uint
        | Rg8Sint => 2,
        R32Uint | R32Sint | R32Float | Rg16Unorm | Rg16Snorm | Rg16Uint | Rg16Sint | Rg16Float
        | Rgba8Unorm | Rgba8UnormSrgb | Rgba8Snorm | Rgba8Uint | Rgba8Sint | Bgra8Unorm
        | Bgra8UnormSrgb | Rgb10a2Unorm | Rg11b10Float => 4,
        Rg32Uint | Rg32Sint | Rg32Float | Rgba16Unorm | Rgba16Snorm | Rgba16Uint | Rgba16Sint
        | Rgba16Float | Rgba32Uint | Rgba32Sint | Rgba32Float => 8,
        Depth32Float | Depth24Plus | Depth24PlusStencil8 => 4,
    }
}

/// Returns `true` if the given `wgpu::Extent3d`s are equal.
pub fn extent_3d_eq(a: &wgpu::Extent3d, b: &wgpu::Extent3d) -> bool {
    a.width == b.width && a.height == b.height && a.depth == b.depth
}

/// Returns `true` if the given texture descriptors are equal.
pub fn descriptor_eq(a: &wgpu::TextureDescriptor, b: &wgpu::TextureDescriptor) -> bool {
    extent_3d_eq(&a.size, &b.size)
        && a.array_layer_count == b.array_layer_count
        && a.mip_level_count == b.mip_level_count
        && a.sample_count == b.sample_count
        && a.dimension == b.dimension
        && a.format == b.format
        && a.usage == b.usage
}
