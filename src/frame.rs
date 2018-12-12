use draw::properties::color::IntoRgba;
use std::sync::{Arc, Mutex};
use vulkano;
use vulkano::buffer::{BufferAccess, TypedBufferAccess};
use vulkano::command_buffer::{AutoCommandBufferBuilderContextError, BeginRenderPassError,
                              BlitImageError, ClearColorImageError, CopyBufferError,
                              CopyBufferImageError, DrawError, DrawIndexedError, DynamicState,
                              FillBufferError, UpdateBufferError};
use vulkano::command_buffer::pool::standard::StandardCommandPoolBuilder;
use vulkano::descriptor::descriptor_set::DescriptorSetsCollection;
use vulkano::device::Queue;
use vulkano::format::{AcceptsPixels, ClearValue, Format};
use vulkano::framebuffer::{FramebufferAbstract, RenderPassDescClearValues};
use vulkano::image::ImageAccess;
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::pipeline::input_assembly::Index;
use vulkano::pipeline::vertex::VertexSource;
use vulkano::sampler::Filter;
use window;
use window::SwapchainImage;

/// Allows the user to draw a single **Frame** to the surface of a window.
///
/// The application's **view** function is called each time the application is ready to retrieve a
/// new image that will be displayed to a window. The **Frame** type can be thought of as the
/// canvas to which you draw this image.
///
/// ## Under the hood - Vulkan
///
/// There are a couple of main goals for the **Frame** type:
///
/// - Allow for maximum flexibility and customisation over the:
///   - Render Pass
///   - Graphics Pipeline
///   - Framebuffer Creation and
///   - Command Buffer
///   to the extent that the user may not interfere with the expected behaviour of the **App**.
/// - Provide reasonable defaults for each step so that it is friendly for new users.
///
/// **Vulkan**
///
/// Nannou uses Vulkan for interacting with the available graphics devices on a machine and for
/// presenting images to the swapchain associated with each window. It does so via the **vulkano**
/// crate, which is exposed publicly from the crate root. **vulkano** aims to be a type-safe,
/// near-zero-cost API around the low-level Vulkan API. As a result **vulkano** tends to be a lot
/// nicer than using the Vulkan API directly where it is the role of the user to maintain all the
/// invariants described within the spec themselves. Due to this higher-level nature, nannou
/// exposes the vulkano API directly in some areas where it is deemed reasonable/safe to do so.
///
/// In order to provide maximum flexibility, nannou allows for fully custom
/// [**RenderPass**](https://docs.rs/vulkano/latest/vulkano/framebuffer/struct.RenderPass.html) and
/// [**GraphicsPipeline**](https://docs.rs/vulkano/latest/vulkano/pipeline/struct.GraphicsPipeline.html)
/// via the **vulkano** API but aims to take care of the surface creation, swapchain tedium and
/// rendering synchronisation behind the scenes.
///
/// ### Render Pass and Swapchain Framebuffers.
/// 
/// The render pass describes the destination for the output of the graphics pipeline. It is
/// essential that the render pass uses the same pixel format of the window's surface. It also must
/// be initialised with the same logical device with which the surface was initialised.
///
/// For now, it is up to the user to ensure these gaurantees are met. In the future nannou may
/// provide a simplified constructor that implicitly uses the same logical device and format
/// associated with the surface.
///
/// The user can create the framebuffers for the swapchain using this render pass by using the
/// **SwapchainFramebuffers** type. While under the hood there is no distinction between the
/// Framebuffer type used to draw to a swapchain image and any other image, nannou chooses to wrap
/// these framebuffers in a type to ensure the following invariants are met:
///
/// - There must be one framebuffer per swapchain image.
/// - Each framebuffer must be recreated to match the dimensions of the swapchain each time the
///   swapchain requires recreation. This will occur any time the window is resized on desktop or
///   when an app comes in or out of focus on Android, and possibly in many other cases not
///   mentioned here.
/// - It should be impossible to write to these framebuffers outside the **view** function to
///   ensure framebuffer availability.
/// - Each call to **view** must draw to the framebuffer associated with the image that is ready as
///   indicated by the `swapchain::acquire_next_image` function.
///
/// As a result, access to the swapchain framebuffers may feel relatively restrictive. If you
/// require greater flexibility (e.g. control over framebuffer dimensions, the ability to draw to a
/// framebuffer outside the **view** function, etc) then consider creating and writing to another
/// intermediary framebuffer before drawing to the swapchain framebuffers.
///
/// See [the vulkano documenation](https://docs.rs/vulkano/latest/vulkano/framebuffer/index.html)
/// for more details on render passes and framebuffers.
///
/// ### Graphics Pipeline
///
/// The role of the `GraphicsPipeline` is similar to that of the GL "program" but much more
/// detailed and explicit. It allows for describing and chaining together a series of custom
/// shaders.
///
/// For more information on the graphics pipeline and how to create one, see [the vulkano
/// documentation](https://docs.rs/vulkano/latest/vulkano/pipeline/index.html#creating-a-graphics-pipeline).
///
/// ### Command Buffer
///
/// The API for the **Frame** type maps directly onto a vulkano `AutoCommandBufferBuilder` under
/// the hood. This `AutoCommandBufferBuilder` is created using the `primary_one_time_submit`
/// constructor. When returned, the **App** will build the command buffer and submit it to the GPU.
/// Note that certain builder methods are *not* provided in order to avoid unexpected behaviour.
/// E.g. the `build` and submit methods are not provided as it is important that the **App** is
/// able to build the command buffer and synchronise its submission with the swapchain.
///
/// Use the **frame.add_commands()** method to begin chaining together commands. You may call this
/// more than once throughout the duration of the **view** function.
///
/// See [the vulkano
/// documentation](https://docs.rs/vulkano/latest/vulkano/command_buffer/index.html) for more
/// details on how command buffers work in vulkano.
///
/// **Note:** If you find you are unable to do something or that this API is too restrictive,
/// please open an issue about it so that might be able to work out a solution!
pub struct Frame {
    // The `AutoCommandBufferBuilder` type used for building the frame's command buffer.
    //
    // An `Option` is used here to allow for taking the builder by `self` as type's builder methods
    // require consuming `self` and returning a new `AutoCommandBufferBuilder` as a result.
    //
    // This `Mutex` is only ever locked for the duration of the addition of a single command.
    command_buffer_builder: Mutex<Option<AutoCommandBufferBuilder>>,
    // The `Id` whose surface the swapchain image is associated with.
    window_id: window::Id,
    // The `nth` frame that has been presented to the window since the start of the application.
    nth: u64,
    // The index associated with the swapchain image.
    swapchain_image_index: usize,
    // The image to which this frame is drawing.
    swapchain_image: Arc<SwapchainImage>,
    // The index of the frame before which this swapchain was created.
    swapchain_frame_created: u64,
    // The queue on which the swapchain image will be drawn.
    queue: Arc<Queue>,
}

