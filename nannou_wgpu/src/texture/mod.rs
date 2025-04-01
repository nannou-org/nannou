use crate::{self as wgpu, RowPaddedBuffer, TextureHandle, TextureViewHandle};
use std::ops::Deref;
use std::sync::Arc;

#[cfg(feature = "image")]
pub mod image;
pub mod reshaper;
pub mod row_padded_buffer;

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
    info: TextureViewInfo,
    texture_extent: wgpu::Extent3d,
    texture_id: TextureId,
}

/// Similar to `TextureViewDescriptor`, but contains the built fields rather than `Option`s.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TextureViewInfo {
    /// Debug label of the texture view.
    ///
    /// This will show up in graphics debuggers for easy identification.
    pub label: &'static str,
    /// Format of the texture view.
    ///
    /// At this time, it must be the same as the underlying format of the texture.
    pub format: wgpu::TextureFormat,
    /// The dimension of the texture view.
    ///
    /// - For 1D textures, this must be 1D.
    /// - For 2D textures it must be one of D2, D2Array, Cube, and CubeArray.
    /// - For 3D textures it must be 3D.
    pub dimension: wgpu::TextureViewDimension,
    /// Aspect of the texture. Color textures must be TextureAspect::All.
    pub aspect: wgpu::TextureAspect,
    pub base_mip_level: u32,
    /// Mip level count.
    ///
    /// If `Some`, base_mip_level + count must be less or equal to underlying texture mip count.
    ///
    /// If `None`, considered to include the rest of the mipmap levels, but at least 1 in total.
    pub level_count: Option<u32>,
    pub base_array_layer: u32,
    /// Layer count.
    ///
    /// If `Some`, base_array_layer + count must be less or equal to the underlying array count.
    ///
    /// If `None`, considered to include the rest of the array layers, but at least 1 in total.
    pub array_layer_count: Option<u32>,
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
    info: TextureViewInfo,
}

impl Texture {
    /// The inner descriptor from which this **Texture** was constructed.
    pub fn descriptor(&self) -> &wgpu::TextureDescriptor<'static> {
        &self.descriptor
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

    /// Mip count of texture. For a texture with no extra mips, this must be 1.
    pub fn mip_level_count(&self) -> u32 {
        self.descriptor.mip_level_count
    }

    /// Sample count of texture. If this is not 1, texture must have BindingType::SampledTexture::multisampled set to true.
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
    pub fn usage(&self) -> wgpu::TextureUsages {
        self.descriptor.usage
    }

    /// The size of the texture data in bytes.
    pub fn size_bytes(&self) -> usize {
        data_size_bytes(&self.descriptor)
    }

    /// The component type associated with the texture's format.
    pub fn sample_type(&self) -> wgpu::TextureSampleType {
        self.format()
            .sample_type(None, None)
            .expect("Expected the format to have a sample type")
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
        TextureId(Arc::as_ptr(&self.handle) as usize)
    }

    /// Begin building a **TextureView** for this **Texture**.
    ///
    /// By default, the produced **TextureViewBuilder** will build a texture view for the
    /// descriptor returned via `default_view_descriptor`.
    pub fn view(&self) -> ViewBuilder {
        ViewBuilder {
            texture: self,
            info: self.default_view_info(),
        }
    }

    /// A `TextureViewDimension` for a full view of the entire texture.
    ///
    /// NOTE: This will never produce the `D2Array`, `Cube` or `CubeArray` variants. You may have to
    /// construct your own `wgpu::TextureViewDimension` via the `view` method if these are desired.
    pub fn view_dimension(&self) -> wgpu::TextureViewDimension {
        match self.dimension() {
            wgpu::TextureDimension::D1 => wgpu::TextureViewDimension::D1,
            wgpu::TextureDimension::D2 => match self.descriptor.size.depth_or_array_layers {
                1 => wgpu::TextureViewDimension::D2,
                _ => wgpu::TextureViewDimension::D2Array,
            },
            wgpu::TextureDimension::D3 => wgpu::TextureViewDimension::D3,
        }
    }

    /// The view info, describing a full view of the texture.
    pub fn default_view_info(&self) -> TextureViewInfo {
        let format = self.format();
        TextureViewInfo {
            label: TextureView::DEFAULT_LABEL,
            format: format,
            dimension: self.view_dimension(),
            aspect: infer_aspect_from_format(format),
            base_mip_level: 0,
            level_count: Some(self.mip_level_count()),
            base_array_layer: 0,
            array_layer_count: Some(1),
        }
    }

