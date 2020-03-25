use nannou::prelude::*;
use nannou::image;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use thiserror::Error;

/// A render pipeline designed for hotloading!
pub struct IsfPipeline {
    isf: Option<(isf::Isf, IsfData)>,
    isf_err: Option<IsfError>,
    vs: Shader,
    fs: Shader,
    isf_uniform_buffer: wgpu::Buffer,
    isf_inputs_uniform_buffer: wgpu::Buffer,
    isf_bind_group_layout: wgpu::BindGroupLayout,
    isf_inputs_bind_group_layout: wgpu::BindGroupLayout,
    isf_textures_bind_group_layout: wgpu::BindGroupLayout,
    isf_bind_group: wgpu::BindGroup,
    isf_inputs_bind_group: wgpu::BindGroup,
    isf_textures_bind_group: wgpu::BindGroup,
    layout: wgpu::PipelineLayout,
    render_pipeline: Option<wgpu::RenderPipeline>,
    vertex_buffer: wgpu::Buffer,
    dst_format: wgpu::TextureFormat,
    dst_texture_size: [u32; 2],
    dst_sample_count: u32,
}

/// The first set of ISF uniforms that are available to every ISF shader.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct IsfUniforms {
    pass_index: i32,
    render_size: [f32; 2],
    time: f32,
    time_delta: f32,
    date: [f32; 4],
    frame_index: i32,
}

/// Created directly after successfully parsing an `Isf`.
///
/// `imported` textures can be accessed by the user.
#[derive(Debug)]
pub struct IsfData {
    imported: HashMap<PathBuf, ImageState>,
    inputs: HashMap<InputName, IsfInputData>,
}

/// The state of the image.
#[derive(Debug)]
pub enum ImageState {
    None,
    Loading(mpsc::Receiver<image::RgbaImage>),
    Ready(Result<ImageData, ImageLoadError>),
}

/// Handles to both the cpu and gpu representations of the image.
#[derive(Debug)]
pub struct ImageData {
    pub image: image::RgbaImage,
    pub texture: wgpu::Texture,
}

pub type InputName = String;

#[derive(Debug)]
pub enum IsfInputData {
    Event,
    Bool(bool),
    Long(i32),
    Float(f32),
    Point2d(Point2),
    Color(LinSrgba),
    // TODO?
    Image,
    Audio,
    AudioFft,
}

pub struct IsfEvent {
    pub happening: bool,
}

#[derive(Clone, Debug)]
struct IsfInputUniforms {
    // Each supported uniform field type is 32-bit long, so store them as such.
    fields: Vec<u32>,
}

/// A shader with some extra information relating to recent compilation success/failure.
pub struct Shader {
    source: ShaderSource,
    module: Option<wgpu::ShaderModule>,
    compile_err: Option<hotglsl::CompileError>,
}

enum ShaderSource {
    Path(PathBuf),
    HardCoded,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 2],
}

/// Errors that can occur while trying to load and parse ISF from the fragment shader.
#[derive(Debug, Error)]
pub enum IsfError {
    #[error("{err}")]
    Parse {
        #[from]
        err: isf::ParseError,
    },
    #[error("failed to read fragment shader for ISF: {err}")]
    Io {
        #[from]
        err: std::io::Error,
    }
}

/// Errors that might occur while loading an image.
#[derive(Debug, Error)]
pub enum ImageLoadError {
    #[error("an IO error: {err}")]
    Io {
        #[from]
        err: std::io::Error,
    }
}

const VERTICES: [Vertex; 4] = [
    Vertex {
        position: [-1.0, -1.0],
    },
    Vertex {
        position: [-1.0, 1.0],
    },
    Vertex {
        position: [1.0, -1.0],
    },
    Vertex {
        position: [1.0, 1.0],
    },
];

impl wgpu::VertexDescriptor for Vertex {
    const STRIDE: wgpu::BufferAddress = std::mem::size_of::<Vertex>() as _;
    const ATTRIBUTES: &'static [wgpu::VertexAttributeDescriptor] =
        &[wgpu::VertexAttributeDescriptor {
            format: wgpu::VertexFormat::Float2,
            offset: 0,
            shader_location: 0,
        }];
}

