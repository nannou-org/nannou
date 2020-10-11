use crate::wgpu::{self, TextureHandle, TextureViewHandle};
use std::ops::Deref;
use std::sync::Arc;

use wgpu::util::{DeviceExt, BufferInitDescriptor};
use wgpu::TextureFormat;

pub mod capturer;
pub mod image;
pub mod reshaper;

/// Types that can produce a texture view.
///
/// The primary purpose of this trait is to allow for APIs to be generic over both `Texture` and
/// `TextureView`. This is particularly useful for the `draw` API as we can avoid needing users to
/// understand the difference between the two. That said, it *is* slightly more efficient to create
/// your texture view once and re-use it, rather than have these APIs create a new one from your
/// texture each time they are invoked.
pub trait ToTextureView {
    fn to_texture_view(&self) -> TextureView;
}

/// A convenient wrapper around a handle to a texture on the GPU along with its descriptor.
///
/// A texture can be thought of as an image that resides in GPU memory (as opposed to CPU memory).
///
/// This type is a thin wrapper around the `wgpu` crate's `Texture` type, but provides access to
/// useful information like size, format, usage, etc.
#[derive(Debug)]
pub struct Texture {
    handle: Arc<TextureHandle>,
    descriptor: wgpu::TextureDescriptor<'static>,
}

/// A convenient wrapper around a handle to a texture view along with its descriptor.
///
/// A **TextureView** is, perhaps unsurprisingly, a view of some existing texture. The view might
/// be of the whole texture, but it might also be of some sub-section of the texture. When an API
/// provides
#[derive(Debug)]
pub struct TextureView {
    handle: Arc<TextureViewHandle>,
    descriptor: wgpu::TextureViewDescriptor<'static>,
    texture_extent: wgpu::Extent3d,
    texture_id: TextureId,
}

/// A unique identifier associated with a **Texture**.
///
/// If a texture is cloned, the result of a call to `id` will return the same result for
/// both the original and the clone.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct TextureId(usize);

/// A unique identifier associated with a **TextureView**.
///
/// A **TextureViewId** is derived from the hash of both the **TextureView**'s parent texture ID
/// and the contents of its **TextureViewDescriptor**. This allows the same **TextureViewId** to
/// represent two separate yet texture views of the same texture that share the exact same
/// descriptor.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct TextureViewId(u64);

/// A type aimed at simplifying the construction of a **Texture**.
///
/// The builder assumes a set of defaults describing a 128x128, non-multisampled, single-layer,
/// non-linear sRGBA-8 texture. A suite of builder methods may be used to specify the exact
/// properties desired.
#[derive(Debug)]
pub struct Builder {
    descriptor: wgpu::TextureDescriptor<'static>,
}

/// A type aimed at simplifying the construction of a **TextureView**.
///
/// The builder assumes a set of defaults that match view produced via `create_view`.
#[derive(Debug)]
pub struct ViewBuilder<'a> {
    texture: &'a wgpu::Texture,
    descriptor: wgpu::TextureViewDescriptor<'static>,
}

/// A wrapper around a `wgpu::Buffer` containing bytes of a known length.
#[derive(Debug)]
pub struct BufferBytes {
    buffer: wgpu::Buffer,
    len_bytes: wgpu::BufferAddress,
}