    /// The view descriptor, describing a full view of the texture.
    pub fn default_view_descriptor(&self) -> wgpu::TextureViewDescriptor<'static> {
        view_info_to_view_descriptor(&self.default_view_info())
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

        let buffer = wgpu::RowPaddedBuffer::for_texture(
            device,
            self,
            wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::MAP_WRITE,
        );
        buffer.write(data);
        buffer.encode_copy_into(encoder, self);
    }

    /// Write the contents of the texture into a new buffer.
    ///
    /// Commands will be added to the given encoder to copy the entire contents of the texture into
    /// the buffer.
    ///
    /// If the texture has a sample count greater than one, it will first be resolved to a
    /// non-multisampled texture before being copied to the buffer.
    /// `copy_texture_to_buffer` command has been performed by the GPU.
    ///
    /// NOTE: `read` should not be called on the returned buffer until the encoded commands have
    /// been submitted to the device queue.
    pub fn to_buffer(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
    ) -> wgpu::RowPaddedBuffer {
        assert_eq!(
            self.extent().depth_or_array_layers,
            1,
            "cannot convert a 3d texture to a RowPaddedBuffer"
        );

        // If this texture is multi-sampled, resolve it first.
        if self.sample_count() > 1 {
            let view = self.create_view(&wgpu::TextureViewDescriptor::default());
            let descriptor = self.descriptor.clone();
            let resolved_texture = wgpu::TextureBuilder::from(descriptor)
                .sample_count(1)
                .usage(wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC)
                .build(device);
            let resolved_view =
                resolved_texture.create_view(&wgpu::TextureViewDescriptor::default());
            wgpu::resolve_texture(&view, &resolved_view, encoder);
            let buffer = RowPaddedBuffer::for_texture(
                device,
                &resolved_texture,
                wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            );
            buffer.encode_copy_from(encoder, &resolved_texture);
            buffer
        } else {
            let buffer = RowPaddedBuffer::for_texture(
                device,
                self,
                wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            );
            buffer.encode_copy_from(encoder, self);
            buffer
        }
    }
}

impl TextureViewInfo {
    /// Produces a `TextureViewDescriptor` that matches this info instance.
    pub fn descriptor(&self) -> wgpu::TextureViewDescriptor<'static> {
        view_info_to_view_descriptor(self)
    }
}

impl TextureView {
    pub const DEFAULT_LABEL: &'static str = "nannou-texture-view";

    pub fn info(&self) -> &TextureViewInfo {
        &self.info
    }

