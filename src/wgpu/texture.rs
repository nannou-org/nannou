use crate::wgpu::TextureHandle;

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

impl Texture {
    /// The inner descriptor from which this **Texture** was constructed.
    pub fn descriptor(&self) -> &wgpu::TextureDescriptor {
        &self.descriptor
    }

    /// The full extent of the texture in three dimensions.
    pub fn size(&self) -> wgpu::Extent3d {
        self.descriptor.size
    }

    pub fn array_layer_count(&self) -> u32 {
        self.descriptor.array_layer_count
    }

    pub fn mip_level_count(&self) -> u32 {
        self.descriptor.mip_level_count
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

    /// Create a **Texture** from the inner wgpu texture handle and the descriptor used to create
    /// it.
    ///
    /// This constructor should only be used in the case that you already have a texture handle and
    /// a descriptor but need a **Texture**. The preferred construction approach is to use the
    /// [**TextureBuilder**](./struct.Builder.html).
    ///
    /// The `descriptor` must be the same used to create the texture.
    pub fn from_texture_and_descriptor(
        texture: TextureHandle,
        descriptor: wgpu::TextureDescriptor,
    ) -> Self {
        Texture {
            texture,
            descriptor,
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

impl std::ops::Deref for Texture {
    type Target = TextureHandle;
    fn deref(&self) -> &Self::Target {
        &self.texture
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
