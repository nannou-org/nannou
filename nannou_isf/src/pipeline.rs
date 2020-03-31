use nannou::image;
use nannou::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use thiserror::Error;
use threadpool::ThreadPool;

/// A render pipeline designed for hotloading!
pub struct IsfPipeline {
    isf: Option<isf::Isf>,
    pub isf_data: IsfData,
    isf_err: Option<IsfError>,
    image_loader: ImageLoader,
    vs: Shader,
    fs: Shader,
    sampler: wgpu::Sampler,
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

/// Timing information passed into the shader.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct IsfTime {
    /// The time since the start of the application.
    pub time: f32,
    /// The time since the last frame was rendered.
    pub time_delta: f32,
    /// The date as year, month, day and seconds.
    pub date: [f32; 4],
    /// The current frame that is to be rendered.
    pub frame_index: i32,
}

/// Created directly after successfully parsing an `Isf`.
///
/// `imported` textures can be accessed by the user.
#[derive(Debug, Default)]
pub struct IsfData {
    imported: HashMap<ImportName, ImageState>,
    inputs: HashMap<InputName, IsfInputData>,
    passes: Vec<wgpu::Texture>,
}

/// The state of the image.
#[derive(Debug)]
pub enum ImageState {
    None,
    Loading(mpsc::Receiver<Result<image::RgbaImage, ImageLoadError>>),
    Ready(Result<ImageData, ImageLoadError>),
}

/// Handles to both the cpu and gpu representations of the image.
#[derive(Debug)]
pub struct ImageData {
    pub image: image::RgbaImage,
    pub texture: wgpu::Texture,
}

pub type ImportName = String;
pub type InputName = String;

#[derive(Debug)]
pub enum IsfInputData {
    Event {
        happening: bool,
    },
    Bool(bool),
    Long(i32),
    Float(f32),
    Point2d(Point2),
    Color(LinSrgba),
    Image(ImageState),
    Audio {
        samples: Vec<f32>,
        texture: wgpu::Texture,
    },
    AudioFft {
        columns: Vec<f32>,
        texture: wgpu::Texture,
    },
}

#[derive(Clone, Debug)]
struct IsfInputUniforms {
    // Each supported uniform field type is 32-bit long, so store them as such.
    fields: Vec<u32>,
}

/// A shader with some extra information relating to recent compilation success/failure.
#[derive(Debug)]
pub struct Shader {
    source: ShaderSource,
    module: Option<wgpu::ShaderModule>,
    error: Option<ShaderError>,
}

#[derive(Debug)]
enum ShaderSource {
    Path(PathBuf),
    HardCoded,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 2],
}

#[derive(Debug)]
struct ImageLoader {
    threadpool: ThreadPool,
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
    },
}

/// Errors that might occur while loading an image.
#[derive(Debug, Error)]
pub enum ImageLoadError {
    #[error("an IO error: {err}")]
    Io {
        #[from]
        err: std::io::Error,
    },
    #[error("{}", err)]
    Image {
        #[from]
        err: image::ImageError,
    },
}

/// Errors that might occur while loading a shader.
#[derive(Debug, Error)]
pub enum ShaderError {
    #[error("{err}")]
    Io {
        #[from]
        err: std::io::Error,
    },
    #[error("an error occurred while parsing ISF: {err}")]
    IsfParse {
        #[from]
        err: isf::ParseError,
    },
    #[error("an error occurred while parsing ISF: {err}")]
    Compile {
        #[from]
        err: hotglsl::CompileError,
    },
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

    /// Update the image state.
    fn update(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        image_loader: &ImageLoader,
        img_path: PathBuf,
    ) {
        *self = match *self {
            ImageState::None => {
                let (tx, rx) = mpsc::channel();
                image_loader.threadpool.execute(move || {
                    let img_res = image::open(img_path)
                        .map(|img| img.to_rgba())
                        .map_err(|err| err.into());
                    tx.send(img_res).ok();
                });
                ImageState::Loading(rx)
            }
            ImageState::Loading(ref rx) => match rx.try_recv() {
                Ok(img_res) => {
                    let usage = wgpu::TextureUsage::COPY_DST | wgpu::TextureUsage::SAMPLED;
                    let res = img_res.map(|image| {
                        let texture = wgpu::Texture::encode_load_from_image_buffer(
                            device, encoder, usage, &image,
                        );
                        ImageData { image, texture }
                    });
                    ImageState::Ready(res)
                }
                _ => return,
            },
            ImageState::Ready(_) => return,
        };
    }
}

