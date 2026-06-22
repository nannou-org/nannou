use crate::{
    PackedSdfCacheConfig, Sdf, SdfBounds, SdfConfig, SdfGpuComputeState, SdfGpuHandles, SdfScene,
};
use bevy::{
    asset::{Asset, uuid_handle},
    core_pipeline::schedule::{Core3d, Core3dSystems},
    ecs::{query::QueryItem, system::lifetimeless::Read},
    pbr::MATERIAL_BIND_GROUP_INDEX,
    prelude::*,
    render::{
        Render, RenderApp, RenderSystems,
        extract_instances::{ExtractInstance, ExtractInstancesPlugin, ExtractedInstances},
        render_asset::{RenderAssets, prepare_assets},
        render_resource::{
            AsBindGroup, BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
            BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferInitDescriptor,
            BufferUsages, CachedComputePipelineId, ComputePassDescriptor,
            ComputePipelineDescriptor, PipelineCache, ShaderStages, ShaderType,
            encase::{UniformBuffer, internal::WriteInto},
        },
        renderer::{RenderContext, RenderDevice},
        storage::{GpuShaderBuffer, ShaderBuffer},
    },
    shader::{Shader, ShaderDefVal, ShaderRef},
};
use nannou_draw::{draw::Draw, render::ShaderModel};
use std::{
    borrow::Cow,
    collections::HashMap,
    sync::{Arc, RwLock},
};

pub const SDF_SHADER_HANDLE: Handle<Shader> = uuid_handle!("6111cf3e-105b-4c2d-9c7a-a8b2b74d9f2c");

const COMPUTE_WORKGROUP_SIZE: u32 = 64;

pub struct SdfComputePlugin;

impl Plugin for SdfComputePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractInstancesPlugin::<SdfComputeInstance>::new());

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<PreparedSdfComputeInstances>()
            .add_systems(
                Render,
                prepare_sdf_compute_instances
                    .after(prepare_assets::<GpuShaderBuffer>)
                    .in_set(RenderSystems::PrepareBindGroups),
            );
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<SdfComputePipeline>()
            .add_systems(Core3d, run_sdf_compute.in_set(Core3dSystems::Prepass));
    }
}

#[derive(Clone)]
struct SdfComputeInstance {
    handles: SdfGpuHandles,
    shared_gpu: Arc<RwLock<SdfGpuHandles>>,
    compute: SdfGpuComputeState,
}

impl ExtractInstance for SdfComputeInstance {
    type QueryData = Read<Sdf>;
    type QueryFilter = ();

    fn extract(item: QueryItem<'_, '_, Self::QueryData>) -> Option<Self> {
        let gpu = item.gpu.read().ok()?.clone();
        Some(Self {
            compute: gpu.compute.clone(),
            handles: gpu,
            shared_gpu: item.gpu.clone(),
        })
    }
}

#[derive(Resource, Default)]
struct PreparedSdfComputeInstances(HashMap<Entity, PreparedSdfCompute>);

struct PreparedSdfCompute {
    init: PreparedSdfComputePass,
    stages: Vec<PreparedSdfComputeStage>,
    finalize: PreparedSdfComputePass,
    shared_gpu: Arc<RwLock<SdfGpuHandles>>,
    cache_version: u64,
    dirty_count: u32,
    sample_workgroups: u32,
}

struct PreparedSdfComputeStage {
    shape_kind: u32,
    pass: PreparedSdfComputePass,
}

struct PreparedSdfComputePass {
    bind_group: BindGroup,
    _uniform: Buffer,
}

#[derive(Resource)]
struct SdfComputePipeline {
    layout_descriptor: BindGroupLayoutDescriptor,
    init: CachedComputePipelineId,
    eval: [CachedComputePipelineId; 9],
    finalize: CachedComputePipelineId,
}

