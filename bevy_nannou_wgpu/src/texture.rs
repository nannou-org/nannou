use bevy::render::render_resource as wgpu;
use bevy::render::renderer::RenderDevice;

/// A type aimed at simplifying the construction of a **Texture**.
///
/// The builder assumes a set of defaults describing a 128x128, non-multisampled, single-layer,
/// non-linear sRGBA-8 texture. A suite of builder methods may be used to specify the exact
/// properties desired.
#[derive(Debug)]
pub struct TextureBuilder {
    descriptor: wgpu::TextureDescriptor<'static>,
}

impl TextureBuilder {
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
    pub fn build(self, device: &RenderDevice) -> wgpu::Texture {
        device.create_texture(&self.descriptor)
    }

    /// Consumes the builder and returns the resulting `wgpu::TextureDescriptor`.
    pub fn into_descriptor(self) -> wgpu::TextureDescriptor<'static> {
        self.into()
    }
}

impl Default for TextureBuilder {
    fn default() -> Self {
        Self {
            descriptor: Self::DEFAULT_DESCRIPTOR,
        }
    }
}

impl From<wgpu::TextureDescriptor<'static>> for TextureBuilder {
    fn from(descriptor: wgpu::TextureDescriptor<'static>) -> Self {
        Self { descriptor }
    }
}

impl Into<wgpu::TextureDescriptor<'static>> for TextureBuilder {
    fn into(self) -> wgpu::TextureDescriptor<'static> {
        self.descriptor
    }
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
        .block_size(None)
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