impl IsfData {
    /// The map of imported images.
    pub fn imported(&self) -> &HashMap<ImportName, ImageState> {
        &self.imported
    }

    /// The map of all declared inputs.
    pub fn inputs(&self) -> &HashMap<InputName, IsfInputData> {
        &self.inputs
    }

    /// The texture stored for each pass.
    pub fn passes(&self) -> &[wgpu::Texture] {
        &self.passes
    }
}

impl IsfInputData {
    /// Initialise a new `IsfInputData` instance.
    fn new(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        image_loader: &ImageLoader,
        images_path: &Path,
        input: &isf::Input,
    ) -> Self {
        match &input.ty {
            isf::InputType::Event => IsfInputData::Event { happening: false },
            isf::InputType::Bool(b) => IsfInputData::Bool(b.default.unwrap_or_default()),
            isf::InputType::Long(n) => {
                let init = n
                    .default
                    .or(n.min)
                    .or(n.values.first().cloned())
                    .unwrap_or_default();
                IsfInputData::Long(init)
            }
            isf::InputType::Float(f) => {
                let init = f.default.or(f.min).unwrap_or_default();
                IsfInputData::Float(init)
            }
            isf::InputType::Point2d(p) => {
                let [x, y] = p.default.or(p.min).unwrap_or_default();
                IsfInputData::Point2d(pt2(x, y))
            }
            isf::InputType::Color(c) => {
                let v = c.default.clone().or(c.min.clone()).unwrap_or_default();
                let r = v.get(0).cloned().unwrap_or_default();
                let g = v.get(1).cloned().unwrap_or_default();
                let b = v.get(2).cloned().unwrap_or_default();
                let a = v.get(3).cloned().unwrap_or_default();
                IsfInputData::Color(lin_srgba(r, g, b, a))
            }
            // For the input images, it's up to us how we want to source them. Perhaps
            // `assets/images/`?  For now we'll black images.
            isf::InputType::Image => {
                let mut image_state = ImageState::None;
                if let Some(img_path) = image_paths(images_path).next() {
                    image_state.update(device, encoder, image_loader, img_path);
                }
                IsfInputData::Image(image_state)
            }
            isf::InputType::Audio(a) => {
                let n_samples = a
                    .num_samples
                    .unwrap_or(IsfPipeline::DEFAULT_AUDIO_SAMPLE_COUNT);
                let samples = vec![0.0; n_samples as usize];
                let size = [n_samples, 1];
                let format = IsfPipeline::DEFAULT_AUDIO_TEXTURE_FORMAT;
                let texture = create_black_texture(device, encoder, size, format);
                IsfInputData::Audio { samples, texture }
            }
            isf::InputType::AudioFft(a) => {
                let n_columns = a
                    .num_columns
                    .unwrap_or(IsfPipeline::DEFAULT_AUDIO_FFT_COLUMNS);
                let columns = vec![0.0; n_columns as usize];
                let size = [n_columns, 1];
                let format = IsfPipeline::DEFAULT_AUDIO_TEXTURE_FORMAT;
                let texture = create_black_texture(device, encoder, size, format);
                IsfInputData::AudioFft { columns, texture }
            }
        }
    }

    /// Short-hand for checking that the input type matches the data.
    ///
    /// This is useful for checking to see if the user has changed the type of data associated with
    /// the name.
    pub fn ty_matches(&self, ty: &isf::InputType) -> bool {
        match (self, ty) {
            (IsfInputData::Event { .. }, isf::InputType::Event)
            | (IsfInputData::Bool(_), isf::InputType::Bool(_))
            | (IsfInputData::Long(_), isf::InputType::Long(_))
            | (IsfInputData::Float(_), isf::InputType::Float(_))
            | (IsfInputData::Point2d(_), isf::InputType::Point2d(_))
            | (IsfInputData::Color(_), isf::InputType::Color(_))
            | (IsfInputData::Image(_), isf::InputType::Image)
            | (IsfInputData::Audio { .. }, isf::InputType::Audio(_))
            | (IsfInputData::AudioFft { .. }, isf::InputType::AudioFft(_)) => true,
            _ => false,
        }
    }