impl ImageState {
    /// Whether or not the texture is currently loading.
    pub fn is_loading(&self) -> bool {
        match *self {
            ImageState::Loading(_) => true,
            _ => false,
        }
    }

    /// If the image has been loaded, provides access to the result.
    ///
    /// Returns `None` if the image is still loading or has not started loading.
    pub fn ready(&self) -> Option<Result<&ImageData, &ImageLoadError>> {
        match *self {
            ImageState::Ready(ref res) => Some(res.as_ref()),
            _ => None,
        }
    }
}

impl ShaderSource {
    fn as_path(&self) -> Option<&Path> {
        match *self {
            ShaderSource::Path(ref path) => Some(path),
            ShaderSource::HardCoded => None,
        }
    }
}

impl Shader {
    pub fn from_path(device: &wgpu::Device, path: PathBuf) -> Self {
        let res = hotglsl::compile(&path);
        let (bytes, compile_err) = split_compile_result(res);
        let module = bytes.map(|b| spirv_bytes_to_mod(device, &b));
        let source = ShaderSource::Path(path);
        Shader {
            source,
            module,
            compile_err,
        }
    }

    /// Create the default vertex shader for ISF fragment shaders.
    pub fn vertex_default(device: &wgpu::Device) -> Self {
        let vs = include_bytes!("shaders/vert.spv");
        let vs_spirv = wgpu::read_spirv(std::io::Cursor::new(&vs[..]))
            .expect("failed to read hard-coded SPIRV");
        let module = Some(device.create_shader_module(&vs_spirv));
        let compile_err = None;
        let source = ShaderSource::HardCoded;
        Shader {
            source,
            module,
            compile_err,
        }
    }

    pub fn update(&mut self, device: &wgpu::Device, res: Result<Vec<u8>, hotglsl::CompileError>) {
        let (bytes, compile_err) = split_compile_result(res);
        self.compile_err = compile_err;
        if let Some(b) = bytes {
            self.module = Some(spirv_bytes_to_mod(device, &b));
        }
    }
}

