pub use egui;
pub use egui::color_picker;
pub use egui_wgpu_backend;

use egui::{pos2, ClippedMesh, CtxRef};
use egui_wgpu_backend::{epi, ScreenDescriptor};
use nannou::{wgpu, winit::event::VirtualKeyCode, winit::event::WindowEvent::*};
use std::{
    cell::RefCell,
    ops::Deref,
    sync::{Arc, Mutex},
    time::Duration,
};

/// All `egui`-related state for a single window.
///
/// Includes the context, a renderer, and an input tracker.
///
/// For multi-window user interfaces, you will need to create an instance of this type per-window.
pub struct Egui {
    context: CtxRef,
    renderer: RefCell<Renderer>,
    input: Input,
}

/// A wrapper around all necessary state for rendering a `Egui` to a single texture (often a window
/// texture).
///
/// For targeting more than one window, users should construct a `Egui` for each.
pub struct Renderer {
    render_pass: egui_wgpu_backend::RenderPass,
    paint_jobs: Vec<ClippedMesh>,
}

/// Tracking user and application event input.
pub struct Input {
    pub pointer_pos: egui::Pos2,
    pub raw: egui::RawInput,
    pub window_size_pixels: [u32; 2],
    pub window_scale_factor: f32,
}

/// A wrapper around a `CtxRef` on which `begin_frame` was called.
///
/// Automatically calls `end_frame` on `drop` in the case that it wasn't already called by the
/// usef.
pub struct FrameCtx<'a> {
    ui: &'a mut Egui,
    ended: bool,
}

struct RepaintSignal(Mutex<nannou::app::Proxy>);

impl Egui {
    /// Construct the `Egui` from its parts.
    ///
    /// The given `device` must be the same used to create the queue to which the Egui's render
    /// commands will be submitted.
    ///
    /// The `target_format`, `target_msaa_samples`, `window_scale_factor` and `window_size_pixels`
    /// must match the window to which the UI will be drawn.
    ///
    /// The `context` should have the desired initial styling and fonts already set.
    pub fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        target_msaa_samples: u32,
        window_scale_factor: f32,
        window_size_pixels: [u32; 2],
    ) -> Self {
        let renderer = RefCell::new(Renderer::new(device, target_format, target_msaa_samples));
        let input = Input::new(window_scale_factor, window_size_pixels);
        let context = Default::default();
        Self {
            renderer,
            input,
            context,
        }
    }

    /// Construct a `Egui` associated with the given window.
    pub fn from_window(window: &nannou::window::Window) -> Self {
        let device = window.swap_chain_device();
        let format = nannou::Frame::TEXTURE_FORMAT;
        let msaa_samples = window.msaa_samples();
        let scale_factor = window.scale_factor();
        let (w_px, h_px) = window.inner_size_pixels();
        Self::new(device, format, msaa_samples, scale_factor, [w_px, h_px])
    }

    /// Access to the inner `egui::CtxRef`.
    pub fn ctx(&self) -> &CtxRef {
        &self.context
    }

    /// Access to the currently tracked input state.
    pub fn input(&self) -> &Input {
        &self.input
    }

    /// Handles a raw window event, tracking all input and events relevant to the UI as necessary.
    pub fn handle_raw_event(&mut self, event: &winit::event::WindowEvent) {
        self.input.handle_raw_event(event);
    }

    /// Set the elapsed time since the `Egui` app started running.
    pub fn set_elapsed_time(&mut self, elapsed: Duration) {
        self.input.set_elapsed_time(elapsed);
    }

    /// Begin describing a UI frame.
    pub fn begin_frame(&mut self) -> FrameCtx {
        self.begin_frame_inner();
        let ui = self;
        let ended = false;
        FrameCtx { ui, ended }
    }

    /// Draws the contents of the inner `context` to the given frame.
    pub fn draw_to_frame(&self, frame: &nannou::Frame) {
        let mut renderer = self.renderer.borrow_mut();
        renderer.draw_to_frame(&self.context, frame);
    }

    /// Provide access to an `epi::Frame` within the given function.
    ///
    /// This method is primarily used for apps based on the `epi` interface.
    pub fn with_epi_frame<F>(&mut self, proxy: nannou::app::Proxy, f: F)
    where
        F: FnOnce(&CtxRef, &mut epi::Frame),
    {
        let mut renderer = self.renderer.borrow_mut();
        let integration_info = epi::IntegrationInfo {
            native_pixels_per_point: Some(self.input.window_scale_factor as _),
            // TODO: Provide access to this stuff.
            web_info: None,
            prefer_dark_mode: None,
            cpu_usage: None,
            seconds_since_midnight: None,
        };
        let mut app_output = epi::backend::AppOutput::default();
        let repaint_signal = Arc::new(RepaintSignal(Mutex::new(proxy)));
        let mut frame = epi::backend::FrameBuilder {
            info: integration_info,
            tex_allocator: &mut renderer.render_pass,
            // TODO: We may want to support a http feature for hyperlinks?
            // #[cfg(feature = "http")]
            // http: http.clone(),
            output: &mut app_output,
            repaint_signal: repaint_signal as Arc<_>,
        }
        .build();
        f(&self.context, &mut frame)
    }

    /// The same as `with_epi_frame`, but calls `begin_frame` before calling the given function,
    /// and then calls `end_frame` before returning.
    pub fn do_frame_with_epi_frame<F>(&mut self, proxy: nannou::app::Proxy, f: F)
    where
        F: FnOnce(&CtxRef, &mut epi::Frame),
    {
        self.begin_frame_inner();
        self.with_epi_frame(proxy, f);
        self.end_frame_inner();
    }

    fn begin_frame_inner(&mut self) {
        self.context.begin_frame(self.input.raw.take());
    }

    fn end_frame_inner(&mut self) {
        let (_, paint_cmds) = self.context.end_frame();
        self.renderer.borrow_mut().paint_jobs = self.context.tessellate(paint_cmds);
    }
}