impl Texture {
    /// The inner descriptor from which this **Texture** was constructed.
    pub fn descriptor(&self) -> &wgpu::TextureDescriptor<'static> {
        &self.descriptor
    }

    /// The inner descriptor from which this **Texture** was constructed.
    ///
    /// TODO: This method should be removed upon updating to wgpu 0.5 as the new version will
    /// include an implementation of `Clone` for `TextureDescriptor`.
    pub fn descriptor_cloned(&self) -> wgpu::TextureDescriptor<'static> {
        wgpu::TextureDescriptor {
            label: Some("nannou"),
            size: self.extent(),
            mip_level_count: self.mip_level_count(),
            sample_count: self.sample_count(),
            dimension: self.dimension(),
            format: self.format(),
            usage: self.usage(),
        }
    }

    /// Consume the **Texture** and produce the inner **Arc<TextureHandle>**.
    pub fn into_inner(self) -> Arc<TextureHandle> {
        self.into()
    }

    /// A reference to the inner **TextureHandle**.
    pub fn inner(&self) -> &Arc<TextureHandle> {
        &self.handle
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

    /// The format of the underlying texture data.
    pub fn format(&self) -> wgpu::TextureFormat {
        self.descriptor.format
    }

    /// The set of usage bits describing the ways in which the **Texture** may be used.
    pub fn usage(&self) -> wgpu::TextureUsage {
        self.descriptor.usage
    }

    /// The size of the texture data in bytes.
    pub fn size_bytes(&self) -> usize {
        data_size_bytes(&self.descriptor)
    }

    /// The component type associated with the texture's format.
    pub fn component_type(&self) -> wgpu::TextureComponentType {
        self.format().into()
    }

    // Custom constructors.

    /// Create a **Texture** from the inner wgpu texture handle and the descriptor used to create
    /// it.
    ///
    /// This constructor should only be used in the case that you already have a texture handle and
    /// a descriptor but need a **Texture**. The preferred construction approach is to use the
    /// [**TextureBuilder**](./struct.TextureBuilder.html).
    ///
    /// The `descriptor` must be the same used to create the texture.
    pub fn from_handle_and_descriptor(
        handle: Arc<TextureHandle>,
        descriptor: wgpu::TextureDescriptor<'static>,
    ) -> Self {
        Texture { handle, descriptor }
    }

    // Custom common use methods.

    /// A unique identifier associated with this texture.
    ///
    /// This is useful for distinguishing between two **Texture**s or for producing a hashable
    /// representation.
    pub fn id(&self) -> TextureId {
        TextureId(Arc::into_raw(self.handle.clone()) as usize)
    }

    /// Begin building a **TextureView** for this **Texture**.
    ///
    /// By default, the produced **TextureViewBuilder** will build a texture view for the
    /// descriptor returned via `default_view_descriptor`.
    pub fn view(&self) -> ViewBuilder {
        ViewBuilder {
            texture: self,
            descriptor: self.default_view_descriptor(),
        }
    }

    /// A `TextureViewDimension` for a full view of the entire texture.
    ///
    /// NOTE: This will never produce the `Cube` or `CubeArray` variants. You may have to construct
    /// your own `wgpu::TextureViewDimension` via the `view` method if these are desired.
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

    /// The view descriptor describing a full view of the texture.
    pub fn default_view_descriptor(&self) -> wgpu::TextureViewDescriptor<'static> {
        let dimension = self.view_dimension();
        // TODO: Is this correct? Should we check the format?
        let aspect = wgpu::TextureAspect::All;
        wgpu::TextureViewDescriptor {
            label: Some("nannou"),
            format: self.format(),
            dimension,
            aspect,
            base_mip_level: 0,
            level_count: self.mip_level_count(),
            base_array_layer: 0,
            array_layer_count: self.array_layer_count(),
        }
    }

    /// Creates a `TextureCopyView` ready for copying to or from the entire texture.
    pub fn default_copy_view(&self) -> wgpu::TextureCopyView {
        wgpu::TextureCopyView {
            texture: &self.handle,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
        }
    }

    /// Creates a `BufferCopyView` ready for copying to or from the given buffer where the given
    /// buffer is assumed to have the same size as the entirety of this texture.
    pub fn default_buffer_copy_view<'a>(
        &self,
        buffer: &'static wgpu::Buffer,
    ) -> wgpu::BufferCopyView<'a> {
        let format_size_bytes = format_size_bytes(self.format());
        let [width, height] = self.size();
        let layout = wgpu::TextureDataLayout {
            offset: 0,
            bytes_per_row: width * format_size_bytes,
            rows_per_image: height,
        };
        wgpu::BufferCopyView { buffer, layout }
    }

    /// Encode a command for uploading the given data to the texture.
    ///
    /// The length of the data must be equal to the length returned by `texture.size_bytes()`.
    pub fn upload_data(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        data: &[u8],
    ) {
        // Ensure data has valid length.
        let texture_size_bytes = self.size_bytes();
        assert_eq!(data.len(), texture_size_bytes);

        // Upload and copy the data.
        let buffer = device.create_buffer_init(&BufferInitDescriptor{ label: None, contents: data, usage: wgpu::BufferUsage::COPY_SRC});
        let buffer_copy_view = self.default_buffer_copy_view(&buffer);
        let texture_copy_view = self.default_copy_view();
        let extent = self.extent();
        encoder.copy_buffer_to_texture(buffer_copy_view, texture_copy_view, extent);
    }

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
                label: Some("nannou_texture_to_buffer"),
                size: data_size_bytes,
                usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::MAP_READ,
                mapped_at_creation: wgpu::BufferUsage::MAP_WRITE,
            };
            let buffer = device.create_buffer(&buffer_descriptor);

            // Copy the full contents of the texture to the buffer.
            let texture_copy_view = texture.default_copy_view();
            let buffer_copy_view = texture.default_buffer_copy_view(&buffer);
            encoder.copy_texture_to_buffer(texture_copy_view, buffer_copy_view, size);

            (buffer, data_size_bytes)
        }

        // If this texture is multi-sampled, resolve it first.
        if self.sample_count() > 1 {
            let view = self.create_view(&wgpu::TextureViewDescriptor::default());
            let descriptor = self.descriptor_cloned();
            let resolved_texture = wgpu::TextureBuilder::from(descriptor)
                .sample_count(1)
                .usage(wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::COPY_SRC)
                .build(device);
            let resolved_view = resolved_texture.create_view(&wgpu::TextureViewDescriptor::default());
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
}

