use std::{borrow::BorrowMut, cell::RefCell, iter, sync::Arc};

pub use egui;
pub use egui_wgpu_backend;

use egui::FontDefinitions;
use egui_wgpu_backend::ScreenDescriptor;
use egui_winit_platform;
use epi;
use winit::event::WindowEvent;

const OUTPUT_FORMAT: egui_wgpu_backend::wgpu::TextureFormat =
    egui_wgpu_backend::wgpu::TextureFormat::Rgba16Float;

pub struct EguiBackend {
    render_pass: RefCell<egui_wgpu_backend::RenderPass>,
    platform: RefCell<egui_winit_platform::Platform>,
    width: u32,
    height: u32,
    scale_factor: f64,
}

struct ExampleRepaintSignal;

impl epi::RepaintSignal for ExampleRepaintSignal {
    fn request_repaint(&self) {}
}

impl EguiBackend {
    pub fn new(
        device: &egui_wgpu_backend::wgpu::Device,
        width: u32,
        height: u32,
        scale_factor: f64,
    ) -> EguiBackend {
        EguiBackend {
            render_pass: RefCell::new(egui_wgpu_backend::RenderPass::new(device, OUTPUT_FORMAT)),
            platform: RefCell::new(egui_winit_platform::Platform::new(
                egui_winit_platform::PlatformDescriptor {
                    physical_width: width,
                    physical_height: height,
                    scale_factor: scale_factor,
                    font_definitions: FontDefinitions::default(),
                    style: Default::default(),
                },
            )),

            width,
            height,
            scale_factor,
        }
    }

    pub fn handle_event(&mut self, _event: &WindowEvent) {
        // self.platform.borrow_mut().handle_event::<winit::event::WindowEvent>(
        //     &winit::event::Event::WindowEvent {
        //         window_id: self.window,
        //         event: event.clone(),
        //     },
        // );
    }

    pub fn update_time(&mut self, dt: f64) {
        self.platform.borrow_mut().update_time(dt);
    }

    pub fn context(&self) -> egui::CtxRef {
        let mut platform = self.platform.borrow_mut();
        platform.begin_frame();
        platform.context()
    }

    pub fn draw_ui_to_frame(&self, frame: &nannou::Frame) {
        let device_queue_pair = frame.device_queue_pair();
        let device = device_queue_pair.device();
        let queue = device_queue_pair.queue();

        let mut platform = self.platform.borrow_mut();
        let mut render_pass = self.render_pass.borrow_mut();

        let (_output, paint_commands) = platform.end_frame();
        let paint_jobs = platform.context().tessellate(paint_commands);

        let mut encoder =
            device.create_command_encoder(&egui_wgpu_backend::wgpu::CommandEncoderDescriptor {
                label: Some("egui_encoder"),
            });

        let screen_descriptor = ScreenDescriptor {
            physical_width: self.width,
            physical_height: self.height,
            scale_factor: self.scale_factor as f32,
        };
        render_pass.update_texture(device, queue, &platform.context().texture());
        render_pass.update_user_textures(&device, &queue);
        render_pass.update_buffers(device, queue, &paint_jobs, &screen_descriptor);

        // Record all render passes.
        render_pass.execute(
            &mut encoder,
            frame.texture_view(),
            &paint_jobs,
            &screen_descriptor,
            Some(egui_wgpu_backend::wgpu::Color::TRANSPARENT),
        );

        queue.submit(iter::once(encoder.finish()));
    }
}