impl IsfPipeline {
    /// Construct a new **IsfPipeline**.
    pub fn new(
        device: &wgpu::Device,
        vs_path: Option<PathBuf>,
        fs_path: PathBuf,
        dst_format: wgpu::TextureFormat,
        dst_texture_size: [u32; 2],
        dst_sample_count: u32,
    ) -> Self {
        // Retrieve the `Isf` instance.
        let isf_res = std::fs::read_to_string(&fs_path)
            .map_err(|err| IsfError::from(err))
            .and_then(|s| isf::parse(&s).map_err(From::from));
        let (isf, isf_err) = match isf_res {
            Ok(isf) => {
                // TODO: ISF data.
                let isf_data = unimplemented!();
                (Some((isf, isf_data)), None)
            },
            Err(err) => (None, Some(err)),
        };

        // Create the shaders.
        let vs = match vs_path {
            None => Shader::vertex_default(device),
            Some(vs_path) => Shader::from_path(device, vs_path),
        };
        let fs = Shader::from_path(device, fs_path);

        // Prepare the uniform buffers.
        let [dst_tex_w, dst_tex_h] = dst_texture_size;
        let isf_uniforms = IsfUniforms {
            pass_index: 0,
            render_size: [dst_tex_w as f32, dst_tex_h as f32],
            time: 0.0,
            time_delta: 0.0,
            date: [0.0; 4],
            frame_index: 0,
        };
        let isf_uniform_buffer = device
            .create_buffer_mapped(1, wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST)
            .fill_from_slice(&[isf_uniforms]);
        type IsfInputUniforms = [u32; 128];
        let isf_input_uniforms = [0u32; 128];
        let isf_inputs_uniform_buffer = device
            .create_buffer_mapped(1, wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST)
            .fill_from_slice(&[isf_input_uniforms]);

        // Prepare the bind group layouts.
        let isf_bind_group_layout = wgpu::BindGroupLayoutBuilder::new()
            .uniform_buffer(wgpu::ShaderStage::FRAGMENT, false)
            .build(device);
        let isf_inputs_bind_group_layout = wgpu::BindGroupLayoutBuilder::new()
            .uniform_buffer(wgpu::ShaderStage::FRAGMENT, false)
            .build(device);
        let isf_textures_bind_group_layout = create_isf_textures_bind_group_layout(device, isf.as_ref().map(|(ref isf, _)| isf));

        // Create the bind groups
        let isf_bind_group = wgpu::BindGroupBuilder::new()
            .buffer::<IsfUniforms>(&isf_uniform_buffer, 0..1)
            .build(device, &isf_bind_group_layout);
        let isf_inputs_bind_group = wgpu::BindGroupBuilder::new()
            .buffer::<IsfInputUniforms>(&isf_inputs_uniform_buffer, 0..1)
            .build(device, &isf_inputs_bind_group_layout);
        let isf_textures_bind_group: wgpu::BindGroup = unimplemented!();

        // Create the render pipeline.
        let layout = create_pipeline_layout(
            device,
            &[&isf_bind_group_layout, &isf_inputs_bind_group_layout, &isf_textures_bind_group_layout],
        );
        let render_pipeline = match (vs.module.as_ref(), fs.module.as_ref()) {
            (Some(vs_mod), Some(fs_mod)) => Some(create_render_pipeline(
                device,
                &layout,
                vs_mod,
                fs_mod,
                dst_format,
                dst_sample_count,
            )),
            _ => None,
        };

        // The quad vertex buffer.
        let vertex_buffer = device
            .create_buffer_mapped(VERTICES.len(), wgpu::BufferUsage::VERTEX)
            .fill_from_slice(&VERTICES[..]);

        Self {
            isf,
            isf_err,
            vs,
            fs,
            isf_uniform_buffer,
            isf_inputs_uniform_buffer,
            isf_bind_group_layout,
            isf_inputs_bind_group_layout,
            isf_textures_bind_group_layout,
            isf_bind_group,
            isf_inputs_bind_group,
            isf_textures_bind_group,
            layout,
            render_pipeline,
            vertex_buffer,
            dst_format,
            dst_texture_size,
            dst_sample_count,
        }
    }

    /// Update the pipeline's shaders with the given result.
    ///
    /// Only shaders that match the yielded path will be updated with the yielded result.
    pub fn update_shaders<I>(&mut self, device: &wgpu::Device, updates: I)
    where
        I: IntoIterator<Item = (PathBuf, Result<Vec<u8>, hotglsl::CompileError>)>,
    {
        let mut needs_recreation = false;
        for (path, result) in updates {
            if self.vs.source.as_path() == Some(&path) {
                self.vs.update(device, result);
                needs_recreation = true;
            } else if self.fs.source.as_path() == Some(&path) {
                // TODO: Update ISF and everything.
                unimplemented!();

                self.fs.update(device, result);
                needs_recreation = true;
            }
        }
        if !needs_recreation {
            return;
        }
        if let (Some(vs_mod), Some(fs_mod)) = (self.vs.module.as_ref(), self.fs.module.as_ref()) {
            self.render_pipeline = Some(create_render_pipeline(
                device,
                &self.layout,
                vs_mod,
                fs_mod,
                self.dst_format,
                self.dst_sample_count,
            ));
        }
    }

    /// Given an encoder, submits a render pass command for drawing the pipeline to the given
    /// texture.
    ///
    /// If the pipeline has not yet been created because it has not yet compiled the necessary
    /// shaders correctly, the render pass will not be encoded.
    pub fn encode_render_pass(
        &self,
        dst_texture: &wgpu::TextureViewHandle,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        if let Some(ref pipeline) = self.render_pipeline {
            let mut render_pass = wgpu::RenderPassBuilder::new()
                .color_attachment(dst_texture, |color| color)
                .begin(encoder);
            render_pass.set_pipeline(pipeline);
            render_pass.set_vertex_buffers(0, &[(&self.vertex_buffer, 0)]);
            render_pass.set_bind_group(0, &self.isf_bind_group, &[]);
            render_pass.set_bind_group(1, &self.isf_inputs_bind_group, &[]);
            render_pass.set_bind_group(2, &self.isf_textures_bind_group, &[]);
            let vertex_range = 0..VERTICES.len() as u32;
            let instance_range = 0..1;
            render_pass.draw(vertex_range, instance_range);
        }
    }