impl TextureView {
    pub fn descriptor(&self) -> &wgpu::TextureViewDescriptor {
        &self.descriptor
    }

    pub fn descriptor_cloned(&self) -> wgpu::TextureViewDescriptor {
        wgpu::TextureViewDescriptor {
            label: None,
            format: self.format(),
            dimension: self.dimension(),
            aspect: self.aspect(),
            base_mip_level: self.base_mip_level(),
            level_count: self.level_count(),
            base_array_layer: self.base_array_layer(),
            array_layer_count: self.array_layer_count(),
        }
    }

    pub fn format(&self) -> Option<wgpu::TextureFormat> {
        self.descriptor.format
    }

    pub fn dimension(&self) -> Option<wgpu::TextureViewDimension> {
        self.descriptor.dimension
    }

    pub fn aspect(&self) -> wgpu::TextureAspect {
        self.descriptor.aspect
    }

    pub fn base_mip_level(&self) -> u32 {
        self.descriptor.base_mip_level
    }

    pub fn level_count(&self) -> Option<u32> {
        self.descriptor.level_count
    }

    pub fn base_array_layer(&self) -> u32 {
        self.descriptor.base_array_layer
    }

    pub fn array_layer_count(&self) -> Option<u32> {
        self.descriptor.array_layer_count
    }

    pub fn component_type(&self) -> wgpu::TextureComponentType {
        self.format().into()
    }

    pub fn id(&self) -> TextureViewId {
        texture_view_id(&self.texture_id, &self.descriptor)
    }

    /// The width and height of the source texture.
    ///
    /// See the `extent` method for producing the full width, height and *depth* of the source
    /// texture.
    pub fn size(&self) -> [u32; 2] {
        [self.texture_extent.width, self.texture_extent.height]
    }

    /// The width, height and depth of the source texture.
    pub fn extent(&self) -> wgpu::Extent3d {
        self.texture_extent.clone()
    }

    /// The unique identifier associated with the texture that this view is derived from.
    pub fn texture_id(&self) -> TextureId {
        self.texture_id
    }

    /// Access to the inner texture view handle.
    pub fn inner(&self) -> &Arc<wgpu::TextureViewHandle> {
        &self.handle
    }