/// A builder type that allows chaining together commands for the command buffer that will be used
/// to draw to the swapchain image framebuffer associated with this **Frame**.
pub struct AddCommands<'a> {
    frame: &'a Frame,
}

// The `AutoCommandBufferBuilder` type used for building the frame's command buffer.
type AutoCommandBufferBuilder =
    vulkano::command_buffer::AutoCommandBufferBuilder<StandardCommandPoolBuilder>;

impl Frame {
    // Initialise a new empty frame ready for "drawing".
    pub(crate) fn new_empty(
        queue: Arc<Queue>,
        window_id: window::Id,
        nth: u64,
        swapchain_image_index: usize,
        swapchain_image: Arc<SwapchainImage>,
        swapchain_frame_created: u64,
    ) -> Result<Self, vulkano::OomError> {
        let device = queue.device().clone();
        let cb_builder = AutoCommandBufferBuilder::primary_one_time_submit(device, queue.family())?;
        let command_buffer_builder = Mutex::new(Some(cb_builder));
        let frame = Frame {
            command_buffer_builder,
            window_id,
            nth,
            swapchain_image_index,
            swapchain_image,
            swapchain_frame_created,
            queue,
        };
        Ok(frame)
    }

    // Called after the user's `view` function, this consumes the `Frame` and returns the inner
    // command buffer builder so that it can be completed.
    pub(crate) fn finish(self) -> AutoCommandBufferBuilder {
        self.command_buffer_builder
            .lock()
            .expect("failed to lock `command_buffer_builder`")
            .take()
            .expect("`command_buffer_builder` was `None`")
    }