    pub fn descriptor(&self) -> wgpu::TextureViewDescriptor<'static> {
        self.info.descriptor()
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.info.format
    }

    pub fn dimension(&self) -> wgpu::TextureViewDimension {
        self.info.dimension
    }

    pub fn aspect(&self) -> wgpu::TextureAspect {
        self.info.aspect
    }

    pub fn base_mip_level(&self) -> u32 {
        self.info.base_mip_level
    }

    pub fn level_count(&self) -> Option<u32> {
        self.info.level_count
    }

    pub fn base_array_layer(&self) -> u32 {
        self.info.base_array_layer
    }

    pub fn array_layer_count(&self) -> Option<u32> {
        self.info.array_layer_count
    }

    pub fn sample_type(&self) -> wgpu::TextureSampleType {
        self.format()
            .sample_type(None, None)
            .expect("Expected the format to have a sample type")
    }

    pub fn id(&self) -> TextureViewId {
        texture_view_id(&self.texture_id, &self.info)
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
        depth_or_array_layers: Self::DEFAULT_DEPTH,
    };
    pub const DEFAULT_ARRAY_LAYER_COUNT: u32 = 1;
    pub const DEFAULT_MIP_LEVEL_COUNT: u32 = 1;
    pub const DEFAULT_SAMPLE_COUNT: u32 = 1;
    pub const DEFAULT_DIMENSION: wgpu::TextureDimension = wgpu::TextureDimension::D2;
    pub const DEFAULT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
    pub const DEFAULT_USAGE: wgpu::TextureUsages = wgpu::TextureUsages::all(); // TODO: is this the right choice?
    pub const DEFAULT_DESCRIPTOR: wgpu::TextureDescriptor<'static> = wgpu::TextureDescriptor {
        label: Some("nannou Texture"),
        size: Self::DEFAULT_SIZE,
        mip_level_count: Self::DEFAULT_MIP_LEVEL_COUNT,
        sample_count: Self::DEFAULT_SAMPLE_COUNT,
        dimension: Self::DEFAULT_DIMENSION,
        format: Self::DEFAULT_FORMAT,
        usage: Self::DEFAULT_USAGE,
        view_formats: &[],
    };

    /// Creates a new `Default` builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Specify the width and height of the texture.
    ///
    /// Note: On calls to `size`, `depth` and `extent` the `Builder` will attempt to infer the
    /// `wgpu::TextureDimension` of its inner `wgpu::TextureDescriptor` by examining its `size`
    /// field. Use `TextureBuilder::dimension()` to override this behavior.
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
    /// field. Use `TextureBuilder::dimension()` to override this behavior.
    pub fn depth(mut self, depth: u32) -> Self {
        self.descriptor.size.depth_or_array_layers = depth;
        self.infer_dimension_from_size();
        self
    }

    /// Specify the width, height and depth of the texture.
    ///
    /// Note: On calls to `size`, `depth` and `extent` the `Builder` will attempt to infer the
    /// `wgpu::TextureDimension` of its inner `wgpu::TextureDescriptor` by examining its `size`
    /// field. Use `TextureBuilder::dimension()` to override this behavior.
    pub fn extent(mut self, extent: wgpu::Extent3d) -> Self {
        self.descriptor.size = extent;
        self.infer_dimension_from_size();
        self
    }

    /// Specify the dimension of the texture, overriding inferred dimension.
    ///
    /// Mainly useful for creating 2d texture arrays -- override dimension with
    /// `wgpu::TextureDimension::D2` on a texture with `extent.depth > 1` in order to create a
    /// texture array (/ cubemap / cubemap array) instead of a 3d texture.
    pub fn dimension(mut self, dimension: wgpu::TextureDimension) -> Self {
        self.descriptor.dimension = dimension;
        self
    }

    /// Specify the number of mip levels of the texture.
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
    pub fn usage(mut self, usage: wgpu::TextureUsages) -> Self {
        self.descriptor.usage = usage;
        self
    }

    // If `depth` is greater than `1` then `D3` is assumed, otherwise if `height` is greater than
    // `1` then `D2` is assumed, otherwise `D1` is assumed.
    fn infer_dimension_from_size(&mut self) {
        if self.descriptor.size.depth_or_array_layers > 1 {
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
    /// Debug label of the texture view.
    ///
    /// This will show up in graphics debuggers for easy identification.
    ///
    /// By default, this is `"nannou-texture-view"`.
    pub fn label(mut self, label: &'static str) -> Self {
        self.info.label = label;
        self
    }

    /// Format of the texture view.
    ///
    /// At this time, it must be the same as the underlying format of the texture.
    ///
    /// By default, this is derived from the parent texture.
    pub fn format(mut self, format: wgpu::TextureFormat) -> Self {
        self.info.format = format;
        self
    }

    /// The dimension of the texture view.
    ///
    /// - For 1D textures, this must be 1D.
    /// - For 2D textures it must be one of D2, D2Array, Cube, and CubeArray.
    /// - For 3D textures it must be 3D.
    ///
    /// By default, this is derived from the parent texture.
    pub fn dimension(mut self, dimension: wgpu::TextureViewDimension) -> Self {
        self.info.dimension = dimension;
        self
    }

    /// Aspect of the texture.
    ///
    /// Color textures **must** be `TextureAspect::All`.
    ///
    /// By default, this is the result of `infer_aspect_from_format` called for the parent
    /// texture's texture format. See the `infer_aspect_from_format` function docs for details.
    pub fn aspect(mut self, aspect: wgpu::TextureAspect) -> Self {
        self.info.aspect = aspect;
        self
    }

    /// Mip level count.
    ///
    /// If `Some`, base_mip_level + count must be less or equal to underlying texture mip count.
    ///
    /// If `None`, considered to include the rest of the mipmap levels, but at least 1 in total.
    pub fn level_count(mut self, level_count: Option<u32>) -> Self {
        self.info.level_count = level_count;
        self
    }

    pub fn base_array_layer(mut self, base_array_layer: u32) -> Self {
        self.info.base_array_layer = base_array_layer;
        self
    }

    /// Layer count.
    ///
    /// If `Some`, base_array_layer + count must be less or equal to the underlying array count.
    ///
    /// If `None`, considered to include the rest of the array layers, but at least 1 in total.
    pub fn array_layer_count(mut self, array_layer_count: Option<u32>) -> Self {
        match (self.info.dimension, array_layer_count) {
            (wgpu::TextureViewDimension::D2Array, Some(1)) => {
                self.info.dimension = wgpu::TextureViewDimension::D2;
            }
            _ => (),
        }
        self.info.array_layer_count = array_layer_count;
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
        self.base_array_layer(layer).array_layer_count(Some(1))
    }

    pub fn build(self) -> TextureView {
        let descriptor = self.info.descriptor();
        TextureView {
            handle: Arc::new(self.texture.inner().create_view(&descriptor)),
            info: self.info,
            texture_id: self.texture.id(),
            texture_extent: self.texture.extent(),
        }
    }

    /// Consumes the texture view builder and returns the resulting `wgpu::TextureViewDescriptor`.
    pub fn into_descriptor(self) -> wgpu::TextureViewDescriptor<'static> {
        self.info.descriptor()
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
            info: self.info.clone(),
            texture_id: self.texture_id(),
            texture_extent: self.extent(),
        }
    }
}

