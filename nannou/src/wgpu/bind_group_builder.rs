use crate::wgpu;

/// A type aimed at simplifying the creation of a bind group layout.
#[derive(Debug, Default)]
pub struct LayoutBuilder {
    bindings: Vec<(wgpu::ShaderStage, wgpu::BindingType)>,
}

/// Simplified creation of a bind group.
#[derive(Debug, Default)]
pub struct Builder<'a> {
    resources: Vec<wgpu::BindingResource<'a>>,
}

impl LayoutBuilder {
    /// Begin building the bind group layout.
    pub fn new() -> Self {
        Self::default()
    }

    /// Specify a new binding.
    ///
    /// The `binding` position of each binding will be inferred as the index within the order that
    /// they are added to this builder type. If you require manually specifying the binding
    /// location, you may be better off not using the `BindGroupLayoutBuilder` and instead
    /// constructing the `BindGroupLayout` and `BindGroup` manually.
    pub fn binding(mut self, visibility: wgpu::ShaderStage, ty: wgpu::BindingType) -> Self {
        self.bindings.push((visibility, ty));
        self
    }

    /// Add a uniform buffer binding to the layout.
    pub fn uniform_buffer(self, visibility: wgpu::ShaderStage, dynamic: bool) -> Self {
        let ty = wgpu::BindingType::UniformBuffer { dynamic };
        self.binding(visibility, ty)
    }

    /// Add a storage buffer binding to the layout.
    pub fn storage_buffer(
        self,
        visibility: wgpu::ShaderStage,
        dynamic: bool,
        readonly: bool,
    ) -> Self {
        let ty = wgpu::BindingType::StorageBuffer { dynamic, readonly };
        self.binding(visibility, ty)
    }

    /// Add a sampler binding to the layout.
    pub fn sampler(self, visibility: wgpu::ShaderStage) -> Self {
        let comparison = false;
        let ty = wgpu::BindingType::Sampler { comparison };
        self.binding(visibility, ty)
    }

    /// Add a sampler binding to the layout.
    pub fn comparison_sampler(self, visibility: wgpu::ShaderStage) -> Self {
        let comparison = true;
        let ty = wgpu::BindingType::Sampler { comparison };
        self.binding(visibility, ty)
    }

    /// Add a sampled texture binding to the layout.
    pub fn sampled_texture(
        self,
        visibility: wgpu::ShaderStage,
        multisampled: bool,
        dimension: wgpu::TextureViewDimension,
        component_type: wgpu::TextureComponentType,
    ) -> Self {
        let ty = wgpu::BindingType::SampledTexture {
            multisampled,
            dimension,
            component_type,
        };
        self.binding(visibility, ty)
    }

    /// Short-hand for adding a sampled textured binding for a full view of the given texture to
    /// the layout.
    ///
    /// The `multisampled` and `dimension` parameters are retrieved from the `Texture` itself.
    ///
    /// Note that if you wish to take a `Cube` or `CubeArray` view of the given texture, you will
    /// need to manually specify the `TextureViewDimension` via the `sampled_texture` method
    /// instead.
    pub fn sampled_texture_from(
        self,
        visibility: wgpu::ShaderStage,
        texture: &wgpu::Texture,
    ) -> Self {
        self.sampled_texture(
            visibility,
            texture.sample_count() > 1,
            texture.view_dimension(),
            texture.component_type(),
        )
    }

    /// Add a storage texture binding to the layout.
    pub fn storage_texture(
        self,
        visibility: wgpu::ShaderStage,
        format: wgpu::TextureFormat,
        dimension: wgpu::TextureViewDimension,
        component_type: wgpu::TextureComponentType,
        readonly: bool,
    ) -> Self {
        let ty = wgpu::BindingType::StorageTexture {
            dimension,
            component_type,
            format,
            readonly,
        };
        self.binding(visibility, ty)
    }