    /// Returns whether or not this is the first time this swapchain image has been presented.
    ///
    /// This will be `true` following each occurrence at which the swapchain has been recreated,
    /// which may occur during resize, loop mode switch, etc.
    ///
    /// It is important to call this each frame to determine whether or not framebuffers associated
    /// with the swapchain need to be recreated.
    pub fn swapchain_image_is_new(&self) -> bool {
        // TODO: This is based on the assumption that the images will be acquired starting from
        // index `0` each time the swapchain is recreated. Verify that this is the case.
        (self.nth - self.swapchain_image_index as u64) == self.swapchain_frame_created
    }

    /// Add commands to be executed by the GPU once the **Frame** is returned.
    pub fn add_commands(&self) -> AddCommands {
        let frame = self;
        AddCommands { frame }
    }

    /// Clear the image with the given color.
    pub fn clear<C>(&self, color: C)
    where
        C: IntoRgba<f32>,
    {
        let rgba = color.into_rgba();
        let value = ClearValue::Float([rgba.red, rgba.green, rgba.blue, rgba.alpha]);
        let image = self.swapchain_image.clone();
        self.add_commands()
            .clear_color_image(image, value)
            .expect("failed to submit `clear_color_image` command");
    }

    /// The `Id` of the window whose vulkan surface is associated with this frame.
    pub fn window_id(&self) -> window::Id {
        self.window_id
    }

    /// The `nth` frame for the associated window since the application started.
    ///
    /// E.g. the first frame yielded will return `0`, the second will return `1`, and so on.
    pub fn nth(&self) -> u64 {
        self.nth
    }

    /// The swapchain image that will be the target for this frame.
    ///
    /// NOTE: You should avoid using the returned `SwapchainImage` outside of the `view` function
    /// as it may become invalid at any moment. The reason we expose the `Arc` is that some of the
    /// vulkano API (including framebuffer creation) requires it to avoid some severe ownsership
    /// issues.
    pub fn swapchain_image(&self) -> &Arc<SwapchainImage> {
        &self.swapchain_image
    }

    /// The index associated with the swapchain image that will be the target for this frame.
    pub fn swapchain_image_index(&self) -> usize {
        self.swapchain_image_index
    }

    /// The queue on which the swapchain image will be drawn.
    pub fn queue(&self) -> &Arc<Queue> {
        &self.queue
    }
}

impl<'a> AddCommands<'a> {
    // Maps a call onto the command buffer builder.
    fn map_cb<F, E>(self, map: F) -> Result<Self, E>
    where
        F: FnOnce(AutoCommandBufferBuilder) -> Result<AutoCommandBufferBuilder, E>
    {
        {
            let mut guard = self.frame.command_buffer_builder
                .lock()
                .expect("failed to lock `Frame`'s inner command buffer builder");
            let mut builder = guard
                .take()
                .expect("the `Frame`'s inner command buffer should always be `Some`");
            builder = map(builder)?;
            *guard = Some(builder);
        }
        Ok(self)
    }

    /// Adds a command that enters a render pass.
    ///
    /// If `secondary` is true, then you will only be able to add secondary command buffers while
    /// you're inside the first subpass of the render pass. If `secondary` is false, you will only
    /// be able to add inline draw commands and not secondary command buffers.
    ///
    /// C must contain exactly one clear value for each attachment in the framebuffer.
    ///
    /// You must call this before you can add draw commands.
    ///
    /// [*Documentation taken from the corresponding vulkano method.*](https://docs.rs/vulkano/latest/vulkano/command_buffer/struct.AutoCommandBufferBuilder.html)
    pub fn begin_render_pass<F, C>(
        self, 
        framebuffer: F, 
        secondary: bool, 
        clear_values: C
    ) -> Result<Self, BeginRenderPassError>
    where
        F: FramebufferAbstract + RenderPassDescClearValues<C> + Clone + Send + Sync + 'static, 
    {
        self.map_cb(move |cb| cb.begin_render_pass(framebuffer, secondary, clear_values))
    }

    /// Adds a command that jumps to the next subpass of the current render pass.
    pub fn next_subpass(
        self, 
        secondary: bool
    ) -> Result<Self, AutoCommandBufferBuilderContextError> {
        self.map_cb(move |cb| cb.next_subpass(secondary))
    }