impl Input {
    /// Initialise user input and window event tracking with the given target scale factor and size
    /// in pixels.
    pub fn new(window_scale_factor: f32, window_size_pixels: [u32; 2]) -> Self {
        let raw = egui::RawInput {
            pixels_per_point: Some(window_scale_factor),
            ..Default::default()
        };
        let pointer_pos = Default::default();
        let mut input = Self {
            raw,
            pointer_pos,
            window_scale_factor,
            window_size_pixels,
        };
        input.raw.screen_rect = Some(input.egui_window_rect());
        input
    }

    /// Handles a raw window event, tracking all input and events relevant to the UI as necessary.
    pub fn handle_raw_event(&mut self, event: &winit::event::WindowEvent) {
        match event {
            Resized(physical_size) => {
                self.window_size_pixels = [physical_size.width, physical_size.height];
                self.raw.screen_rect = Some(self.egui_window_rect());
            }
            ScaleFactorChanged {
                scale_factor,
                new_inner_size,
            } => {
                self.window_scale_factor = *scale_factor as f32;
                self.window_size_pixels = [new_inner_size.width, new_inner_size.height];
                self.raw.pixels_per_point = Some(self.window_scale_factor);
                self.raw.screen_rect = Some(self.egui_window_rect());
            }
            MouseInput { state, button, .. } => {
                if let winit::event::MouseButton::Other(..) = button {
                } else {
                    self.raw.events.push(egui::Event::PointerButton {
                        pos: self.pointer_pos,
                        button: match button {
                            winit::event::MouseButton::Left => egui::PointerButton::Primary,
                            winit::event::MouseButton::Right => egui::PointerButton::Secondary,
                            winit::event::MouseButton::Middle => egui::PointerButton::Middle,
                            winit::event::MouseButton::Other(_) => unreachable!(),
                        },
                        pressed: *state == winit::event::ElementState::Pressed,
                        modifiers: self.raw.modifiers,
                    });
                }
            }
            MouseWheel { delta, .. } => {
                match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => {
                        let line_height = 24.0;
                        self.raw.scroll_delta = egui::vec2(*x, *y) * line_height;
                    }
                    winit::event::MouseScrollDelta::PixelDelta(delta) => {
                        // Actually point delta
                        self.raw.scroll_delta = egui::vec2(delta.x as f32, delta.y as f32);
                    }
                }
            }
            CursorMoved { position, .. } => {
                self.pointer_pos = pos2(
                    position.x as f32 / self.window_scale_factor as f32,
                    position.y as f32 / self.window_scale_factor as f32,
                );
                self.raw
                    .events
                    .push(egui::Event::PointerMoved(self.pointer_pos));
            }
            CursorLeft { .. } => {
                self.raw.events.push(egui::Event::PointerGone);
            }
            ModifiersChanged(input) => {
                self.raw.modifiers = winit_to_egui_modifiers(*input);
            }
            KeyboardInput { input, .. } => {
                if let Some(virtual_keycode) = input.virtual_keycode {
                    if let Some(key) = winit_to_egui_key_code(virtual_keycode) {
                        // TODO figure out why if I enable this the characters get ignored
                        self.raw.events.push(egui::Event::Key {
                            key,
                            pressed: input.state == winit::event::ElementState::Pressed,
                            modifiers: self.raw.modifiers,
                        });
                    }
                }
            }
            ReceivedCharacter(ch) => {
                if is_printable(*ch) && !self.raw.modifiers.ctrl && !self.raw.modifiers.command {
                    self.raw.events.push(egui::Event::Text(ch.to_string()));
                }
            }
            _ => {}
        }
    }

    /// Set the elapsed time since the `Egui` app started running.
    pub fn set_elapsed_time(&mut self, elapsed: Duration) {
        self.raw.time = Some(elapsed.as_secs_f64());
    }

    /// Small helper for the common task of producing an `egui::Rect` describing the window.
    fn egui_window_rect(&self) -> egui::Rect {
        let [w, h] = self.window_size_pixels;
        egui::Rect::from_min_size(
            Default::default(),
            egui::vec2(w as f32, h as f32) / self.window_scale_factor as f32,
        )
    }
}

