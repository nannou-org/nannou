use bevy::asset::embedded_asset;
use bevy::core_pipeline::core_3d::graph::{Core3d, Node3d};
use bevy::core_pipeline::core_3d::{Opaque3d, Opaque3dBinKey, CORE_3D_DEPTH_FORMAT};
use bevy::core_pipeline::fullscreen_vertex_shader::{
    fullscreen_shader_vertex_state, FULLSCREEN_SHADER_HANDLE,
};
use bevy::ecs::entity::EntityHashMap;
use bevy::ecs::query::{QueryItem, ROQueryItem};
use bevy::ecs::system::lifetimeless::SRes;
use bevy::ecs::system::SystemParamItem;
use bevy::prelude::*;
use bevy::render::extract_component::DynamicUniformIndex;
use bevy::render::render_asset::{PrepareAssetError, RenderAsset, RenderAssets};
use bevy::render::render_graph::{
    NodeRunError, RenderGraphApp, RenderGraphContext, RenderLabel, ViewNode, ViewNodeRunner,
};
use bevy::render::render_phase::{
    BinnedRenderPhaseType, DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult,
    SetItemPipeline, TrackedRenderPass, ViewBinnedRenderPhases,
};
use bevy::render::render_resource::binding_types::{
    sampler, texture_2d, uniform_buffer, uniform_buffer_sized,
};
use bevy::render::render_resource::*;
use bevy::render::renderer::{RenderContext, RenderDevice, RenderQueue};
use bevy::render::texture::{BevyDefault, DefaultImageSampler, GpuImage};
use bevy::render::view::{ExtractedView, ViewTarget, VisibleEntities};
use bevy::render::{Extract, Render, RenderApp, RenderSet};
use bevy::utils::HashMap;
use std::num::{NonZero, NonZeroU64};

use crate::asset::{GpuIsf, Isf};
use crate::inputs::{IsfInputValue, IsfInputs};
use crate::WithIsfInputs;

pub struct IsfRenderPlugin;

impl Plugin for IsfRenderPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "src/", "fullscreen.vert");
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_systems(
                Render,
                (
                    queue_isf.in_set(RenderSet::Queue),
                    prepare_isf_bind_groups.in_set(RenderSet::PrepareBindGroups),
                ),
            )
            .init_resource::<SpecializedRenderPipelines<IsfPipeline>>()
            .add_render_graph_node::<ViewNodeRunner<IsfNode>>(Core3d, IsfLabel)
            .add_render_graph_edges(
                Core3d,
                (Node3d::StartMainPass, IsfLabel, Node3d::EndMainPass),
            );
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.init_resource::<IsfPipeline>();
    }
}

#[derive(Component)]
pub struct IsfPipelineId(CachedRenderPipelineId);

fn queue_isf(
    mut commands: Commands,
    mut render_device: ResMut<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    mut isf_pipeline: ResMut<IsfPipeline>,
    isf_assets: Res<RenderAssets<GpuIsf>>,
    msaa: Res<Msaa>,
    mut specialized_render_pipelines: ResMut<SpecializedRenderPipelines<IsfPipeline>>,
    views: Query<(Entity, &Handle<Isf>, &IsfInputs), With<ExtractedView>>,
) {
    for (view_entity, isf, isf_inputs) in views.iter() {
        let isf = isf_assets.get(isf).unwrap();

        // Prepare any new layouts
        if let None = isf_pipeline
            .isf_input_uniforms_layouts
            .get(&isf_inputs.uniform_size())
        {
            isf_pipeline.isf_input_uniforms_layouts.insert(
                isf_inputs.uniform_size(),
                render_device.create_bind_group_layout(
                    "isf_input_uniforms_layout",
                    &BindGroupLayoutEntries::single(
                        ShaderStages::FRAGMENT,
                        uniform_buffer_sized(false, NonZero::new(isf_inputs.uniform_size() as u64)),
                    ),
                ),
            );
        }

        let image_count = isf.isf.num_images();
        if let None = isf_pipeline
            .isf_textures_bind_group_layouts
            .get(&image_count)
        {
            isf_pipeline
                .isf_textures_bind_group_layouts
                .insert(image_count, {
                    let mut entries = vec![];
                    entries.push(
                        sampler(SamplerBindingType::Filtering).build(0, ShaderStages::FRAGMENT),
                    );
                    for i in 0..image_count {
                        entries.push(
                            texture_2d(TextureSampleType::Float { filterable: true })
                                .build((i + 1) as u32, ShaderStages::FRAGMENT),
                        );
                    }

                    info!("isf_textures_bind_group_layouts: {:?}", entries);
                    render_device
                        .create_bind_group_layout("isf_textures_bind_group_layout", &entries)
                });
        }

        let pipeline_id = specialized_render_pipelines.specialize(
            &pipeline_cache,
            &isf_pipeline,
            IsfPipelineKey {
                shader: isf.isf.shader.clone(),
                size: isf_inputs.uniform_size(),
                textures: isf.isf.num_images(),
                msaa: *msaa,
            },
        );

        commands
            .entity(view_entity)
            .insert(IsfPipelineId(pipeline_id));
    }
}

