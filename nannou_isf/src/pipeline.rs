use nannou::image;
use nannou::prelude::*;
use nannou::wgpu::BufferInitDescriptor;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use thiserror::Error;
use threadpool::ThreadPool;

/// A render pipeline designed for hotloading!
pub struct IsfPipeline {
    isf: Option<isf::Isf>,
    isf_data: IsfData,
    isf_err: Option<IsfError>,
    image_loader: ImageLoader,
    vs: Shader,
    fs: Shader,
    sampler: wgpu::Sampler,
    sampler_filtering: bool,
    isf_uniform_buffer: wgpu::Buffer,
    isf_inputs_uniform_buffer: wgpu::Buffer,
    isf_bind_group_layout: wgpu::BindGroupLayout,
    isf_inputs_bind_group_layout: wgpu::BindGroupLayout,
    isf_textures_bind_group_layout: wgpu::BindGroupLayout,
    isf_textures_bind_group_descriptors: Vec<wgpu::TextureDescriptor<'static>>,
    isf_bind_group: wgpu::BindGroup,
    isf_inputs_bind_group: wgpu::BindGroup,
    isf_textures_bind_group: wgpu::BindGroup,
    layout: wgpu::PipelineLayout,
    render_pipeline: Option<wgpu::RenderPipeline>,
    vertex_buffer: wgpu::Buffer,
    dst_format: wgpu::TextureFormat,
    dst_texture_size: [u32; 2],
    dst_sample_count: u32,
    placeholder_texture: wgpu::Texture,
}

