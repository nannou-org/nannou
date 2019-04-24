use nannou::prelude::*;
use std::fs::File;
use crate::sr;
use std::borrow::Cow;
use nannou::vk::pipeline::shader::{ShaderInterfaceDef, ShaderInterfaceDefEntry};
use std::path::PathBuf;
use std::io::Read;
use crate::watch::ShaderMsg;
use crate::Model;
use std::ffi::CStr;
use std::sync::Arc;
use crate::Vertex;

pub struct ShaderInterfaces {
    pub inputs: Vec<ShaderInterfaceDefEntry>,
    pub outputs: Vec<ShaderInterfaceDefEntry>,
}

#[derive(Debug, Clone)]
struct FragInput {
    inputs: Vec<ShaderInterfaceDefEntry>,
}

unsafe impl ShaderInterfaceDef for FragInput {
    type Iter = FragInputIter;

    fn elements(&self) -> FragInputIter {
        self.inputs.clone().into_iter()
    }
}

type FragInputIter = std::vec::IntoIter<ShaderInterfaceDefEntry>;

#[derive(Debug, Clone)]
struct FragOutput {
    outputs: Vec<ShaderInterfaceDefEntry>,
}

unsafe impl ShaderInterfaceDef for FragOutput {
    type Iter = FragOutputIter;

    fn elements(&self) -> FragOutputIter {
        self.outputs.clone().into_iter()
    }
}

type FragOutputIter = std::vec::IntoIter<ShaderInterfaceDefEntry>;

// Layout same as with vertex shader.
#[derive(Debug, Copy, Clone)]
struct FragLayout(pub vk::ShaderStages);
unsafe impl vk::PipelineLayoutDesc for FragLayout {
    fn num_sets(&self) -> usize {
        0
    }
    fn num_bindings_in_set(&self, _set: usize) -> Option<usize> {
        None
    }
    fn descriptor(&self, _set: usize, _binding: usize) -> Option<vk::DescriptorDesc> {
        None
    }
    fn num_push_constants_ranges(&self) -> usize {
        0
    }
    fn push_constants_range(&self, _num: usize) -> Option<vk::PipelineLayoutDescPcRange> {
        None
    }
}

#[derive(Debug, Clone)]
struct VertInput {
    inputs: Vec<ShaderInterfaceDefEntry>,
}

unsafe impl ShaderInterfaceDef for VertInput {
    type Iter = VertInputIter;

    fn elements(&self) -> VertInputIter {
        self.inputs.clone().into_iter()
    }
}

type VertInputIter = std::vec::IntoIter<ShaderInterfaceDefEntry>;

#[derive(Debug, Clone)]
struct VertOutput {
    outputs: Vec<ShaderInterfaceDefEntry>,
}

unsafe impl ShaderInterfaceDef for VertOutput {
    type Iter = VertOutputIter;

    fn elements(&self) -> VertOutputIter {
        self.outputs.clone().into_iter()
    }
}

type VertOutputIter = std::vec::IntoIter<ShaderInterfaceDefEntry>;

// This structure describes layout of this stage.
#[derive(Debug, Copy, Clone)]
struct VertLayout(pub vk::ShaderStages);
unsafe impl vk::PipelineLayoutDesc for VertLayout {
    // Number of descriptor sets it takes.
    fn num_sets(&self) -> usize {
        0
    }
    // Number of entries (bindings) in each set.
    fn num_bindings_in_set(&self, _set: usize) -> Option<usize> {
        None
    }
    // Descriptor descriptions.
    fn descriptor(&self, _set: usize, _binding: usize) -> Option<vk::DescriptorDesc> {
        None
    }
    // Number of push constants ranges (think: number of push constants).
    fn num_push_constants_ranges(&self) -> usize {
        0
    }
    // Each push constant range in memory.
    fn push_constants_range(&self, _num: usize) -> Option<vk::PipelineLayoutDescPcRange> {
        None
    }
}

pub(crate) fn update(model: &mut Model) {
    if let Ok(msg) = model.shader_change.try_recv() {
        match msg {
            ShaderMsg::Vert(v) => {
                model.vert_shader = unsafe {
                    vk::pipeline::shader::ShaderModule::from_words(model.device.clone(), &v)
                }
                .unwrap();
                model.vert_interfaces = create_interfaces(&v);
                update_pipeline(model);
            }
            ShaderMsg::Frag(v) => {
                model.frag_shader = unsafe {
                    vk::pipeline::shader::ShaderModule::from_words(model.device.clone(), &v)
                }
                .unwrap();
                model.frag_interfaces = create_interfaces(&v);
                update_pipeline(model);
            }
        }
    }
}