impl FromWorld for SdfComputePipeline {
    fn from_world(world: &mut World) -> Self {
        let pipeline_cache = world.resource::<PipelineCache>();
        let layout_entries = [
            uniform_entry(0),
            storage_entry(1, true),
            storage_entry(2, true),
            storage_entry(3, true),
            storage_entry(4, false),
            storage_entry(5, false),
            storage_entry(6, false),
            storage_entry(7, false),
            storage_entry(8, false),
        ];
        let layout_descriptor =
            BindGroupLayoutDescriptor::new("nannou_sdf_compute_layout", &layout_entries);

        let init = queue_compute_pipeline(pipeline_cache, &layout_descriptor, "sdf_init_bricks");
        let eval = [
            queue_compute_pipeline(pipeline_cache, &layout_descriptor, "sdf_eval_sphere"),
            queue_compute_pipeline(pipeline_cache, &layout_descriptor, "sdf_eval_cuboid"),
            queue_compute_pipeline(pipeline_cache, &layout_descriptor, "sdf_eval_capsule"),
            queue_compute_pipeline(pipeline_cache, &layout_descriptor, "sdf_eval_cylinder"),
            queue_compute_pipeline(pipeline_cache, &layout_descriptor, "sdf_eval_cone"),
            queue_compute_pipeline(pipeline_cache, &layout_descriptor, "sdf_eval_torus"),
            queue_compute_pipeline(pipeline_cache, &layout_descriptor, "sdf_eval_ellipsoid"),
            queue_compute_pipeline(pipeline_cache, &layout_descriptor, "sdf_eval_plane"),
            queue_compute_pipeline(pipeline_cache, &layout_descriptor, "sdf_eval_terrain"),
        ];
        let finalize =
            queue_compute_pipeline(pipeline_cache, &layout_descriptor, "sdf_finalize_bricks");

        Self {
            layout_descriptor,
            init,
            eval,
            finalize,
        }
    }
}

fn uniform_entry(binding: u32) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding,
        visibility: ShaderStages::COMPUTE,
        ty: BindingType::Buffer {
            ty: BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn storage_entry(binding: u32, read_only: bool) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding,
        visibility: ShaderStages::COMPUTE,
        ty: BindingType::Buffer {
            ty: BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn queue_compute_pipeline(
    pipeline_cache: &PipelineCache,
    layout: &BindGroupLayoutDescriptor,
    entry_point: &'static str,
) -> CachedComputePipelineId {
    pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some(format!("nannou_sdf_{entry_point}").into()),
        layout: vec![layout.clone()],
        immediate_size: 0,
        shader: SDF_SHADER_HANDLE,
        shader_defs: vec![
            ShaderDefVal::UInt(
                "MATERIAL_BIND_GROUP".into(),
                MATERIAL_BIND_GROUP_INDEX as u32,
            ),
            ShaderDefVal::from("VERTEX_POSITIONS"),
        ],
        entry_point: Some(Cow::Borrowed(entry_point)),
        zero_initialize_workgroup_memory: false,
    })
}

#[derive(Clone, Copy, Debug, ShaderType)]
struct SdfComputeUniform {
    cache: PackedSdfCacheConfig,
    counts: UVec4,
}

fn prepare_sdf_compute_instances(
    mut prepared: ResMut<PreparedSdfComputeInstances>,
    extracted: Res<ExtractedInstances<SdfComputeInstance>>,
    pipeline: Res<SdfComputePipeline>,
    pipeline_cache: Res<PipelineCache>,
    render_device: Res<RenderDevice>,
    gpu_buffers: Res<RenderAssets<GpuShaderBuffer>>,
) {
    let layout = pipeline_cache.get_bind_group_layout(&pipeline.layout_descriptor);
    for (entity, instance) in extracted.iter() {
        let compute = &instance.compute;
        if !compute.has_content || compute.dirty_count == 0 || compute.stage_count == 0 {
            continue;
        }
        if prepared
            .0
            .get(&entity.id())
            .is_some_and(|work| work.cache_version == compute.cache_version)
        {
            continue;
        }

        let Some(resources) = SdfComputeBuffers::from_handles(&instance.handles, &gpu_buffers)
        else {
            continue;
        };

        let sample_workgroups = compute
            .config
            .atlas
            .y
            .div_ceil(COMPUTE_WORKGROUP_SIZE)
            .max(1);
        let init = create_compute_pass(
            &render_device,
            &layout,
            &resources,
            compute,
            0,
            "nannou_sdf_init_uniform",
        );
        let finalize = create_compute_pass(
            &render_device,
            &layout,
            &resources,
            compute,
            0,
            "nannou_sdf_finalize_uniform",
        );
        let stages = compute
            .stage_shape_kinds
            .iter()
            .enumerate()
            .filter_map(|(stage_index, shape_kind)| {
                if *shape_kind >= pipeline.eval.len() as u32 {
                    return None;
                }
                Some(PreparedSdfComputeStage {
                    shape_kind: *shape_kind,
                    pass: create_compute_pass(
                        &render_device,
                        &layout,
                        &resources,
                        compute,
                        stage_index as u32,
                        "nannou_sdf_stage_uniform",
                    ),
                })
            })
            .collect::<Vec<_>>();

        prepared.0.insert(
            entity.id(),
            PreparedSdfCompute {
                init,
                stages,
                finalize,
                shared_gpu: instance.shared_gpu.clone(),
                cache_version: compute.cache_version,
                dirty_count: compute.dirty_count,
                sample_workgroups,
            },
        );
    }
}

