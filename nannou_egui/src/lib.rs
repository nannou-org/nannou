use std::{borrow::BorrowMut, cell::RefCell, sync::{Arc, Mutex}};

pub use egui;
pub use egui::color_picker;
pub use egui_wgpu_backend;

use egui::{pos2, ClippedMesh, CtxRef};
use egui_wgpu_backend::{epi, ScreenDescriptor};
use winit::event::VirtualKeyCode;
use winit::event::WindowEvent::*;

const OUTPUT_FORMAT: egui_wgpu_backend::wgpu::TextureFormat = nannou::Frame::TEXTURE_FORMAT;

pub struct EguiBackend {
    render_pass: RefCell<egui_wgpu_backend::RenderPass>,
    raw_input: egui::RawInput,
    modifier_state: winit::event::ModifiersState,
    pointer_pos: egui::Pos2,
    width: u32,
    height: u32,
    scale_factor: f64,
    context: egui::CtxRef,
    paint_jobs: Vec<ClippedMesh>,
    repaint_signal: Arc<dyn epi::RepaintSignal + 'static>,
}

struct RepaintSignal(Mutex<nannou::app::Proxy>);

impl EguiBackend {
    pub fn from_window(window: &nannou::window::Window, proxy: nannou::app::Proxy) -> EguiBackend {
        let scale_factor = window.scale_factor() as f64;
        let width = window.inner_size_pixels().0;
        let height = window.inner_size_pixels().1;

        let raw_input = egui::RawInput {
            pixels_per_point: Some(scale_factor as f32),
            screen_rect: Some(egui::Rect::from_min_size(
                Default::default(),
                egui::vec2(width as f32, height as f32) / scale_factor as f32,
            )),
            ..Default::default()
        };

        let context = egui::CtxRef::default();
        context.set_fonts(egui::FontDefinitions::default());
        context.set_style(egui::Style::default());

        EguiBackend {
            render_pass: RefCell::new(egui_wgpu_backend::RenderPass::new(
                window.swap_chain_device(),
                OUTPUT_FORMAT,
                window.msaa_samples(),
            )),
            context,
            modifier_state: winit::event::ModifiersState::empty(),
            width,
            height,
            scale_factor,
            raw_input,
            pointer_pos: Default::default(),
            paint_jobs: Vec::new(),
            repaint_signal: Arc::new(RepaintSignal(Mutex::new(proxy))) as _,
        }
    }

