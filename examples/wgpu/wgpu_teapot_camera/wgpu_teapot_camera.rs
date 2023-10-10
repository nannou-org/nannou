use nannou::prelude::*;
use nannou::winit;
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
    eye: Point3,
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
    world: Mat4,
    view: Mat4,
    proj: Mat4,
}

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

impl Camera {
    // Calculate the direction vector from the pitch and yaw.
    fn direction(&self) -> Vec3 {
        pitch_yaw_to_direction(self.pitch, self.yaw)
    }

    // The camera's "view" matrix.
    fn view(&self) -> Mat4 {
        let direction = self.direction();
        let up = Vec3::Y;
        Mat4::look_to_rh(self.eye, direction, up)
    }
}

fn pitch_yaw_to_direction(pitch: f32, yaw: f32) -> Vec3 {
    let xz_unit_len = pitch.cos();
    let x = xz_unit_len * yaw.cos();
    let y = pitch.sin();
    let z = xz_unit_len * (-yaw).sin();
    vec3(x, y, z)
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
    let device = window.device();
    let format = Frame::TEXTURE_FORMAT;
    let msaa_samples = window.msaa_samples();
    let (win_w, win_h) = window.inner_size_pixels();

    let vs_desc = wgpu::include_wgsl!("shaders/vs.wgsl");
    let fs_desc = wgpu::include_wgsl!("shaders/fs.wgsl");
    let vs_mod = device.create_shader_module(vs_desc);
    let fs_mod = device.create_shader_module(fs_desc);

    // Create the vertex, normal and index buffers.
    let vertices_bytes = vertices_as_bytes(&data::VERTICES);
    let normals_bytes = normals_as_bytes(&data::NORMALS);
    let indices_bytes = indices_as_bytes(&data::INDICES);
    let vertex_usage = wgpu::BufferUsages::VERTEX;
    let index_usage = wgpu::BufferUsages::INDEX;
    let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: vertices_bytes,
        usage: vertex_usage,
    });
    let normal_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: normals_bytes,
        usage: vertex_usage,
    });
    let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: indices_bytes,
        usage: index_usage,
    });

    let depth_texture = create_depth_texture(device, [win_w, win_h], DEPTH_FORMAT, msaa_samples);
    let depth_texture_view = depth_texture.view().build();

    let eye = Point3::new(0.0, 0.0, 1.0);
    let pitch = 0.0;
    let yaw = std::f32::consts::PI * 0.5;
    let camera = Camera { eye, pitch, yaw };

    let uniforms = create_uniforms([win_w, win_h], camera.view());
    let uniforms_bytes = uniforms_as_bytes(&uniforms);
    let usage = wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST;
    let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: uniforms_bytes,
        usage,
    });

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
            let yaw = model.camera.yaw + std::f32::consts::PI * 0.5;
            let direction = pitch_yaw_to_direction(pitch, yaw);
            model.camera.eye += direction * velocity;
        }
        // Strafe right on D.
        if app.keys.down.contains(&Key::D) {
            let pitch = 0.0;
            let yaw = model.camera.yaw - std::f32::consts::PI * 0.5;
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
                    0 => model.camera.yaw -= (value * sensitivity) as f32,
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
        g.depth_texture_view = g.depth_texture.view().build();
    }

    // Update the uniforms (rotate around the teapot).
    let uniforms = create_uniforms(frame_size, model.camera.view());
    let uniforms_size = std::mem::size_of::<Uniforms>() as wgpu::BufferAddress;
    let uniforms_bytes = uniforms_as_bytes(&uniforms);
    let usage = wgpu::BufferUsages::COPY_SRC;
    let new_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: uniforms_bytes,
        usage,
    });

    let mut encoder = frame.command_encoder();
    encoder.copy_buffer_to_buffer(&new_uniform_buffer, 0, &g.uniform_buffer, 0, uniforms_size);
    let mut render_pass = wgpu::RenderPassBuilder::new()
        .color_attachment(frame.texture_view(), |color| color)
        // We'll use a depth texture to assist with the order of rendering fragments based on depth.
        .depth_stencil_attachment(&g.depth_texture_view, |depth| depth)
        .begin(&mut encoder);
    render_pass.set_bind_group(0, &g.bind_group, &[]);
    render_pass.set_pipeline(&g.render_pipeline);
    render_pass.set_vertex_buffer(0, g.vertex_buffer.slice(..));
    render_pass.set_vertex_buffer(1, g.normal_buffer.slice(..));
    render_pass.set_index_buffer(g.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
    let index_range = 0..data::INDICES.len() as u32;
    let start_vertex = 0;
    let instance_range = 0..1;
    render_pass.draw_indexed(index_range, start_vertex, instance_range);
}

fn create_uniforms([w, h]: [u32; 2], view: Mat4) -> Uniforms {
    let rotation = Mat4::from_rotation_y(0f32);
    let aspect_ratio = w as f32 / h as f32;
    let fov_y = std::f32::consts::FRAC_PI_2;
    let near = 0.01;
    let far = 100.0;
    let proj = Mat4::perspective_rh_gl(fov_y, aspect_ratio, near, far);
    let scale = Mat4::from_scale(Vec3::splat(0.01));
    Uniforms {
        world: rotation,
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
        .usage(wgpu::TextureUsages::RENDER_ATTACHMENT)
        .sample_count(sample_count)
        .build(device)
}

fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    wgpu::BindGroupLayoutBuilder::new()
        .uniform_buffer(wgpu::ShaderStages::VERTEX, false)
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
        label: None,
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
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
        .color_blend(wgpu::BlendComponent::REPLACE)
        .alpha_blend(wgpu::BlendComponent::REPLACE)
        .add_vertex_buffer::<Vertex>(&wgpu::vertex_attr_array![0 => Float32x3])
        .add_vertex_buffer::<Normal>(&wgpu::vertex_attr_array![1 => Float32x3])
        .depth_format(depth_format)
        .sample_count(sample_count)
        .build(device)
}

// See the `nannou::wgpu::bytes` documentation for why the following are necessary.

fn vertices_as_bytes(data: &[Vertex]) -> &[u8] {
    unsafe { wgpu::bytes::from_slice(data) }
}

fn normals_as_bytes(data: &[Normal]) -> &[u8] {
    unsafe { wgpu::bytes::from_slice(data) }
}

fn indices_as_bytes(data: &[u16]) -> &[u8] {
    unsafe { wgpu::bytes::from_slice(data) }
}

fn uniforms_as_bytes(uniforms: &Uniforms) -> &[u8] {
    unsafe { wgpu::bytes::from(uniforms) }
}
