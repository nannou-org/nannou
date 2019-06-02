use nannou::audio::Buffer;
use nannou::prelude::*;
use std::cell::RefCell;
use std::ffi::CStr;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::time::Duration;
use std::iter::Cycle;
use std::ops::Range;

struct Model {
    render_pass: Arc<vk::RenderPassAbstract + Send + Sync>,
    pipeline: Option<Arc<vk::GraphicsPipelineAbstract + Send + Sync>>,
    vertex_buffer: Arc<vk::ImmutableBuffer<[Vertex]>>,
    view_fbo: RefCell<ViewFbo>,
    shade_watcher: shade_runner::Watch,
    shade_msg: shade_runner::Message,
    vert_shader: Arc<vk::pipeline::shader::ShaderModule>,
    frag_shader: Arc<vk::pipeline::shader::ShaderModule>,
    device: Arc<vk::Device>,
    pool: Option<
        Arc<
            RefCell<
                vk::FixedSizeDescriptorSetsPool<Arc<vk::GraphicsPipelineAbstract + Send + Sync>>,
            >,
        >,
    >,
    buffer_pool: vk::CpuBufferPool<UniformData>,
    audio_buffer_pool: vk::CpuBufferPool<[u8; 4]>,
    shader_info: ShaderInfo,
    channel_data: ChannelData,
    _audio_stream: audio::Stream<Audio>,
    audio_data: Receiver<[u8; 64]>,
    audio_buffer: Vec<[u8; 4]>,
    audio_index: Cycle<Range<usize>>,
}

struct Audio {
    tx: Sender<[u8; 64]>,
    buf: [u8; 64],
}

struct ShaderInfo {
    time_delta: f32,
}

#[derive(Debug, Clone)]
struct Vertex {
    position: [f32; 2],
}

#[derive(Debug, Clone)]
struct PushConstantData {
    time: f32,
    time_delta: f32,
    frame: u32,
    frame_rate: u32,
}

#[derive(Debug, Clone)]
struct UniformData {
    resolution: [f32; 3],
}

vk::impl_vertex!(Vertex, position);

struct ChannelData {
    static_channels: Vec<Arc<vk::ImmutableImage<vk::Format>>>,
    dynamic_channels: Vec<Arc<vk::AttachmentImage<vk::Format>>>,
    samplers: Vec<Arc<vk::Sampler>>,
}

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

    let positions = [[-1.0, -1.0], [-1.0, 1.0], [1.0, -1.0], [1.0, 1.0]];
    let data = positions.iter().map(|&position| Vertex { position });
    let queue = app.main_window().swapchain_queue().clone();
    let usage = vk::BufferUsage::all();
    let (vertex_buffer, buffer_future) =
        vk::ImmutableBuffer::from_iter(data, usage, queue).unwrap();

    buffer_future
        .then_signal_fence_and_flush()
        .expect("failed to signal_fence_and_flush buffer and image creation future")
        .wait(None)
        .expect("failed to wait for buffer and image creation future");
    let buffer_pool = vk::CpuBufferPool::uniform_buffer(device.clone());
    let audio_buffer_pool = vk::CpuBufferPool::upload(device.clone());

    // Get the paths to your vertex and fragment shaders.
    let vert_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/vulkan/shaders/hottoy_vert.glsl"
    );
    let frag_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/vulkan/shaders/hottoy_frag.glsl"
    );
    // Create a watcher that will reload both shaders if there is any change to the
    // parent directory eg. '/examples/vulkan/shaders/'
    let shade_watcher =
        shade_runner::Watch::create(vert_path, frag_path, Duration::from_millis(50))
            .expect("failed to create watcher");
    // Wait on the first message,
    // which is the shaders compiling and parsing.
    // The message is a Result which indicates if
    // the shader successfully compiled and parsed.
    let shade_msg = shade_watcher
        .rx
        .recv()
        .expect("Failed to receive shader")
        .expect("failed to compile shader");
    // Create the shader module from the compiled
    // shader in the message. It is simply
    // a Vec<u8>.
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
    let pool = None;
    // The render pass we created above only describes the layout of our framebuffer. Before we
    // can draw we also need to create the actual framebuffer.
    let view_fbo = RefCell::new(ViewFbo::default());

    let shader_info = ShaderInfo { time_delta: 0.0 };

    let channel_data = load_textures(app, device.clone());

    let (tx, audio_data) = mpsc::channel();
    let buf = [0u8; 64];
    let audio_stream = app
        .audio
        .new_input_stream(Audio { tx, buf }, audio)
        .build()
        .expect("Failed to create audio stream");
    audio_stream.play();
    let audio_buffer = vec![[0u8; 4]; 512 * 512];
    let audio_index = (0..(512 * 512)).cycle();
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
        pool,
        buffer_pool,
        audio_buffer_pool,
        shader_info,
        channel_data,
        _audio_stream: audio_stream,
        audio_data,
        audio_buffer,
        audio_index,
    };

    // Here we won't to update the pipeline but
    // we need data from the model so it has to
    // happen after model is created.
    update_pipeline(&mut model);
    let pool = vk::FixedSizeDescriptorSetsPool::new(model.pipeline.as_ref().unwrap().clone(), 0);
    model.pool = Some(Arc::new(RefCell::new(pool)));
    model
}

