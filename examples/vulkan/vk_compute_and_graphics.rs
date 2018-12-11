extern crate nannou;

use nannou::prelude::*;
use nannou::vulkano;
use std::sync::Arc;
use std::cell::RefCell;

use nannou::vulkano::sync;
use nannou::vulkano::sync::GpuFuture;
use nannou::vulkano::instance::{PhysicalDevice};
use nannou::vulkano::command_buffer::AutoCommandBufferBuilder;
use nannou::vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use nannou::vulkano::command_buffer::DynamicState;
use nannou::vulkano::device::{Device, DeviceExtensions, DeviceOwned, Queue};
use nannou::vulkano::pipeline::{ComputePipeline, ComputePipelineAbstract, GraphicsPipeline, GraphicsPipelineAbstract};
use nannou::vulkano::pipeline::viewport::Viewport;
use nannou::vulkano::framebuffer::{Framebuffer, FramebufferAbstract, Subpass, RenderPassAbstract, FramebufferCreationError};
use nannou::vulkano::descriptor::descriptor_set::{DescriptorSet, PersistentDescriptorSet};

fn main() {
    nannou::app(model)
        //.vulkan_debug_callback(Default::default())
        .event(event) // The function that will be called when the app receives events.
        .view(view) // The function that will be called for drawing to the window.
        .run();
}

struct Model {
    // Store the window ID so we can refer to this specific window later if needed.
    _window: WindowId,
    device: Arc<Device>,
    compute_pipeline: Arc<ComputePipelineAbstract + Send + Sync>,
    compute_queue: Arc<Queue>,    
    compute_desciptor_set: Arc<DescriptorSet + Send + Sync>,      
    graphics_pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
    vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
    render_pass: Arc<RenderPassAbstract + Send + Sync>,
    framebuffers: RefCell<Vec<Arc<FramebufferAbstract + Send + Sync>>>,
}

#[derive(Debug, Clone)]
struct Vertex { position: [f32; 2] }
nannou::vulkano::impl_vertex!(Vertex, position);

fn model(app: &App) -> Model {
    // Create a new window! Store the ID so we can refer to it later.
    let _window = app.new_window().with_dimensions(512, 512).with_title("nannou").build().unwrap();

    // let instance = app.vulkan_instance().clone();

    // // Choose which physical device to use.
    // let physical = PhysicalDevice::enumerate(&instance).next().unwrap();

    // // Choose the queue of the physical device which is going to run our compute operation.
    // //
    // // The Vulkan specs guarantee that a compliant implementation must provide at least one queue
    // // that supports compute operations.
    // let queue_family = physical.queue_families().find(|&q| q.supports_compute()).unwrap();

    // // Now initializing the device.
    // let (_, mut queues) = Device::new(physical, physical.supported_features(),
    //     &DeviceExtensions::none(), [(queue_family, 0.5)].iter().cloned()).unwrap();

    // Since we can request multiple queues, the `queues` variable is in fact an iterator. In this
    // example we use only one queue, so we just retrieve the first and only element of the
    // iterator and throw it away.
    //let compute_queue = queues.next().unwrap();
    let compute_queue = app.main_window().queue().clone();

    // The gpu device associated with the window's swapchain
    let device = app.main_window().swapchain().device().clone();

    // We start by creating the buffer that will store the data.
    let vertex_buffer = {
        // Iterator that produces the data.
        let data_iter = (0 .. 1024).map(|n| Vertex{position: [0.0; 2]});
        // Builds the buffer and fills it with this iterator.
        CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), data_iter).unwrap()
    };

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

    // We need to create the compute pipeline that describes our operation.
    //
    // If you are familiar with graphics pipeline, the principle is the same except that compute
    // pipelines are much simpler to create.
    let compute_pipeline = Arc::new({
        let compute_shader = cs::Shader::load(device.clone()).unwrap();
        ComputePipeline::new(device.clone(), &compute_shader.main_entry_point(), &()).unwrap()        
    });

    // Before we draw we have to create what is called a pipeline. This is similar to an OpenGL
    // program, but much more specific.
    let graphics_pipeline = Arc::new(GraphicsPipeline::start()
        // We need to indicate the layout of the vertices.
        // The type `SingleBufferDefinition` actually contains a template parameter corresponding
        // to the type of each vertex. But in this code it is automatically inferred.
        .vertex_input_single_buffer::<Vertex>()
        // A Vulkan shader can in theory contain multiple entry points, so we have to specify
        // which one. The `main` word of `main_entry_point` actually corresponds to the name of
        // the entry point.
        .vertex_shader(vertex_shader.main_entry_point(), ())
        // The content of the vertex buffer describes a list of triangles.
        .triangle_list()
        //.point_list()
        //.line_width(1.0)
        // Use a resizable viewport set to draw over the entire window
        .viewports_dynamic_scissors_irrelevant(1)
        // See `vertex_shader`.
        .fragment_shader(fragment_shader.main_entry_point(), ())
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

    // In order to let the shader access the buffer, we need to build a *descriptor set* that
    // contains the buffer.
    //
    // The resources that we bind to the descriptor set must match the resources expected by the
    // pipeline which we pass as the first parameter.
    //
    // If you want to run the pipeline on multiple different buffers, you need to create multiple
    // descriptor sets that each contain the buffer you want to run the shader on.
    let compute_desciptor_set = Arc::new(PersistentDescriptorSet::start(compute_pipeline.clone(), 0)
        .add_buffer(vertex_buffer.clone()).unwrap()
        .build().unwrap()
    );

    Model { 
        _window, 
        device, 
        compute_pipeline, 
        compute_queue, 
        compute_desciptor_set, 
        graphics_pipeline, 
        vertex_buffer, 
        render_pass,
        framebuffers 
    }
}

