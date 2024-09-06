use bevy::ecs::entity::EntityHashMap;
use bevy::prelude::*;
use bevy::render::render_resource::Extent3d;
use bevy::render::renderer::{RenderContext, RenderDevice};
use bevy::render::view::{ExtractedView, ExtractedWindows, ViewTarget};
use bevy::render::{Extract, RenderApp};
use nannou_core::geom;
use std::cell::RefCell;
use std::ops::Deref;

pub struct FramePlugin;

impl Plugin for FramePlugin {
    fn build(&self, app: &mut App) {
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .init_resource::<ExtractedWindowsScaleFactor>()
                .add_systems(ExtractSchedule, extract_scale_factors);
        }
    }
}

fn extract_scale_factors(
    mut window_scale_factors: ResMut<ExtractedWindowsScaleFactor>,
    windows_q: Extract<Query<(Entity, &Window)>>,
) {
    window_scale_factors.clear();
    for (entity, window) in windows_q.iter() {
        window_scale_factors.insert(entity, window.scale_factor());
    }
}

#[derive(Resource, Deref, DerefMut, Default)]
struct ExtractedWindowsScaleFactor(EntityHashMap<f32>);

pub struct Frame<'a, 'w> {
    window_id: Entity,
    world: &'w World,
    view_target: &'w ViewTarget,
    extracted_windows: &'w ExtractedWindows,
    extracted_view: &'w ExtractedView,
    render_device: &'w RenderDevice,
    render_context: RefCell<&'a mut RenderContext<'w>>,
}

impl<'a, 'w> Frame<'a, 'w> {
    pub const TEXTURE_FORMAT: wgpu::TextureFormat =
        nannou_wgpu::RenderPipelineBuilder::DEFAULT_COLOR_FORMAT;

    pub(crate) fn new(
        world: &'w World,
        view_target_id: Entity,
        view_target: &'w ViewTarget,
        extracted_windows: &'w ExtractedWindows,
        extracted_view: &'w ExtractedView,
        render_context: &'a mut RenderContext<'w>,
    ) -> Self {
        let render_device = world.resource::<RenderDevice>();
        Frame {
            window_id: view_target_id,
            world,
            view_target,
            render_device,
            render_context: RefCell::new(render_context),
            extracted_windows,
            extracted_view,
        }
    }

    /// Access the command encoder in order to encode commands that will be submitted to the swap
    /// chain queue at the end of the call to **view**.
    pub fn command_encoder(&self) -> std::cell::RefMut<wgpu::CommandEncoder> {
        std::cell::RefMut::map(self.render_context.borrow_mut(), |x| x.command_encoder())
    }

    /// The `Id` of the window whose wgpu surface is associated with this frame.
    pub fn window_id(&self) -> Entity {
        self.window_id
    }

    /// A **Rect** representing the full surface of the frame.
    ///
    /// The returned **Rect** is equivalent to the result of calling **Window::rect** on the window
    /// associated with this **Frame**.
    pub fn rect(&self) -> geom::Rect {
        let window = self.extracted_windows.windows.get(&self.window_id).unwrap();
        let scale_factor = self.world.resource::<ExtractedWindowsScaleFactor>();
        let scale_factor = scale_factor.get(&self.window_id).unwrap();
        let scale_factor = *scale_factor as f32;
        let [width, height] = [window.physical_width, window.physical_height];
        geom::Rect::from_x_y_w_h(
            0.0,
            0.0,
            width as f32 / scale_factor,
            height as f32 / scale_factor,
        )
    }

    /// The `nth` frame for the associated window since the application started.
    ///
    /// E.g. the first frame yielded will return `0`, the second will return `1`, and so on.
    pub fn nth(&self) -> u64 {
        todo!()
    }

    /// The swap chain texture that will be the target for drawing this frame.
    pub fn swap_chain_texture(&self) -> &wgpu::TextureView {
        self.view_target.out_texture()
    }

    /// The texture format of the frame's swap chain texture.
    pub fn swap_chain_texture_format(&self) -> wgpu::TextureFormat {
        self.view_target.out_texture_format()
    }