    /// Encode a render pass command for drawing the output of the pipeline to the given frame.
    ///
    /// Uses `encode_render_pass` internally.
    pub fn encode_to_frame(&self, frame: &Frame) {
        let mut encoder = frame.command_encoder();
        self.encode_render_pass(frame.texture_view(), &mut *encoder);
    }

    /// Returns the current compilation error for the vertex shader if there is one.
    ///
    /// Returns `Some` if the last call to `update_shaders` contained a compilation error for the
    /// vertex shader.
    pub fn vs_compile_err(&self) -> Option<&hotglsl::CompileError> {
        self.vs.compile_err.as_ref()
    }

    /// Returns the current compilation error for the vertex shader if there is one.
    ///
    /// Returns `Some` if the last call to `update_shaders` contained a compilation error for the
    /// vertex shader.
    pub fn fs_compile_err(&self) -> Option<&hotglsl::CompileError> {
        self.fs.compile_err.as_ref()
    }
}

fn split_compile_result(
    res: Result<Vec<u8>, hotglsl::CompileError>,
) -> (Option<Vec<u8>>, Option<hotglsl::CompileError>) {
    match res {
        Ok(bytes) => (Some(bytes), None),
        Err(err) => (None, Some(err)),
    }
}

fn spirv_bytes_to_mod(device: &wgpu::Device, bytes: &[u8]) -> wgpu::ShaderModule {
    let cursor = std::io::Cursor::new(&bytes[..]);
    let vs_spirv = wgpu::read_spirv(cursor).expect("failed to read hard-coded SPIRV");
    device.create_shader_module(&vs_spirv)
}

// The name of each texture specified within the ISF source file.
fn isf_texture_names(isf: &isf::Isf) -> impl Iterator<Item = &str> {
    let imported = isf.imported.keys().map(|s| &s[..]);
    let inputs = isf
        .inputs
        .iter()
        .filter_map(|input| {
            match input.ty {
                isf::InputType::Image | isf::InputType::Audio(_) | isf::InputType::AudioFft(_) => {
                    Some(&input.name[..])
                }
                _ => None,
            }
        });
    let passes = isf.passes.iter().filter_map(|pass| pass.target.as_ref().map(|s| &s[..]));
    imported.chain(inputs).chain(passes)
}

// // Create a texture for each imported image, input and pass that requires one.
// fn create_isf_textures(device: &wgpu::Device, isf: &Isf) -> Vec<wgpu::Texture> {
//     for (key, img) in isf.imported.keys()
// }

// Includes the sampler and then all textures for all images and passes.
fn create_isf_textures_bind_group_layout(
    device: &wgpu::Device,
    isf: Option<&isf::Isf>,
) -> wgpu::BindGroupLayout {
    // Begin with the sampler.
    let mut builder = wgpu::BindGroupLayoutBuilder::new()
        .sampler(wgpu::ShaderStage::FRAGMENT);
    if let Some(isf) = isf {
        for _ in isf_texture_names(isf) {
            builder = builder.sampled_texture(
                wgpu::ShaderStage::FRAGMENT,
                false,
                wgpu::TextureViewDimension::D2,
            );
        }
    }
    builder.build(device)
}

fn create_pipeline_layout(
    device: &wgpu::Device,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
) -> wgpu::PipelineLayout {
    let desc = wgpu::PipelineLayoutDescriptor {
        bind_group_layouts,
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
    wgpu::RenderPipelineBuilder::from_layout(layout, &vs_mod)
        .fragment_shader(fs_mod)
        .color_format(dst_format)
        .add_vertex_buffer::<Vertex>()
        .sample_count(sample_count)
        .primitive_topology(wgpu::PrimitiveTopology::TriangleStrip)
        .build(device)
}