struct SdfComputeBuffers<'a> {
    edits: &'a GpuShaderBuffer,
    stages: &'a GpuShaderBuffer,
    dirty_bricks: &'a GpuShaderBuffer,
    brick_map: &'a GpuShaderBuffer,
    brick_meta: &'a GpuShaderBuffer,
    distance_atlas: &'a GpuShaderBuffer,
    color_atlas: &'a GpuShaderBuffer,
    material_atlas: &'a GpuShaderBuffer,
}

impl<'a> SdfComputeBuffers<'a> {
    fn from_handles(
        gpu: &SdfGpuHandles,
        buffers: &'a RenderAssets<GpuShaderBuffer>,
    ) -> Option<Self> {
        Some(Self {
            edits: buffers.get(&gpu.edits)?,
            stages: buffers.get(&gpu.stages)?,
            dirty_bricks: buffers.get(&gpu.dirty_bricks)?,
            brick_map: buffers.get(&gpu.brick_map)?,
            brick_meta: buffers.get(&gpu.brick_meta)?,
            distance_atlas: buffers.get(&gpu.distance_atlas)?,
            color_atlas: buffers.get(&gpu.color_atlas)?,
            material_atlas: buffers.get(&gpu.material_atlas)?,
        })
    }
}

fn create_compute_pass(
    render_device: &RenderDevice,
    layout: &BindGroupLayout,
    resources: &SdfComputeBuffers<'_>,
    compute: &SdfGpuComputeState,
    stage_index: u32,
    label: &'static str,
) -> PreparedSdfComputePass {
    let uniform = SdfComputeUniform {
        cache: compute.config,
        counts: UVec4::new(stage_index, compute.dirty_count, compute.stage_count, 0),
    };
    let uniform = create_uniform_buffer(render_device, &uniform, label);
    let bind_group = render_device.create_bind_group(
        Some("nannou_sdf_compute_bind_group"),
        layout,
        &pipeline_entries(
            &uniform,
            resources.edits,
            resources.stages,
            resources.dirty_bricks,
            resources.brick_map,
            resources.brick_meta,
            resources.distance_atlas,
            resources.color_atlas,
            resources.material_atlas,
        ),
    );
    PreparedSdfComputePass {
        bind_group,
        _uniform: uniform,
    }
}

fn pipeline_entries<'a>(
    uniform: &'a Buffer,
    edits: &'a GpuShaderBuffer,
    stages: &'a GpuShaderBuffer,
    dirty_bricks: &'a GpuShaderBuffer,
    brick_map: &'a GpuShaderBuffer,
    brick_meta: &'a GpuShaderBuffer,
    distance_atlas: &'a GpuShaderBuffer,
    color_atlas: &'a GpuShaderBuffer,
    material_atlas: &'a GpuShaderBuffer,
) -> [BindGroupEntry<'a>; 9] {
    [
        BindGroupEntry {
            binding: 0,
            resource: uniform.as_entire_binding(),
        },
        storage_binding(1, edits),
        storage_binding(2, stages),
        storage_binding(3, dirty_bricks),
        storage_binding(4, brick_map),
        storage_binding(5, brick_meta),
        storage_binding(6, distance_atlas),
        storage_binding(7, color_atlas),
        storage_binding(8, material_atlas),
    ]
}

fn storage_binding<'a>(binding: u32, buffer: &'a GpuShaderBuffer) -> BindGroupEntry<'a> {
    BindGroupEntry {
        binding,
        resource: buffer.buffer.as_entire_binding(),
    }
}

fn create_uniform_buffer<T: ShaderType + WriteInto>(
    render_device: &RenderDevice,
    value: &T,
    label: &'static str,
) -> Buffer {
    let mut writer = UniformBuffer::<Vec<u8>>::new(Vec::new());
    writer.write(value).unwrap();
    render_device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some(label),
        contents: writer.as_ref(),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    })
}

