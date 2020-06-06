use crate::wgpu;

/// A builder type to simplify the process of creating a render pass descriptor.
#[derive(Debug, Default)]
pub struct Builder<'a> {
    color_attachments: Vec<wgpu::RenderPassColorAttachmentDescriptor<'a>>,
    depth_stencil_attachment: Option<wgpu::RenderPassDepthStencilAttachmentDescriptor<'a>>,
}

/// A builder type to simplify the process of creating a render pass descriptor.
#[derive(Debug)]
pub struct ColorAttachmentDescriptorBuilder<'a> {
    descriptor: wgpu::RenderPassColorAttachmentDescriptor<'a>,
}

/// A builder type to simplify the process of creating a render pass descriptor.
#[derive(Debug)]
pub struct DepthStencilAttachmentDescriptorBuilder<'a> {
    descriptor: wgpu::RenderPassDepthStencilAttachmentDescriptor<'a>,
}

impl<'a> ColorAttachmentDescriptorBuilder<'a> {
    pub const DEFAULT_LOAD_OP: wgpu::LoadOp = wgpu::LoadOp::Clear;
    pub const DEFAULT_STORE_OP: wgpu::StoreOp = wgpu::StoreOp::Store;
    pub const DEFAULT_CLEAR_COLOR: wgpu::Color = wgpu::Color::TRANSPARENT;

    /// Begin building a new render pass color attachment descriptor.
    fn new(attachment: &'a wgpu::TextureViewHandle) -> Self {
        ColorAttachmentDescriptorBuilder {
            descriptor: wgpu::RenderPassColorAttachmentDescriptor {
                attachment,
                resolve_target: None,
                load_op: Self::DEFAULT_LOAD_OP,
                store_op: Self::DEFAULT_STORE_OP,
                clear_color: Self::DEFAULT_CLEAR_COLOR,
            },
        }
    }

    /// Specify the resolve target for this render pass color attachment.
    pub fn resolve_target(mut self, target: Option<&'a wgpu::TextureView>) -> Self {
        self.descriptor.resolve_target = target.map(|t| &**t);
        self
    }

    /// Specify the resolve target for this render pass color attachment.
    pub fn resolve_target_handle(mut self, target: Option<&'a wgpu::TextureViewHandle>) -> Self {
        self.descriptor.resolve_target = target;
        self
    }

    /// The beginning-of-pass load operation for this color attachment.
    pub fn load_op(mut self, load_op: wgpu::LoadOp) -> Self {
        self.descriptor.load_op = load_op;
        self
    }

    /// The end-of-pass store operation for this color attachment.
    pub fn store_op(mut self, store_op: wgpu::StoreOp) -> Self {
        self.descriptor.store_op = store_op;
        self
    }

    /// The color that will be assigned to every pixel of this attachment when cleared.
    pub fn clear_color(mut self, color: wgpu::Color) -> Self {
        self.descriptor.clear_color = color;
        self
    }
}

impl<'a> DepthStencilAttachmentDescriptorBuilder<'a> {
    pub const DEFAULT_DEPTH_LOAD_OP: wgpu::LoadOp = wgpu::LoadOp::Clear;
    pub const DEFAULT_DEPTH_STORE_OP: wgpu::StoreOp = wgpu::StoreOp::Store;
    pub const DEFAULT_CLEAR_DEPTH: f32 = 1.0;
    pub const DEFAULT_STENCIL_LOAD_OP: wgpu::LoadOp = wgpu::LoadOp::Clear;
    pub const DEFAULT_STENCIL_STORE_OP: wgpu::StoreOp = wgpu::StoreOp::Store;
    pub const DEFAULT_CLEAR_STENCIL: u32 = 0;

    fn new(attachment: &'a wgpu::TextureViewHandle) -> Self {
        DepthStencilAttachmentDescriptorBuilder {
            descriptor: wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment,
                depth_load_op: Self::DEFAULT_DEPTH_LOAD_OP,
                depth_store_op: Self::DEFAULT_DEPTH_STORE_OP,
                clear_depth: Self::DEFAULT_CLEAR_DEPTH,
                stencil_load_op: Self::DEFAULT_STENCIL_LOAD_OP,
                stencil_store_op: Self::DEFAULT_STENCIL_STORE_OP,
                clear_stencil: Self::DEFAULT_CLEAR_STENCIL,
            },
        }
    }

    /// The beginning-of-pass load operation for this depth attachment.
    pub fn depth_load_op(mut self, load_op: wgpu::LoadOp) -> Self {
        self.descriptor.depth_load_op = load_op;
        self
    }

    /// The end-of-pass store operation for this depth attachment.
    pub fn depth_store_op(mut self, store_op: wgpu::StoreOp) -> Self {
        self.descriptor.depth_store_op = store_op;
        self
    }

    /// The value that will be assigned to every pixel of this depth attachment when cleared.
    pub fn clear_depth(mut self, depth: f32) -> Self {
        self.descriptor.clear_depth = depth;
        self
    }

    /// The beginning-of-pass load operation for this stencil attachment.
    pub fn stencil_load_op(mut self, load_op: wgpu::LoadOp) -> Self {
        self.descriptor.stencil_load_op = load_op;
        self
    }

    /// The end-of-pass store operation for this stencil attachment.
    pub fn stencil_store_op(mut self, store_op: wgpu::StoreOp) -> Self {
        self.descriptor.stencil_store_op = store_op;
        self
    }

    /// The value that will be assigned to every pixel of this stencil attachment when cleared.
    pub fn clear_stencil(mut self, stencil: u32) -> Self {
        self.descriptor.clear_stencil = stencil;
        self
    }
}

