extern crate nannou;

mod controls;
mod teapot_verts; 
mod warp;
mod homography;

use nannou::prelude::*;

use self::warp::Warp;
use self::teapot_verts::{VERTICES, INDICES, NORMALS};
use self::controls::{Controls, Corners, Corner, Ids};
use nannou::ui::{Ui, widget};
use nannou::math::cgmath::{self, Matrix3, Matrix4, Rad, Point3, Vector3};
use nannou::vulkano;
use nannou::vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use nannou::vulkano::buffer::cpu_pool::CpuBufferPool;
use nannou::vulkano::command_buffer::DynamicState;
use nannou::vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use nannou::vulkano::device::{Device, DeviceOwned};
use nannou::vulkano::format::Format;
use nannou::vulkano::framebuffer::{RenderPassAbstract, Subpass, Framebuffer, FramebufferAbstract};
use nannou::vulkano::image::attachment::AttachmentImage;
use nannou::vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract,
                                GraphicsPipelineCreationError};
use nannou::vulkano::pipeline::vertex::TwoBuffersDefinition;
use nannou::vulkano::pipeline::viewport::Viewport;
use nannou::window::SwapchainFramebuffers;
use nannou::geom::Quad;
use std::cell::RefCell;
use std::sync::Arc;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    graphics: RefCell<Graphics>,
    warp: Warp,
    controls: Controls,
    ui: Ui,
    ids: Ids,
}


struct Graphics {
    vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
    normal_buffer: Arc<CpuAccessibleBuffer<[Normal]>>,
    index_buffer: Arc<CpuAccessibleBuffer<[u16]>>,
    uniform_buffer: CpuBufferPool<vs::ty::Data>,
    vertex_shader: vs::Shader,
    fragment_shader: fs::Shader,
    render_pass: Arc<RenderPassAbstract + Send + Sync>,
    graphics_pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
    depth_image: Arc<AttachmentImage>,
    inter_image: Arc<AttachmentImage>,
    framebuffer: Arc<FramebufferAbstract + Send + Sync>
    //framebuffers: SwapchainFramebuffers,
}


nannou::vulkano::impl_vertex!(Vertex, position);

nannou::vulkano::impl_vertex!(Normal, normal);

// Teapot data, sourced from `vulkano-examples`.

#[derive(Copy, Clone)]
pub struct Vertex {
    position: (f32, f32, f32)
}

#[derive(Copy, Clone)]
pub struct Normal {
    normal: (f32, f32, f32)
}