    /// Update an existing instance ISF input data instance with the given input.
    fn update(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        image_loader: &ImageLoader,
        images_path: &Path,
        input: &isf::Input,
    ) {
        match (self, &input.ty) {
            (IsfInputData::Event { .. }, isf::InputType::Event) => (),
            (IsfInputData::Bool(_), isf::InputType::Bool(_)) => (),
            (IsfInputData::Long(_), isf::InputType::Long(_)) => {}
            (IsfInputData::Float(_), isf::InputType::Float(_)) => {}
            (IsfInputData::Point2d(_), isf::InputType::Point2d(_)) => {}
            (IsfInputData::Color(_), isf::InputType::Color(_)) => {}
            (IsfInputData::Image(ref mut state), isf::InputType::Image) => {
                if let Some(img_path) = image_paths(images_path).next() {
                    state.update(device, encoder, image_loader, img_path);
                }
            }
            (IsfInputData::Audio { .. }, isf::InputType::Audio(_)) => {}
            (IsfInputData::AudioFft { .. }, isf::InputType::AudioFft(_)) => {}
            (data, _) => *data = Self::new(device, encoder, image_loader, images_path, input),
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

/// Compile an ISF fragment shader.
///
/// This is used for compiling the ISF fragment shader.
pub fn compile_isf_shader(
    device: &wgpu::Device,
    path: &Path,
) -> (Option<wgpu::ShaderModule>, Option<ShaderError>) {
    let res = std::fs::read_to_string(&path)
        .map_err(ShaderError::from)
        .and_then(|s| isf::parse(&s).map(|isf| (s, isf)).map_err(From::from))
        .and_then(|(old_str, isf)| {
            let isf_str = crate::glsl_string_from_isf(&isf);
            let new_str = crate::prefix_isf_glsl_str(&isf_str, old_str);
            let ty = hotglsl::ShaderType::Fragment;
            hotglsl::compile_str(&new_str, ty).map_err(From::from)
        });
    let (bytes, error) = split_result(res);
    let module = bytes.map(|b| spirv_bytes_to_mod(device, &b));
    (module, error)
}

/// Compile a regular, non-ISF shader.
///
/// This is used for compiling the vertex shaders.
pub fn compile_shader(
    device: &wgpu::Device,
    path: &Path,
) -> (Option<wgpu::ShaderModule>, Option<ShaderError>) {
    let res = hotglsl::compile(&path).map_err(ShaderError::from);
    let (bytes, compile_err) = split_result(res);
    let module = bytes.map(|b| spirv_bytes_to_mod(device, &b));
    (module, compile_err)
}

impl Shader {
    pub fn fragment_from_path(device: &wgpu::Device, path: PathBuf) -> Self {
        let (module, error) = compile_isf_shader(device, &path);
        let source = ShaderSource::Path(path);
        Shader {
            source,
            module,
            error,
        }
    }

    pub fn vertex_from_path(device: &wgpu::Device, path: PathBuf) -> Self {
        let (module, error) = compile_shader(device, &path);
        let source = ShaderSource::Path(path);
        Shader {
            source,
            module,
            error,
        }
    }

    /// Create the default vertex shader for ISF fragment shaders.
    pub fn vertex_default(device: &wgpu::Device) -> Self {
        let vs = include_bytes!("shaders/vert.spv");
        let module = Some(spirv_bytes_to_mod(device, &vs[..]));
        let error = None;
        let source = ShaderSource::HardCoded;
        Shader {
            source,
            module,
            error,
        }
    }
}

impl IsfPipeline {
    pub const DEFAULT_IMAGE_TEXTURE_FORMAT: wgpu::TextureFormat =
        wgpu::TextureFormat::Rgba8UnormSrgb;
    pub const DEFAULT_AUDIO_SAMPLE_COUNT: u32 = 64;
    pub const DEFAULT_AUDIO_FFT_COLUMNS: u32 = 64;
    pub const DEFAULT_AUDIO_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R32Float;

    /// Construct a new **IsfPipeline**.
    pub fn new(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        vs_path: Option<PathBuf>,
        fs_path: PathBuf,
        dst_format: wgpu::TextureFormat,
        dst_texture_size: [u32; 2],
        dst_sample_count: u32,
        images_path: &Path,
    ) -> Self {
        // Retrieve the `Isf` instance.
        let isf_res = read_isf_from_path(&fs_path);
        let (isf, isf_err) = split_result(isf_res);

        // Create the shaders.
        let vs = match vs_path {
            None => Shader::vertex_default(device),
            Some(vs_path) => Shader::vertex_from_path(device, vs_path),
        };
        let fs = Shader::fragment_from_path(device, fs_path);

        dbg!(&vs);
        dbg!(&fs);

        // Create a threadpool for loading images.
        let threadpool = ThreadPool::default();
        let image_loader = ImageLoader { threadpool };

        // Initialise the ISF imported images, input data and passes.
        let mut isf_data = IsfData::default();
        if let Some(ref isf) = isf {
            sync_isf_data(
                device,
                encoder,
                isf,
                dst_texture_size,
                &image_loader,
                &images_path,
                &mut isf_data,
            );
        }

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
        let isf_textures_bind_group_layout =
            create_isf_textures_bind_group_layout(device, &isf_data);

        // Create the sampler.
        let sampler = wgpu::SamplerBuilder::new().build(device);

        // Create the bind groups
        let isf_bind_group = wgpu::BindGroupBuilder::new()
            .buffer::<IsfUniforms>(&isf_uniform_buffer, 0..1)
            .build(device, &isf_bind_group_layout);
        let isf_inputs_bind_group = wgpu::BindGroupBuilder::new()
            .buffer::<IsfInputUniforms>(&isf_inputs_uniform_buffer, 0..1)
            .build(device, &isf_inputs_bind_group_layout);
        let isf_textures_bind_group = create_isf_textures_bind_group(
            device,
            &isf_textures_bind_group_layout,
            &sampler,
            &isf_data,
        );

        // Create the render pipeline.
        let layout = create_pipeline_layout(
            device,
            &[
                &isf_bind_group_layout,
                &isf_inputs_bind_group_layout,
                &isf_textures_bind_group_layout,
            ],
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
            isf_data,
            isf_err,
            image_loader,
            vs,
            fs,
            sampler,
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

    /// Update the ISF pipeline.
    ///
    /// Updating the ISF pipeline does the following:
    ///
    /// - First attempts to recompile the given sequence of touched shaders, both for ISF and GLSL.
    /// - Synchronises the ISF data with the latest successfully parsed `Isf` instance. Any images
    ///   that have completed loading will be uploaded to textures.
    /// - If the number of textures has changed, recreates the texture bind group, layout and
    ///   render pipeline layout.
    /// - If any of the shaders successfully recompiled, or if the number of textures changed, the
    ///   pipeline is recreated.
    pub fn encode_update<I>(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        images_path: &Path,
        touched_shaders: I,
    ) where
        I: IntoIterator,
        I::Item: AsRef<Path>,
    {
        // UPDATE SHADERS
        // --------------

        // Attempt to recompile touched shaders.
        let mut shader_recompiled = false;
        for path in touched_shaders {
            let path = path.as_ref();
            if self.vs.source.as_path() == Some(&path) {
                let (module, error) = compile_shader(device, &path);
                self.vs.error = error;
                if module.is_some() {
                    shader_recompiled = true;
                    self.vs.module = module;
                }
            } else if self.fs.source.as_path() == Some(&path) {
                let (module, error) = compile_isf_shader(device, &path);
                self.fs.error = error;
                if module.is_some() {
                    shader_recompiled = true;
                    self.fs.module = module;
                }
                // Update the `Isf` instance.
                let isf_res = read_isf_from_path(&path);
                let (new_isf, new_isf_err) = split_result(isf_res);
                self.isf_err = new_isf_err;
                if self.isf.is_none() {
                    self.isf = new_isf;
                }
            }
        }

        // UPDATE ISF DATA
        // ---------------

        // We can only update the isf data if we have an isf instance to work with.
        let isf = match self.isf {
            None => return,
            Some(ref isf) => isf,
        };

        // Keep track of whether the number of textures change for our bind groups.
        let texture_count = isf_data_textures(&self.isf_data).count();

        // Synchronise the ISF data.
        sync_isf_data(
            device,
            encoder,
            isf,
            self.dst_texture_size,
            &self.image_loader,
            images_path,
            &mut self.isf_data,
        );

        // UPDATE TEXTURE BIND GROUP
        // -------------------------

        // If the number of textures have changed, update the bind group and pipeline layout.
        let new_texture_count = isf_data_textures(&self.isf_data).count();
        let texture_count_changed = texture_count != new_texture_count;
        if texture_count_changed {
            self.isf_textures_bind_group_layout =
                create_isf_textures_bind_group_layout(device, &self.isf_data);
            self.isf_textures_bind_group = create_isf_textures_bind_group(
                device,
                &self.isf_textures_bind_group_layout,
                &self.sampler,
                &self.isf_data,
            );
            self.layout = create_pipeline_layout(
                device,
                &[
                    &self.isf_bind_group_layout,
                    &self.isf_inputs_bind_group_layout,
                    &self.isf_textures_bind_group_layout,
                ],
            );
        }

        // UPDATE RENDER PIPELINE
        // ----------------------

        if shader_recompiled || texture_count_changed {
            if let (Some(vs_mod), Some(fs_mod)) = (self.vs.module.as_ref(), self.fs.module.as_ref())
            {
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
    }

    /// Given an encoder, submits a render pass command for drawing the pipeline to the given
    /// texture.
    ///
    /// If the pipeline has not yet been created because it has not yet compiled the necessary
    /// shaders correctly, the render pass will not be encoded.
    pub fn encode_render_pass(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        dst_texture: &wgpu::TextureViewHandle,
        isf_time: IsfTime,
    ) {
        if let Some(ref pipeline) = self.render_pipeline {
            // Encode an update for the ISF uniform buffer.
            let [w, h] = self.dst_texture_size;
            let isf_uniforms = IsfUniforms {
                pass_index: 0,
                render_size: [w as f32, h as f32],
                time: isf_time.time,
                time_delta: isf_time.time_delta,
                date: isf_time.date,
                frame_index: isf_time.frame_index,
            };
            let size = std::mem::size_of::<IsfUniforms>() as wgpu::BufferAddress;
            let new_buffer = device
                .create_buffer_mapped(1, wgpu::BufferUsage::COPY_SRC)
                .fill_from_slice(&[isf_uniforms]);
            encoder.copy_buffer_to_buffer(&new_buffer, 0, &self.isf_uniform_buffer, 0, size);

            // TODO: Update the inputs.
            let _ = &self.isf_inputs_uniform_buffer;
            //let size = std::mem::size_of::<IsfInputUniforms>() as wgpu::BufferAddress;
            //encoder.copy_buffer_to_buffer(&new_buffer, 0, &self.isf_inputs_uniform_buffer, 0, size);

            // Encode the render pass.
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
    pub fn encode_to_frame(&self, frame: &Frame, isf_time: IsfTime) {
        let device = frame.device_queue_pair().device();
        let mut encoder = frame.command_encoder();
        self.encode_render_pass(device, &mut *encoder, frame.texture_view(), isf_time);
    }

    /// Returns the current compilation error for the vertex shader if there is one.
    ///
    /// Returns `Some` if the last call to `update_shaders` contained a compilation error for the
    /// vertex shader.
    pub fn vs_err(&self) -> Option<&ShaderError> {
        self.vs.error.as_ref()
    }

    /// Returns the current compilation error for the vertex shader if there is one.
    ///
    /// Returns `Some` if the last call to `update_shaders` contained a compilation error for the
    /// vertex shader.
    pub fn fs_err(&self) -> Option<&ShaderError> {
        self.fs.error.as_ref()
    }
}

fn split_result<T, E>(res: Result<T, E>) -> (Option<T>, Option<E>) {
    match res {
        Ok(t) => (Some(t), None),
        Err(e) => (None, Some(e)),
    }
}

fn spirv_bytes_to_mod(device: &wgpu::Device, bytes: &[u8]) -> wgpu::ShaderModule {
    let cursor = std::io::Cursor::new(&bytes[..]);
    let vs_spirv = wgpu::read_spirv(cursor).expect("failed to read hard-coded SPIRV");
    device.create_shader_module(&vs_spirv)
}

// Includes the sampler and then all textures for all images and passes.
fn create_isf_textures_bind_group_layout(
    device: &wgpu::Device,
    isf_data: &IsfData,
) -> wgpu::BindGroupLayout {
    // Begin with the sampler.
    let mut builder = wgpu::BindGroupLayoutBuilder::new().sampler(wgpu::ShaderStage::FRAGMENT);
    for _ in isf_data_textures(isf_data) {
        builder = builder.sampled_texture(
            wgpu::ShaderStage::FRAGMENT,
            false,
            wgpu::TextureViewDimension::D2,
        );
    }
    builder.build(device)
}

fn create_isf_textures_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    isf_data: &IsfData,
) -> wgpu::BindGroup {
    let mut builder = wgpu::BindGroupBuilder::new().sampler(sampler);
    let texture_views: Vec<_> = isf_data_textures(isf_data)
        .map(|tex| tex.view().build())
        .collect();
    for texture_view in &texture_views {
        builder = builder.texture_view(texture_view);
    }
    builder.build(device, layout)
}

fn create_pipeline_layout(
    device: &wgpu::Device,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
) -> wgpu::PipelineLayout {
    let desc = wgpu::PipelineLayoutDescriptor { bind_group_layouts };
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

// All textures stored within the `IsfData` instance in the order that they should be declared in
// the order expected by the isf textures bind group.
fn isf_data_textures(isf_data: &IsfData) -> impl Iterator<Item = &wgpu::Texture> {
    let imported = isf_data.imported.values().filter_map(|state| match state {
        ImageState::Ready(ref img_res) => match img_res {
            Ok(ref img_data) => Some(&img_data.texture),
            _ => None,
        },
        _ => None,
    });
    let inputs = isf_data
        .inputs
        .values()
        .filter_map(|input_data| match input_data {
            IsfInputData::Image(ref img_state) => match *img_state {
                ImageState::Ready(Ok(ref data)) => Some(&data.texture),
                _ => None,
            },
            IsfInputData::Audio { ref texture, .. }
            | IsfInputData::AudioFft { ref texture, .. } => Some(texture),
            _ => None,
        });
    let passes = isf_data.passes.iter();
    imported.chain(inputs).chain(passes)
}

// Ensure the image state map is up to date.
fn sync_isf_data(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    isf: &isf::Isf,
    output_attachment_size: [u32; 2],
    image_loader: &ImageLoader,
    images_path: &Path,
    isf_data: &mut IsfData,
) {
    // Update imported images. first.
    isf_data
        .imported
        .retain(|name, _| isf.imported.contains_key(name));
    for (key, img) in &isf.imported {
        let state = isf_data
            .imported
            .entry(key.clone())
            .or_insert(ImageState::None);
        state.update(device, encoder, image_loader, img.path.clone());
    }

    // First, check all imported textures are loading.
    isf_data
        .inputs
        .retain(|key, _| isf.inputs.iter().map(|i| &i.name).any(|n| n == key));
    for input in &isf.inputs {
        let input_data = isf_data
            .inputs
            .entry(input.name.clone())
            .or_insert_with(|| {
                IsfInputData::new(device, encoder, image_loader, images_path, input)
            });
        input_data.update(device, encoder, image_loader, images_path, input);
    }

    // Prepare the textures that will be written to for passes.
    isf_data.passes.resize_with(isf.passes.len(), || {
        let texture = wgpu::TextureBuilder::new()
            .format(Frame::TEXTURE_FORMAT)
            .size(output_attachment_size)
            .usage(default_isf_texture_usage())
            .build(device);
        let data = vec![0u8; texture.size_bytes()];
        texture.upload_data(device, encoder, &data);
        texture
    });
}

fn create_black_texture(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    size: [u32; 2],
    format: wgpu::TextureFormat,
) -> wgpu::Texture {
    let texture = wgpu::TextureBuilder::new()
        .usage(default_isf_texture_usage())
        .size(size)
        .format(format)
        .build(device);
    let data = vec![0u8; texture.size_bytes()];
    texture.upload_data(device, encoder, &data);
    texture
}

fn default_isf_texture_usage() -> wgpu::TextureUsage {
    wgpu::TextureUsage::COPY_DST | wgpu::TextureUsage::SAMPLED
}

fn read_isf_from_path(path: &Path) -> Result<isf::Isf, IsfError> {
    std::fs::read_to_string(path)
        .map_err(|err| IsfError::from(err))
        .and_then(|s| isf::parse(&s).map_err(From::from))
}

/// Given a path to a directory, produces the paths of all images within it.
fn image_paths(dir: &Path) -> impl Iterator<Item = PathBuf> {
    walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|res| res.ok())
        .map(|entry| entry.path().to_path_buf())
        .filter(|path| image::image_dimensions(path).ok().is_some())
}