impl Renderer {
    /// Create a new `Renderer` from its parts.
    ///
    /// The `device` must be the same that was used to create the queue to which the `Renderer`s
    /// render passes will be submitted.
    ///
    /// The `target_format` and `target_msaa_samples` should describe the target texture to which
    /// the `Egui` will be rendered.
    pub fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        target_msaa_samples: u32,
    ) -> Self {
        Self {
            render_pass: egui_wgpu_backend::RenderPass::new(
                device,
                target_format,
                target_msaa_samples,
            ),
            paint_jobs: Vec::new(),
        }
    }

    /// Construct a `Renderer` ready for drawing to the given window.
    pub fn from_window(window: &nannou::window::Window) -> Self {
        let device = window.swap_chain_device();
        let format = nannou::Frame::TEXTURE_FORMAT;
        let msaa_samples = window.msaa_samples();
        Self::new(device, format, msaa_samples)
    }

    /// Encode a render pass for drawing the given context's texture to the given `dst_texture`.
    pub fn encode_render_pass(
        &mut self,
        context: &CtxRef,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        dst_size_pixels: [u32; 2],
        dst_scale_factor: f32,
        dst_texture: &wgpu::TextureView,
    ) {
        let render_pass = &mut self.render_pass;
        let paint_jobs = &self.paint_jobs;
        let [physical_width, physical_height] = dst_size_pixels;
        let screen_descriptor = ScreenDescriptor {
            physical_width,
            physical_height,
            scale_factor: dst_scale_factor,
        };
        render_pass.update_texture(device, queue, &context.texture());
        render_pass.update_user_textures(&device, &queue);
        render_pass.update_buffers(device, queue, &paint_jobs, &screen_descriptor);
        render_pass.execute(encoder, dst_texture, &paint_jobs, &screen_descriptor, None);
    }

    /// Encodes a render pass for drawing the given context's texture to the given frame.
    pub fn draw_to_frame(&mut self, context: &CtxRef, frame: &nannou::Frame) {
        let device_queue_pair = frame.device_queue_pair();
        let device = device_queue_pair.device();
        let queue = device_queue_pair.queue();
        let size_pixels = frame.texture_size();
        let [width_px, _] = size_pixels;
        let scale_factor = width_px as f32 / frame.rect().w();
        let texture_view = frame.texture_view();
        let mut encoder = frame.command_encoder();
        self.encode_render_pass(
            context,
            device,
            queue,
            &mut encoder,
            size_pixels,
            scale_factor,
            texture_view,
        );
    }
}

impl<'a> FrameCtx<'a> {
    /// Produces a `CtxRef` ready for describing the UI for this frame.
    pub fn context(&self) -> CtxRef {
        self.ui.context.clone()
    }

    /// End the current frame,
    pub fn end(mut self) {
        self.end_inner();
    }

    // The inner `end` implementation, shared between `end` and `drop`.
    fn end_inner(&mut self) {
        if !self.ended {
            self.ui.end_frame_inner();
            self.ended = true;
        }
    }
}

impl<'a> Drop for FrameCtx<'a> {
    fn drop(&mut self) {
        self.end_inner();
    }
}

impl<'a> Deref for FrameCtx<'a> {
    type Target = egui::CtxRef;
    fn deref(&self) -> &Self::Target {
        &self.ui.context
    }
}

impl epi::RepaintSignal for RepaintSignal {
    fn request_repaint(&self) {
        if let Ok(guard) = self.0.lock() {
            guard.wakeup().ok();
        }
    }
}

