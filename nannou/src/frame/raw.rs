//! The lower-level "raw" frame type allowing to draw directly to the window's swap chain image.

use crate::geom;
use crate::wgpu;
use crate::window;
use std::cell::{RefCell, RefMut};
use std::sync::Arc;

/// Allows the user to draw a single **RawFrame** to the surface of a window.
///
/// The application's **view** function is called each time the application is ready to retrieve a
/// new image that will be displayed to a window. The **RawFrame** type can be thought of as the
/// canvas to which you draw this image.
///
/// ## Under the hood - WGPU
///
/// **RawFrame** provides access to the **wgpu::TextureViewHandle** associated with the swap
/// chain's current target texture for a single window.
///
/// In the case that your **view** function is shared between multiple windows, can determine which
/// window the **RawFrame** is associated with via the **RawFrame::window_id** method.
///
/// The user can draw to the swap chain texture by building a list of commands via a
/// `wgpu::CommandEncoder` and submitting them to the `wgpu::Queue` associated with the
/// `wgpu::Device` that was used to create the swap chain. It is important that the queue
/// matches the device. In an effort to reduce the chance for errors to occur, **RawFrame**
/// provides access to a `wgpu::CommandEncoder` whose commands are guaranteed to be submitted to
/// the correct `wgpu::Queue` at the end of the **view** function.
pub struct RawFrame<'swap_chain> {
    command_encoder: Option<RefCell<wgpu::CommandEncoder>>,
    window_id: window::Id,
    nth: u64,
    swap_chain_texture: Option<&'swap_chain wgpu::TextureViewHandle>,
    device_queue_pair: Arc<wgpu::DeviceQueuePair>,
    texture_format: wgpu::TextureFormat,
    window_rect: geom::Rect,
}

impl<'swap_chain> RawFrame<'swap_chain> {
    fn new_inner(
        device_queue_pair: Arc<wgpu::DeviceQueuePair>,
        window_id: window::Id,
        nth: u64,
        swap_chain_texture: Option<&'swap_chain wgpu::TextureViewHandle>,
        texture_format: wgpu::TextureFormat,
        window_rect: geom::Rect,
    ) -> Self {
        let ce_desc = wgpu::CommandEncoderDescriptor {
            label: Some("nannou_raw_frame"),
        };
        let command_encoder = device_queue_pair.device().create_command_encoder(&ce_desc);
        let command_encoder = Some(RefCell::new(command_encoder));
        let frame = RawFrame {
            command_encoder,
            window_id,
            nth,
            swap_chain_texture,
            device_queue_pair,
            texture_format,
            window_rect,
        };
        frame
    }
    // Initialise a new empty frame ready for "drawing".
    pub(crate) fn new_empty(
        device_queue_pair: Arc<wgpu::DeviceQueuePair>,
        window_id: window::Id,
        nth: u64,
        swap_chain_texture: &'swap_chain wgpu::TextureViewHandle,
        texture_format: wgpu::TextureFormat,
        window_rect: geom::Rect,
    ) -> Self {
        Self::new_inner(
            device_queue_pair,
            window_id,
            nth,
            Some(swap_chain_texture),
            texture_format,
            window_rect,
        )
    }
    pub(crate) fn new_fake(
        device_queue_pair: Arc<wgpu::DeviceQueuePair>,
        nth: u64,
        texture_format: wgpu::TextureFormat,
        window_rect: geom::Rect,
    ) -> Self {
        Self::new_inner(
            device_queue_pair,
            unsafe { window::Id::dummy() },
            nth,
            None,
            texture_format,
            window_rect,
        )
    }
    // Submit the encoded commands to the queue of the device that was used to create the swap
    // chain texture.
    pub(crate) fn submit_inner(&mut self) {
        let command_encoder = self
            .command_encoder
            .take()
            .expect("the command encoder should always be `Some` at the time of submission")
            .into_inner();
        let command_buffer = command_encoder.finish();
        let queue = self.device_queue_pair.queue();
        queue.submit(std::iter::once(command_buffer));
    }

    // Allow the `Frame` to check if the raw frame has already been submitted on drop.
    pub(crate) fn is_submitted(&self) -> bool {
        self.command_encoder.is_none()
    }

    /// Access the command encoder in order to encode commands that will be submitted to the swap
    /// chain queue at the end of the call to **view**.
    pub fn command_encoder(&self) -> RefMut<wgpu::CommandEncoder> {
        match self.command_encoder {
            Some(ref ce) => ce.borrow_mut(),
            None => unreachable!("`RawFrame`'s command_encoder was `None`"),
        }
    }

    /// The `Id` of the window whose wgpu surface is associated with this frame.
    pub fn window_id(&self) -> window::Id {
        self.window_id
    }

    /// A **Rect** representing the full surface of the frame.
    ///
    /// The returned **Rect** is equivalent to the result of calling **Window::rect** on the window
    /// associated with this **Frame**.
    pub fn rect(&self) -> geom::Rect {
        self.window_rect
    }

    /// The `nth` frame for the associated window since the application started.
    ///
    /// E.g. the first frame yielded will return `0`, the second will return `1`, and so on.
    pub fn nth(&self) -> u64 {
        self.nth
    }

    /// The swap chain texture that will be the target for drawing this frame.
    pub fn swap_chain_texture(&self) -> Option<&wgpu::TextureViewHandle> {
        self.swap_chain_texture
    }

    /// The texture format of the frame's swap chain texture.
    pub fn texture_format(&self) -> wgpu::TextureFormat {
        self.texture_format
    }

    /// The device and queue on which the swap chain was created and which will be used to submit
    /// the **RawFrame**'s encoded commands.
    ///
    /// This refers to the same **DeviceQueuePair** as held by the window associated with this
    /// frame.
    pub fn device_queue_pair(&self) -> &Arc<wgpu::DeviceQueuePair> {
        &self.device_queue_pair
    }

    /// Submit the frame to the GPU!
    ///
    /// Specifically, this submits the encoded commands to the queue of the device that was used to
    /// create the swap chain texture.
    ///
    /// Note: You do not need to call this manually as submission will occur automatically when
    /// the **Frame** is dropped.
    ///
    /// Note: Be careful that you do not currently possess a lock to either the frame's command
    /// encoder *or* the queue of the window associated with this frame or this method will lock
    /// and block forever.
    pub fn submit(mut self) {
        self.submit_inner();
    }
}

impl<'swap_chain> Drop for RawFrame<'swap_chain> {
    fn drop(&mut self) {
        // Submit the commands if the user hasn't done so already.
        if !self.is_submitted() {
            self.submit_inner();
        }
    }
}