fn run_sdf_compute(
    mut ctx: RenderContext,
    pipeline_cache: Res<PipelineCache>,
    pipeline: Res<SdfComputePipeline>,
    mut prepared: ResMut<PreparedSdfComputeInstances>,
) {
    if prepared.0.is_empty() {
        return;
    }

    let Some(init_pipeline) = pipeline_cache.get_compute_pipeline(pipeline.init) else {
        return;
    };
    let Some(finalize_pipeline) = pipeline_cache.get_compute_pipeline(pipeline.finalize) else {
        return;
    };

    let mut ready_work = Vec::new();
    for (entity, work) in &prepared.0 {
        let mut eval_pipelines = Vec::with_capacity(work.stages.len());
        let mut all_ready = true;
        for stage in &work.stages {
            let Some(pipeline_id) = pipeline.eval.get(stage.shape_kind as usize) else {
                all_ready = false;
                break;
            };
            let Some(eval_pipeline) = pipeline_cache.get_compute_pipeline(*pipeline_id) else {
                all_ready = false;
                break;
            };
            eval_pipelines.push(eval_pipeline);
        }
        if all_ready {
            ready_work.push((*entity, eval_pipelines));
        }
    }

    if ready_work.is_empty() {
        return;
    }

    let mut pass = ctx
        .command_encoder()
        .begin_compute_pass(&ComputePassDescriptor {
            label: Some("nannou_sdf_compute_pass"),
            timestamp_writes: None,
        });

    let mut completed = Vec::with_capacity(ready_work.len());
    for (entity, eval_pipelines) in ready_work {
        let Some(work) = prepared.0.get(&entity) else {
            continue;
        };

        pass.set_pipeline(init_pipeline);
        pass.set_bind_group(0, &work.init.bind_group, &[]);
        pass.dispatch_workgroups(work.sample_workgroups, work.dirty_count, 1);

        for (stage, eval_pipeline) in work.stages.iter().zip(eval_pipelines) {
            pass.set_pipeline(eval_pipeline);
            pass.set_bind_group(0, &stage.pass.bind_group, &[]);
            pass.dispatch_workgroups(work.sample_workgroups, work.dirty_count, 1);
        }

        pass.set_pipeline(finalize_pipeline);
        pass.set_bind_group(0, &work.finalize.bind_group, &[]);
        pass.dispatch_workgroups(work.dirty_count, 1, 1);
        completed.push(entity);
    }

    drop(pass);

    for entity in completed {
        if let Some(work) = prepared.0.remove(&entity) {
            if let Ok(mut gpu) = work.shared_gpu.write() {
                gpu.completed_cache_version = gpu.completed_cache_version.max(work.cache_version);
                if gpu.pending_cache_version == work.cache_version {
                    gpu.pending_cache_version = 0;
                    gpu.compute.dirty_count = 0;
                    gpu.compute.stage_count = 0;
                    gpu.compute.stage_shape_kinds.clear();
                }
            }
        }
    }
}

/// Camera parameters used by the SDF raymarch renderer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SdfCamera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    /// Vertical field of view, in radians.
    pub fov_y_radians: f32,
    /// Horizontal/vertical viewport aspect ratio. Values <= 0.0 use the render target aspect.
    pub aspect_ratio: f32,
}

impl SdfCamera {
    pub fn look_at(position: Vec3, target: Vec3) -> Self {
        Self {
            position,
            target,
            ..Default::default()
        }
    }

    pub fn fov_degrees(mut self, degrees: f32) -> Self {
        self.fov_y_radians = degrees.to_radians();
        self
    }
}