    /// Short-hand for adding a storage texture binding for a full view of the given texture to the
    /// layout.
    ///
    /// The `format`, `dimension` and `component_type` are inferred from the given `texture`.
    pub fn storage_texture_from(
        self,
        visibility: wgpu::ShaderStage,
        texture: &wgpu::Texture,
        readonly: bool,
    ) -> Self {
        self.storage_texture(
            visibility,
            texture.format(),
            texture.view_dimension(),
            texture.component_type(),
            readonly,
        )
    }

    /// Build the bind group layout from the specified parameters.
    pub fn build(self, device: &wgpu::Device) -> wgpu::BindGroupLayout {
        let mut bindings = Vec::with_capacity(self.bindings.len());
        for (i, (visibility, ty)) in self.bindings.into_iter().enumerate() {
            let layout_binding = wgpu::BindGroupLayoutEntry {
                binding: i as u32,
                visibility,
                ty,
            };
            bindings.push(layout_binding);
        }
        let descriptor = wgpu::BindGroupLayoutDescriptor {
            label: Some("nannou"),
            bindings: &bindings,
        };
        device.create_bind_group_layout(&descriptor)
    }
}

impl<'a> Builder<'a> {
    /// Begin building the bind group.
    pub fn new() -> Self {
        Self::default()
    }

    /// Specify a new binding.
    ///
    /// The `binding` position of each binding will be inferred as the index within the order that
    /// they are added to this builder type. If you require manually specifying the binding
    /// location, you may be better off not using the `BindGroupBuilder` and instead constructing
    /// the `BindGroupLayout` and `BindGroup` manually.
    pub fn binding(mut self, resource: wgpu::BindingResource<'a>) -> Self {
        self.resources.push(resource);
        self
    }

    /// Specify a slice of a buffer to be bound.
    ///
    /// The given `range` represents the start and end point of the buffer to be bound in bytes.
    pub fn buffer_bytes(
        self,
        buffer: &'a wgpu::Buffer,
        range: std::ops::Range<wgpu::BufferAddress>,
    ) -> Self {
        let resource = wgpu::BindingResource::Buffer { buffer, range };
        self.binding(resource)
    }

    /// Specify a slice of a buffer of elements of type `T` to be bound.
    ///
    /// This method is similar to `buffer_bytes`, but expects a range of **elements** rather than a
    /// range of **bytes**.
    ///
    /// Type `T` *must* be either `#[repr(C)]` or `#[repr(transparent)]`.
    pub fn buffer<T>(self, buffer: &'a wgpu::Buffer, range: std::ops::Range<usize>) -> Self
    where
        T: Copy,
    {
        let size_bytes = std::mem::size_of::<T>() as wgpu::BufferAddress;
        let start = range.start as wgpu::BufferAddress * size_bytes;
        let end = range.end as wgpu::BufferAddress * size_bytes;
        let byte_range = start..end;
        self.buffer_bytes(buffer, byte_range)
    }

    /// Specify a sampler to be bound.
    pub fn sampler(self, sampler: &'a wgpu::Sampler) -> Self {
        let resource = wgpu::BindingResource::Sampler(sampler);
        self.binding(resource)
    }

    /// Specify a texture view to be bound.
    pub fn texture_view(self, view: &'a wgpu::TextureViewHandle) -> Self {
        let resource = wgpu::BindingResource::TextureView(view);
        self.binding(resource)
    }

    /// Build the bind group with the specified resources.
    pub fn build(self, device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup {
        let mut bindings = Vec::with_capacity(self.resources.len());
        for (i, resource) in self.resources.into_iter().enumerate() {
            let binding = wgpu::Binding {
                binding: i as u32,
                resource,
            };
            bindings.push(binding);
        }
        let descriptor = wgpu::BindGroupDescriptor {
            label: Some("nannou"),
            layout,
            bindings: &bindings,
        };
        device.create_bind_group(&descriptor)
    }
}