/// Translates winit to egui keycodes.
#[inline]
fn winit_to_egui_key_code(key: VirtualKeyCode) -> Option<egui::Key> {
    use egui::Key;

    Some(match key {
        VirtualKeyCode::Escape => Key::Escape,
        VirtualKeyCode::Insert => Key::Insert,
        VirtualKeyCode::Home => Key::Home,
        VirtualKeyCode::Delete => Key::Delete,
        VirtualKeyCode::End => Key::End,
        VirtualKeyCode::PageDown => Key::PageDown,
        VirtualKeyCode::PageUp => Key::PageUp,
        VirtualKeyCode::Left => Key::ArrowLeft,
        VirtualKeyCode::Up => Key::ArrowUp,
        VirtualKeyCode::Right => Key::ArrowRight,
        VirtualKeyCode::Down => Key::ArrowDown,
        VirtualKeyCode::Back => Key::Backspace,
        VirtualKeyCode::Return => Key::Enter,
        VirtualKeyCode::Tab => Key::Tab,
        VirtualKeyCode::Space => Key::Space,

        VirtualKeyCode::A => Key::A,
        VirtualKeyCode::B => Key::B,
        VirtualKeyCode::C => Key::C,
        VirtualKeyCode::D => Key::D,
        VirtualKeyCode::E => Key::E,
        VirtualKeyCode::F => Key::F,
        VirtualKeyCode::G => Key::G,
        VirtualKeyCode::H => Key::H,
        VirtualKeyCode::I => Key::I,
        VirtualKeyCode::J => Key::J,
        VirtualKeyCode::K => Key::K,
        VirtualKeyCode::L => Key::L,
        VirtualKeyCode::M => Key::M,
        VirtualKeyCode::N => Key::N,
        VirtualKeyCode::O => Key::O,
        VirtualKeyCode::P => Key::P,
        VirtualKeyCode::Q => Key::Q,
        VirtualKeyCode::R => Key::R,
        VirtualKeyCode::S => Key::S,
        VirtualKeyCode::T => Key::T,
        VirtualKeyCode::U => Key::U,
        VirtualKeyCode::V => Key::V,
        VirtualKeyCode::W => Key::W,
        VirtualKeyCode::X => Key::X,
        VirtualKeyCode::Y => Key::Y,
        VirtualKeyCode::Z => Key::Z,

        VirtualKeyCode::Key0 => Key::Num0,
        VirtualKeyCode::Key1 => Key::Num1,
        VirtualKeyCode::Key2 => Key::Num2,
        VirtualKeyCode::Key3 => Key::Num3,
        VirtualKeyCode::Key4 => Key::Num4,
        VirtualKeyCode::Key5 => Key::Num5,
        VirtualKeyCode::Key6 => Key::Num6,
        VirtualKeyCode::Key7 => Key::Num7,
        VirtualKeyCode::Key8 => Key::Num8,
        VirtualKeyCode::Key9 => Key::Num9,

        _ => {
            return None;
        }
    })
}

/// Translates winit to egui modifier keys.
#[inline]
fn winit_to_egui_modifiers(modifiers: winit::event::ModifiersState) -> egui::Modifiers {
    egui::Modifiers {
        alt: modifiers.alt(),
        ctrl: modifiers.ctrl(),
        shift: modifiers.shift(),
        #[cfg(target_os = "macos")]
        mac_cmd: modifiers.logo(),
        #[cfg(target_os = "macos")]
        command: modifiers.logo(),
        #[cfg(not(target_os = "macos"))]
        mac_cmd: false,
        #[cfg(not(target_os = "macos"))]
        command: modifiers.ctrl(),
    }
}

pub fn edit_color(ui: &mut egui::Ui, color: &mut nannou::color::Hsv) {
    let mut egui_hsv = egui::color::Hsva::new(
        color.hue.to_positive_radians() as f32 / (std::f32::consts::PI * 2.0),
        color.saturation,
        color.value,
        1.0,
    );

    if egui::color_picker::color_edit_button_hsva(
        ui,
        &mut egui_hsv,
        egui::color_picker::Alpha::Opaque,
    )
    .changed()
    {
        *color = nannou::color::hsv(egui_hsv.h, egui_hsv.s, egui_hsv.v);
    }
}

/// We only want printable characters and ignore all special keys.
fn is_printable(chr: char) -> bool {
    let is_in_private_use_area = '\u{e000}' <= chr && chr <= '\u{f8ff}'
        || '\u{f0000}' <= chr && chr <= '\u{ffffd}'
        || '\u{100000}' <= chr && chr <= '\u{10fffd}';
    !is_in_private_use_area && !chr.is_ascii_control()
}