// Handle events related to the window and update the model if necessary
fn event(app: &App, model: Model, event: Event) -> Model {
    if let Event::Update(_update) = event {
// Lets pass through the app.time to our Compute Shader
        // using a push constants. This will allow us to animate the 
        // Waveform. 
        let push_constants = cs::ty::PushConstantData {
            time: app.time,
        };

        // In order to execute our operation, we have to build a command buffer.
        let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(model.device.clone(), model.compute_queue.family()).unwrap()
            // The command buffer only does one thing: execute the compute pipeline.
            // This is called a *dispatch* operation.
            //
            // Note that we clone the pipeline and the set. Since they are both wrapped around an
            // `Arc`, this only clones the `Arc` and not the whole pipeline or set (which aren't
            // cloneable anyway). In this example we would avoid cloning them since this is the last
            // time we use them, but in a real code you would probably need to clone them.            
            .dispatch([1024, 1, 1], model.compute_pipeline.clone(), model.compute_desciptor_set.clone(), push_constants).unwrap()
            // Finish building the command buffer by calling `build`.
            .build().unwrap();

        // Let's execute this command buffer now.
        let future = sync::now(model.device.clone())
            .then_execute(model.compute_queue.clone(), command_buffer).unwrap()    
            // This line instructs the GPU to signal a *fence* once the command buffer has finished
            // execution. A fence is a Vulkan object that allows the CPU to know when the GPU has
            // reached a certain point.
            // We need to signal a fence here because below we want to block the CPU until the GPU has
            // reached that point in the execution.
            .then_signal_fence_and_flush().unwrap();        

        // Blocks execution until the GPU has finished the operation. This method only exists on the
        // future that corresponds to a signalled fence. In other words, this method wouldn't be
        // available if we didn't call `.then_signal_fence_and_flush()` earlier.
        // The `None` parameter is an optional timeout.
        //
        // Note however that dropping the `future` variable (with `drop(future)` for example) would
        // block execution as well, and this would be the case even if we didn't call
        // `.then_signal_fence_and_flush()`.
        // Therefore the actual point of calling `.then_signal_fence_and_flush()` and `.wait()` is to
        // make things more explicit. In the future, if the Rust language gets linear types vulkano may
        // get modified so that only fence-signalled futures can get destroyed like this.
        future.wait(None).unwrap();  

        
    }
    model
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(_app: &App, model: &Model, frame: Frame) -> Frame {
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
    let clear_values = vec!([0.0, 0.0, 1.0, 1.0].into());

    //println!("values = {:#?}", &*model.vertex_buffer.read().unwrap());
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
            model.graphics_pipeline.clone(),
            &dynamic_state,
            vec![model.vertex_buffer.clone()],
            (),
            (),
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

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
}"
        }
    }

mod fs {
    nannou::vulkano_shaders::shader!{
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

mod cs {
    nannou::vulkano_shaders::shader!{
        ty: "compute",
        src: "
#version 450

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) buffer Data {
    vec2 data[];
} data;

layout(push_constant) uniform PushConstantData {
    float time;
} pc;

void main() {
    uint idx = gl_GlobalInvocationID.x;
    float lfo = 1.0;// cos(pc.time * 0.15) * 0.01;
    if(mod(idx,3) == 0) {
        data.data[idx] = vec2(0.0);
        return;
    }
    data.data[idx] = vec2(sin(idx * lfo + pc.time * 1000.0), cos(idx * -lfo + pc.time * -1000.0));
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