impl Clone for Texture {
    fn clone(&self) -> Self {
        let handle = self.handle.clone();
        let descriptor = self.descriptor.clone();
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

impl Default for Builder {
    fn default() -> Self {
        Self {
            descriptor: Self::DEFAULT_DESCRIPTOR,
        }
    }
}

impl From<Texture> for Arc<TextureHandle> {
    fn from(t: Texture) -> Self {
        t.handle
    }
}

impl From<TextureViewInfo> for wgpu::TextureViewDescriptor<'static> {
    fn from(info: TextureViewInfo) -> Self {
        view_info_to_view_descriptor(&info)
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

impl<'a> From<ViewBuilder<'a>> for TextureViewInfo {
    fn from(builder: ViewBuilder<'a>) -> Self {
        builder.info
    }
}

/// Create a texture ID by hashing the source texture ID along with the contents of the descriptor.
fn texture_view_id(texture_id: &TextureId, view_info: &TextureViewInfo) -> TextureViewId {
    use std::hash::{Hash, Hasher};
    let mut s = std::collections::hash_map::DefaultHasher::new();
    texture_id.hash(&mut s);
    view_info.hash(&mut s);
    TextureViewId(s.finish())
}

/// The size of the texture data in bytes as described by the given descriptor.
pub fn data_size_bytes(desc: &wgpu::TextureDescriptor) -> usize {
    desc.size.width as usize
        * desc.size.height as usize
        * desc.size.depth_or_array_layers as usize
        * format_size_bytes(desc.format) as usize
}

/// Return the size of the given texture format in bytes.
pub fn format_size_bytes(format: wgpu::TextureFormat) -> u32 {
    format
        .block_copy_size(None)
        .expect("Expected the format to have a block size") as u32
}

/// Returns `true` if the given `wgpu::Extent3d`s are equal.
pub fn extent_3d_eq(a: &wgpu::Extent3d, b: &wgpu::Extent3d) -> bool {
    a.width == b.width && a.height == b.height && a.depth_or_array_layers == b.depth_or_array_layers
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

/// Used to infer the `TextureAspect` for a `TextureView` from a specific `TextureFormat`.
///
/// Does the following:
///
/// - If the format is `Depth32Float` or `Depth24Plus`, `TextureAspect::DepthOnly` is assumed.
/// - Otherwise, `TextureAspect::All` is assumed.
///
/// Please note that `wgpu::TextureAspect::StencilOnly` can never be inferred with this function.
/// If you require using a `TextureView` as a stencil, consider explicitly specify the
/// `TextureAspect` you require.
pub fn infer_aspect_from_format(format: wgpu::TextureFormat) -> wgpu::TextureAspect {
    use wgpu::TextureFormat::*;
    match format {
        Depth32Float | Depth24Plus => wgpu::TextureAspect::DepthOnly,
        _ => wgpu::TextureAspect::All,
    }
}

fn view_info_to_view_descriptor(info: &TextureViewInfo) -> wgpu::TextureViewDescriptor<'static> {
    wgpu::TextureViewDescriptor {
        label: Some(info.label),
        format: Some(info.format),
        dimension: Some(info.dimension),
        usage: None,
        aspect: info.aspect,
        base_mip_level: info.base_mip_level,
        mip_level_count: info.level_count,
        base_array_layer: info.base_array_layer,
        array_layer_count: info.array_layer_count,
    }
}