    /// Consume the **TextureView** and produce the inner **Arc<TextureViewHandle>**.
    pub fn into_inner(self) -> Arc<wgpu::TextureViewHandle> {
        self.handle
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
    pub const DEFAULT_DESCRIPTOR: wgpu::TextureDescriptor<'static> = wgpu::TextureDescriptor {
        label: Some("nannou_texture_descriptor"),
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
        let handle = Arc::new(device.create_texture(&self.descriptor));
        let descriptor = self.into();
        Texture { handle, descriptor }
    }

    /// Consumes the builder and returns the resulting `wgpu::TextureDescriptor`.
    pub fn into_descriptor(self) -> wgpu::TextureDescriptor<'static> {
        self.into()
    }
}

impl<'a> ViewBuilder<'a> {
    pub fn format(mut self, format: Option<wgpu::TextureFormat>) -> Self {
        self.descriptor.format = format;
        self
    }

    pub fn dimension(mut self, dimension: wgpu::TextureViewDimension) -> Self {
        self.descriptor.dimension = dimension;
        self
    }

    pub fn aspect(mut self, aspect: wgpu::TextureAspect) -> Self {
        self.descriptor.aspect = aspect;
        self
    }

    pub fn level_count(mut self, level_count: Option<u32>) -> Self {
        self.descriptor.level_count = level_count;
        self
    }

    pub fn base_array_layer(mut self, base_array_layer: u32) -> Self {
        self.descriptor.base_array_layer = base_array_layer;
        self
    }

    pub fn array_layer_count(mut self, array_layer_count: Option<u32>) -> Self {
        self.descriptor.array_layer_count = array_layer_count;
        self
    }

    /// Short-hand for specifying a **TextureView** for a single given base array layer.
    ///
    /// In other words, this is short-hand for the following:
    ///
    /// ```ignore
    /// builder
    ///     .base_array_layer(layer)
    ///     .array_layer_count(1)
    /// ```
    pub fn layer(self, layer: u32) -> Self {
        self.base_array_layer(layer).array_layer_count(1)
    }

    pub fn build(self) -> TextureView {
        TextureView {
            handle: Arc::new(self.texture.inner().create_view(&self.descriptor)),
            descriptor: self.descriptor,
            texture_id: self.texture.id(),
            texture_extent: self.texture.extent(),
        }
    }

    /// Consumes the texture view builder and returns the resulting `wgpu::TextureViewDescriptor`.
    pub fn into_descriptor(self) -> wgpu::TextureViewDescriptor<'static> {
        self.into()
    }
}

impl BufferBytes {
    /// Asynchronously maps the buffer of bytes to host memory and, once mapped, calls the given
    /// user callback with the data as a slice of bytes.
    ///
    /// Note: The given callback will not be called until the memory is mapped and the device is
    /// polled. You should not rely on the callback being called immediately.
    pub async fn read(&self) -> Result<(), wgpu::BufferAsyncError> {
        // TODO(jhg): fix
        panic!()
        //self.buffer.map_read(0, self.len_bytes).await
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

impl<'a, T> ToTextureView for &'a T
where
    T: ToTextureView,
{
    fn to_texture_view(&self) -> TextureView {
        (**self).to_texture_view()
    }
}

impl<'a, T> ToTextureView for &'a mut T
where
    T: ToTextureView,
{
    fn to_texture_view(&self) -> TextureView {
        (**self).to_texture_view()
    }
}

impl ToTextureView for TextureView {
    fn to_texture_view(&self) -> TextureView {
        self.clone()
    }
}

impl ToTextureView for Texture {
    fn to_texture_view(&self) -> TextureView {
        self.view().build()
    }
}

impl Clone for TextureView {
    fn clone(&self) -> Self {
        TextureView {
            handle: self.handle.clone(),
            descriptor: self.descriptor_cloned(),
            texture_id: self.texture_id(),
            texture_extent: self.extent(),
        }
    }
}

impl Clone for Texture {
    fn clone(&self) -> Self {
        let handle = self.handle.clone();
        let descriptor = self.descriptor_cloned();
        Self { handle, descriptor }
    }
}

impl Deref for Texture {
    type Target = TextureHandle;
    fn deref(&self) -> &Self::Target {
        &*self.handle
    }
}

impl Deref for TextureView {
    type Target = TextureViewHandle;
    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl Into<Arc<TextureHandle>> for Texture {
    fn into(self) -> Arc<TextureHandle> {
        self.handle
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            descriptor: Self::DEFAULT_DESCRIPTOR,
        }
    }
}

impl From<wgpu::TextureDescriptor<'static>> for Builder {
    fn from(descriptor: wgpu::TextureDescriptor<'static>) -> Self {
        Self { descriptor }
    }
}

impl Into<wgpu::TextureDescriptor<'static>> for Builder {
    fn into(self) -> wgpu::TextureDescriptor<'static> {
        self.descriptor
    }
}

