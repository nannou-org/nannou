use nannou::math::cgmath::{self, Matrix3, Matrix4, Point3, Rad, Vector3};
use nannou::prelude::*;
use std::cell::RefCell;

mod data;

struct Model {
    camera_is_active: bool,
    graphics: RefCell<Graphics>,
    camera: Camera,
}

struct Graphics {
    vertex_buffer: wgpu::Buffer,
    normal_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    depth_texture: wgpu::Texture,
    depth_texture_view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
}

// A simple first person camera.
struct Camera {
    // The position of the camera.
    eye: Point3<f32>,
    // Rotation around the x axis.
    pitch: f32,
    // Rotation around the y axis.
    yaw: f32,
}

// The vertex type that we will use to represent a point on our triangle.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex {
    position: (f32, f32, f32),
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Normal {
    normal: (f32, f32, f32),
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Uniforms {
    world: Matrix4<f32>,
    view: Matrix4<f32>,
    proj: Matrix4<f32>,
}

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

impl Camera {
    // Calculate the direction vector from the pitch and yaw.
    fn direction(&self) -> Vector3<f32> {
        pitch_yaw_to_direction(self.pitch, self.yaw)
    }

    // The camera's "view" matrix.
    fn view(&self) -> Matrix4<f32> {
        let direction = self.direction();
        let up = Vector3::new(0.0, -1.0, 0.0);
        Matrix4::look_at_dir(self.eye, direction, up)
    }
}

impl wgpu::VertexDescriptor for Vertex {
    const STRIDE: wgpu::BufferAddress = std::mem::size_of::<Vertex>() as _;
    const ATTRIBUTES: &'static [wgpu::VertexAttributeDescriptor] =
        &[wgpu::VertexAttributeDescriptor {
            format: wgpu::VertexFormat::Float3,
            offset: 0,
            shader_location: 0,
        }];
}

impl wgpu::VertexDescriptor for Normal {
    const STRIDE: wgpu::BufferAddress = std::mem::size_of::<Normal>() as _;
    const ATTRIBUTES: &'static [wgpu::VertexAttributeDescriptor] =
        &[wgpu::VertexAttributeDescriptor {
            format: wgpu::VertexFormat::Float3,
            offset: 0,
            shader_location: 1,
        }];
}

fn pitch_yaw_to_direction(pitch: f32, yaw: f32) -> Vector3<f32> {
    let xz_unit_len = pitch.cos();
    let x = xz_unit_len * yaw.cos();
    let y = pitch.sin();
    let z = xz_unit_len * (-yaw).sin();
    Vector3::new(x, y, z)
}

fn main() {
    nannou::app(model).event(event).update(update).run();
}

fn model(app: &App) -> Model {
    let w_id = app
        .new_window()
        .size(1024, 576)
        .key_pressed(key_pressed)
        .view(view)
        .build()
        .unwrap();

    let window = app.window(w_id).unwrap();
    let camera_is_active = true;
    window.set_cursor_grab(true).unwrap();
    window.set_cursor_visible(false);
    let device = window.swap_chain_device();
    let format = Frame::TEXTURE_FORMAT;
    let msaa_samples = window.msaa_samples();
    let (win_w, win_h) = window.inner_size_pixels();

    let vs = include_bytes!("shaders/vert.spv");
    let vs_spirv =
        wgpu::read_spirv(std::io::Cursor::new(&vs[..])).expect("failed to read hard-coded SPIRV");
    let vs_mod = device.create_shader_module(&vs_spirv);
    let fs = include_bytes!("shaders/frag.spv");
    let fs_spirv =
        wgpu::read_spirv(std::io::Cursor::new(&fs[..])).expect("failed to read hard-coded SPIRV");
    let fs_mod = device.create_shader_module(&fs_spirv);

    let vertex_buffer = device
        .create_buffer_mapped(data::VERTICES.len(), wgpu::BufferUsage::VERTEX)
        .fill_from_slice(&data::VERTICES[..]);
    let normal_buffer = device
        .create_buffer_mapped(data::NORMALS.len(), wgpu::BufferUsage::VERTEX)
        .fill_from_slice(&data::NORMALS[..]);
    let index_buffer = device
        .create_buffer_mapped(data::INDICES.len(), wgpu::BufferUsage::INDEX)
        .fill_from_slice(&data::INDICES[..]);

    let depth_texture = create_depth_texture(device, [win_w, win_h], DEPTH_FORMAT, msaa_samples);
    let depth_texture_view = depth_texture.create_default_view();

    let eye = Point3::new(0.0, 0.0, 1.0);
    let pitch = 0.0;
    let yaw = std::f32::consts::PI * 0.5;
    let camera = Camera { eye, pitch, yaw };

    let uniforms = create_uniforms([win_w, win_h], camera.view());
    let uniform_buffer = device
        .create_buffer_mapped(1, wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST)
        .fill_from_slice(&[uniforms]);

    let bind_group_layout = create_bind_group_layout(device);
    let bind_group = create_bind_group(device, &bind_group_layout, &uniform_buffer);
    let pipeline_layout = create_pipeline_layout(device, &bind_group_layout);
    let render_pipeline = create_render_pipeline(
        device,
        &pipeline_layout,
        &vs_mod,
        &fs_mod,
        format,
        DEPTH_FORMAT,
        msaa_samples,
    );

    let graphics = RefCell::new(Graphics {
        vertex_buffer,
        normal_buffer,
        index_buffer,
        uniform_buffer,
        depth_texture,
        depth_texture_view,
        bind_group,
        render_pipeline,
    });

    println!("Use the `W`, `A`, `S`, `D`, `Q` and `E` keys to move the camera.");
    println!("Use the mouse to orient the pitch and yaw of the camera.");
    println!("Press the `Space` key to toggle camera mode.");

    Model {
        camera_is_active,
        graphics,
        camera,
    }
}

// Move the camera based on the current key pressed and its current direction.
fn update(app: &App, model: &mut Model, update: Update) {
    const CAM_SPEED_HZ: f64 = 0.5;
    if model.camera_is_active {
        let velocity = (update.since_last.secs() * CAM_SPEED_HZ) as f32;
        // Go forwards on W.
        if app.keys.down.contains(&Key::W) {
            model.camera.eye += model.camera.direction() * velocity;
        }
        // Go backwards on S.
        if app.keys.down.contains(&Key::S) {
            model.camera.eye -= model.camera.direction() * velocity;
        }
        // Strafe left on A.
        if app.keys.down.contains(&Key::A) {
            let pitch = 0.0;
            let yaw = model.camera.yaw - std::f32::consts::PI * 0.5;
            let direction = pitch_yaw_to_direction(pitch, yaw);
            model.camera.eye += direction * velocity;
        }
        // Strafe right on D.
        if app.keys.down.contains(&Key::D) {
            let pitch = 0.0;
            let yaw = model.camera.yaw + std::f32::consts::PI * 0.5;
            let direction = pitch_yaw_to_direction(pitch, yaw);
            model.camera.eye += direction * velocity;
        }
        // Float down on Q.
        if app.keys.down.contains(&Key::Q) {
            let pitch = model.camera.pitch - std::f32::consts::PI * 0.5;
            let direction = pitch_yaw_to_direction(pitch, model.camera.yaw);
            model.camera.eye += direction * velocity;
        }
        // Float up on E.
        if app.keys.down.contains(&Key::E) {
            let pitch = model.camera.pitch + std::f32::consts::PI * 0.5;
            let direction = pitch_yaw_to_direction(pitch, model.camera.yaw);
            model.camera.eye += direction * velocity;
        }
    }
}

// Use raw device motion event for camera pitch and yaw.
// TODO: Check device ID for mouse here - not sure if possible with winit currently.
fn event(_app: &App, model: &mut Model, event: Event) {
    if model.camera_is_active {
        if let Event::DeviceEvent(_device_id, event) = event {
            if let winit::event::DeviceEvent::Motion { axis, value } = event {
                let sensitivity = 0.004;
                match axis {
                    // Yaw left and right on mouse x axis movement.
                    0 => model.camera.yaw += (value * sensitivity) as f32,
                    // Pitch up and down on mouse y axis movement.
                    _ => {
                        let max_pitch = std::f32::consts::PI * 0.5 - 0.0001;
                        let min_pitch = -max_pitch;
                        model.camera.pitch = (model.camera.pitch + (-value * sensitivity) as f32)
                            .min(max_pitch)
                            .max(min_pitch)
                    }
                }
            }
        }
    }
}

// Toggle cursor grabbing and hiding on Space key.
fn key_pressed(app: &App, model: &mut Model, key: Key) {
    if let Key::Space = key {
        let window = app.main_window();
        if !model.camera_is_active {
            if window.set_cursor_grab(true).is_ok() {
                model.camera_is_active = true;
            }
        } else {
            if window.set_cursor_grab(false).is_ok() {
                model.camera_is_active = false;
            }
        }
        window.set_cursor_visible(!model.camera_is_active);
    }
}

fn view(_app: &App, model: &Model, frame: Frame) {
    let mut g = model.graphics.borrow_mut();

    // If the window has changed size, recreate our depth texture to match.
    let depth_size = g.depth_texture.size();
    let frame_size = frame.texture_size();
    let device = frame.device_queue_pair().device();
    if frame_size != depth_size {
        let depth_format = g.depth_texture.format();
        let sample_count = frame.texture_msaa_samples();
        g.depth_texture = create_depth_texture(device, frame_size, depth_format, sample_count);
        g.depth_texture_view = g.depth_texture.create_default_view();
    }

    // Update the uniforms (rotate around the teapot).
    let uniforms = create_uniforms(frame_size, model.camera.view());
    let uniforms_size = std::mem::size_of::<Uniforms>() as wgpu::BufferAddress;
    let new_uniform_buffer = device
        .create_buffer_mapped(1, wgpu::BufferUsage::COPY_SRC)
        .fill_from_slice(&[uniforms]);

    let mut encoder = frame.command_encoder();
    encoder.copy_buffer_to_buffer(&new_uniform_buffer, 0, &g.uniform_buffer, 0, uniforms_size);
    let mut render_pass = wgpu::RenderPassBuilder::new()
        .color_attachment(frame.texture_view(), |color| color)
        // We'll use a depth texture to assist with the order of rendering fragments based on depth.
        .depth_stencil_attachment(&g.depth_texture_view, |depth| depth)
        .begin(&mut encoder);
    render_pass.set_bind_group(0, &g.bind_group, &[]);
    render_pass.set_pipeline(&g.render_pipeline);
    render_pass.set_vertex_buffers(0, &[(&g.vertex_buffer, 0), (&g.normal_buffer, 0)]);
    render_pass.set_index_buffer(&g.index_buffer, 0);
    let index_range = 0..data::INDICES.len() as u32;
    let start_vertex = 0;
    let instance_range = 0..1;
    render_pass.draw_indexed(index_range, start_vertex, instance_range);
}

fn create_uniforms([w, h]: [u32; 2], view: Matrix4<f32>) -> Uniforms {
    let rotation = Matrix3::from_angle_y(Rad(0f32));
    // note: this teapot was meant for OpenGL where the origin is at the lower left instead the
    // origin is at the upper left in Vulkan, so we reverse the Y axis
    let aspect_ratio = w as f32 / h as f32;
    let proj = cgmath::perspective(Rad(std::f32::consts::FRAC_PI_2), aspect_ratio, 0.01, 100.0);
    let scale = Matrix4::from_scale(0.01);
    Uniforms {
        world: Matrix4::from(rotation).into(),
        view: (view * scale).into(),
        proj: proj.into(),
    }
}

fn create_depth_texture(
    device: &wgpu::Device,
    size: [u32; 2],
    depth_format: wgpu::TextureFormat,
    sample_count: u32,
) -> wgpu::Texture {
    wgpu::TextureBuilder::new()
        .size(size)
        .format(depth_format)
        .usage(wgpu::TextureUsage::OUTPUT_ATTACHMENT)
        .sample_count(sample_count)
        .build(device)
}

fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    wgpu::BindGroupLayoutBuilder::new()
        .uniform_buffer(wgpu::ShaderStage::VERTEX, false)
        .build(device)
}

fn create_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    uniform_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    wgpu::BindGroupBuilder::new()
        .buffer::<Uniforms>(uniform_buffer, 0..1)
        .build(device, layout)
}

fn create_pipeline_layout(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::PipelineLayout {
    let desc = wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[&bind_group_layout],
    };
    device.create_pipeline_layout(&desc)
}

fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    vs_mod: &wgpu::ShaderModule,
    fs_mod: &wgpu::ShaderModule,
    dst_format: wgpu::TextureFormat,
    depth_format: wgpu::TextureFormat,
    sample_count: u32,
) -> wgpu::RenderPipeline {
    wgpu::RenderPipelineBuilder::from_layout(layout, vs_mod)
        .fragment_shader(&fs_mod)
        .color_format(dst_format)
        .color_blend(wgpu::BlendDescriptor::REPLACE)
        .alpha_blend(wgpu::BlendDescriptor::REPLACE)
        .add_vertex_buffer::<Vertex>()
        .add_vertex_buffer::<Normal>()
        .depth_format(depth_format)
        .index_format(wgpu::IndexFormat::Uint16)
        .sample_count(sample_count)
        .build(device)
}