    /// Adds a command that ends the current render pass.
    /// 
    /// This must be called after you went through all the subpasses and before you can add further
    /// commands.
    pub fn end_render_pass(self) -> Result<Self, AutoCommandBufferBuilderContextError> {
        self.map_cb(move |cb| cb.end_render_pass())
    }

    /// Adds a command that blits an image to another.
    ///
    /// A *blit* is similar to an image copy operation, except that the portion of the image that
    /// is transferred can be resized. You choose an area of the source and an area of the
    /// destination, and the implementation will resize the area of the source so that it matches
    /// the size of the area of the destination before writing it.
    ///
    /// Blit operations have several restrictions:
    ///
    /// - Blit operations are only allowed on queue families that support graphics operations.
    /// - The format of the source and destination images must support blit operations, which
    ///   depends on the Vulkan implementation. Vulkan guarantees that some specific formats must
    ///   always be supported. See tables 52 to 61 of the specifications.
    /// - Only single-sampled images are allowed.
    /// - You can only blit between two images whose formats belong to the same type. The types
    ///   are: floating-point, signed integers, unsigned integers, depth-stencil.
    /// - If you blit between depth, stencil or depth-stencil images, the format of both images
    ///   must match exactly.
    /// - If you blit between depth, stencil or depth-stencil images, only the `Nearest` filter is
    ///   allowed.
    /// - For two-dimensional images, the Z coordinate must be 0 for the top-left offset and 1 for
    ///   the bottom-right offset. Same for the Y coordinate for one-dimensional images.
    /// - For non-array images, the base array layer must be 0 and the number of layers must be 1.
    ///
    /// If `layer_count` is greater than 1, the blit will happen between each individual layer as
    /// if they were separate images.
    ///
    /// # Panic
    ///
    /// - Panics if the source or the destination was not created with `device`.
    ///
    /// [*Documentation taken from the corresponding vulkano method.*](https://docs.rs/vulkano/latest/vulkano/command_buffer/struct.AutoCommandBufferBuilder.html)
    pub fn blit_image<S, D>(
        self, 
        source: S, 
        source_top_left: [i32; 3], 
        source_bottom_right: [i32; 3], 
        source_base_array_layer: u32, 
        source_mip_level: u32, 
        destination: D, 
        destination_top_left: [i32; 3], 
        destination_bottom_right: [i32; 3], 
        destination_base_array_layer: u32, 
        destination_mip_level: u32, 
        layer_count: u32, 
        filter: Filter,
    ) -> Result<Self, BlitImageError>
    where
        S: ImageAccess + Send + Sync + 'static,
        D: ImageAccess + Send + Sync + 'static, 
    {
        self.map_cb(move |cb| {
            cb.blit_image(
                source,
                source_top_left,
                source_bottom_right,
                source_base_array_layer,
                source_mip_level,
                destination,
                destination_top_left,
                destination_bottom_right,
                destination_base_array_layer,
                destination_mip_level,
                layer_count,
                filter,
            )
        })
    }

    /// Adds a command that copies an image to another.
    /// 
    /// Copy operations have several restrictions:
    /// 
    /// - Copy operations are only allowed on queue families that support transfer, graphics, or
    ///   compute operations.
    /// - The number of samples in the source and destination images must be equal.
    /// - The size of the uncompressed element format of the source image must be equal to the
    ///   compressed element format of the destination.
    /// - If you copy between depth, stencil or depth-stencil images, the format of both images
    ///   must match exactly.
    /// - For two-dimensional images, the Z coordinate must be 0 for the image offsets and 1 for
    ///   the extent. Same for the Y coordinate for one-dimensional images.
    /// - For non-array images, the base array layer must be 0 and the number of layers must be 1.
    /// 
    /// If layer_count is greater than 1, the copy will happen between each individual layer as if
    /// they were separate images.
    ///
    /// # Panic
    /// 
    /// - Panics if the source or the destination was not created with device.
    ///
    /// [*Documentation taken from the corresponding vulkano method.*](https://docs.rs/vulkano/latest/vulkano/command_buffer/struct.AutoCommandBufferBuilder.html)
    pub fn copy_image<S, D>(
        self, 
        source: S, 
        source_offset: [i32; 3], 
        source_base_array_layer: u32, 
        source_mip_level: u32, 
        destination: D, 
        destination_offset: [i32; 3], 
        destination_base_array_layer: u32, 
        destination_mip_level: u32, 
        extent: [u32; 3], 
        layer_count: u32,
    ) -> Result<Self, ()> // TODO: Expose error: https://github.com/vulkano-rs/vulkano/pull/1112
    where
        S: ImageAccess + Send + Sync + 'static,
        D: ImageAccess + Send + Sync + 'static, 
    {
        self.map_cb(move |cb| {
                cb.copy_image(
                    source,
                    source_offset,
                    source_base_array_layer,
                    source_mip_level,
                    destination,
                    destination_offset,
                    destination_base_array_layer,
                    destination_mip_level,
                    extent,
                    layer_count,
                )
            })
            .map_err(|err| panic!("{}", err))
    }

