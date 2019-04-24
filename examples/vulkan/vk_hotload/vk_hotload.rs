extern crate nannou;
extern crate notify;
extern crate shaderc;
extern crate spirv_reflect;

mod shader;
mod watch;

use nannou::prelude::*;
use spirv_reflect as sr;
use std::cell::RefCell;
use std::path::PathBuf;
use std::sync::{mpsc, Arc};

use shader::{compile_shader, create_interfaces, ShaderInterfaces};
use watch::{Handler, ShaderMsg};

struct Model {
    render_pass: Arc<vk::RenderPassAbstract + Send + Sync>,
    pipeline: Option<Arc<vk::GraphicsPipelineAbstract + Send + Sync>>,
    vertex_buffer: Arc<vk::CpuAccessibleBuffer<[Vertex]>>,
    view_fbo: RefCell<ViewFbo>,
    _shader_watch: Handler,
    shader_change: mpsc::Receiver<ShaderMsg>,
    vert_shader: Arc<vk::pipeline::shader::ShaderModule>,
    frag_shader: Arc<vk::pipeline::shader::ShaderModule>,
    vert_interfaces: ShaderInterfaces,
    frag_interfaces: ShaderInterfaces,
    device: Arc<vk::Device>,
}

#[derive(Debug, Clone)]
struct Vertex {
    position: [f32; 2],
}

vk::impl_vertex!(Vertex, position);

fn main() {
    nannou::app(model).view(view).update(update).run();
}

fn model(app: &App) -> Model {
    app.new_window()
        .with_dimensions(512, 512)
        .view(view)
        .build()
        .unwrap();

    // The gpu device associated with the window's swapchain
    let device = app.main_window().swapchain().device().clone();

    // We now create a buffer that will store the shape of our triangle.
    let vertex_buffer = {
        let positions = [[-0.5, -0.25], [0.0, 0.5], [0.25, -0.1]];
        let vertices = positions.iter().map(|&position| Vertex { position });
        vk::CpuAccessibleBuffer::from_iter(device.clone(), vk::BufferUsage::all(), vertices)
            .unwrap()
    };
    let nannou_root = std::env::current_dir().expect("failed to get root directory");
    let mut vert_path = nannou_root.clone();
    vert_path.push(PathBuf::from("examples/vulkan/shaders/hotload_vert.glsl"));
    let mut frag_path = nannou_root.clone();
    frag_path.push(PathBuf::from("examples/vulkan/shaders/hotload_frag.glsl"));
    let (vs, vert_interfaces) = {
        let v = compile_shader(vert_path.clone(), shaderc::ShaderKind::Vertex)
            .expect("Failed to load shader");
        let s = create_interfaces(&v);
        (
            unsafe { vk::pipeline::shader::ShaderModule::from_words(device.clone(), &v) }.unwrap(),
            s,
        )
    };

    let (fs, frag_interfaces) = {
        let v = compile_shader(frag_path.clone(), shaderc::ShaderKind::Fragment)
            .expect("Failed to load shader");
        let s = create_interfaces(&v);
        (
            unsafe { vk::pipeline::shader::ShaderModule::from_words(device.clone(), &v) }.unwrap(),
            s,
        )
    };

    // The next step is to create a *render pass*, which is an object that describes where the
    // output of the graphics pipeline will go. It describes the layout of the images
    // where the colors, depth and/or stencil information will be written.
    let render_pass = Arc::new(
        vk::single_pass_renderpass!(
            device.clone(),
            attachments: {
                // `color` is a custom name we give to the first and only attachment.
                color: {
                    // `load: Clear` means that we ask the GPU to clear the content of this
                    // attachment at the start of the drawing.
                    load: Clear,
                    // `store: Store` means that we ask the GPU to store the output of the draw
                    // in the actual image. We could also ask it to discard the result.
                    store: Store,
                    // `format: <ty>` indicates the type of the format of the image. This has to
                    // be one of the types of the `vulkano::format` module (or alternatively one
                    // of your structs that implements the `FormatDesc` trait). Here we use the
                    // same format as the swapchain.
                    format: app.main_window().swapchain().format(),
                    // TODO:
                    samples: app.main_window().msaa_samples(),
                }
            },
            pass: {
                // We use the attachment named `color` as the one and only color attachment.
                color: [color],
                // No depth-stencil attachment is indicated with empty brackets.
                depth_stencil: {}
            }
        )
        .unwrap(),
    );

    let pipeline = None;
    // The render pass we created above only describes the layout of our framebuffer. Before we
    // can draw we also need to create the actual framebuffer.
    let view_fbo = RefCell::new(ViewFbo::default());

    let (shader_watch, shader_change) = watch::new(&vert_path, &frag_path);

    let mut model = Model {
        render_pass,
        pipeline,
        vertex_buffer,
        view_fbo,
        _shader_watch: shader_watch,
        shader_change,
        vert_shader: vs,
        frag_shader: fs,
        vert_interfaces,
        frag_interfaces,
        device,
    };
    shader::update_pipeline(&mut model);
    model
}

fn update(_app: &App, model: &mut Model, _: Update) {
    shader::update(model);
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(_app: &App, model: &Model, frame: Frame) -> Frame {
    // Dynamic viewports allow us to recreate just the viewport when the window is resized
    // Otherwise we would have to recreate the whole pipeline.
    let [w, h] = frame.swapchain_image().dimensions();
    let viewport = vk::ViewportBuilder::new().build([w as _, h as _]);
    let dynamic_state = vk::DynamicState::default().viewports(vec![viewport]);

    // Update the view_fbo.
    model
        .view_fbo
        .borrow_mut()
        .update(&frame, model.render_pass.clone(), |builder, image| {
            builder.add(image)
        })
        .unwrap();

    // Specify the color to clear the framebuffer with i.e. blue.
    let clear_values = vec![[0.0, 0.0, 1.0, 1.0].into()];

    // Submit the draw commands.
    frame
        .add_commands()
        .begin_render_pass(model.view_fbo.borrow().expect_inner(), false, clear_values)
        .unwrap()
        .draw(
            model.pipeline.clone().unwrap(),
            &dynamic_state,
            vec![model.vertex_buffer.clone()],
            (),
            (),
        )
        .unwrap()
        .end_render_pass()
        .expect("failed to add `end_render_pass` command");

    frame
}