impl<'a> Into<wgpu::TextureViewDescriptor<'static>> for ViewBuilder<'a> {
    fn into(self) -> wgpu::TextureViewDescriptor<'static> {
        self.descriptor
    }
}

/// Create a texture ID by hashing the source texture ID along with the contents of the descriptor.
fn texture_view_id(texture_id: &TextureId, desc: &wgpu::TextureViewDescriptor<'static>) -> TextureViewId {
    use std::hash::{Hash, Hasher};
    let mut s = std::collections::hash_map::DefaultHasher::new();

    // Hash source texture ID.
    texture_id.hash(&mut s);

    // Hash descriptor contents.
    desc.format.hash(&mut s);
    desc.dimension.hash(&mut s);
    desc.aspect.hash(&mut s);
    desc.base_mip_level.hash(&mut s);
    desc.level_count.hash(&mut s);
    desc.base_array_layer.hash(&mut s);
    desc.array_layer_count.hash(&mut s);

    TextureViewId(s.finish())
}

/// The size of the texture data in bytes as described by the given descriptor.
pub fn data_size_bytes(desc: &wgpu::TextureDescriptor) -> usize {
    desc.size.width as usize
        * desc.size.height as usize
        * desc.size.depth as usize
        * desc.array_layer_count as usize
        * format_size_bytes(desc.format) as usize
}

/// Return the size of the given texture format in bytes.
pub fn format_size_bytes(format: wgpu::TextureFormat) -> u32 {
    use crate::wgpu::TextureFormat::*;
    match format {
        R8Unorm | R8Snorm | R8Uint | R8Sint => 1,

        R16Uint | R16Sint | R16Float | Rg8Unorm | Rg8Snorm | Rg8Uint | Rg8Sint => 2,

        R32Uint | R32Sint | R32Float | Rg16Uint | Rg16Sint | Rg16Float | Rgba8Unorm
        | Rgba8UnormSrgb | Rgba8Snorm | Rgba8Uint | Rgba8Sint | Bgra8Unorm | Bgra8UnormSrgb
        | Rgb10a2Unorm | Rg11b10Float => 4,

        Rg32Uint | Rg32Sint | Rg32Float | Rgba16Uint | Rgba16Sint | Rgba16Float | Rgba32Uint
        | Rgba32Sint | Rgba32Float => 8,

        Depth32Float | Depth24Plus | Depth24PlusStencil8 => 4,

        _ => unimplemented!()
    }
}

/// Returns `true` if the given `wgpu::Extent3d`s are equal.
pub fn extent_3d_eq(a: &wgpu::Extent3d, b: &wgpu::Extent3d) -> bool {
    a.width == b.width && a.height == b.height && a.depth == b.depth
}

/// Returns `true` if the given texture descriptors are equal.
pub fn descriptor_eq(a: &wgpu::TextureDescriptor, b: &wgpu::TextureDescriptor) -> bool {
    extent_3d_eq(&a.size, &b.size)
        && a.mip_level_count == b.mip_level_count
        && a.sample_count == b.sample_count
        && a.dimension == b.dimension
        && a.format == b.format
        && a.usage == b.usage
}