impl Default for SdfCamera {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 600.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            fov_y_radians: 45.0_f32.to_radians(),
            aspect_ratio: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SdfLighting {
    /// Direction the light travels in world space.
    pub direction: Vec3,
    /// Linear RGB light tint. Values above 1.0 increase intensity.
    pub color: Vec3,
    /// Ambient contribution mixed into every shaded hit.
    pub ambient: f32,
    /// Lambertian diffuse contribution.
    pub diffuse: f32,
}

impl Default for SdfLighting {
    fn default() -> Self {
        Self {
            direction: Vec3::new(-0.4, -0.8, -0.6),
            color: Vec3::ONE,
            ambient: 0.25,
            diffuse: 0.75,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[repr(u32)]
pub enum SdfDebugView {
    None = 0,
    DirtyBricks = 1,
    BrickResidency = 2,
    Distance = 3,
    Normals = 4,
}

impl Default for SdfDebugView {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SdfRenderSettings {
    pub camera: SdfCamera,
    pub lighting: SdfLighting,
    pub debug: SdfDebugView,
    pub max_steps: u32,
    pub hit_epsilon: f32,
    pub normal_epsilon: f32,
    pub max_distance: f32,
}

impl Default for SdfRenderSettings {
    fn default() -> Self {
        Self {
            camera: SdfCamera::default(),
            lighting: SdfLighting::default(),
            debug: SdfDebugView::None,
            max_steps: 512,
            hit_epsilon: 0.5,
            normal_epsilon: 1.0,
            max_distance: 1000.0,
        }
    }
}

/// Extension trait that adds `draw.sdf(&sdf)` to nannou's [`Draw`] API.
pub trait SdfDrawExt {
    fn sdf<'a>(&'a self, sdf: &'a Sdf) -> SdfRenderBuilder<'a>;
}

impl SdfDrawExt for Draw {
    fn sdf<'a>(&'a self, sdf: &'a Sdf) -> SdfRenderBuilder<'a> {
        SdfRenderBuilder {
            draw: self,
            sdf,
            settings: SdfRenderSettings::default(),
            finished: false,
        }
    }
}

/// Builder returned by [`SdfDrawExt::sdf`].
pub struct SdfRenderBuilder<'a> {
    draw: &'a Draw,
    sdf: &'a Sdf,
    settings: SdfRenderSettings,
    finished: bool,
}

impl<'a> SdfRenderBuilder<'a> {
    pub fn camera(mut self, camera: SdfCamera) -> Self {
        self.settings.camera = camera;
        self
    }

    pub fn look_at(mut self, position: Vec3, target: Vec3) -> Self {
        self.settings.camera.position = position;
        self.settings.camera.target = target;
        self
    }

    pub fn fov_degrees(mut self, degrees: f32) -> Self {
        self.settings.camera.fov_y_radians = degrees.to_radians();
        self
    }

    pub fn lighting(mut self, lighting: SdfLighting) -> Self {
        self.settings.lighting = lighting;
        self
    }

    pub fn light_dir(mut self, direction: Vec3) -> Self {
        self.settings.lighting.direction = direction;
        self
    }

    pub fn light_color(mut self, color: Vec3) -> Self {
        self.settings.lighting.color = color;
        self
    }

    pub fn ambient(mut self, ambient: f32) -> Self {
        self.settings.lighting.ambient = ambient;
        self
    }

    pub fn diffuse(mut self, diffuse: f32) -> Self {
        self.settings.lighting.diffuse = diffuse;
        self
    }

    pub fn debug(mut self, debug: SdfDebugView) -> Self {
        self.settings.debug = debug;
        self
    }

    pub fn max_steps(mut self, max_steps: u32) -> Self {
        self.settings.max_steps = max_steps.max(1);
        self
    }

    pub fn hit_epsilon(mut self, hit_epsilon: f32) -> Self {
        self.settings.hit_epsilon = hit_epsilon.max(f32::EPSILON);
        self
    }

    pub fn normal_epsilon(mut self, normal_epsilon: f32) -> Self {
        self.settings.normal_epsilon = normal_epsilon.max(f32::EPSILON);
        self
    }

    pub fn max_distance(mut self, max_distance: f32) -> Self {
        self.settings.max_distance = max_distance.max(0.0);
        self
    }

    pub fn finish(mut self) {
        self.submit();
        self.finished = true;
    }

    fn submit(&mut self) {
        let model = self.sdf.shader_model(self.settings);
        let draw = self.draw.shader_model(model);
        let points = [
            Vec3::new(-1.0, -1.0, 0.0),
            Vec3::new(1.0, -1.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(-1.0, 1.0, 0.0),
        ];
        let indices = [0usize, 1, 2, 0, 2, 3];
        draw.mesh().indexed(points, indices);
    }
}

impl Drop for SdfRenderBuilder<'_> {
    fn drop(&mut self) {
        if !self.finished {
            self.submit();
        }
    }
}

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone, Default)]
pub struct SdfShaderModel {
    #[uniform(0)]
    pub uniform: SdfRenderUniform,
    #[storage(1, read_only)]
    pub brick_map: Handle<ShaderBuffer>,
    #[storage(2, read_only)]
    pub brick_meta: Handle<ShaderBuffer>,
    #[storage(3, read_only)]
    pub distance_atlas: Handle<ShaderBuffer>,
    #[storage(4, read_only)]
    pub color_atlas: Handle<ShaderBuffer>,
    #[storage(5, read_only)]
    pub material_atlas: Handle<ShaderBuffer>,
}

impl ShaderModel for SdfShaderModel {
    fn vertex_shader() -> ShaderRef {
        SDF_SHADER_HANDLE.into()
    }

