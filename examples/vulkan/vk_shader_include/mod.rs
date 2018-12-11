extern crate nannou;

use nannou::prelude::*;
use nannou::vulkano;
use std::sync::Arc;
use std::cell::RefCell;

use nannou::vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use nannou::vulkano::command_buffer::DynamicState;
use nannou::vulkano::device::DeviceOwned;
use nannou::vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use nannou::vulkano::pipeline::viewport::Viewport;
use nannou::vulkano::framebuffer::{Framebuffer, FramebufferAbstract, Subpass, RenderPassAbstract, FramebufferCreationError};

fn main() {
    nannou::app(model)
        .event(event) // The function that will be called when the app receives events.
        .view(view) // The function that will be called for drawing to the window.
        .run();
}

struct Model {
    // Store the window ID so we can refer to this specific window later if needed.
    _window: WindowId,
    render_pass: Arc<RenderPassAbstract + Send + Sync>,
    pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
    vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
    framebuffers: RefCell<Vec<Arc<FramebufferAbstract + Send + Sync>>>,
}

#[derive(Debug, Clone)]
struct Vertex { position: [f32; 2] }
nannou::vulkano::impl_vertex!(Vertex, position);

fn model(app: &App) -> Model {
    // Create a new window! Store the ID so we can refer to it later.
    let _window = app.new_window().with_dimensions(1024, 512).with_title("nannou").build().unwrap();

    // The gpu device associated with the window's swapchain
    let device = app.main_window().swapchain().device().clone();

    // We now create a buffer that will store the shape of our triangle.
    let vertex_buffer = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::all(),
        [
            Vertex { position: [-1.0, -1.0 ] },
            Vertex { position: [-1.0,  1.0 ] },
            Vertex { position: [ 1.0, -1.0 ] },
            Vertex { position: [ 1.0,  1.0 ] },
        ].iter().cloned()
    ).unwrap();

    let vertex_shader = vs::Shader::load(device.clone()).unwrap();
    let fragment_shader = fs::Shader::load(device.clone()).unwrap();

    // The next step is to create a *render pass*, which is an object that describes where the
    // output of the graphics pipeline will go. It describes the layout of the images
    // where the colors, depth and/or stencil information will be written.
    let render_pass = Arc::new(nannou::vulkano::single_pass_renderpass!(
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
                samples: 1,
                initial_layout: ImageLayout::PresentSrc,
                final_layout: ImageLayout::PresentSrc,
            }
        },
        pass: {
            // We use the attachment named `color` as the one and only color attachment.
            color: [color],
            // No depth-stencil attachment is indicated with empty brackets.
            depth_stencil: {}
        }
    ).unwrap());

    // Before we draw we have to create what is called a pipeline. This is similar to an OpenGL
    // program, but much more specific.
    let pipeline = Arc::new(GraphicsPipeline::start()
        // We need to indicate the layout of the vertices.
        // The type `SingleBufferDefinition` actually contains a template parameter corresponding
        // to the type of each vertex. But in this code it is automatically inferred.
        .vertex_input_single_buffer::<Vertex>()
        // A Vulkan shader can in theory contain multiple entry points, so we have to specify
        // which one. The `main` word of `main_entry_point` actually corresponds to the name of
        // the entry point.
        .vertex_shader(vertex_shader.main_entry_point(), ())
        // The content of the vertex buffer describes a list of triangles.
        .triangle_strip()
        // Use a resizable viewport set to draw over the entire window
        .viewports_dynamic_scissors_irrelevant(1)
        // See `vertex_shader`.
        .fragment_shader(fragment_shader.main_entry_point(), ())
        // Enable Alpha Blending
        .blend_alpha_blending()
        // We have to indicate which subpass of which render pass this pipeline is going to be used
        // in. The pipeline will only be usable from this particular subpass.
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        // Now that our builder is filled, we call `build()` to obtain an actual pipeline.
        .build(device.clone())
        .unwrap());

    // The render pass we created above only describes the layout of our framebuffers. Before we
    // can draw we also need to create the actual framebuffers.
    //
    // Since we need to draw to multiple images, we are going to create a different framebuffer for
    // each image.
    let framebuffers = RefCell::new(Vec::new());    

    Model { _window, render_pass, pipeline, vertex_buffer, framebuffers }
}

