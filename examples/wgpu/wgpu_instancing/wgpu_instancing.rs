use nannou::math::cgmath::{self, Matrix3, Matrix4, Point3, Rad, Vector3};
use nannou::prelude::*;
use std::cell::RefCell;

mod data;

/* Generates vertices on a subdivided isocahedron, normalized to 1.
 * It is used for creating a sphere for which the vertices are spread more
 * regularly than it would be with for instance polar coordinates (which is the case for UV map).
 * See https://en.wikipedia.org/wiki/Geodesic_polyhedron
 */
fn make_geodesic_isocahedron(subdivisions: usize) -> Vec<Vector3<f32>> {
    let sqrt5 = 5f32.sqrt();
    let phi = (1f32 + sqrt5) * 0.5f32;
    let ratio = (10f32 + (2f32 * sqrt5)).sqrt() / (4f32 * phi);
    let a = (1f32 / ratio) * 0.5f32;
    let b = (1f32 / ratio) / (2f32 * phi);

    let mut points = vec![
        Vector3::new(0f32, b, -a),
        Vector3::new(b, a, 0f32),
        Vector3::new(-b, a, 0f32),
        Vector3::new(0f32, b, a),
        Vector3::new(0f32, -b, a),
        Vector3::new(-a, 0f32, b),
        Vector3::new(0f32, -b, -a),
        Vector3::new(a, 0f32, -b),
        Vector3::new(a, 0f32, b),
        Vector3::new(-a, 0f32, -b),
        Vector3::new(b, -a, 0f32),
        Vector3::new(-b, -a, 0f32),
    ];

    let triangles = vec![
        [0, 1, 2],
        [3, 2, 1],
        [3, 4, 5],
        [3, 8, 4],
        [0, 6, 7],
        [0, 9, 6],
        [4, 10, 11],
        [6, 11, 10],
        [2, 5, 9],
        [11, 9, 5],
        [1, 7, 8],
        [10, 8, 7],
        [3, 5, 2],
        [3, 1, 8],
        [0, 2, 9],
        [0, 7, 1],
        [6, 9, 11],
        [6, 10, 7],
        [4, 11, 5],
        [4, 8, 10],
    ];

    let missing_edges = vec![
        [1, 2],
        [4, 5],
        [6, 7],
        [10, 11],
        [5, 9],
        [9, 2],
        [5, 11],
        [7, 8],
        [8, 1],
        [7, 10],
    ];

    points.reserve(
        triangles.len() * subdivisions * (subdivisions + 1) / 2
            + missing_edges.len() * subdivisions,
    );

    for indices in triangles.iter() {
        let p0 = points[indices[0]];
        let p1 = points[indices[1]];
        let p2 = points[indices[2]];

        let step0 = (p1 - p0) / (subdivisions + 1) as f32;
        let step1 = (p2 - p0) / (subdivisions + 1) as f32;

        for i in 1..subdivisions + 1 {
            for j in 0..subdivisions + 1 - i {
                let pt = p0 + i as f32 * step0 + j as f32 * step1;
                points.push(pt.normalize());
            }
        }
    }

    for edge in missing_edges.iter() {
        let p0 = points[edge[0]];
        let p1 = points[edge[1]];
        let step = (p1 - p0) / (subdivisions + 1) as f32;
        for i in 1..subdivisions + 1 {
            let pt = p0 + i as f32 * step;
            points.push(pt.normalize());
        }
    }

    points
}

struct Model {
    graphics: RefCell<Graphics>,
    sphere: Vec<Vector3<f32>>,
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

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Instance {
    transformation: Matrix4<f32>,
    color: [f32; 3],
}

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

fn main() {
    nannou::app(model).run();
}

fn model(app: &App) -> Model {
    let w_id = app.new_window().size(1024, 576).view(view).build().unwrap();

    // The gpu device associated with the window's swapchain
    let window = app.window(w_id).unwrap();
    let device = window.swap_chain_device();
    let format = Frame::TEXTURE_FORMAT;
    let msaa_samples = window.msaa_samples();
    let (win_w, win_h) = window.inner_size_pixels();

    // Load shader modules.
    let vs_mod = wgpu::shader_from_spirv_bytes(device, include_bytes!("shaders/vert.spv"));
    let fs_mod = wgpu::shader_from_spirv_bytes(device, include_bytes!("shaders/frag.spv"));

    // Create the vertex, normal and index buffers.
    let vertices_bytes = vertices_as_bytes(&data::VERTICES);
    let normals_bytes = normals_as_bytes(&data::NORMALS);
    let indices_bytes = indices_as_bytes(&data::INDICES);
    let vertex_usage = wgpu::BufferUsage::VERTEX;
    let index_usage = wgpu::BufferUsage::INDEX;
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

    let sphere = make_geodesic_isocahedron(10);
    println!("Number of points on the sphere: {}", sphere.len());

    // Create the depth texture.
    let depth_texture = create_depth_texture(device, [win_w, win_h], DEPTH_FORMAT, msaa_samples);
    let depth_texture_view = depth_texture.view().build();

    // Create the uniform buffer.
    let uniforms = create_uniforms(0.0, [win_w, win_h]);
    let uniforms_bytes = uniforms_as_bytes(&uniforms);
    let usage = wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST;
    let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: uniforms_bytes,
        usage,
    });

    // Create the render pipeline.
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

    Model { graphics, sphere }
}

