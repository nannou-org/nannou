//! Items related to the **Frame** - a window's render target for a single update.
//!
//! A [`Frame`] wraps Bevy's [`ViewTarget`] for a particular window, giving access to the
//! intermediary texture that nannou draws into and the window's final surface texture, along with
//! the GPU device and render context used to encode draw commands.

use bevy::ecs::entity::EntityHashMap;
use bevy::prelude::*;
use bevy::render::render_resource::Extent3d;
use bevy::render::renderer::{RenderContext, RenderDevice};
use bevy::render::view::{ExtractedWindows, ViewTarget};
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

/// The per-window scale factors extracted into the render world by [`FramePlugin`].
///
/// Take this as a `Res` in a render-world system to construct a [`Frame`] via [`Frame::new`].
#[derive(Resource, Deref, DerefMut, Default)]
pub struct ExtractedWindowsScaleFactor(EntityHashMap<f32>);

pub struct Frame<'a, 'r, 'w, 's> {
    window_id: Entity,
    view_target: &'r ViewTarget,
    extracted_windows: &'r ExtractedWindows,
    scale_factors: &'r ExtractedWindowsScaleFactor,
    render_device: &'r RenderDevice,
    render_context: RefCell<&'a mut RenderContext<'w, 's>>,
}

impl<'a, 'r, 'w, 's> Frame<'a, 'r, 'w, 's> {
    pub const TEXTURE_FORMAT: wgpu::TextureFormat =
        nannou_wgpu::RenderPipelineBuilder::DEFAULT_COLOR_FORMAT;

    /// Construct a `Frame` from render-world resources.
    ///
    /// Use this to do custom wgpu rendering from your own render-world system: add
    /// [`FramePlugin`] (bundled in [`NannouPlugin`](crate::NannouPlugin)), then take the
    /// [`RenderContext`], a [`ViewTarget`] (e.g. via `ViewQuery`), and `Res`-access to
    /// [`ExtractedWindows`], [`RenderDevice`] and [`ExtractedWindowsScaleFactor`]. See
    /// `nannou::render` and the `wgpu_*` examples for the pattern the classic `app(..).render(..)`
    /// builder uses internally.
    pub fn new(
        render_device: &'r RenderDevice,
        scale_factors: &'r ExtractedWindowsScaleFactor,
        view_target_id: Entity,
        view_target: &'r ViewTarget,
        extracted_windows: &'r ExtractedWindows,
        render_context: &'a mut RenderContext<'w, 's>,
    ) -> Self {
        Frame {
            window_id: view_target_id,
            view_target,
            render_device,
            scale_factors,
            render_context: RefCell::new(render_context),
            extracted_windows,
        }
    }

    /// Access the command encoder in order to encode commands that will be submitted to the GPU
    /// queue at the end of the call to **view**.
    pub fn command_encoder(&self) -> std::cell::RefMut<'_, wgpu::CommandEncoder> {
        std::cell::RefMut::map(self.render_context.borrow_mut(), |x| x.command_encoder())
    }

    /// The [`Entity`] of the window whose surface is associated with this frame.
    pub fn window_id(&self) -> Entity {
        self.window_id
    }

    /// A **Rect** representing the full surface of the frame.
    ///
    /// The returned **Rect** is equivalent to the result of calling **Window::rect** on the window
    /// associated with this **Frame**.
    pub fn rect(&self) -> geom::Rect {
        let window = self.extracted_windows.windows.get(&self.window_id).unwrap();
        let scale_factor = self.scale_factors.get(&self.window_id).unwrap();
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

    /// The window's surface texture that will be the target for presenting this frame.
    pub fn swap_chain_texture(&self) -> &wgpu::TextureView {
        // Bevy 0.19's `ViewTarget` exposes the output texture as an `Option`.
        self.view_target
            .out_texture()
            .expect("frame has no output texture")
    }

    /// The texture format of the window's surface texture.
    pub fn swap_chain_texture_format(&self) -> wgpu::TextureFormat {
        self.view_target
            .out_texture_view_format()
            .expect("frame has no output texture")
    }

    /// The GPU device used to submit this frame's encoded commands.
    pub fn device(&self) -> &wgpu::Device {
        self.render_device.wgpu_device()
    }

    /// The texture to which all graphics should be drawn this frame.
    ///
    /// This is **not** the window's surface texture, but rather an intermediary linear sRGBA image. This
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
    /// within a graphics pipeline used to draw to the window's surface texture.
    pub fn texture(&self) -> &wgpu::Texture {
        self.view_target.main_texture()
    }

    /// A full view into the frame's single-sample texture.
    ///
    /// This is the texture that is presented to the window's surface (after tonemapping).
    /// When MSAA is enabled it is the *resolve target* - the multisampled texture returned by
    /// [`Frame::resolve_target_view`] is resolved into it. Prefer [`Frame::color_attachment`],
    /// which wires this up for you, over targeting these views by hand.
    ///
    /// See `texture` for details.
    pub fn texture_view(&self) -> &wgpu::TextureView {
        self.view_target.main_texture_view()
    }

    /// Returns the multisampled texture in the case that MSAA is enabled.
    ///
    /// This is the texture a render pass should draw *into* when MSAA is enabled; it is
    /// resolved into the single-sample [`Frame::texture`] before presentation.
    pub fn resolve_target(&self) -> Option<&wgpu::Texture> {
        self.view_target.sampled_main_texture().map(|x| x.deref())
    }

    /// Returns the multisampled texture view in the case that MSAA is enabled.
    ///
    /// This is the view a render pass should draw *into* when MSAA is enabled; it is resolved
    /// into the single-sample [`Frame::texture_view`] before presentation. Prefer
    /// [`Frame::color_attachment`], which wires this up for you.
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

    /// A [`wgpu::RenderPassColorAttachment`] that targets this frame, with the MSAA resolve
    /// already wired up.
    ///
    /// This is the recommended way to target the frame from a custom render pass. When MSAA is
    /// enabled the frame's multisampled texture is used as the attachment and is automatically
    /// resolved into the single-sample texture presented to the window's surface; when MSAA is
    /// disabled the single-sample texture is targeted directly. Either way the result is read
    /// by Bevy's tonemapping pass and drawn to the surface.
    ///
    /// `load` selects how the target is initialised at the start of the pass - e.g.
    /// [`wgpu::LoadOp::Clear`] to clear it each frame, or [`wgpu::LoadOp::Load`] to build upon
    /// the previous frame's contents.
    pub fn color_attachment(
        &self,
        load: wgpu::LoadOp<wgpu::Color>,
    ) -> wgpu::RenderPassColorAttachment<'_> {
        let (view, resolve_target): (&wgpu::TextureView, Option<&wgpu::TextureView>) =
            match self.view_target.sampled_main_texture_view() {
                Some(sampled) => (sampled, Some(self.view_target.main_texture_view())),
                None => (self.view_target.main_texture_view(), None),
            };
        wgpu::RenderPassColorAttachment {
            view,
            resolve_target,
            ops: wgpu::Operations {
                load,
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        }
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
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    }
}