/// The first set of ISF uniforms that are available to every ISF shader.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct IsfUniforms {
    date: [f32; 4],
    render_size: [f32; 2],
    time: f32,
    time_delta: f32,
    pass_index: i32,
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
    imported: BTreeMap<ImportName, ImageState>,
    inputs: BTreeMap<InputName, IsfInputData>,
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
                        .map(|img| img.to_rgba8())
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
    pub fn imported(&self) -> &BTreeMap<ImportName, ImageState> {
        &self.imported
    }

    /// The map of all declared inputs.
    pub fn inputs(&self) -> &BTreeMap<InputName, IsfInputData> {
        &self.inputs
    }

    /// An iterator yielding only the uniform inputs.
    ///
    /// Yields inputs in the order in which they are laid out in the generated shader.
    pub fn uniform_inputs<'a>(&'a self) -> impl Iterator<Item = (&'a InputName, &'a IsfInputData)> {
        isf_inputs_by_uniform_order(&self.inputs)
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
                if let Some(img_path) = image_paths_ordered(images_path).into_iter().next() {
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

    /// The size of this field when laid out within the dynamically sized input uniform buffer.
    pub fn uniform_size_bytes(&self) -> Option<usize> {
        let size = match *self {
            IsfInputData::Color(_) => 4 * 4,
            IsfInputData::Point2d(_) => 2 * 2,
            IsfInputData::Long(_)
            | IsfInputData::Float(_)
            | IsfInputData::Bool(_)
            | IsfInputData::Event { .. } => 1 * 4,
            IsfInputData::Image(_) | IsfInputData::Audio { .. } | IsfInputData::AudioFft { .. } => {
                return None
            }
        };
        Some(size)
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
                if let Some(img_path) = image_paths_ordered(images_path).into_iter().next() {
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
    let module = bytes.map(|b| wgpu::shader_from_spirv_bytes(device, &b));
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
    let module = bytes.map(|b| wgpu::shader_from_spirv_bytes(device, &b));
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
        let module = Some(wgpu::shader_from_spirv_bytes(device, vs));
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

        // Prepare the uniform data.
        let [dst_tex_w, dst_tex_h] = dst_texture_size;
        let isf_uniforms = IsfUniforms {
            date: [0.0; 4],
            render_size: [dst_tex_w as f32, dst_tex_h as f32],
            time: 0.0,
            time_delta: 0.0,
            pass_index: 0,
            frame_index: 0,
        };
        let isf_input_uniforms = isf_inputs_to_uniform_data(&isf_data.inputs);

        // Convert uniform data to bytes for upload.
        let isf_uniforms_bytes = isf_uniforms_as_bytes(&isf_uniforms);
        let isf_input_uniforms_bytes = isf_input_uniforms_as_bytes(&isf_input_uniforms);

        // Create the uniform buffers.
        let uniforms_usage = wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST;
        let isf_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: &isf_uniforms_bytes,
            usage: uniforms_usage,
        });
        let isf_inputs_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: &isf_input_uniforms_bytes,
            usage: uniforms_usage,
        });

        // Create the sampler.
        let sampler_desc = wgpu::SamplerBuilder::new().into_descriptor();
        let sampler_filtering = wgpu::sampler_filtering(&sampler_desc);
        let sampler = device.create_sampler(&sampler_desc);

        // Create the placeholder texture for the case where an image is not yet loaded.
        let placeholder_texture = wgpu::TextureBuilder::new()
            .usage(wgpu::TextureUsage::SAMPLED)
            .build(device);

        // Prepare the bind group layouts.
        let isf_bind_group_layout = wgpu::BindGroupLayoutBuilder::new()
            .uniform_buffer(wgpu::ShaderStage::FRAGMENT, false)
            .build(device);
        let isf_inputs_bind_group_layout = wgpu::BindGroupLayoutBuilder::new()
            .uniform_buffer(wgpu::ShaderStage::FRAGMENT, false)
            .build(device);
        let isf_textures_bind_group_layout = create_isf_textures_bind_group_layout(
            device,
            sampler_filtering,
            &isf_data,
            &placeholder_texture,
        );

        // Track the descriptors in case we need to rebuild pipeline.
        let isf_textures_bind_group_descriptors =
            isf_data_textures(&isf_data, &placeholder_texture)
                .map(|texture| texture.descriptor().clone())
                .collect();

        // Create the bind groups
        let isf_bind_group = wgpu::BindGroupBuilder::new()
            .buffer::<IsfUniforms>(&isf_uniform_buffer, 0..1)
            .build(device, &isf_bind_group_layout);
        let isf_inputs_bind_group = wgpu::BindGroupBuilder::new()
            .buffer::<u32>(&isf_inputs_uniform_buffer, 0..isf_input_uniforms.len())
            .build(device, &isf_inputs_bind_group_layout);
        let isf_textures_bind_group = create_isf_textures_bind_group(
            device,
            &isf_textures_bind_group_layout,
            &sampler,
            &isf_data,
            &placeholder_texture,
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
        let vertices_bytes = vertices_as_bytes(&VERTICES[..]);
        let vertex_usage = wgpu::BufferUsage::VERTEX;
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: vertices_bytes,
            usage: vertex_usage,
        });

        Self {
            isf,
            isf_data,
            isf_err,
            image_loader,
            vs,
            fs,
            sampler,
            sampler_filtering,
            isf_uniform_buffer,
            isf_inputs_uniform_buffer,
            isf_bind_group_layout,
            isf_inputs_bind_group_layout,
            isf_textures_bind_group_layout,
            isf_textures_bind_group_descriptors,
            isf_bind_group,
            isf_inputs_bind_group,
            isf_textures_bind_group,
            layout,
            render_pipeline,
            vertex_buffer,
            dst_format,
            dst_texture_size,
            dst_sample_count,
            placeholder_texture,
        }
    }

    /// Data loaded after successfully parsing the ISF descriptor.
    ///
    /// Provides access to the resources loaded for textures, inputs and passes.
    pub fn isf_data(&self) -> &IsfData {
        &self.isf_data
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

        // If the textures have changed, update the bind group and pipeline layout.
        let new_texture_descriptors = isf_data_textures(&self.isf_data, &self.placeholder_texture)
            .map(|texture| texture.descriptor().clone())
            .collect::<Vec<_>>();
        let textures_changed = self.isf_textures_bind_group_descriptors != new_texture_descriptors;
        if textures_changed {
            self.isf_textures_bind_group_descriptors = new_texture_descriptors;
            self.isf_textures_bind_group_layout = create_isf_textures_bind_group_layout(
                device,
                self.sampler_filtering,
                &self.isf_data,
                &self.placeholder_texture,
            );
            self.isf_textures_bind_group = create_isf_textures_bind_group(
                device,
                &self.isf_textures_bind_group_layout,
                &self.sampler,
                &self.isf_data,
                &self.placeholder_texture,
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

        if shader_recompiled || textures_changed {
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
            let isf_uniforms_bytes = isf_uniforms_as_bytes(&isf_uniforms);
            let usage = wgpu::BufferUsage::COPY_SRC;
            let new_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: &isf_uniforms_bytes,
                usage,
            });
            let size = isf_uniforms_bytes.len() as wgpu::BufferAddress;
            encoder.copy_buffer_to_buffer(&new_buffer, 0, &self.isf_uniform_buffer, 0, size);

            // TODO: Update the inputs, but only if changed.
            let isf_inputs_changed = true;
            if isf_inputs_changed {
                let isf_input_uniforms = isf_inputs_to_uniform_data(&self.isf_data.inputs);
                let isf_input_uniforms_bytes = isf_input_uniforms_as_bytes(&isf_input_uniforms);
                let usage = wgpu::BufferUsage::COPY_SRC;
                let new_buffer = device.create_buffer_init(&BufferInitDescriptor {
                    label: Some("nannou_isf-input_uniforms"),
                    contents: &isf_input_uniforms_bytes,
                    usage,
                });
                let size = isf_input_uniforms_bytes.len() as wgpu::BufferAddress;
                encoder.copy_buffer_to_buffer(
                    &new_buffer,
                    0,
                    &self.isf_inputs_uniform_buffer,
                    0,
                    size,
                );
            }

            // Encode the render pass.
            let mut render_pass = wgpu::RenderPassBuilder::new()
                .color_attachment(dst_texture, |color| color)
                .begin(encoder);
            render_pass.set_pipeline(pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
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

    /// Returns the current ISF read error for the fragment shader if there is one.
    ///
    /// Returns `Some` if an error occurred when parsing the ISF from the fragment shader the last
    /// time the fragment shader file was touched.
    pub fn isf_err(&self) -> Option<&IsfError> {
        self.isf_err.as_ref()
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

// Includes the sampler and then all textures for all images and passes.
fn create_isf_textures_bind_group_layout(
    device: &wgpu::Device,
    sampler_filtering: bool,
    isf_data: &IsfData,
    placeholder_texture: &wgpu::Texture,
) -> wgpu::BindGroupLayout {
    // Begin with the sampler.
    let mut builder =
        wgpu::BindGroupLayoutBuilder::new().sampler(wgpu::ShaderStage::FRAGMENT, sampler_filtering);
    for texture in isf_data_textures(isf_data, placeholder_texture) {
        builder = builder.texture(
            wgpu::ShaderStage::FRAGMENT,
            false,
            wgpu::TextureViewDimension::D2,
            texture.sample_type(),
        );
    }
    builder.build(device)
}

fn create_isf_textures_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    isf_data: &IsfData,
    placeholder_texture: &wgpu::Texture,
) -> wgpu::BindGroup {
    let mut builder = wgpu::BindGroupBuilder::new().sampler(sampler);
    let texture_views: Vec<_> = isf_data_textures(isf_data, placeholder_texture)
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
    let desc = wgpu::PipelineLayoutDescriptor {
        label: Some("nannou_isf-pipeline_layout"),
        bind_group_layouts,
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
    wgpu::RenderPipelineBuilder::from_layout(layout, &vs_mod)
        .fragment_shader(fs_mod)
        .color_format(dst_format)
        .add_vertex_buffer::<Vertex>(&wgpu::vertex_attr_array![0 => Float32x2])
        .sample_count(sample_count)
        .primitive_topology(wgpu::PrimitiveTopology::TriangleStrip)
        .build(device)
}

// All textures stored within the `IsfData` instance in the order that they should be declared in
// the order expected by the isf textures bind group.
fn isf_data_textures<'a>(
    isf_data: &'a IsfData,
    placeholder: &'a wgpu::Texture,
) -> impl Iterator<Item = &'a wgpu::Texture> {
    let imported = isf_data.imported.values().map(move |state| match state {
        ImageState::Ready(Ok(ref img_data)) => &img_data.texture,
        ImageState::Ready(Err(_)) | ImageState::None | ImageState::Loading(_) => placeholder,
    });
    let inputs = isf_data
        .inputs
        .values()
        .filter_map(move |input_data| match input_data {
            IsfInputData::Image(ref img_state) => match *img_state {
                ImageState::Ready(Ok(ref data)) => Some(&data.texture),
                ImageState::Ready(Err(_)) | ImageState::None | ImageState::Loading(_) => {
                    Some(placeholder)
                }
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
    btreemap_retain(&mut isf_data.imported, |k, _| isf.imported.contains_key(k));
    for (key, img) in &isf.imported {
        let state = isf_data
            .imported
            .entry(key.clone())
            .or_insert(ImageState::None);
        state.update(device, encoder, image_loader, img.path.clone());
    }

    // First, check all imported textures are loading.
    btreemap_retain(&mut isf_data.inputs, |k, _| {
        isf.inputs.iter().map(|i| &i.name).any(|n| n == k)
    });
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

// The order in which inputs are laid out in the uniform buffer.
//
// This is important to meet the conditions required by wgpu uniform layout. Specifically, 16-byte
// types must be aligned to 16 bytes, 8-byte types must be aligned to 8-bytes, etc.
//
// This must match the order specified in the generated glsl shader.
fn isf_inputs_by_uniform_order(
    inputs: &BTreeMap<InputName, IsfInputData>,
) -> impl Iterator<Item = (&InputName, &IsfInputData)> {
    let b16 = inputs
        .iter()
        .filter(|(_k, v)| v.uniform_size_bytes() == Some(16));
    let b8 = inputs
        .iter()
        .filter(|(_k, v)| v.uniform_size_bytes() == Some(8));
    let b4 = inputs
        .iter()
        .filter(|(_k, v)| v.uniform_size_bytes() == Some(4));
    b16.chain(b8).chain(b4)
}

// Encodes the ISF inputs to a slice of `u32` values, ready for uploading to the GPU.
fn isf_inputs_to_uniform_data(inputs: &BTreeMap<InputName, IsfInputData>) -> Vec<u32> {
    let mut u32s: Vec<u32> = vec![];
    for (_k, v) in isf_inputs_by_uniform_order(inputs) {
        match *v {
            IsfInputData::Event { happening } => {
                u32s.push(if happening { 1 } else { 0 });
            }
            IsfInputData::Bool(b) => {
                u32s.push(if b { 1 } else { 0 });
            }
            IsfInputData::Long(l) => {
                u32s.push(u32::from_le_bytes(l.to_le_bytes()));
            }
            IsfInputData::Float(f) => {
                u32s.push(f.to_bits());
            }
            IsfInputData::Point2d(Point2 { x, y }) => {
                u32s.push(x.to_bits());
                u32s.push(y.to_bits());
            }
            IsfInputData::Color(ref color) => {
                u32s.push(color.red.to_bits());
                u32s.push(color.green.to_bits());
                u32s.push(color.blue.to_bits());
                u32s.push(color.alpha.to_bits());
            }
            IsfInputData::Image(_) | IsfInputData::Audio { .. } | IsfInputData::AudioFft { .. } => {
                ()
            }
        }
    }
    u32s
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

fn image_paths_ordered(dir: &Path) -> Vec<PathBuf> {
    let mut paths: Vec<_> = image_paths(dir).collect();
    paths.sort();
    paths
}

fn btreemap_retain<K, V, F>(map: &mut BTreeMap<K, V>, mut pred: F)
where
    K: Ord,
    F: FnMut(&mut K, &mut V) -> bool,
{
    let temp = std::mem::replace(map, Default::default());
    for (mut k, mut v) in temp {
        if pred(&mut k, &mut v) {
            map.insert(k, v);
        }
    }
}

// Conversions to bytes for GPU buffer uploads.

fn isf_uniforms_as_bytes(data: &IsfUniforms) -> &[u8] {
    unsafe { wgpu::bytes::from(data) }
}

fn isf_input_uniforms_as_bytes(data: &[u32]) -> &[u8] {
    unsafe { wgpu::bytes::from_slice(data) }
}

fn vertices_as_bytes(data: &[Vertex]) -> &[u8] {
    unsafe { wgpu::bytes::from_slice(data) }
}
