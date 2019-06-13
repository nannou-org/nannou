use nannou::prelude::*;
use std::cell::RefCell;
use std::sync::Arc;

fn main() {
    nannou::app(model).run();
}

struct Model {
    render_pass: Arc<dyn vk::RenderPassAbstract + Send + Sync>,
    pipeline: Arc<dyn vk::GraphicsPipelineAbstract + Send + Sync>,
    vertex_buffer: Arc<vk::CpuAccessibleBuffer<[Vertex]>>,
    framebuffers: RefCell<window::SwapchainFramebuffers>,
}

#[derive(Debug, Clone)]
struct Vertex {
    position: [f32; 2],
}

vk::impl_vertex!(Vertex, position);

fn model(app: &App) -> Model {
    app.new_window()
        .with_dimensions(512, 512)
        .raw_view(view)
        .build()
        .unwrap();

    // The gpu device associated with the window's swapchain
    let device = app.main_window().swapchain().device().clone();

    // We now create a buffer that will store the shape of our triangle.
    let vertex_buffer = {
        let positions = [[-0.5, -0.25], [0.0, 0.5], [0.25, -0.1]];
        let data = positions.iter().map(|&position| Vertex { position });
        let usage = vk::BufferUsage::all();
        vk::CpuAccessibleBuffer::from_iter(device.clone(), usage, data).unwrap()
    };

    let vertex_shader = vs::Shader::load(device.clone()).unwrap();
    let fragment_shader = fs::Shader::load(device.clone()).unwrap();

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
                    // be one of the types of the `vk::format` module (or alternatively one
                    // of your structs that implements the `FormatDesc` trait). Here we use the
                    // same format as the swapchain.
                    format: app.main_window().swapchain().format(),
                    // TODO:
                    samples: 1,
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

    // Before we draw we have to create what is called a pipeline. This is similar to an OpenGL
    // program, but much more specific.
    let pipeline = Arc::new(
        vk::GraphicsPipeline::start()
            // We need to indicate the layout of the vertices.
            // The type `SingleBufferDefinition` actually contains a template parameter
            // corresponding to the type of each vertex.
            .vertex_input_single_buffer::<Vertex>()
            // A Vulkan shader can in theory contain multiple entry points, so we have to specify
            // which one. The `main` word of `main_entry_point` actually corresponds to the name of
            // the entry point.
            .vertex_shader(vertex_shader.main_entry_point(), ())
            // The content of the vertex buffer describes a list of triangles.
            .triangle_list()
            // Use a resizable viewport set to draw over the entire window
            .viewports_dynamic_scissors_irrelevant(1)
            // See `vertex_shader`.
            .fragment_shader(fragment_shader.main_entry_point(), ())
            // We have to indicate which subpass of which render pass this pipeline is going to be
            // used in. The pipeline will only be usable from this particular subpass.
            .render_pass(vk::Subpass::from(render_pass.clone(), 0).unwrap())
            // Now that our builder is filled, we call `build()` to obtain an actual pipeline.
            .build(device.clone())
            .unwrap(),
    );

    // The render pass we created above only describes the layout of our framebuffers. Before we
    // can draw we also need to create the actual framebuffers.
    //
    // Since we need to draw to multiple images, we are going to create a different framebuffer for
    // each image.
    let framebuffers = RefCell::new(window::SwapchainFramebuffers::default());

    Model {
        render_pass,
        pipeline,
        vertex_buffer,
        framebuffers,
    }
}

// Draw the state of your `Model` into the given `RawFrame` here.
fn view(_app: &App, model: &Model, frame: RawFrame) -> RawFrame {
    // Dynamic viewports allow us to recreate just the viewport when the window is resized
    // Otherwise we would have to recreate the whole pipeline.
    let [w, h] = frame.swapchain_image().dimensions();
    let viewport = vk::ViewportBuilder::new().build([w as _, h as _]);
    let dynamic_state = vk::DynamicState::default().viewports(vec![viewport]);

    // Update framebuffers so that count matches swapchain image count and dimensions match.
    model
        .framebuffers
        .borrow_mut()
        .update(&frame, model.render_pass.clone(), |builder, image| {
            builder.add(image)
        })
        .unwrap();

    // Specify the color to clear the framebuffer with i.e. blue
    let clear_values = vec![[0.0, 0.0, 1.0, 1.0].into()];

    // Submit the draw commands.
    frame
        .add_commands()
        .begin_render_pass(
            model.framebuffers.borrow()[frame.swapchain_image_index()].clone(),
            false,
            clear_values,
        )
        .unwrap()
        .draw(
            model.pipeline.clone(),
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

mod vs {
    nannou::vk::shaders::shader! {
    ty: "vertex",
            src: "
#version 450

layout(location = 0) in vec2 position;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
}"
    }
}

mod fs {
    nannou::vk::shaders::shader! {
    ty: "fragment",
        src: "
#version 450

layout(location = 0) out vec4 f_color;

void main() {
    f_color = vec4(1.0, 0.0, 0.0, 1.0);
}
"
    }
}