fn model(app: &App) -> Model {
    app.new_window()
        .with_dimensions(500, 400)
        .view(view)
        .build()
        .unwrap();

    let device = app.main_window().swapchain().device().clone();

    let vertex_buffer = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::all(),
        VERTICES.iter().cloned(),
    ).unwrap();
    let normal_buffer = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::all(),
        NORMALS.iter().cloned(),
    ).unwrap();
    let index_buffer = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::all(),
        INDICES.iter().cloned(),
    ).unwrap();

    let uniform_buffer = CpuBufferPool::<vs::ty::Data>::new(device.clone(), BufferUsage::all());

    let vertex_shader = vs::Shader::load(device.clone()).unwrap();
    let fragment_shader = fs::Shader::load(device.clone()).unwrap();

    let render_pass = Arc::new(
        nannou::vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: app.main_window().swapchain().format(),
                    samples: 1,
                    initial_layout: ImageLayout::Undefined,
                    final_layout: ImageLayout::ShaderReadOnlyOptimal,
                },
                depth: {
                    load: Clear,
                    store: DontCare,
                    format: Format::D16Unorm,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {depth}
            }
        ).unwrap(),
    );

    let [w, h] = app.main_window().swapchain().dimensions();

    let graphics_pipeline = create_graphics_pipeline(
        device.clone(),
        &vertex_shader,
        &fragment_shader,
        render_pass.clone(),
        [w as f32, h as f32],
    ).unwrap();

    let depth_image = AttachmentImage::transient(device.clone(), [w, h], Format::D16Unorm)
        .unwrap();

    let inter_image = AttachmentImage::sampled(device.clone(), [w, h], app.main_window().swapchain().format()).unwrap();
    //let framebuffers = SwapchainFramebuffers::default();
    let framebuffer = Framebuffer::start(render_pass.clone())
    .add(inter_image.clone()).unwrap()
    .add(depth_image.clone()).unwrap()
    .build().unwrap();
    let framebuffer = Arc::new(framebuffer);

    let graphics = RefCell::new(Graphics {
        vertex_buffer,
        normal_buffer,
        index_buffer,
        uniform_buffer,
        vertex_shader,
        fragment_shader,
        render_pass,
        graphics_pipeline,
        depth_image,
        framebuffer,
        inter_image,
        //framebuffers,
    });
    
    let gui_window = app.new_window()
        .with_dimensions(500, 400)
        .view(ui_view)
        .event(controls::event)
        .build()
        .expect("Failed to build second window");
    let mut ui = app.new_ui().window(gui_window).build().unwrap();
    
    let ids = Ids {
        top_left_corner: ui.generate_widget_id(),
        top_right_corner: ui.generate_widget_id(),
        bottom_left_corner: ui.generate_widget_id(),
        bottom_right_corner: ui.generate_widget_id(),
        background: ui.generate_widget_id(),
        points: ui.generate_widget_id(),
        tl_text: ui.generate_widget_id(),
        tr_text: ui.generate_widget_id(),
        bl_text: ui.generate_widget_id(),
        br_text: ui.generate_widget_id(),
    };

    let (window_w, window_h) = app.window(gui_window).expect("Gui window doesn't exist").inner_size_points();
    let w = window_w / 2.0;
    let h = window_h / 2.0;
    let corners = Corners {
        window_w,
        window_h,
        top_left: Corner{ drag: false, pos: pt2(-w, h) },
        top_right: Corner{ drag: false, pos: pt2(w, h) },
        bottom_left: Corner{ drag: false, pos: pt2(-w, -h) },
        bottom_right: Corner{ drag: false, pos: pt2(w, -h) },
    };
    let controls = Controls {
        corners,
    };

    let warp = warp::warp(app);

    Model { graphics, controls, ui, ids, warp }
}

fn update(_: &App, model: &mut Model, update: Update) {
    controls::update(model);
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(app: &App, model: &Model, frame: Frame) -> Frame {
    let mut graphics = model.graphics.borrow_mut();
    let inter_image = graphics.inter_image.clone();

    let [w, h] = frame.swapchain_image().dimensions();

    // If the dimensions have changed, recreate the pipeline and depth buffer.
    if [w, h] != graphics.depth_image.dimensions() {
        let device = frame.swapchain_image().swapchain().device().clone();
        graphics.graphics_pipeline = create_graphics_pipeline(
            device.clone(),
            &graphics.vertex_shader,
            &graphics.fragment_shader,
            graphics.render_pass.clone(),
            [w as f32, h as f32],
        ).unwrap();

        graphics.depth_image = AttachmentImage::transient(
            device.clone(),
            [w, h],
            Format::D16Unorm,
        ).unwrap();
    }

    // Update framebuffers so that count matches swapchain image count and dimensions match.
    let render_pass = graphics.render_pass.clone();
    let depth_image = graphics.depth_image.clone();
    /*
    graphics.framebuffers
        .update(&frame, render_pass, |builder, image| builder.add(image)?.add(depth_image.clone()))
        .unwrap();
        */

    // Create a uniform buffer slice with the world, view and projection matrices.
    let uniform_buffer_slice = {
        let rotation = app.time;
        let rotation = Matrix3::from_angle_y(Rad(rotation as f32));
        // note: this teapot was meant for OpenGL where the origin is at the lower left instead the
        // origin is at the upper left in Vulkan, so we reverse the Y axis
        let aspect_ratio = w as f32 / h as f32;
        let proj = cgmath::perspective(
            Rad(std::f32::consts::FRAC_PI_2),
            aspect_ratio,
            0.01,
            100.0,
        );
        let view = Matrix4::look_at(
            Point3::new(0.3, 0.3, 1.0),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, -1.0, 0.0),
        );
        let scale = Matrix4::from_scale(0.01);

        let uniform_data = vs::ty::Data {
            world: Matrix4::from(rotation).into(),
            view: (view * scale).into(),
            proj: proj.into(),
        };

        graphics.uniform_buffer.next(uniform_data).unwrap()
    };

    let descriptor_set = Arc::new(
        PersistentDescriptorSet::start(graphics.graphics_pipeline.clone(), 0)
            .add_buffer(uniform_buffer_slice)
            .unwrap()
            .build()
            .unwrap()
    );

    let clear_color = [0.0, 0.0, 0.0, 1.0].into();
    let clear_depth = 1f32.into();
    let clear_values = vec![clear_color, clear_depth];

    // Submit the draw commands.
    frame
        .add_commands()
        .begin_render_pass(
            graphics.framebuffer.clone(),
            //graphics.framebuffers[frame.swapchain_image_index()].clone(),
            false,
            clear_values,
        )
        .unwrap()
        .draw_indexed(
            graphics.graphics_pipeline.clone(),
            &DynamicState::none(),
            vec![graphics.vertex_buffer.clone(), graphics.normal_buffer.clone()],
            graphics.index_buffer.clone(),
            descriptor_set,
            (),
        )
        .unwrap()
        .end_render_pass()
        .expect("failed to add `end_render_pass` command");

    warp::view(&app, model, inter_image, frame)
    //frame
}

