//! A demonstration of playing back a sequence of images.
//!
//! This approach loads a directory of images into a single texture array. We only ever present a
//! single layer of the texture at a time by creating a texture view. We select which layer to view
//! by using a `current_layer` variable and updating it based on a frame rate that we determine by
//! the mouse x position.
//!
//! An interesting exercise might be to make a copy of this example and attempt to smooth the slow
//! frame rates by interpolating between two of the layers at a time. Hint: this would likely
//! require adding a second texture view binding to the bind group and its layout.

use nannou::image;
use nannou::image::RgbaImage;
use nannou::prelude::*;
use std::path::{Path, PathBuf};

struct Model {
    current_layer: f32,
    texture_array: wgpu::Texture,
    texture_view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
}

// The vertex type that we will use to represent a point on our triangle.
#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 2],
}

// The vertices that make up the rectangle to which the image will be drawn.
const VERTICES: [Vertex; 4] = [
    Vertex {
        position: [-1.0, 1.0],
    },
    Vertex {
        position: [-1.0, -1.0],
    },
    Vertex {
        position: [1.0, 1.0],
    },
    Vertex {
        position: [1.0, -1.0],
    },
];

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    // Load the images.
    let sequence_path = app
        .assets_path()
        .unwrap()
        .join("images")
        .join("spinning_dancer");

    println!("Loading images...");
    let (images, (img_w, img_h)) = load_images(&sequence_path);
    println!("Done!");

    let w_id = app
        .new_window()
        .size(img_w, img_h)
        .view(view)
        .build()
        .unwrap();

    let window = app.window(w_id).unwrap();
    let device = window.swap_chain_device();
    let format = Frame::TEXTURE_FORMAT;
    let msaa_samples = window.msaa_samples();

    let vs_mod = wgpu::shader_from_spirv_bytes(device, include_bytes!("shaders/vert.spv"));
    let fs_mod = wgpu::shader_from_spirv_bytes(device, include_bytes!("shaders/frag.spv"));

    let texture_array = {
        // The wgpu device queue used to load the image data.
        let queue = window.swap_chain_queue();
        // Describe how we will use the texture so that the GPU may handle it efficiently.
        let usage = wgpu::TextureUsage::SAMPLED;
        let iter = images.iter().map(|&(_, ref img)| img);
        wgpu::Texture::load_array_from_image_buffers(device, queue, usage, iter)
            .expect("tied to load texture array with an empty image buffer sequence")
    };
    let layer = 0;
    let texture_view = texture_array.view().layer(layer).build();

    // Create the sampler for sampling from the source texture.
    let sampler_desc = wgpu::SamplerBuilder::new().into_descriptor();
    let sampler_filtering = wgpu::sampler_filtering(&sampler_desc);
    let sampler = device.create_sampler(&sampler_desc);

    let bind_group_layout =
        create_bind_group_layout(device, texture_view.sample_type(), sampler_filtering);
    let bind_group = create_bind_group(device, &bind_group_layout, &texture_view, &sampler);
    let pipeline_layout = create_pipeline_layout(device, &bind_group_layout);
    let render_pipeline = create_render_pipeline(
        device,
        &pipeline_layout,
        &vs_mod,
        &fs_mod,
        format,
        msaa_samples,
    );

    // Create the vertex buffer.
    let vertices_bytes = vertices_as_bytes(&VERTICES[..]);
    let usage = wgpu::BufferUsage::VERTEX;
    let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: vertices_bytes,
        usage,
    });

    Model {
        current_layer: 0.0,
        texture_array,
        texture_view,
        sampler,
        bind_group_layout,
        bind_group,
        vertex_buffer,
        render_pipeline,
    }
}

fn update(app: &App, model: &mut Model, update: Update) {
    // Update which layer in the texture array that are viewing.
    let window = app.main_window();
    let device = window.swap_chain_device();

    // Determine how fast to play back the frames based on the mouse x.
    let win_rect = window.rect();
    let fps = map_range(
        app.mouse.x,
        win_rect.left(),
        win_rect.right(),
        -100.0,
        100.0,
    );

    // Update which layer we are viewing based on the playback speed and layer count.
    let layer_count = model.texture_array.extent().depth;
    model.current_layer = fmod(
        model.current_layer + update.since_last.secs() as f32 * fps,
        layer_count as f32,
    );

    // Update the view and the bind group ready for drawing.
    let layer = model.current_layer as u32;
    model.texture_view = model.texture_array.view().layer(layer).build();
    model.bind_group = create_bind_group(
        device,
        &model.bind_group_layout,
        &model.texture_view,
        &model.sampler,
    );
}

fn view(_app: &App, model: &Model, frame: Frame) {
    let mut encoder = frame.command_encoder();
    let mut render_pass = wgpu::RenderPassBuilder::new()
        .color_attachment(frame.texture_view(), |color| color)
        .begin(&mut encoder);
    render_pass.set_bind_group(0, &model.bind_group, &[]);
    render_pass.set_pipeline(&model.render_pipeline);
    render_pass.set_vertex_buffer(0, model.vertex_buffer.slice(..));
    let vertex_range = 0..VERTICES.len() as u32;
    let instance_range = 0..1;
    render_pass.draw(vertex_range, instance_range);
}

// Load a directory of images and returns them sorted by filename alongside their dimensions.
// This function assumes all the images have the same dimensions.
fn load_images(dir: &Path) -> (Vec<(PathBuf, RgbaImage)>, (u32, u32)) {
    let mut images = vec![];
    let mut dims = (0, 0);
    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let image = match image::open(&path) {
            Ok(img) => img.into_rgba8(),
            Err(err) => {
                eprintln!("failed to open {} as an image: {}", path.display(), err);
                continue;
            }
        };
        let (w, h) = image.dimensions();
        dims = (w, h);
        images.push((path, image));
    }
    images.sort_by_key(|(path, _)| path.clone());
    (images, dims)
}

fn create_bind_group_layout(
    device: &wgpu::Device,
    texture_sample_type: wgpu::TextureSampleType,
    sampler_filtering: bool,
) -> wgpu::BindGroupLayout {
    wgpu::BindGroupLayoutBuilder::new()
        .texture(
            wgpu::ShaderStage::FRAGMENT,
            false,
            wgpu::TextureViewDimension::D2,
            texture_sample_type,
        )
        .sampler(wgpu::ShaderStage::FRAGMENT, sampler_filtering)
        .build(device)
}

fn create_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    texture: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    wgpu::BindGroupBuilder::new()
        .texture_view(texture)
        .sampler(sampler)
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
    sample_count: u32,
) -> wgpu::RenderPipeline {
    wgpu::RenderPipelineBuilder::from_layout(layout, vs_mod)
        .fragment_shader(fs_mod)
        .color_format(dst_format)
        .add_vertex_buffer::<Vertex>(&wgpu::vertex_attr_array![0 => Float2])
        .sample_count(sample_count)
        .primitive_topology(wgpu::PrimitiveTopology::TriangleStrip)
        .build(device)
}

// See the `nannou::wgpu::bytes` documentation for why this is necessary.
fn vertices_as_bytes(data: &[Vertex]) -> &[u8] {
    unsafe { wgpu::bytes::from_slice(data) }
}
