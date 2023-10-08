use std::num::NonZeroU8;

use crate as wgpu;

/// Simplifies the construction of a `Sampler` with a set of reasonable defaults.
#[derive(Debug)]
pub struct SamplerBuilder {
    pub descriptor: wgpu::SamplerDescriptor<'static>,
}

impl<'b> SamplerBuilder {
    pub const DEFAULT_ADDRESS_MODE_U: wgpu::AddressMode = wgpu::AddressMode::ClampToEdge;
    pub const DEFAULT_ADDRESS_MODE_V: wgpu::AddressMode = wgpu::AddressMode::ClampToEdge;
    pub const DEFAULT_ADDRESS_MODE_W: wgpu::AddressMode = wgpu::AddressMode::ClampToEdge;
    pub const DEFAULT_MAG_FILTER: wgpu::FilterMode = wgpu::FilterMode::Linear;
    pub const DEFAULT_MIN_FILTER: wgpu::FilterMode = wgpu::FilterMode::Linear;
    pub const DEFAULT_MIPMAP_FILTER: wgpu::FilterMode = wgpu::FilterMode::Nearest;
    pub const DEFAULT_LOD_MIN_CLAMP: f32 = 0.0;
    pub const DEFAULT_LOD_MAX_CLAMP: f32 = 100.0;
    pub const DEFAULT_COMPARE: Option<wgpu::CompareFunction> = None;
    pub const DEFAULT_ANISOTROPY_CLAMP: u16 = 1;
    pub const DEFAULT_LABEL: &'static str = "nannou-sampler";
    pub const DEFAULT_BORDER_COLOR: Option<wgpu::SamplerBorderColor> = None;
    pub const DEFAULT_DESCRIPTOR: wgpu::SamplerDescriptor<'static> = wgpu::SamplerDescriptor {
        address_mode_u: Self::DEFAULT_ADDRESS_MODE_U,
        address_mode_v: Self::DEFAULT_ADDRESS_MODE_V,
        address_mode_w: Self::DEFAULT_ADDRESS_MODE_W,
        mag_filter: Self::DEFAULT_MAG_FILTER,
        min_filter: Self::DEFAULT_MIN_FILTER,
        mipmap_filter: Self::DEFAULT_MIPMAP_FILTER,
        lod_min_clamp: Self::DEFAULT_LOD_MIN_CLAMP,
        lod_max_clamp: Self::DEFAULT_LOD_MAX_CLAMP,
        compare: Self::DEFAULT_COMPARE,
        anisotropy_clamp: Self::DEFAULT_ANISOTROPY_CLAMP,
        label: Some(Self::DEFAULT_LABEL),
        border_color: Self::DEFAULT_BORDER_COLOR,
    };

    /// Begin building a `Sampler`, starting with the `Default` parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// How the implementation should behave when sampling outside of the texture coordinates range
    /// [0.0, 1.0].
    pub fn address_mode_u(mut self, mode: wgpu::AddressMode) -> Self {
        self.descriptor.address_mode_u = mode;
        self
    }

    /// How the implementation should behave when sampling outside of the texture coordinates range
    /// [0.0, 1.0].
    pub fn address_mode_v(mut self, mode: wgpu::AddressMode) -> Self {
        self.descriptor.address_mode_v = mode;
        self
    }

    /// How the implementation should behave when sampling outside of the texture coordinates range
    /// [0.0, 1.0].
    pub fn address_mode_w(mut self, mode: wgpu::AddressMode) -> Self {
        self.descriptor.address_mode_w = mode;
        self
    }

    /// How the implementation should behave when sampling outside of the texture coordinates range
    /// [0.0, 1.0].
    ///
    /// Applies the same address mode to all axes.
    pub fn address_mode(self, mode: wgpu::AddressMode) -> Self {
        self.address_mode_u(mode)
            .address_mode_v(mode)
            .address_mode_w(mode)
    }

    /// How the implementation should sample from the image when it is respectively larger than the
    /// original.
    pub fn mag_filter(mut self, filter: wgpu::FilterMode) -> Self {
        self.descriptor.mag_filter = filter;
        self
    }

    /// How the implementation should sample from the image when it is respectively smaller than
    /// the original.
    pub fn min_filter(mut self, filter: wgpu::FilterMode) -> Self {
        self.descriptor.min_filter = filter;
        self
    }

    /// How the implementation should choose which mipmap to use.
    pub fn mipmap_filter(mut self, filter: wgpu::FilterMode) -> Self {
        self.descriptor.mipmap_filter = filter;
        self
    }

    /// The minimum mipmap level to use.
    pub fn lod_min_clamp(mut self, min: f32) -> Self {
        self.descriptor.lod_min_clamp = min;
        self
    }

    /// The maximum mipmap level to use.
    pub fn lod_max_clamp(mut self, max: f32) -> Self {
        self.descriptor.lod_max_clamp = max;
        self
    }

    /// The comparison function to use, if any.
    pub fn compare(mut self, f: Option<wgpu::CompareFunction>) -> Self {
        self.descriptor.compare = f;
        self
    }

    /// The anisotropy level to clamp to, if any.
    pub fn anisotropy_clamp(mut self, clamp: u16) -> Self {
        self.descriptor.anisotropy_clamp = clamp;
        self
    }

    /// The label to use, if any.
    pub fn label(mut self, label: Option<&'static str>) -> Self {
        self.descriptor.label = label;
        self
    }

    /// Calls `device.create_sampler(&self.descriptor)` internally.
    pub fn build(&self, device: &wgpu::Device) -> wgpu::Sampler {
        device.create_sampler(&self.descriptor)
    }

    /// Consume the builder and produce the inner `SamplerDescriptor`.
    pub fn into_descriptor(self) -> wgpu::SamplerDescriptor<'static> {
        self.into()
    }
}

impl Default for SamplerBuilder {
    fn default() -> Self {
        SamplerBuilder {
            descriptor: Self::DEFAULT_DESCRIPTOR,
        }
    }
}

impl Into<wgpu::SamplerDescriptor<'static>> for SamplerBuilder {
    fn into(self) -> wgpu::SamplerDescriptor<'static> {
        self.descriptor
    }
}

impl From<wgpu::SamplerDescriptor<'static>> for SamplerBuilder {
    fn from(descriptor: wgpu::SamplerDescriptor<'static>) -> Self {
        SamplerBuilder { descriptor }
    }
}