    /// Adds a command that clears all the layers and mipmap levels of a color image with a
    /// specific value.
    ///
    /// # Panic
    ///
    /// Panics if `color` is not a color value.
    ///
    /// [*Documentation taken from the corresponding vulkano method.*](https://docs.rs/vulkano/latest/vulkano/command_buffer/struct.AutoCommandBufferBuilder.html)
    pub fn clear_color_image<I>(
        self, 
        image: I, 
        color: ClearValue,
    ) -> Result<Self, ClearColorImageError>
    where
        I: ImageAccess + Send + Sync + 'static,
    {
        self.map_cb(move |cb| cb.clear_color_image(image, color))
    }

    /// Adds a command that clears a color image with a specific value.
    ///
    /// # Panic
    /// 
    /// Panics if color is not a color value.
    pub fn clear_color_image_dimensions<I>(
        self, 
        image: I, 
        first_layer: u32, 
        num_layers: u32, 
        first_mipmap: u32, 
        num_mipmaps: u32, 
        color: ClearValue
    ) -> Result<Self, ClearColorImageError>
    where
        I: ImageAccess + Send + Sync + 'static, 
    {
        self.map_cb(move |cb| {
            cb.clear_color_image_dimensions(
                image,
                first_layer,
                num_layers,
                first_mipmap,
                num_mipmaps,
                color,
            )
        })
    }

    /// Adds a command that copies from a buffer to another.
    /// 
    /// This command will copy from the source to the destination. If their size is not equal, then
    /// the amount of data copied is equal to the smallest of the two.
    pub fn copy_buffer<S, D, T>(
        self, 
        source: S, 
        destination: D
    ) -> Result<Self, CopyBufferError>
    where
        S: TypedBufferAccess<Content = T> + Send + Sync + 'static,
        D: TypedBufferAccess<Content = T> + Send + Sync + 'static,
        T: ?Sized,
    {
        self.map_cb(move |cb| cb.copy_buffer(source, destination))
    }

    /// Adds a command that copies from a buffer to an image.
    pub fn copy_buffer_to_image<S, D, Px>(
        self, 
        source: S, 
        destination: D
    ) -> Result<Self, CopyBufferImageError>
    where
        S: TypedBufferAccess<Content = [Px]> + Send + Sync + 'static,
        D: ImageAccess + Send + Sync + 'static,
        Format: AcceptsPixels<Px>, 
    {
        self.map_cb(move |cb| cb.copy_buffer_to_image(source, destination))
    }

    /// Adds a command that copies from a buffer to an image.
    pub fn copy_buffer_to_image_dimensions<S, D, Px>(
        self, 
        source: S, 
        destination: D, 
        offset: [u32; 3], 
        size: [u32; 3], 
        first_layer: u32, 
        num_layers: u32, 
        mipmap: u32
    ) -> Result<Self, CopyBufferImageError>
    where
        S: TypedBufferAccess<Content = [Px]> + Send + Sync + 'static,
        D: ImageAccess + Send + Sync + 'static,
        Format: AcceptsPixels<Px>, 
    {
        self.map_cb(move |cb| {
            cb.copy_buffer_to_image_dimensions(
                source,
                destination,
                offset,
                size,
                first_layer,
                num_layers,
                mipmap,
            )
        })
    }

