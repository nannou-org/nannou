use nannou::color;
use nannou::prelude::*;
use palette::named;
use std::cell::{Cell, RefCell};
use std::convert::TryInto;
use std::sync::Arc;
use std::slice;

// These must be smaller then your actual screen
const IMAGE_DIMS: (usize, usize) = (1366, 600);
// This must match the number of colours per
// pixel.
// RGBA = 4
// RGB = 3
// RG = 2 etc.
const NUM_COLOURS: usize = 4;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    render_pass: Arc<vk::RenderPassAbstract + Send + Sync>,
    pipeline: Arc<vk::GraphicsPipelineAbstract + Send + Sync>,
    vertex_buffer_pool: vk::CpuBufferPool<[Vertex; 3]>,
    view_fbo: RefCell<ViewFbo>,
    screenshot_buffer: Arc<vk::CpuAccessibleBuffer<[[u8; NUM_COLOURS]]>>,
    take_screenshot: Cell<bool>,
    save_screenshot: Cell<bool>,
}

#[derive(Debug, Clone)]
struct Vertex {
    position: [f32; 2],
}

vk::impl_vertex!(Vertex, position);

fn model(app: &App) -> Model {
    app.new_window()
        .with_dimensions(
            IMAGE_DIMS.0.try_into().unwrap(),
            IMAGE_DIMS.1.try_into().unwrap(),
        )
        .view(view)
        .event(window_event)
        .build()
        .unwrap();

    // The gpu device associated with the window's swapchain
    let device = app.main_window().swapchain().device().clone();
    dbg!(app.main_window().swapchain().format());

    let buf = vec![[0u8; NUM_COLOURS]; IMAGE_DIMS.0 * IMAGE_DIMS.1];
    let vertex_buffer_pool = vk::CpuBufferPool::vertex_buffer(device.clone());
    let screenshot_buffer = vk::CpuAccessibleBuffer::from_iter(
        device.clone(),
        vk::BufferUsage::transfer_destination(),
        buf.into_iter(),
    )
    .expect("Failed to create screenshot buffer");

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

    // The render pass we created above only describes the layout of our framebuffer. Before we
    // can draw we also need to create the actual framebuffer.
    let view_fbo = RefCell::new(ViewFbo::default());

    Model {
        render_pass,
        pipeline,
        vertex_buffer_pool,
        view_fbo,
        screenshot_buffer,
        take_screenshot: Cell::new(false),
        save_screenshot: Cell::new(false),
    }
}

fn write(screenshot_buffer: &[[u8; NUM_COLOURS]]) {
    // This is pretty unsafe but
    // there is no fast and safe way to go from
    // [[u8; NUM_COLOURS]; IMAGE_DIMS.0 * IMAGE_DIMS.1]
    // to
    // [u8; NUM_COLOURS * IMAGE_DIMS.0 * IMAGE_DIMS.1]
    let buf: &[u8] = unsafe {
        slice::from_raw_parts(&screenshot_buffer[0] as *const u8, NUM_COLOURS * IMAGE_DIMS.0 * IMAGE_DIMS.1)
    };

    let screenshot_path = concat!(env!("CARGO_MANIFEST_DIR"), "/screenshot.png");
    // It is vital that ColorType(bit_depth) matches the
    // type that is used in the screenshot buffer
    nannou::image::save_buffer(
        screenshot_path,
        buf,
        IMAGE_DIMS.0 as u32,
        IMAGE_DIMS.1 as u32,
        nannou::image::ColorType::BGRA(8),
        )
        .expect("Failed to save image");
}

fn update(_app: &App, model: &mut Model, _: Update) {
    let ss = model.save_screenshot.replace(false);
    if ss {
        let screenshot_buffer = match model.screenshot_buffer.read() {
            Ok(s) => s,
            Err(_) => {
                model.save_screenshot.set(true);
                return ();
            },
        };
        write(&(*screenshot_buffer));
        model.take_screenshot.set(true);
    }
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(app: &App, model: &Model, frame: Frame) -> Frame {
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

    let n = (1.0 + app.time.sin()) / 2.0;

    let vertices = [
        Vertex {
            position: [-0.5 * n, -0.25],
        },
        Vertex {
            position: [0.0, 0.5 * n],
        },
        Vertex {
            position: [0.25 * n, -0.1],
        },
    ];

    let vertex_buffer = model
        .vertex_buffer_pool
        .next(vertices)
        .expect("Failed to get next uniform buffer");

    let vertex_buffer = Arc::new(vertex_buffer);

    // Specify the color to clear the framebuffer with i.e. blue.
    let b = color::Alpha {
        color: Rgb::from_pixel(&named::ALICEBLUE),
        alpha: 1.0,
    };
    let b: [f32; 4] = b.to_pixel();
    let clear_values = vec![b.into()];

    // Submit the draw commands.
    let ts = model.take_screenshot.replace(false);
    if ts {
        model.save_screenshot.set(true);
        frame
            .add_commands()
            .begin_render_pass(model.view_fbo.borrow().expect_inner(), false, clear_values)
            .unwrap()
            .draw(
                model.pipeline.clone(),
                &dynamic_state,
                vec![vertex_buffer],
                (),
                (),
            )
            .unwrap()
            .end_render_pass()
            .expect("failed to add `end_render_pass` command")
            .copy_image_to_buffer_dimensions(
                frame.swapchain_image().clone(),
                model.screenshot_buffer.clone(),
                [0, 0, 0],
                [IMAGE_DIMS.0 as u32, IMAGE_DIMS.1 as u32, 1],
                0,
                1,
                0,
            )
            .expect("failed to copy image");

        frame
    } else {
        frame
            .add_commands()
            .begin_render_pass(model.view_fbo.borrow().expect_inner(), false, clear_values)
            .unwrap()
            .draw(
                model.pipeline.clone(),
                &dynamic_state,
                vec![vertex_buffer],
                (),
                (),
            )
            .unwrap()
            .end_render_pass()
            .expect("failed to add `end_render_pass` command");

        frame
    }
}

fn window_event(_app: &App, model: &mut Model, event: WindowEvent) {
    match event {
        KeyPressed(key) => {
            if let Key::S = key {
                if !model.save_screenshot.get() {
                    model.take_screenshot.set(true);
                }
            }
        }
        KeyReleased(_key) => {}
        MouseMoved(_pos) => {}
        MousePressed(_button) => {}
        MouseReleased(_button) => {}
        MouseEntered => {}
        MouseExited => {}
        MouseWheel(_amount, _phase) => {}
        Moved(_pos) => {}
        Resized(_size) => {}
        Touch(_touch) => {}
        TouchPressure(_pressure) => {}
        HoveredFile(_path) => {}
        DroppedFile(_path) => {}
        HoveredFileCancelled => {}
        Focused => {}
        Unfocused => {}
        Closed => {}
    }
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
