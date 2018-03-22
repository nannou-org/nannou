use draw::properties::color::IntoRgba;
use glium::{self, Surface};
use glium::framebuffer::{MultiOutputFrameBuffer, SimpleFrameBuffer};
use glium::uniforms::{MagnifySamplerFilter, Uniforms};
use glium::vertex::MultiVerticesSource;
use glium::index::IndicesSource;
use std::collections::{hash_map, HashMap};
use std::cell::{RefMut, RefCell};
use std::ops::{Deref, DerefMut};
use window;

/// A **Frame** represents all graphics for the application for a single "frame" of time.
///
/// The **Frame** itself consists of a `WindowFrame` for each window in the `App`.
pub struct Frame {
    gl_frames: HashMap<window::Id, RefCell<GlFrame>>,
    main_window: Option<window::Id>,
}

/// An iterator yielding the `WindowFrame` for each open window in the application.
pub struct WindowFrames<'a> {
    iter: hash_map::Iter<'a, window::Id, RefCell<GlFrame>>,
}

impl Frame {
    /// Return the part of the `Frame` associated with the given window.
    pub fn window(&self, id: window::Id) -> Option<WindowFrame> {
        self.gl_frames
            .get(&id)
            .map(|wf| WindowFrame { frame: wf.borrow_mut() })
    }

    /// Return the part of the `Frame` associated with the main window.
    pub fn main_window(&self) -> WindowFrame {
        self.main_window
            .and_then(|id| self.window(id))
            .or_else(|| {
                self.gl_frames
                    .values()
                    .next()
                    .map(|wf| WindowFrame { frame: wf.borrow_mut() })
            })
            .expect("no `main_window` in `Frame`")
    }

    /// Return an iterator yielding each `window::Id` along with its `WindowFrame` for drawing.
    pub fn windows(&self) -> WindowFrames {
        let iter = self.gl_frames.iter();
        WindowFrames { iter }
    }

    /// Short-hand for clearing all windows with the given color.
    pub fn clear_all<C>(&self, color: C)
    where
        C: IntoRgba<f32>,
    {
        let rgba = color.into_rgba();
        for (_, mut frame) in self.windows() {
            frame.clear(rgba);
        }
    }
}

impl<'a> Iterator for WindowFrames<'a> {
    type Item = (window::Id, WindowFrame<'a>);
    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(&id, wf)| (id, WindowFrame { frame: wf.borrow_mut() }))
    }
}

// A function (private to the crate) for creating a new `Frame`.
pub fn new(gl_frames: HashMap<window::Id, RefCell<GlFrame>>, main_window: Option<window::Id>) -> Frame {
    Frame { gl_frames, main_window }
}

// A function (private to the crate) for finishing and submitting a `Frame`.
pub fn finish(Frame { gl_frames, .. }: Frame) -> Result<(), glium::SwapBuffersError> {
    for (_, gl_frame) in gl_frames {
        gl_frame.into_inner().frame.finish()?;
    }
    Ok(())
}

pub struct WindowFrame<'a> {
    pub(crate) frame: RefMut<'a, GlFrame>,
}

impl<'a> Deref for WindowFrame<'a> {
    type Target = RefMut<'a, GlFrame>;
    fn deref(&self) -> &Self::Target {
        &self.frame
    }
}

impl<'a> DerefMut for WindowFrame<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.frame
    }
}

/// A graphics surface, targeting the default framebuffer.
///
/// This is a simple wrapper around the `glium::Frame` type that restricts some undesired
/// functionality and removes the need for importing the `Surface` trait.
pub struct GlFrame {
    pub(crate) frame: glium::Frame,
}

impl GlFrame {
    pub fn new(frame: glium::Frame) -> Self {
        GlFrame { frame }
    }

    /// Clears some attachments of the target.
    pub fn clear_raw(
        &mut self,
        rect: Option<&glium::Rect>,
        color: Option<(f32, f32, f32, f32)>,
        color_srgb: bool,
        depth: Option<f32>,
        stencil: Option<i32>,
    ) {
        self.frame.clear(rect, color, color_srgb, depth, stencil);
    }

    /// The dimensions of the target in pixels.
    pub fn dimensions_pixels(&self) -> (u32, u32) {
        self.frame.get_dimensions()
    }

    /// The number of bits of each pixel of the depth buffer.
    pub fn depth_buffer_bits(&self) -> Option<u16> {
        self.frame.get_depth_buffer_bits()
    }

    /// The number of bits of each pixel of the stencil buffer.
    pub fn stencil_buffer_bits(&self) -> Option<u16> {
        self.frame.get_stencil_buffer_bits()
    }

    /// Performs the drawing.
    pub fn draw<'a, 'b, V, I, U>(
        &mut self,
        vertex_buffer: V,
        index_buffer: I,
        program: &glium::Program,
        uniforms: &U,
        draw_parameters: &glium::DrawParameters,
    ) -> Result<(), glium::DrawError>
    where
        I: Into<IndicesSource<'a>>,
        U: Uniforms,
        V: MultiVerticesSource<'b>,
    {
        self.frame.draw(vertex_buffer, index_buffer, program, uniforms, draw_parameters)
    }

