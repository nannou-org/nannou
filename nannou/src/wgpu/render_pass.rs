use crate::wgpu::{self, Color, LoadOp};

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
    pub const DEFAULT_CLEAR_COLOR: Color = Color::TRANSPARENT;
    pub const DEFAULT_LOAD_OP: LoadOp<Color> = LoadOp::Clear(Self::DEFAULT_CLEAR_COLOR);
    pub const DEFAULT_STORE_OP: bool = true;

    /// Begin building a new render pass color attachment descriptor.
    fn new(attachment: &'a wgpu::TextureViewHandle) -> Self {
        ColorAttachmentDescriptorBuilder {
            descriptor: wgpu::RenderPassColorAttachmentDescriptor {
                attachment,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: LoadOp::Clear(Color::TRANSPARENT),
                    store: true,
                },
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
    pub fn load_op(mut self, load_op: LoadOp<Color>) -> Self {
        self.descriptor.ops.load = load_op;
        self
    }

    /// The end-of-pass store operation for this color attachment.
    pub fn store_op(mut self, store_op: bool) -> Self {
        self.descriptor.ops.store = store_op;
        self
    }
}

impl<'a> DepthStencilAttachmentDescriptorBuilder<'a> {
    pub const DEFAULT_DEPTH_LOAD_OP: LoadOp<f32> = LoadOp::Clear(0.);
    pub const DEFAULT_DEPTH_STORE_OP: bool = true;
    pub const DEFAULT_CLEAR_DEPTH: f32 = 1.0;
    pub const DEFAULT_STENCIL_LOAD_OP: LoadOp<u32> = LoadOp::Clear(0);
    pub const DEFAULT_STENCIL_STORE_OP: bool = true;
    pub const DEFAULT_CLEAR_STENCIL: u32 = 0;

    fn new(attachment: &'a wgpu::TextureViewHandle) -> Self {
        DepthStencilAttachmentDescriptorBuilder {
            descriptor: wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment,
                depth_ops: Some(wgpu::Operations {
                    load: LoadOp::Clear(0.),
                    store: true,
                }),
                stencil_ops: Some(wgpu::Operations {
                    load: LoadOp::Clear(0),
                    store: true,
                }),
            },
        }
    }

    /// The beginning-of-pass load operation for this depth attachment.
    pub fn depth_load_op(mut self, load: LoadOp<f32>) -> Self {
        self.descriptor.depth_ops = Some(wgpu::Operations {
            load,
            store: self.descriptor.depth_ops.expect("no depth ops field").store,
        });
        self
    }

    /// The end-of-pass store operation for this depth attachment.
    pub fn depth_store_op(mut self, store: bool) -> Self {
        self.descriptor.depth_ops = Some(wgpu::Operations {
            load: self.descriptor.depth_ops.expect("no depth ops field").load,
            store,
        });
        self
    }

    /// The beginning-of-pass load operation for this stencil attachment.
    pub fn stencil_load_op(mut self, load: LoadOp<u32>) -> Self {
        self.descriptor.stencil_ops = Some(wgpu::Operations {
            load,
            store: self
                .descriptor
                .stencil_ops
                .expect("no stencil ops field")
                .store,
        });
        self
    }

    /// The end-of-pass store operation for this stencil attachment.
    pub fn stencil_store_op(mut self, store: bool) -> Self {
        self.descriptor.stencil_ops = Some(wgpu::Operations {
            load: self
                .descriptor
                .stencil_ops
                .expect("no stencil ops field")
                .load,
            store,
        });
        self
    }
}

impl<'a> Builder<'a> {
    pub const DEFAULT_COLOR_LOAD_OP: LoadOp<Color> =
        ColorAttachmentDescriptorBuilder::DEFAULT_LOAD_OP;
    pub const DEFAULT_COLOR_STORE_OP: bool = ColorAttachmentDescriptorBuilder::DEFAULT_STORE_OP;
    pub const DEFAULT_CLEAR_COLOR: Color = ColorAttachmentDescriptorBuilder::DEFAULT_CLEAR_COLOR;
    pub const DEFAULT_DEPTH_LOAD_OP: LoadOp<f32> =
        DepthStencilAttachmentDescriptorBuilder::DEFAULT_DEPTH_LOAD_OP;
    pub const DEFAULT_DEPTH_STORE_OP: bool =
        DepthStencilAttachmentDescriptorBuilder::DEFAULT_DEPTH_STORE_OP;
    pub const DEFAULT_CLEAR_DEPTH: f32 =
        DepthStencilAttachmentDescriptorBuilder::DEFAULT_CLEAR_DEPTH;
    pub const DEFAULT_STENCIL_LOAD_OP: LoadOp<u32> =
        DepthStencilAttachmentDescriptorBuilder::DEFAULT_STENCIL_LOAD_OP;
    pub const DEFAULT_STENCIL_STORE_OP: bool =
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