    /// The device and queue on which the swap chain was created and which will be used to submit
    /// the **RawFrame**'s encoded commands.
    ///
    /// This refers to the same **DeviceQueuePair** as held by the window associated with this
    /// frame.
    pub fn device(&self) -> &wgpu::Device {
        self.render_device.wgpu_device()
    }

    /// The texture to which all graphics should be drawn this frame.
    ///
    /// This is **not** the swapchain texture, but rather an intermediary linear sRGBA image. This
    /// intermediary image is used in order to:
    ///
    /// - Ensure consistent MSAA resolve behaviour across platforms.
    /// - Avoid the need for multiple implicit conversions to and from linear sRGBA for each
    /// graphics pipeline render pass that is used.
    /// - Allow for the user's rendered image to persist between frames.
    ///
    /// The exact format of the texture is equal to `Frame::TEXTURE_FORMAT`.
    ///
    /// If the number of MSAA samples specified is greater than `1` (which it is by default if
    /// supported by the platform), this will be a multisampled texture. After the **view**
    /// function returns, this texture will be resolved to a non-multisampled linear sRGBA texture.
    /// After the texture has been resolved if necessary, it will then be used as a shader input
    /// within a graphics pipeline used to draw the swapchain texture.
    pub fn texture(&self) -> &wgpu::Texture {
        self.view_target.main_texture()
    }

    /// A full view into the frame's texture.
    ///
    /// See `texture` for details.
    pub fn texture_view(&self) -> &wgpu::TextureView {
        self.view_target.main_texture_view()
    }

    /// Returns the resolve target texture in the case that MSAA is enabled.
    pub fn resolve_target(&self) -> Option<&wgpu::Texture> {
        self.view_target.sampled_main_texture().map(|x| x.deref())
    }

    /// Returns the resolve target texture view in the case that MSAA is enabled.
    pub fn resolve_target_view(&self) -> Option<&wgpu::TextureView> {
        self.view_target
            .sampled_main_texture_view()
            .map(|x| x.deref())
    }

    pub fn resolve_target_msaa_samples(&self) -> u32 {
        self.view_target
            .sampled_main_texture()
            .map(|x| x.sample_count())
            .unwrap_or(1)
    }

    /// The color format of the `Frame`'s intermediary linear sRGBA texture (equal to
    /// `Frame::TEXTURE_FORMAT`).
    pub fn texture_format(&self) -> wgpu::TextureFormat {
        self.view_target.main_texture_format()
    }

    /// The number of MSAA samples of the `Frame`'s intermediary linear sRGBA texture.
    pub fn texture_msaa_samples(&self) -> u32 {
        self.view_target.main_texture().sample_count()
    }

    /// The size of the frame's texture in pixels.
    pub fn texture_size(&self) -> [u32; 2] {
        let Extent3d { width, height, .. } = self.view_target.main_texture().size();
        [width, height]
    }

    /// Short-hand for constructing a `wgpu::RenderPassColorAttachment` for use within a
    /// render pass that targets this frame's texture. The returned descriptor's `attachment` will
    /// the same `wgpu::TextureView` returned by the `Frame::texture` method.
    ///
    /// Note that this method will not perform any resolving. In the case that `msaa_samples` is
    /// greater than `1`, a render pass will be automatically added after the `view` completes and
    /// before the texture is drawn to the swapchain.
    pub fn color_attachment_descriptor(&self) -> wgpu::RenderPassColorAttachment {
        self.view_target.get_color_attachment()
    }

    /// Clear the texture with the given color.
    pub fn clear<C>(&mut self, color: C)
    where
        C: Into<Color>,
    {
        let linear_color = color.into().to_linear();
        let view = self.view_target.main_texture_view();
        let mut render_ctx = self.render_context.borrow_mut();
        render_ctx.begin_tracked_render_pass(wgpu::RenderPassDescriptor {
            label: Some("nannou_frame_clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(linear_color.into()),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
    }
}
