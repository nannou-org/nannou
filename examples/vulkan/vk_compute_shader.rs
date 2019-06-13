use nannou::prelude::*;
use std::sync::Arc;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    device: Arc<vk::Device>,
    queue: Arc<vk::Queue>,
    pipeline: Arc<dyn vk::ComputePipelineAbstract + Send + Sync>,
    desciptor_set: Arc<dyn vk::DescriptorSet + Send + Sync>,
    data_buffer: Arc<vk::CpuAccessibleBuffer<[f32]>>,
}

fn model(app: &App) -> Model {
    app.new_window()
        .with_dimensions(1440, 512)
        .with_title("nannou")
        .view(view)
        .build()
        .unwrap();

    let instance = app.vk_instance().clone();

    // Choose which physical device to use.
    let physical = vk::PhysicalDevice::enumerate(&instance).next().unwrap();

    // Choose the queue of the physical device which is going to run our compute operation.
    //
    // The Vulkan specs guarantee that a compliant implementation must provide at least one queue
    // that supports compute operations.
    let queue_family = physical
        .queue_families()
        .find(|&q| q.supports_compute())
        .unwrap();

    // Now initializing the device.
    let (device, mut queues) = vk::Device::new(
        physical,
        physical.supported_features(),
        &vk::DeviceExtensions::none(),
        [(queue_family, 0.5)].iter().cloned(),
    )
    .unwrap();

    // Since we can request multiple queues, the `queues` variable is in fact an iterator. In this
    // example we use only one queue, so we just retrieve the first and only element of the
    // iterator and throw it away.
    let queue = queues.next().unwrap();

    // We need to create the compute pipeline that describes our operation.
    //
    // If you are familiar with graphics pipeline, the principle is the same except that compute
    // pipelines are much simpler to create.
    let pipeline = Arc::new({
        let compute_shader = cs::Shader::load(device.clone()).unwrap();
        vk::ComputePipeline::new(device.clone(), &compute_shader.main_entry_point(), &()).unwrap()
    });

    // We start by creating the buffer that will store the data.
    let data_buffer = {
        // Iterator that produces the data.
        let data_iter = (0..1024).map(|n| n as f32);
        // Builds the buffer and fills it with this iterator.
        let usage = vk::BufferUsage::all();
        vk::CpuAccessibleBuffer::from_iter(device.clone(), usage, data_iter).unwrap()
    };

    // In order to let the shader access the buffer, we need to build a *descriptor set* that
    // contains the buffer.
    //
    // The resources that we bind to the descriptor set must match the resources expected by the
    // pipeline which we pass as the first parameter.
    //
    // If you want to run the pipeline on multiple different buffers, you need to create multiple
    // descriptor sets that each contain the buffer you want to run the shader on.
    let desciptor_set = Arc::new(
        vk::PersistentDescriptorSet::start(pipeline.clone(), 0)
            .add_buffer(data_buffer.clone())
            .unwrap()
            .build()
            .unwrap(),
    );

    Model {
        pipeline,
        device,
        queue,
        desciptor_set,
        data_buffer,
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    // Lets pass through the app.time to our Compute Shader
    // using a push constants. This will allow us to animate the
    // Waveform.
    let push_constants = cs::ty::PushConstantData { time: app.time };

    // In order to execute our operation, we have to build a command buffer.
    let device = model.device.clone();
    let queue_fam = model.queue.family();
    let command_buffer = vk::AutoCommandBufferBuilder::primary_one_time_submit(device, queue_fam)
        .unwrap()
        // The command buffer only does one thing: execute the compute pipeline.
        // This is called a *dispatch* operation.
        //
        // Note that we clone the pipeline and the set. Since they are both wrapped around an
        // `Arc`, this only clones the `Arc` and not the whole pipeline or set (which aren't
        // cloneable anyway). In this example we would avoid cloning them since this is the last
        // time we use them, but in a real code you would probably need to clone them.
        .dispatch(
            [1024, 1, 1],
            model.pipeline.clone(),
            model.desciptor_set.clone(),
            push_constants,
        )
        .unwrap()
        // Finish building the command buffer by calling `build`.
        .build()
        .unwrap();

    // Let's execute this command buffer now.
    let future = vk::sync::now(model.device.clone())
        .then_execute(model.queue.clone(), command_buffer)
        .unwrap()
        // This line instructs the GPU to signal a *fence* once the command buffer has finished
        // execution. A fence is a Vulkan object that allows the CPU to know when the GPU has
        // reached a certain point.
        // We need to signal a fence here because below we want to block the CPU until the GPU
        // has reached that point in the execution.
        .then_signal_fence_and_flush()
        .unwrap();

    // Blocks execution until the GPU has finished the operation. This method only exists on
    // the future that corresponds to a signalled fence. In other words, this method wouldn't
    // be available if we didn't call `.then_signal_fence_and_flush()` earlier.
    //
    // The `None` parameter is an optional timeout.
    //
    // Note however that dropping the `future` variable (with `drop(future)` for example) would
    // block execution as well, and this would be the case even if we didn't call
    // `.then_signal_fence_and_flush()`. Therefore the actual point of calling
    // `.then_signal_fence_and_flush()` and `.wait()` is to make things more explicit. In the
    // future, if the Rust language gets linear types vulkano may get modified so that only
    // fence-signalled futures can get destroyed like this.
    future.wait(None).unwrap();
}

fn view(app: &App, model: &Model, frame: Frame) -> Frame {
    // Begin drawing
    let draw = app.draw();
    let win = app.window_rect();

    // Clear the background to blue.
    draw.background().color(BLACK);

    // Now that the GPU is done, the content of the buffer should have been modified. Let's
    // check it out.
    //
    // The call to `read()` would return an error if the buffer was still in use by the GPU.
    let data_buffer_content = model.data_buffer.read().unwrap();
    for (n, &f) in data_buffer_content.iter().enumerate() {
        let x = map_range(n as f32, 0.0, 1024.0, win.left(), win.right());
        let y = 0.0;
        let h = f;
        let hue = map_range(h, 0.0, 512.0, 0.4, 0.6);
        draw.rect().x_y(x, y).w_h(1.0, h).hsv(hue, 1.0, 1.0);
    }

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the cleared frame.
    frame
}

mod cs {
    nannou::vk::shaders::shader! {
        ty: "compute",
        src: "
#version 450

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) buffer Data {
    float data[];
} data;

layout(push_constant) uniform PushConstantData {
    float time;
} pc;

void main() {
    uint idx = gl_GlobalInvocationID.x;
    float lfo = cos(pc.time * 0.15) * 0.01;
    data.data[idx] = sin(idx * lfo + pc.time) * 512.0;
}"
    }
}