    /// Adds a command that copies from an image to a buffer.
    pub fn copy_image_to_buffer<S, D, Px>(
        self, 
        source: S, 
        destination: D
    ) -> Result<Self, CopyBufferImageError>
    where
        S: ImageAccess + Send + Sync + 'static,
        D: TypedBufferAccess<Content = [Px]> + Send + Sync + 'static,
        Format: AcceptsPixels<Px>, 
    {
        self.map_cb(move |cb| cb.copy_image_to_buffer(source, destination))
    }

    /// Adds a command that copies from an image to a buffer.
    pub fn copy_image_to_buffer_dimensions<S, D, Px>(
        self, 
        source: S, 
        destination: D, 
        offset: [u32; 3], 
        size: [u32; 3], 
        first_layer: u32, 
        num_layers: u32, 
        mipmap: u32
    ) -> Result<Self, CopyBufferImageError>
    where
        S: ImageAccess + Send + Sync + 'static,
        D: TypedBufferAccess<Content = [Px]> + Send + Sync + 'static,
        Format: AcceptsPixels<Px>,
    {
        self.map_cb(move |cb| {
            cb.copy_image_to_buffer_dimensions(
                source,
                destination,
                offset,
                size,
                first_layer,
                num_layers,
                mipmap,
            )
        })
    }

    /// Draw once, using the vertex_buffer.
    /// 
    /// To use only some data in the buffer, wrap it in a `vulkano::buffer::BufferSlice`.
    pub fn draw<V, Gp, S, Pc>(
        self, 
        pipeline: Gp, 
        dynamic: &DynamicState, 
        vertex_buffer: V, 
        sets: S, 
        constants: Pc
    ) -> Result<Self, DrawError>
    where
        Gp: GraphicsPipelineAbstract + VertexSource<V> + Send + Sync + 'static + Clone,
        S: DescriptorSetsCollection,
    {
        self.map_cb(move |cb| {
            cb.draw(
                pipeline,
                dynamic,
                vertex_buffer,
                sets,
                constants,
            )
        })
    }


    /// Draw once, using the vertex_buffer and the index_buffer.
    /// 
    /// To use only some data in a buffer, wrap it in a `vulkano::buffer::BufferSlice`.
    pub fn draw_indexed<V, Gp, S, Pc, Ib, I>(
        self, 
        pipeline: Gp, 
        dynamic: &DynamicState, 
        vertex_buffer: V, 
        index_buffer: Ib, 
        sets: S, 
        constants: Pc
    ) -> Result<Self, DrawIndexedError>
    where
        Gp: GraphicsPipelineAbstract + VertexSource<V> + Send + Sync + 'static + Clone,
        S: DescriptorSetsCollection,
        Ib: BufferAccess + TypedBufferAccess<Content = [I]> + Send + Sync + 'static,
        I: Index + 'static,
    {
        self.map_cb(move |cb| {
            cb.draw_indexed(
                pipeline,
                dynamic,
                vertex_buffer,
                index_buffer,
                sets,
                constants,
            )
        })
    }

    /// Adds a command that writes the content of a buffer.
    ///
    /// This function is similar to the `memset` function in C. The `data` parameter is a number
    /// that will be repeatedly written through the entire buffer.
    ///
    /// > **Note**: This function is technically safe because buffers can only contain integers or
    /// > floating point numbers, which are always valid whatever their memory representation is.
    /// > But unless your buffer actually contains only 32-bits integers, you are encouraged to use
    /// > this function only for zeroing the content of a buffer by passing `0` for the data.
    pub fn fill_buffer<B>(self, buffer: B, data: u32) -> Result<Self, FillBufferError>
    where
        B: BufferAccess + Send + Sync + 'static,
    {
        self.map_cb(move |cb| cb.fill_buffer(buffer, data))
    }


    /// Adds a command that writes data to a buffer.
    /// 
    /// If data is larger than the buffer, only the part of data that fits is written. If the
    /// buffer is larger than data, only the start of the buffer is written.
    pub fn update_buffer<B, D>(
        self, 
        buffer: B, 
        data: D
    ) -> Result<Self, UpdateBufferError>
    where
        B: TypedBufferAccess<Content = D> + Send + Sync + 'static,
        D: Send + Sync + 'static,
    {
        self.map_cb(move |cb| cb.update_buffer(buffer, data))
    }
}