impl<'a> Builder<'a> {
    pub const DEFAULT_COLOR_LOAD_OP: wgpu::LoadOp =
        ColorAttachmentDescriptorBuilder::DEFAULT_LOAD_OP;
    pub const DEFAULT_COLOR_STORE_OP: wgpu::StoreOp =
        ColorAttachmentDescriptorBuilder::DEFAULT_STORE_OP;
    pub const DEFAULT_CLEAR_COLOR: wgpu::Color =
        ColorAttachmentDescriptorBuilder::DEFAULT_CLEAR_COLOR;
    pub const DEFAULT_DEPTH_LOAD_OP: wgpu::LoadOp =
        DepthStencilAttachmentDescriptorBuilder::DEFAULT_DEPTH_LOAD_OP;
    pub const DEFAULT_DEPTH_STORE_OP: wgpu::StoreOp =
        DepthStencilAttachmentDescriptorBuilder::DEFAULT_DEPTH_STORE_OP;
    pub const DEFAULT_CLEAR_DEPTH: f32 =
        DepthStencilAttachmentDescriptorBuilder::DEFAULT_CLEAR_DEPTH;
    pub const DEFAULT_STENCIL_LOAD_OP: wgpu::LoadOp =
        DepthStencilAttachmentDescriptorBuilder::DEFAULT_STENCIL_LOAD_OP;
    pub const DEFAULT_STENCIL_STORE_OP: wgpu::StoreOp =
        DepthStencilAttachmentDescriptorBuilder::DEFAULT_STENCIL_STORE_OP;
    pub const DEFAULT_CLEAR_STENCIL: u32 =
        DepthStencilAttachmentDescriptorBuilder::DEFAULT_CLEAR_STENCIL;

    /// Begin building a new render pass descriptor.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a single color attachment descriptor to the render pass descriptor.
    ///
    /// Call this multiple times in succession to add multiple color attachments.
    pub fn color_attachment<F>(
        mut self,
        attachment: &'a wgpu::TextureViewHandle,
        color_builder: F,
    ) -> Self
    where
        F: FnOnce(ColorAttachmentDescriptorBuilder<'a>) -> ColorAttachmentDescriptorBuilder<'a>,
    {
        let builder = ColorAttachmentDescriptorBuilder::new(attachment);
        let descriptor = color_builder(builder).descriptor;
        self.color_attachments.push(descriptor);
        self
    }

    /// Add a depth stencil attachment to the render pass.
    ///
    /// This should only be called once, as only a single depth stencil attachment is valid. Only
    /// the attachment submitted last will be used.
    pub fn depth_stencil_attachment<F>(
        mut self,
        attachment: &'a wgpu::TextureViewHandle,
        depth_stencil_builder: F,
    ) -> Self
    where
        F: FnOnce(
            DepthStencilAttachmentDescriptorBuilder<'a>,
        ) -> DepthStencilAttachmentDescriptorBuilder<'a>,
    {
        let builder = DepthStencilAttachmentDescriptorBuilder::new(attachment);
        let descriptor = depth_stencil_builder(builder).descriptor;
        self.depth_stencil_attachment = Some(descriptor);
        self
    }

    /// Return the built color and depth attachments.
    pub fn into_inner(
        self,
    ) -> (
        Vec<wgpu::RenderPassColorAttachmentDescriptor<'a>>,
        Option<wgpu::RenderPassDepthStencilAttachmentDescriptor<'a>>,
    ) {
        let Builder {
            color_attachments,
            depth_stencil_attachment,
        } = self;
        (color_attachments, depth_stencil_attachment)
    }

    /// Begin a render pass with the specified parameters on the given encoder.
    pub fn begin(self, encoder: &'a mut wgpu::CommandEncoder) -> wgpu::RenderPass<'a> {
        let (color_attachments, depth_stencil_attachment) = self.into_inner();
        let descriptor = wgpu::RenderPassDescriptor {
            color_attachments: &color_attachments,
            depth_stencil_attachment,
        };
        encoder.begin_render_pass(&descriptor)
    }
}