// Create the graphics pipeline.
fn create_graphics_pipeline(
    device: Arc<Device>,
    vertex_shader: &vs::Shader,
    fragment_shader: &fs::Shader,
    render_pass: Arc<RenderPassAbstract + Send + Sync>,
    dimensions: [f32; 2],
) -> Result<Arc<GraphicsPipelineAbstract + Send + Sync>, GraphicsPipelineCreationError> {
    let pipeline = GraphicsPipeline::start()
        .vertex_input(TwoBuffersDefinition::<Vertex, Normal>::new())
        .vertex_shader(vertex_shader.main_entry_point(), ())
        .triangle_list()
        .viewports_dynamic_scissors_irrelevant(1)
        .viewports(Some(Viewport {
            origin: [0.0, 0.0],
            dimensions,
            depth_range: 0.0..1.0,
        }))
        .fragment_shader(fragment_shader.main_entry_point(), ())
        .depth_stencil_simple_depth()
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        .build(device.clone())?;
    Ok(Arc::new(pipeline) as Arc<_>)
}

fn ui_view(app: &App, model: &Model, frame: Frame) -> Frame {
    // Draw the state of the `Ui` to the frame.
    model.ui.draw_to_frame(app, &frame).unwrap();
    frame
}

// GLSL Shaders

mod vs {
    nannou::vulkano_shaders::shader! {
        ty: "vertex",
        src: "
#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;

layout(location = 0) out vec3 v_normal;

layout(set = 0, binding = 0) uniform Data {
    mat4 world;
    mat4 view;
    mat4 proj;
} uniforms;

void main() {
    mat4 worldview = uniforms.view * uniforms.world;
    v_normal = transpose(inverse(mat3(worldview))) * normal;
    gl_Position = uniforms.proj * worldview * vec4(position, 1.0);
}
"
    }
}

mod fs {
    nannou::vulkano_shaders::shader! {
        ty: "fragment",
        src: "
#version 450

layout(location = 0) in vec3 v_normal;
layout(location = 0) out vec4 f_color;

const vec3 LIGHT = vec3(0.0, 0.0, 1.0);

void main() {
    float brightness = dot(normalize(v_normal), normalize(LIGHT));
    vec3 dark_color = vec3(0.1, 0.1, 0.1);
    vec3 regular_color = vec3(1.0, 1.0, 1.0);

    f_color = vec4(mix(dark_color, regular_color, brightness), 1.0);
}
"
    }
}