    pub fn handle_event(&mut self, event: &winit::event::WindowEvent) {
        let mut raw_input = &mut self.raw_input;
        match event {
            Resized(physical_size) => {
                self.width = physical_size.width;
                self.height = physical_size.height;
                raw_input.screen_rect = Some(egui::Rect::from_min_size(
                    Default::default(),
                    egui::vec2(physical_size.width as f32, physical_size.height as f32)
                        / self.scale_factor as f32,
                ));
            }
            ScaleFactorChanged {
                scale_factor,
                new_inner_size,
            } => {
                self.scale_factor = *scale_factor;
                raw_input.pixels_per_point = Some(*scale_factor as f32);
                raw_input.screen_rect = Some(egui::Rect::from_min_size(
                    Default::default(),
                    egui::vec2(new_inner_size.width as f32, new_inner_size.height as f32)
                        / self.scale_factor as f32,
                ));
            }
            MouseInput { state, button, .. } => {
                if let winit::event::MouseButton::Other(..) = button {
                } else {
                    raw_input.events.push(egui::Event::PointerButton {
                        pos: self.pointer_pos,
                        button: match button {
                            winit::event::MouseButton::Left => egui::PointerButton::Primary,
                            winit::event::MouseButton::Right => egui::PointerButton::Secondary,
                            winit::event::MouseButton::Middle => egui::PointerButton::Middle,
                            winit::event::MouseButton::Other(_) => unreachable!(),
                        },
                        pressed: *state == winit::event::ElementState::Pressed,
                        modifiers: Default::default(),
                    });
                }
            }
            MouseWheel { delta, .. } => {
                match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => {
                        let line_height = 24.0;
                        raw_input.scroll_delta = egui::vec2(*x, *y) * line_height;
                    }
                    winit::event::MouseScrollDelta::PixelDelta(delta) => {
                        // Actually point delta
                        raw_input.scroll_delta = egui::vec2(delta.x as f32, delta.y as f32);
                    }
                }
            }
            CursorMoved { position, .. } => {
                self.pointer_pos = pos2(
                    position.x as f32 / self.scale_factor as f32,
                    position.y as f32 / self.scale_factor as f32,
                );
                raw_input
                    .events
                    .push(egui::Event::PointerMoved(self.pointer_pos));
            }
            CursorLeft { .. } => {
                raw_input.events.push(egui::Event::PointerGone);
            }
            ModifiersChanged(input) => self.modifier_state = *input,
            KeyboardInput { input, .. } => {
                if let Some(virtual_keycode) = input.virtual_keycode {
                    if let Some(key) = winit_to_egui_key_code(virtual_keycode) {
                        // TODO figure out why if I enable this the characters get ignored

                        raw_input.events.push(egui::Event::Key {
                            key,
                            pressed: input.state == winit::event::ElementState::Pressed,
                            // modifiers: winit_to_egui_modifiers(self.modifier_state),
                            modifiers: winit_to_egui_modifiers(self.modifier_state),
                        });
                    }
                }
            }
            ReceivedCharacter(ch) => {
                if ch.is_alphanumeric() && !self.modifier_state.ctrl() && !self.modifier_state.logo()
                {
                    raw_input.events.push(egui::Event::Text(ch.to_string()));
                }
            }
            _ => {}
        }
    }

    pub fn begin_frame(&mut self) -> CtxRef {
        self.context.begin_frame(self.raw_input.borrow_mut().take());
        self.context.clone()
    }

    pub fn end_frame(&mut self) {
        let (_, paint_cmds) = self.context.end_frame();
        self.paint_jobs = self.context.tessellate(paint_cmds);
    }

    pub fn update_time(&mut self, elapsed_seconds: f64) {
        self.raw_input.time = Some(elapsed_seconds);
    }

    pub fn with_ctxt_and_frame<F>(&mut self, f: F)
    where
        F: for<'a> FnOnce(&CtxRef, &mut epi::Frame<'a>),
    {
        let mut render_pass = self.render_pass.borrow_mut();

        let integration_info = epi::IntegrationInfo {
            web_info: None,
            prefer_dark_mode: None, // TODO: figure out system default
            cpu_usage: None,
            seconds_since_midnight: None,
            native_pixels_per_point: Some(self.scale_factor as _),
        };
        let mut app_output = epi::backend::AppOutput::default();

        let mut frame = epi::backend::FrameBuilder {
            info: integration_info,
            tex_allocator: &mut *render_pass,
            // #[cfg(feature = "http")]
            // http: http.clone(),
            output: &mut app_output,
            repaint_signal: self.repaint_signal.clone(),
        }
        .build();
        f(&self.context, &mut frame)
    }

    pub fn draw_ui_to_frame(&self, frame: &nannou::Frame) {
        let device_queue_pair = frame.device_queue_pair();
        let device = device_queue_pair.device();
        let queue = device_queue_pair.queue();
        let mut render_pass = self.render_pass.borrow_mut();
        let paint_jobs = &self.paint_jobs;
        let mut encoder = frame.command_encoder();

        let screen_descriptor = ScreenDescriptor {
            physical_width: self.width,
            physical_height: self.height,
            scale_factor: self.scale_factor as f32,
        };
        render_pass.update_texture(device, queue, &self.context.texture());
        render_pass.update_user_textures(&device, &queue);
        render_pass.update_buffers(device, queue, &paint_jobs, &screen_descriptor);

        // Record all render passes.
        render_pass.execute(
            &mut encoder,
            frame.texture_view(),
            &paint_jobs,
            &screen_descriptor,
            None,
        );
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
        VirtualKeyCode::K => Key::K,
        VirtualKeyCode::U => Key::U,
        VirtualKeyCode::W => Key::W,
        VirtualKeyCode::Z => Key::Z,

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

    if color_picker::color_edit_button_hsva(ui, &mut egui_hsv, color_picker::Alpha::Opaque)
        .changed()
    {
        *color = nannou::color::hsv(egui_hsv.h, egui_hsv.s, egui_hsv.v);
    }
}