#[derive(Component)]
pub struct IsfBindGroups {
    isf_inputs_bind_group: BindGroup,
    isf_textures_bind_group: BindGroup,
}

fn prepare_isf_bind_groups(
    mut commands: Commands,
    pipeline: Res<IsfPipeline>,
    sampler: Res<DefaultImageSampler>,
    views: Query<(Entity, &ExtractedView, &Handle<Isf>, &IsfInputs)>,
    render_device: Res<RenderDevice>,
    isf_assets: Res<RenderAssets<GpuIsf>>,
    gpu_images: Res<RenderAssets<GpuImage>>,
) {
    for (entity, extracted_view, isf, isf_inputs) in views.iter() {
        let gpu_isf = isf_assets.get(isf).unwrap();

        let isf_inputs_uniform_buffer =
            render_device.create_buffer_with_data(&BufferInitDescriptor {
                label: None,
                contents: &bytemuck::cast_slice(&isf_inputs.to_uniform()),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });

        let inputs_bind_group = render_device.create_bind_group(
            "isf_inputs_bind_group",
            &pipeline.isf_input_uniforms_layouts[&isf_inputs.uniform_size()],
            &[BindGroupEntry {
                binding: 0,
                resource: isf_inputs_uniform_buffer.as_entire_binding(),
            }],
        );

        let image_count = gpu_isf.isf.num_images();
        let mut bindings = vec![BindGroupEntry {
            binding: 0,
            resource: sampler.into_binding(),
        }];

        let mut binding = 1;

        for (_name, image) in gpu_isf.isf.imported_images.iter() {
            let gpu_image = gpu_images.get(image).unwrap();
            bindings.push(BindGroupEntry {
                binding,
                resource: gpu_image.texture_view.into_binding(),
            });
            binding += 1;
        }
        for input in &gpu_isf.isf.isf.inputs {
            match input.ty {
                isf::InputType::Image { .. }
                | isf::InputType::Audio(_)
                | isf::InputType::AudioFft(_) => {
                    let input_value = &isf_inputs[&input.name];
                    match input_value {
                        IsfInputValue::Image(image) => {
                            let gpu_image = gpu_images
                                .get(image)
                                .or_else(|| gpu_images.get(&Handle::<Image>::default()))
                                .unwrap();
                            bindings.push(BindGroupEntry {
                                binding,
                                resource: gpu_image.texture_view.into_binding(),
                            });
                            binding += 1;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        let textures_bind_group = render_device.create_bind_group(
            "isf_textures_bind_group",
            &pipeline.isf_textures_bind_group_layouts[&image_count],
            &bindings,
        );

        commands.entity(entity).insert(IsfBindGroups {
            isf_inputs_bind_group: inputs_bind_group,
            isf_textures_bind_group: textures_bind_group,
        });
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct IsfLabel;

#[derive(Default)]
struct IsfNode;

impl ViewNode for IsfNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static IsfBindGroups,
        &'static IsfPipelineId,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, bind_groups, pipeline_id): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let pipeline = world.resource::<IsfPipeline>();
        let uniform = IsfUniform {
            pass_index: 0,
            render_size: [0.0; 2],
            time: 0.0,
            time_delta: 0.0,
            date: [0.0; 4],
            frame_index: 0,
        };

        let isf_uniform_buffer =
            render_context
                .render_device()
                .create_buffer_with_data(&BufferInitDescriptor {
                    label: None,
                    contents: &bytemuck::cast_slice(&[0; 10]),
                    usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                });

        let uniform_bind_group = render_context.render_device().create_bind_group(
            "isf_inputs_bind_group",
            &pipeline.isf_uniforms_layout,
            &[BindGroupEntry {
                binding: 0,
                resource: isf_uniform_buffer.as_entire_binding(),
            }],
        );

        let pipeline_cache = world.resource::<PipelineCache>();
        let Some(pipeline) = pipeline_cache.get_render_pipeline(pipeline_id.0) else {
            warn!("Failed to get render pipeline");
            return Ok(());
        };

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("isf_pass"),
            color_attachments: &[Some(view_target.get_color_attachment())],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &uniform_bind_group, &[]);
        render_pass.set_bind_group(1, &bind_groups.isf_inputs_bind_group, &[]);
        render_pass.set_bind_group(2, &bind_groups.isf_textures_bind_group, &[]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

struct DrawIsf;

impl<P> RenderCommand<P> for DrawIsf
where
    P: PhaseItem,
{
    type Param = SRes<IsfBuffers>;

    type ViewQuery = ();

    type ItemQuery = ();

    fn render<'w>(
        _: &P,
        _: ROQueryItem<'w, Self::ViewQuery>,
        _: Option<ROQueryItem<'w, Self::ItemQuery>>,
        custom_phase_item_buffers: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        // Borrow check workaround.
        let custom_phase_item_buffers = custom_phase_item_buffers.into_inner();

        pass.draw(0..3, 0..1);

        RenderCommandResult::Success
    }
}

#[derive(Resource)]
struct IsfBuffers {}

#[derive(Component)]
pub struct ExtractedIsf {
    inputs: IsfInputs,
    images: Vec<Handle<Image>>,
}

struct IsfPass {
    pub texture: Texture,
    pub texture_view: TextureView,
    pub texture_format: TextureFormat,
    pub sampler: Sampler,
    pub size: UVec2,
    pub mip_level_count: u32,
}

#[derive(Resource)]
pub struct IsfPipeline {
    isf_uniforms_layout: BindGroupLayout,
    isf_input_uniforms_layouts: HashMap<usize, BindGroupLayout>,
    isf_textures_bind_group_layouts: HashMap<usize, BindGroupLayout>,
    fullscreen_shader: Handle<Shader>,
}

impl FromWorld for IsfPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let isf_uniforms_layout = render_device.create_bind_group_layout(
            "isf_uniforms_layout",
            &BindGroupLayoutEntries::single(
                ShaderStages::FRAGMENT,
                uniform_buffer::<IsfUniform>(false),
            ),
        );
        Self {
            isf_uniforms_layout,
            isf_input_uniforms_layouts: default(),
            isf_textures_bind_group_layouts: default(),
            fullscreen_shader: world
                .resource_mut::<AssetServer>()
                .load("embedded://bevy_nannou_isf/fullscreen.vert"),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct IsfPipelineKey {
    shader: Handle<Shader>,
    size: usize,
    textures: usize,
    msaa: Msaa,
}

impl SpecializedRenderPipeline for IsfPipeline {
    type Key = IsfPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some("isf_pipeline".into()),
            layout: vec![
                self.isf_uniforms_layout.clone(),
                self.isf_input_uniforms_layouts[&key.size].clone(),
                self.isf_textures_bind_group_layouts[&key.textures].clone(),
            ],
            push_constant_ranges: vec![],
            vertex: VertexState {
                shader: self.fullscreen_shader.clone(),
                shader_defs: Vec::new(),
                entry_point: "main".into(),
                buffers: vec![],
            },
            fragment: Some(FragmentState {
                shader: key.shader,
                shader_defs: vec![],
                entry_point: "main".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState {
                count: key.msaa.samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        }
    }
}

#[derive(ShaderType, Clone)]
struct IsfUniform {
    pass_index: i32,
    render_size: [f32; 2],
    time: f32,
    time_delta: f32,
    date: [f32; 4],
    frame_index: i32,
}

#[derive(Component, Deref, DerefMut)]
pub struct IsfInputsUniform(pub Vec<u8>);