fn make_instance(
    orientation: Vector3<f32>,
    offset: f32,
    local_rotation: f32,
    scale: f32,
    color: [f32; 3],
) -> Instance {
    let scale_m = Matrix4::from_scale(scale);
    let local_rotation_m = Matrix4::from(Matrix3::from_angle_y(Rad(local_rotation)));
    let orientation_m = {
        let up = Vector3::new(0f32, 1f32, 0f32);
        let cosine = orientation.dot(up);
        if cosine > 0.999 {
            Matrix4::identity()
        } else if cosine < -0.999 {
            Matrix4::from_axis_angle(Vector3::new(1f32, 0f32, 0f32), Rad(std::f32::consts::PI))
        } else {
            Matrix4::from_axis_angle(up.cross(orientation).normalize(), up.angle(orientation))
        }
    };

    let translation_m = Matrix4::from_translation(offset * orientation);

    Instance {
        transformation: translation_m * orientation_m * local_rotation_m * scale_m,
        color,
    }
}

fn special_color(index: usize) -> [f32; 3] {
    let value = 1.5f32 * (index as f32).sin() + 1f32;
    if value < 1f32 {
        [0.6f32, 0.6f32, 0.0f32]
    } else if value < 2f32 {
        [0.6f32, 0.0f32, 0.6f32]
    } else {
        [0.0f32, 0.6f32, 0.6f32]
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
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

    let inner_sphere_instance_rotation = app.time;
    let outer_sphere_instance_rotation = 0.2f32 * app.time;
    let inner_sphere_radius = 35f32;
    let outer_sphere_radius = 50f32;
    let inner_sphere_scale = 0.04f32;
    let outer_sphere_scale = 0.03f32;

    let instances: Vec<_> = model
        .sphere
        .iter()
        .map(|direction| {
            make_instance(
                direction.clone(),
                outer_sphere_radius,
                outer_sphere_instance_rotation,
                outer_sphere_scale,
                [1f32, 1f32, 1f32],
            )
        })
        .chain(model.sphere.iter().enumerate().map(|(i, direction)| {
            make_instance(
                direction.clone(),
                inner_sphere_radius + 2f32 * (i as f32 + app.time).sin().pow(4f32),
                inner_sphere_instance_rotation,
                inner_sphere_scale,
                special_color(i),
            )
        }))
        .collect();

    let instances_bytes = instances_as_bytes(&instances);
    let usage = wgpu::BufferUsage::VERTEX;
    let instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: instances_bytes,
        usage,
    });

    // Update the uniforms (rotate around the teapot).
    let world_rotation = 0.05f32 * app.time;
    let uniforms = create_uniforms(world_rotation, frame_size);
    let uniforms_size = std::mem::size_of::<Uniforms>() as wgpu::BufferAddress;
    let uniforms_bytes = uniforms_as_bytes(&uniforms);
    let usage = wgpu::BufferUsage::COPY_SRC;
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
    render_pass.set_vertex_buffer(2, instance_buffer.slice(..));
    render_pass.set_index_buffer(g.index_buffer.slice(..));
    let index_range = 0..data::INDICES.len() as u32;
    let start_vertex = 0;
    let instance_range = 0..instances.len() as u32;
    render_pass.draw_indexed(index_range, start_vertex, instance_range);
}

fn create_uniforms(world_rotation: f32, [w, h]: [u32; 2]) -> Uniforms {
    let world_rotation = Matrix3::from_angle_y(Rad(world_rotation as f32));
    let aspect_ratio = w as f32 / h as f32;
    let proj = cgmath::perspective(Rad(std::f32::consts::FRAC_PI_2), aspect_ratio, 0.01, 100.0);
    let view = Matrix4::look_at(
        Point3::new(0.3, 0.3, 1.0),
        Point3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
    );

    let world_scale = Matrix4::from_scale(0.015);

    Uniforms {
        world: Matrix4::from(world_rotation).into(),
        view: (view * world_scale).into(),
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
        .color_blend(wgpu::BlendDescriptor::REPLACE)
        .alpha_blend(wgpu::BlendDescriptor::REPLACE)
        .add_vertex_buffer::<Vertex>(&wgpu::vertex_attr_array![0 => Float3])
        .add_vertex_buffer::<Normal>(&wgpu::vertex_attr_array![1 => Float3])
        // TODO: this can use the macro again when https://github.com/gfx-rs/wgpu/issues/836 is fixed
        .add_instance_buffer::<Instance>(&[
            wgpu::VertexAttributeDescriptor {
                shader_location: 2,
                format: wgpu::VertexFormat::Float4,
                offset: std::mem::size_of::<[f32; 4]>() as u64 * 0,
            },
            wgpu::VertexAttributeDescriptor {
                shader_location: 3,
                format: wgpu::VertexFormat::Float4,
                offset: std::mem::size_of::<[f32; 4]>() as u64 * 1,
            },
            wgpu::VertexAttributeDescriptor {
                shader_location: 4,
                format: wgpu::VertexFormat::Float4,
                offset: std::mem::size_of::<[f32; 4]>() as u64 * 2,
            },
            wgpu::VertexAttributeDescriptor {
                shader_location: 5,
                format: wgpu::VertexFormat::Float4,
                offset: std::mem::size_of::<[f32; 4]>() as u64 * 3,
            },
            wgpu::VertexAttributeDescriptor {
                shader_location: 6,
                format: wgpu::VertexFormat::Float4,
                offset: std::mem::size_of::<[f32; 4]>() as u64 * 4,
            },
        ])
        .depth_format(depth_format)
        .index_format(wgpu::IndexFormat::Uint16)
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

fn instances_as_bytes(data: &[Instance]) -> &[u8] {
    unsafe { wgpu::bytes::from_slice(data) }
}