pub fn compile_shader(path: PathBuf, shader_kind: shaderc::ShaderKind) -> shaderc::Result<Vec<u32>> {
    // TODO Probably shouldn't create this every time.
    let mut compiler = shaderc::Compiler::new().expect("failed to create compiler");
    let mut f = File::open(&path).expect("failed to open shader src");
    let mut src = String::new();
    f.read_to_string(&mut src).expect("failed to read src");
    let mut options = shaderc::CompileOptions::new().unwrap();
    options.add_macro_definition("EP", Some("main"));
    let result = compiler
        .compile_into_spirv(
            src.as_str(),
            shader_kind,
            path.to_str().expect("failed to make path string"),
            "main",
            None,
        )?;
    let data = result.as_binary();
    Ok(data.to_owned())
}

pub fn create_interfaces(data: &[u32]) -> ShaderInterfaces {
    sr::ShaderModule::load_u32_data(data)
        .map(|m| {
            let inputs = m
                .enumerate_input_variables(None)
                .map(|inputs| {
                    inputs
                        .iter()
                        .filter(|i| {
                            !i.decoration_flags
                                .contains(sr::types::ReflectDecorationFlags::BUILT_IN)
                        })
                        .map(|i| ShaderInterfaceDefEntry {
                            location: i.location..(i.location + 1),
                            format: to_format(i.format),
                            name: Some(Cow::from(i.name.clone())),
                        })
                        .collect::<Vec<ShaderInterfaceDefEntry>>()
                })
                .expect("Failed to pass inputs");
            let outputs = m
                .enumerate_output_variables(None)
                .map(|outputs| {
                    outputs
                        .iter()
                        .filter(|i| {
                            !i.decoration_flags
                                .contains(sr::types::ReflectDecorationFlags::BUILT_IN)
                        })
                        .map(|i| ShaderInterfaceDefEntry {
                            location: i.location..(i.location + 1),
                            format: to_format(i.format),
                            name: Some(Cow::from(i.name.clone())),
                        })
                        .collect::<Vec<ShaderInterfaceDefEntry>>()
                })
                .expect("Failed to pass outputs");
            let sets = m
                .enumerate_descriptor_sets(None)
                .map(|sets| {
                    sets.iter()
                        .map(|i| {
                            dbg!(i);
                        })
                        .collect::<Vec<()>>()
                })
                .expect("Failed to pass outputs");
            ShaderInterfaces { inputs, outputs }
        })
        .expect("failed to load module")
}

fn to_format(f: sr::types::ReflectFormat) -> vk::Format {
    use sr::types::ReflectFormat::*;
    use vk::Format::*;
    match f {
        Undefined => unreachable!(),
        R32_UINT => R32Uint,
        R32_SINT => R32Sint,
        R32_SFLOAT => R32Sfloat,
        R32G32_UINT => R32G32Uint,
        R32G32_SINT => R32G32Sint,
        R32G32_SFLOAT => R32G32Sfloat,
        R32G32B32_UINT => R32G32B32Uint,
        R32G32B32_SINT => R32G32B32Sint,
        R32G32B32_SFLOAT => R32G32B32Sfloat,
        R32G32B32A32_UINT => R32G32B32A32Uint,
        R32G32B32A32_SINT => R32G32B32A32Sint,
        R32G32B32A32_SFLOAT => R32G32B32A32Sfloat,
    }
}

pub(crate) fn update_pipeline(model: &mut Model) {
    let Model {
        ref vert_shader,
        ref frag_shader,
        ref vert_interfaces,
        ref frag_interfaces,
        ref device,
        ref render_pass,
        ref mut pipeline,
        ..
    } = model;
    let vert_main = unsafe {
        vert_shader.graphics_entry_point(
            CStr::from_bytes_with_nul_unchecked(b"main\0"),
            VertInput {
                inputs: vert_interfaces.inputs.clone(),
            },
            VertOutput {
                outputs: vert_interfaces.outputs.clone(),
            },
            VertLayout(vk::ShaderStages {
                vertex: true,
                ..vk::ShaderStages::none()
            }),
            vk::pipeline::shader::GraphicsShaderType::Vertex,
        )
    };
    let frag_main = unsafe {
        frag_shader.graphics_entry_point(
            CStr::from_bytes_with_nul_unchecked(b"main\0"),
            FragInput {
                inputs: frag_interfaces.inputs.clone(),
            },
            FragOutput {
                outputs: frag_interfaces.outputs.clone(),
            },
            FragLayout(vk::ShaderStages {
                fragment: true,
                ..vk::ShaderStages::none()
            }),
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
            .triangle_list()
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