    /// Copies a rectangle of pixels from this surface to another surface.
    pub fn blit_color<S>(
        &self,
        source_rect: &glium::Rect,
        target: &S,
        target_rect: &glium::BlitTarget,
        filter: MagnifySamplerFilter,
    )
    where
        S: Surface,
    {
        self.frame.blit_color(source_rect, target, target_rect, filter)
    }

    /// Blits from the default framebuffer.
    pub fn blit_from_frame(
        &self,
        source_rect: &glium::Rect,
        target_rect: &glium::BlitTarget,
        filter: MagnifySamplerFilter,
    ) {
        self.frame.blit_from_frame(source_rect, target_rect, filter)
    }

    /// Blits from a simple framebuffer.
    pub fn blit_from_simple_framebuffer(
        &self,
        source: &SimpleFrameBuffer,
        source_rect: &glium::Rect,
        target_rect: &glium::BlitTarget,
        filter: MagnifySamplerFilter,
    ) {
        self.frame.blit_from_simple_framebuffer(source, source_rect, target_rect, filter)
    }

    /// Blits from a multi-output framebuffer.
    pub fn blit_from_multioutput_framebuffer(
        &self,
        source: &MultiOutputFrameBuffer,
        source_rect: &glium::Rect,
        target_rect: &glium::BlitTarget,
        filter: MagnifySamplerFilter,
    ) {
        self.frame.blit_from_multioutput_framebuffer(source, source_rect, target_rect, filter)
    }

    /// Copies the entire surface to a target surface.
    pub fn blit_whole_color_to<S>(
        &self,
        target: &S,
        target_rect: &glium::BlitTarget,
        filter: MagnifySamplerFilter,
    )
    where
        S: Surface,
    {
        self.frame.blit_whole_color_to(target, target_rect, filter)
    }

    /// Copies the entire surface to a target surface.
    pub fn fill<S>(&self, target: &S, filter: MagnifySamplerFilter)
        where S: Surface,
    {
        self.frame.fill(target, filter)
    }

    /// Clears the color attachment of the target.
    pub fn clear<C>(&mut self, color: C)
    where
        C: IntoRgba<f32>,
    {
        let color = color.into_rgba();
        self.clear_color(color.red, color.green, color.blue, color.alpha);
    }

    /// Clears the color attachment of the target.
    pub fn clear_color(&mut self, red: f32, green: f32, blue: f32, alpha: f32) {
        self.frame.clear_color(red, green, blue, alpha)
    }

    /// Clears the color attachment of the target.
    ///
    /// The color is in sRGB format.
    pub fn clear_color_srgb(&mut self, red: f32, green: f32, blue: f32, alpha: f32) {
        self.frame.clear_color_srgb(red, green, blue, alpha)
    }

    /// Clears the depth attachment of the target.
    pub fn clear_depth(&mut self, value: f32) {
        self.frame.clear_depth(value)
    }

    /// Clears the stencil attachment of the target.
    pub fn clear_stencil(&mut self, value: i32) {
        self.frame.clear_stencil(value)
    }

    /// Clears the color and depth attachments of the target.
    pub fn clear_color_and_depth(&mut self, color: (f32, f32, f32, f32), depth: f32) {
        self.frame.clear_color_and_depth(color, depth)
    }

    /// Clears the color and depth attachments of the target.
    pub fn clear_color_srgb_and_depth(&mut self, color: (f32, f32, f32, f32), depth: f32) {
        self.frame.clear_color_srgb_and_depth(color, depth)
    }

    /// Clears the color and stencil attachments of the target.
    pub fn clear_color_and_stencil(&mut self, color: (f32, f32, f32, f32), stencil: i32) {
        self.frame.clear_color_and_stencil(color, stencil)
    }

    /// Clears the color and stencil attachments of the target.
    pub fn clear_color_srgb_and_stencil(&mut self, color: (f32, f32, f32, f32), stencil: i32) {
        self.frame.clear_color_srgb_and_stencil(color, stencil)
    }

    /// Clears the depth and stencil attachments of the target.
    pub fn clear_depth_and_stencil(&mut self, depth: f32, stencil: i32) {
        self.frame.clear_depth_and_stencil(depth, stencil)
    }

    /// Clears the color, depth and stencil attachments of the target.
    pub fn clear_all(&mut self, color: (f32, f32, f32, f32), depth: f32, stencil: i32) {
        self.frame.clear_all(color, depth, stencil)
    }

    /// Clears the color, depth and stencil attachments of the target.
    ///
    /// The color is in sRGB format.
    pub fn clear_all_srgb(&mut self, color: (f32, f32, f32, f32), depth: f32, stencil: i32) {
        self.frame.clear_all_srgb(color, depth, stencil)
    }

    /// True if the surface has a depth buffer available.
    pub fn has_depth_buffer(&self) -> bool {
        self.frame.has_depth_buffer()
    }

    /// True if the surface has a stencil buffer available.
    pub fn has_stencil_buffer(&self) -> bool {
        self.frame.has_stencil_buffer()
    }
}
