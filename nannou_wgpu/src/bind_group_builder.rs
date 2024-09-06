use wgpu::{BufferBinding, SamplerBindingType};

/// A type aimed at simplifying the creation of a bind group layout.
#[derive(Debug, Default)]
pub struct LayoutBuilder {
    bindings: Vec<(wgpu::ShaderStages, wgpu::BindingType)>,
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
    pub fn binding(mut self, visibility: wgpu::ShaderStages, ty: wgpu::BindingType) -> Self {
        self.bindings.push((visibility, ty));
        self
    }

    /// Add a uniform buffer binding to the layout.
    pub fn uniform_buffer(self, visibility: wgpu::ShaderStages, has_dynamic_offset: bool) -> Self {
        let ty = wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset,
            // wgpu 0.5-0.6 TODO: potential perf hit, investigate this field
            min_binding_size: None,
        };
        self.binding(visibility, ty)
    }

    /// Add a storage buffer binding to the layout.
    pub fn storage_buffer(
        self,
        visibility: wgpu::ShaderStages,
        has_dynamic_offset: bool,
        read_only: bool,
    ) -> Self {
        let ty = wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset,
            // wgpu 0.5-0.6 TODO: potential perf hit, investigate this field
            min_binding_size: None,
        };
        self.binding(visibility, ty)
    }

    /// Add a sampler binding to the layout.
    pub fn sampler(self, visibility: wgpu::ShaderStages, filtering: bool) -> Self {
        let ty = wgpu::BindingType::Sampler(if filtering {
            SamplerBindingType::Filtering
        } else {
            SamplerBindingType::NonFiltering
        });
        self.binding(visibility, ty)
    }

    /// Add a sampler binding to the layout.
    pub fn comparison_sampler(self, visibility: wgpu::ShaderStages) -> Self {
        let ty = wgpu::BindingType::Sampler(SamplerBindingType::Comparison);
        self.binding(visibility, ty)
    }

    /// Add a texture binding to the layout.
    pub fn texture(
        self,
        visibility: wgpu::ShaderStages,
        multisampled: bool,
        view_dimension: wgpu::TextureViewDimension,
        sample_type: wgpu::TextureSampleType,
    ) -> Self {
        // fix sample type in certain scenarios (constraint given by wgpu)
        let sample_type = if multisampled
            && matches!(
                sample_type,
                wgpu::TextureSampleType::Float { filterable: true }
            ) {
            wgpu::TextureSampleType::Float { filterable: false }
        } else {
            sample_type
        };
        let ty = wgpu::BindingType::Texture {
            multisampled,
            view_dimension,
            sample_type,
        };
        self.binding(visibility, ty)
    }

    /// Short-hand for adding a texture binding for a full view of the given texture to the layout.
    ///
    /// The `multisampled` and `dimension` parameters are retrieved from the `Texture` itself.
    ///
    /// Note that if you wish to take a `Cube` or `CubeArray` view of the given texture, you will
    /// need to manually specify the `TextureViewDimension` via the `sampled_texture` method
    /// instead.
    pub fn texture_from(self, visibility: wgpu::ShaderStages, texture: &crate::Texture) -> Self {
        self.texture(
            visibility,
            texture.sample_count() > 1,
            texture.view_dimension(),
            texture.sample_type(),
        )
    }

    /// Add a storage texture binding to the layout.
    pub fn storage_texture(
        self,
        visibility: wgpu::ShaderStages,
        format: wgpu::TextureFormat,
        view_dimension: wgpu::TextureViewDimension,
        access: wgpu::StorageTextureAccess,
    ) -> Self {
        let ty = wgpu::BindingType::StorageTexture {
            view_dimension,
            format,
            access,
        };
        self.binding(visibility, ty)
    }

    /// Short-hand for adding a storage texture binding for a full view of the given texture to the
    /// layout.
    ///
    /// The `format`, `dimension` and `sample_type` are inferred from the given `texture`.
    pub fn storage_texture_from(
        self,
        visibility: wgpu::ShaderStages,
        texture: &crate::Texture,
        access: wgpu::StorageTextureAccess,
    ) -> Self {
        self.storage_texture(
            visibility,
            texture.format(),
            texture.view_dimension(),
            access,
        )
    }

    /// Build the bind group layout from the specified parameters.
    pub fn build(self, device: &wgpu::Device) -> wgpu::BindGroupLayout {
        let mut entries = Vec::with_capacity(self.bindings.len());
        for (i, (visibility, ty)) in self.bindings.into_iter().enumerate() {
            let layout_binding = wgpu::BindGroupLayoutEntry {
                binding: i as u32,
                visibility,
                ty,
                count: None,
            };
            entries.push(layout_binding);
        }
        let descriptor = wgpu::BindGroupLayoutDescriptor {
            label: Some("nannou bind group layout"),
            entries: &entries,
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
        offset: wgpu::BufferAddress,
        size: Option<wgpu::BufferSize>,
    ) -> Self {
        let resource = wgpu::BindingResource::Buffer(BufferBinding {
            buffer,
            offset,
            size,
        });
        self.binding(resource)
    }

    /// Specify a slice of a buffer of elements of type `T` to be bound.
    ///
    /// This method is similar to `buffer_bytes`, but expects a range of **elements** rather than a
    /// range of **bytes**.
    ///
    /// Type `T` *must* be either `#[repr(C)]` or `#[repr(transparent)]`.
    // NOTE: We might want to change this to match the wgpu API by using a NonZeroU64 for size.
    pub fn buffer<T>(self, buffer: &'a wgpu::Buffer, range: std::ops::Range<usize>) -> Self
    where
        T: Copy,
    {
        let size_bytes = std::mem::size_of::<T>() as wgpu::BufferAddress;
        let start = range.start as wgpu::BufferAddress * size_bytes;
        let end = range.end as wgpu::BufferAddress * size_bytes;
        let size = std::num::NonZeroU64::new(end - start).expect("buffer slice must not be empty");
        self.buffer_bytes(buffer, start, Some(size))
    }

    /// Specify a sampler to be bound.
    pub fn sampler(self, sampler: &'a wgpu::Sampler) -> Self {
        let resource = wgpu::BindingResource::Sampler(sampler);
        self.binding(resource)
    }

    /// Specify a texture view to be bound.
    pub fn texture_view(self, view: &'a crate::TextureViewHandle) -> Self {
        let resource = wgpu::BindingResource::TextureView(view);
        self.binding(resource)
    }

    /// Build the bind group with the specified resources.
    pub fn build(self, device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup {
        let mut entries = Vec::with_capacity(self.resources.len());
        for (i, resource) in self.resources.into_iter().enumerate() {
            let binding = wgpu::BindGroupEntry {
                binding: i as u32,
                resource,
            };
            entries.push(binding);
        }
        let descriptor = wgpu::BindGroupDescriptor {
            label: Some("nannou bind group"),
            layout,
            entries: &entries,
        };
        device.create_bind_group(&descriptor)
    }
}