// Handle events related to the window and update the model if necessary
fn event(_app: &App, model: Model, event: Event) -> Model {
    if let Event::Update(_update) = event {
    }
    model
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(app: &App, model: &Model, frame: Frame) -> Frame {
    // Clear the window with a "dark charcoal" shade.
    frame.clear(BLUE);

    // Dynamic viewports allow us to recreate just the viewport when the window is resized
    // Otherwise we would have to recreate the whole pipeline.
    let [w, h] = frame.swapchain_image().dimensions();
    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [w as _, h as _],
        depth_range: 0.0 .. 1.0,
    };
    let dynamic_state = DynamicState {
        line_width: None,
        viewports: Some(vec![viewport]),
        scissors: None 
    };

    // Update the framebuffers if necessary.
    while frame.swapchain_image_index() >= model.framebuffers.borrow().len() {
        let fb = create_framebuffer(
            model.render_pass.clone(),
            frame.swapchain_image().clone(),
        ).unwrap();
        model.framebuffers.borrow_mut().push(Arc::new(fb));
    }

    // If the dimensions for the current framebuffer do not match, recreate it.
    if frame.swapchain_image_is_new() {
        let fb = &mut model.framebuffers.borrow_mut()[frame.swapchain_image_index()];
        let new_fb = create_framebuffer(
            model.render_pass.clone(),
            frame.swapchain_image().clone(),
        ).unwrap();
        *fb = Arc::new(new_fb);
    }

    // Specify the color to clear the framebuffer with i.e. blue
    let clear_values = vec!([0.0, 1.0, 0.0, 1.0].into());

    let push_constants = fs::ty::PushConstantData {
        time: app.time,
        width: w as f32,
        height: h as f32,
    };

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
            push_constants,
        )
        .unwrap()
        .end_render_pass()
        .expect("failed to add `end_render_pass` command");

    // Return the cleared frame.
    frame
}

mod vs {
    nannou::vulkano_shaders::shader!{
        ty: "vertex",
        src: "
#version 450

layout(location = 0) in vec2 position;
layout(location = 0) out vec2 tex_coords;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    tex_coords = position + vec2(0.5);
}"
    }
}

mod fs {
    nannou::vulkano_shaders::shader!{
        ty: "fragment",
        // We declare what directories to search for when using the `#include <...>`
        // syntax. Specified directories have descending priorities based on their order.
        include: [ "examples/vulkan/vk_shader_include/common_shaders" ],
        src: "
#version 450
// Substitutes this line with the contents of the file `lfos.glsl` found in one of the standard
// `include` directories specified above.
// Note, that relative inclusion (`#include \"...\"`), although it falls back to standard
// inclusion, should not be used for **embedded** shader source, as it may be misleading and/or
// confusing.
#include <lfos.glsl>

layout(location = 0) in vec2 tex_coords;
layout(location = 0) out vec4 f_color;

layout(push_constant) uniform PushConstantData {
    float time;
    float width;
    float height;
} pc;

float circle(in vec2 _st, in float _radius){
    vec2 dist = _st-vec2(0.5);
	return 1.-smoothstep(_radius-(_radius*0.01),
                         _radius+(_radius*0.01),
                         dot(dist,dist)*4.0);
}

void main() {

    // Background
	vec3 bg = vec3(0.8,0.9,0.9);

    float aspect = pc.width / pc.height;
    vec2 center = vec2(0.5,0.5);
    float radius = 0.25 * aspect;

    // Here we use the 'lfo' function imported from our include shader
    float x = tex_coords.x + lfo(2, pc.time * 3.0) * 0.7;
    vec3 color = vec3(lfo(3,pc.time * 6.0),0.0,0.9);
    vec3 shape = vec3(circle(vec2((x * aspect) - 0.45, tex_coords.y), radius) );
    shape *= color;

    // Blend the two
	f_color = vec4( vec3(mix(shape, bg, 0.5)),1.0 );
}"
    }
}
// Create the framebuffer for the image.
fn create_framebuffer(
    render_pass: Arc<RenderPassAbstract + Send + Sync>,
    swapchain_image: Arc<nannou::window::SwapchainImage>,
) -> Result<Arc<FramebufferAbstract + Send + Sync>, FramebufferCreationError> {
    let fb = Framebuffer::start(render_pass)
        .add(swapchain_image)?
        .build()?;
    Ok(Arc::new(fb) as _)
}