fn audio(audio: &mut Audio, s: &Buffer<u16>) {
    for (i, j) in s.iter().step_by(2).map(|&b| b as u8).zip(audio.buf.iter_mut()) {
        *j = i;
    }
    audio.tx.send(audio.buf).ok();
}

fn update(_app: &App, model: &mut Model, update: Update) {
    for buffer in model.audio_data.try_iter().take(4096) {
        model.audio_index.next();
        for d in buffer.chunks_exact(4) {
            let n = model.audio_index.next().unwrap();
            model.audio_buffer[n].copy_from_slice(d);
        }
    }

    model.shader_info.time_delta = (update.since_last.as_secs() as f64
        + update.since_last.subsec_nanos() as f64 * 1e-9) as f32;
    // Get the latest message from the watcher.
    // There will be a message for any change to the shaders.
    let shader_msg = model.shade_watcher.rx.try_iter().last();
    if let Some(shade_msg) = shader_msg {
        match shade_msg {
            Ok(shade_msg) => {
                // Got a successfully compiled and parsed message.
                // Recreate the shader modules and update the pipeline.
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
                // The shader changed but failed to compile.
                // This contains the reason why it failed.
                println!("Error compiling shader {:?}", e);
            }
        }
    }
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(app: &App, model: &Model, frame: Frame) -> Frame {
    // Dynamic viewports allow us to recreate just the viewport when the window is resized
    // Otherwise we would have to recreate the whole pipeline.
    let [w, h] = frame.swapchain_image().dimensions();
    let viewport = vk::ViewportBuilder::new().build([w as _, h as _]);
    let dynamic_state = vk::DynamicState::default().viewports(vec![viewport]);

    let push_constants = PushConstantData {
        time: app.time,
        time_delta: model.shader_info.time_delta,
        frame: app.elapsed_frames() as u32,
        frame_rate: (60.0 / model.shader_info.time_delta) as u32,
    };

    let uniform_buffer = UniformData {
        resolution: [w as _, h as _, 1.0],
    };

    let sub_buffer = model
        .buffer_pool
        .next(uniform_buffer)
        .expect("Failed to get next uniform buffer");
    let audio_sub_buffer = model
        .audio_buffer_pool
        .chunk(model.audio_buffer.iter().cloned())
        .expect("Failed to get next uniform buffer");

    let descriptor_set = model
        .pool
        .as_ref()
        .unwrap()
        .borrow_mut()
        .next()
        .add_buffer(sub_buffer)
        .expect("Failed to add uniform buffer")
        .add_sampled_image(
            model.channel_data.static_channels[0].clone(),
            model.channel_data.samplers[0].clone(),
        )
        .expect("Failed to add texture sampler")
        .add_sampled_image(
            model.channel_data.static_channels[1].clone(),
            model.channel_data.samplers[1].clone(),
        )
        .expect("Failed to add texture sampler")
        .add_sampled_image(
            model.channel_data.static_channels[2].clone(),
            model.channel_data.samplers[2].clone(),
        )
        .expect("Failed to add texture sampler")
        .add_sampled_image(
            model.channel_data.dynamic_channels[0].clone(),
            model.channel_data.samplers[3].clone(),
        )
        .expect("Failed to add texture sampler")
        .build()
        .expect("Failed to get next descriptor set");

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
        .copy_buffer_to_image(
            audio_sub_buffer,
            model.channel_data.dynamic_channels[0].clone(),
        )
        .unwrap()
        .begin_render_pass(model.view_fbo.borrow().expect_inner(), false, clear_values)
        .unwrap()
        .draw(
            model.pipeline.clone().unwrap(),
            &dynamic_state,
            vec![model.vertex_buffer.clone()],
            descriptor_set,
            push_constants,
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
    // Here we use the other part of the latest message
    // from shade_runner. It is the entry point for vulkano
    // to use your compiled shader.
    // If there is any errors here make sure you have the
    // same version of vulkano in your application and shade_runner.
    // Cargo patch can be handy for this.
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
            .triangle_strip()
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

fn load_textures(app: &App, device: Arc<vk::Device>) -> ChannelData {
    let dims = [512, 512];
    let (texture, tex_future) = {
        let mut texture_path = app.assets_path().expect("Failed to load asset path");
        texture_path.push(PathBuf::from("images/noise/blue/512_512/LDR_RGBA_0.png"));
        let image = image::open(texture_path).unwrap().to_rgba();
        let (width, height) = image.dimensions();
        let image_data = image.into_raw().clone();

        vk::ImmutableImage::from_iter(
            image_data.iter().cloned(),
            vk::image::Dimensions::Dim2d { width, height },
            vk::Format::R8G8B8A8Srgb,
            app.main_window().swapchain_queue().clone(),
        )
        .unwrap()
    };
    let (green_texture, empty_tex_future_1) = vk::ImmutableBuffer::from_iter(
        (0..(512 * 512)).map(|_| [0u8, 255u8, 0u8, 0u8]),
        vk::BufferUsage::transfer_source(),
        app.main_window().swapchain_queue().clone(),
    )
    .unwrap();

    let dynamic_image = vk::AttachmentImage::with_usage(
        device.clone(),
        dims,
        vk::Format::R8G8B8A8Srgb,
        vk::ImageUsage {
            transfer_destination: true,
            sampled: true,
            ..vk::ImageUsage::none()
        },
    )
    .expect("Failed to create attachment image");
    let samplers = (0..4)
        .map(|_| {
            vk::SamplerBuilder::new()
                .address_u(vk::SamplerAddressMode::Repeat)
                .address_v(vk::SamplerAddressMode::Repeat)
                .build(device.clone())
                .expect("Failed to create sampler")
        })
        .collect::<Vec<_>>();
    tex_future
        .join(empty_tex_future_1)
        .then_signal_fence_and_flush()
        .expect("`then_signal_fence_and_flush` failed")
        .wait(None)
        .expect("failed to wait for futures");

    let (green_image_1, green_tex_future_1) = vk::ImmutableImage::from_buffer(
        green_texture.clone(),
        vk::image::Dimensions::Dim2d {
            width: dims[0],
            height: dims[1],
        },
        vk::Format::R8G8B8A8Srgb,
        app.main_window().swapchain_queue().clone(),
    )
    .expect("Failed to create empty image");
    let (green_image_2, green_tex_future_2) = vk::ImmutableImage::from_buffer(
        green_texture,
        vk::image::Dimensions::Dim2d {
            width: dims[0],
            height: dims[1],
        },
        vk::Format::R8G8B8A8Srgb,
        app.main_window().swapchain_queue().clone(),
    )
    .expect("Failed to create empty image");

    green_tex_future_1
        .join(green_tex_future_2)
        .then_signal_fence_and_flush()
        .expect("`then_signal_fence_and_flush` failed")
        .wait(None)
        .expect("failed to wait for futures");

    let static_channels = vec![texture, green_image_1, green_image_2];
    let dynamic_channels = vec![dynamic_image];
    ChannelData {
        static_channels,
        dynamic_channels,
        samplers,
    }
}