    fn fragment_shader() -> ShaderRef {
        SDF_SHADER_HANDLE.into()
    }
}

#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct SdfRenderUniform {
    pub bounds_min: Vec4,
    pub bounds_max: Vec4,
    pub camera_position: Vec4,
    pub camera_forward: Vec4,
    pub camera_right: Vec4,
    pub camera_up: Vec4,
    pub lighting_direction: Vec4,
    pub lighting_color: Vec4,
    pub render_params: Vec4,
    pub grid: UVec4,
    pub atlas: UVec4,
    pub cache_params: Vec4,
    pub counts: UVec4,
}

pub(crate) fn shader_model(
    scene: &SdfScene,
    gpu: &SdfGpuHandles,
    settings: SdfRenderSettings,
) -> SdfShaderModel {
    let config = scene.config();
    let camera = camera_for_scene(settings.camera, config);
    let forward = (camera.target - camera.position).normalize_or(Vec3::new(0.0, 0.0, -1.0));
    let right = forward.cross(camera.up).normalize_or(Vec3::X);
    let up = right.cross(forward).normalize_or(Vec3::Y);
    let half_fov = (camera.fov_y_radians * 0.5).tan();
    let right = right * half_fov;
    let up = up * half_fov;

    let brick_dims = crate::brick_dimensions(config).unwrap_or(UVec3::ONE);
    let cache = &gpu.compute;
    let has_content = cache.has_content && cache.resident_count > 0;

    SdfShaderModel {
        uniform: SdfRenderUniform {
            bounds_min: config.bounds.min.extend(0.0),
            bounds_max: config.bounds.max.extend(0.0),
            camera_position: camera.position.extend(0.0),
            camera_forward: forward.extend(0.0),
            camera_right: right.extend(camera.aspect_ratio),
            camera_up: up.extend(0.0),
            lighting_direction: settings
                .lighting
                .direction
                .normalize_or(Vec3::new(-0.4, -0.8, -0.6))
                .extend(settings.lighting.ambient.clamp(0.0, 1.0)),
            lighting_color: settings
                .lighting
                .color
                .max(Vec3::ZERO)
                .extend(settings.lighting.diffuse.max(0.0)),
            render_params: Vec4::new(
                settings.max_steps as f32,
                settings.hit_epsilon,
                settings.normal_epsilon,
                settings.max_distance,
            ),
            grid: brick_dims.extend(config.brick_size),
            atlas: UVec4::new(
                config.atlas_capacity,
                crate::samples_per_brick(config),
                crate::samples_per_axis(config),
                0,
            ),
            cache_params: Vec4::new(config.voxel_size, config.narrow_band, 0.0, 0.0),
            counts: UVec4::new(
                has_content as u32,
                cache.resident_count,
                settings.debug as u32,
                cache.dirty_count,
            ),
        },
        brick_map: gpu.brick_map.clone(),
        brick_meta: gpu.brick_meta.clone(),
        distance_atlas: gpu.distance_atlas.clone(),
        color_atlas: gpu.color_atlas.clone(),
        material_atlas: gpu.material_atlas.clone(),
    }
}

fn camera_for_scene(camera: SdfCamera, config: &SdfConfig) -> SdfCamera {
    if camera == SdfCamera::default() {
        let bounds = config.bounds;
        let radius = bounds.size().length().max(1.0) * 0.8;
        SdfCamera {
            position: bounds.center() + Vec3::new(0.0, 0.0, radius),
            target: bounds.center(),
            ..camera
        }
    } else {
        camera
    }
}

#[allow(dead_code)]
fn _bounds_for_docs(_: SdfBounds) {}
