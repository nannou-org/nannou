extern crate nannou;
extern crate shade_runner;

use nannou::prelude::*;
use std::cell::RefCell;
use std::ffi::CStr;
use std::sync::Arc;
use std::time::Duration;

struct Model {
    render_pass: Arc<vk::RenderPassAbstract + Send + Sync>,
    pipeline: Option<Arc<vk::GraphicsPipelineAbstract + Send + Sync>>,
    vertex_buffer: Arc<vk::CpuAccessibleBuffer<[Vertex]>>,
    view_fbo: RefCell<ViewFbo>,
    shade_watcher: shade_runner::Watch,
    shade_msg: shade_runner::Message,
    vert_shader: Arc<vk::pipeline::shader::ShaderModule>,
    frag_shader: Arc<vk::pipeline::shader::ShaderModule>,
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
    let vert_path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/vulkan/shaders/hotload_vert.glsl");
    let frag_path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/vulkan/shaders/hotload_frag.glsl");
    let shade_watcher = shade_runner::Watch::create(vert_path, frag_path, Duration::from_millis(50)).expect("failed to create watcher");
    let shade_msg = shade_watcher
        .rx
        .recv()
        .expect("Failed to receive shader")
        .expect("failed to compile shader");
    let vs = unsafe {
        vk::pipeline::shader::ShaderModule::from_words(device.clone(), &shade_msg.shaders.vertex)
    }
    .unwrap();

    let fs = unsafe {
        vk::pipeline::shader::ShaderModule::from_words(device.clone(), &shade_msg.shaders.fragment)
    }
    .unwrap();

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

    let mut model = Model {
        render_pass,
        pipeline,
        vertex_buffer,
        view_fbo,
        shade_watcher,
        shade_msg,
        vert_shader: vs,
        frag_shader: fs,
        device,
    };
    update_pipeline(&mut model);
    model
}

fn update(_app: &App, model: &mut Model, _: Update) {
    let shader_msg = model.shade_watcher.rx.try_iter().last();
    if let Some(shade_msg) = shader_msg {
        match shade_msg {
            Ok(shade_msg) => {
                model.vert_shader = unsafe {
                    vk::pipeline::shader::ShaderModule::from_words(
                        model.device.clone(),
                        &shade_msg.shaders.vertex,
                    )
                }
                .unwrap();

                model.frag_shader = unsafe {
                    vk::pipeline::shader::ShaderModule::from_words(
                        model.device.clone(),
                        &shade_msg.shaders.fragment,
                    )
                }
                .unwrap();
                model.shade_msg = shade_msg;
                update_pipeline(model);
            }
            Err(e) => {
                println!("Error compiling shader {:?}", e);
            }
        }
    }
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

fn update_pipeline(model: &mut Model) {
    let Model {
        ref vert_shader,
        ref frag_shader,
        ref shade_msg,
        ref device,
        ref render_pass,
        ref mut pipeline,
        ..
    } = model;
    let entry = shade_msg.entry.clone();
    let vert_main = unsafe {
        vert_shader.graphics_entry_point(
            CStr::from_bytes_with_nul_unchecked(b"main\0"),
            entry.vert_input,
            entry.vert_output,
            entry.vert_layout,
            vk::pipeline::shader::GraphicsShaderType::Vertex,
        )
    };
    let frag_main = unsafe {
        frag_shader.graphics_entry_point(
            CStr::from_bytes_with_nul_unchecked(b"main\0"),
            entry.frag_input,
            entry.frag_output,
            entry.frag_layout,
            vk::pipeline::shader::GraphicsShaderType::Fragment,
        )
    };
    *pipeline = Some(Arc::new(
        vk::GraphicsPipeline::start()
            // We need to indicate the layout of the vertices.
            // The type `SingleBufferDefinition` actually contains a template parameter
            // corresponding to the type of each vertex.
            .vertex_input_single_buffer::<Vertex>()
            // A Vulkan shader can in theory contain multiple entry points, so we have to specify
            // which one. The `main` word of `main_entry_point` actually corresponds to the name of
            // the entry point.
            .vertex_shader(vert_main, ())
            // The content of the vertex buffer describes a list of triangles.
            .triangle_list()
            // Use a resizable viewport set to draw over the entire window
            .viewports_dynamic_scissors_irrelevant(1)
            // See `vertex_shader`.
            .fragment_shader(frag_main, ())
            // We have to indicate which subpass of which render pass this pipeline is going to be
            // used in. The pipeline will only be usable from this particular subpass.
            .render_pass(vk::Subpass::from(render_pass.clone(), 0).unwrap())
            // Now that our builder is filled, we call `build()` to obtain an actual pipeline.
            .build(device.clone())
            .unwrap(),
    ));
}
