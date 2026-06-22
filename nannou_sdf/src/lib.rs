//! Persistent 3D signed-distance-field scene support for nannou.
//!
//! The public API mirrors nannou's [`Draw`](nannou_draw::draw::Draw) builder style while keeping
//! SDF scene state persistent so edits can be diffed and dirty regions can be updated
//! incrementally.

mod render;

use bevy::{
    asset::{RenderAssetUsages, load_internal_asset},
    color::LinearRgba,
    prelude::*,
    render::{
        render_resource::{BufferUsages, ShaderType},
        storage::ShaderBuffer,
    },
};
use nannou_draw::render::NannouShaderModelPlugin;
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
    fmt,
    hash::{Hash, Hasher},
    sync::{Arc, RwLock},
};

pub use render::{
    SdfCamera, SdfDebugView, SdfDrawExt, SdfLighting, SdfRenderBuilder, SdfRenderSettings,
    SdfShaderModel,
};

pub(crate) const INVALID_ATLAS_SLOT: u32 = u32::MAX;
const DEFAULT_ATLAS_CAPACITY: u32 = 4096;
const MAX_AUTO_ATLAS_CAPACITY: u32 = 1 << 20;

/// Standard Hermite smoothstep interpolation.
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    if edge0 == edge1 {
        return if x < edge0 { 0.0 } else { 1.0 };
    }
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Quintic smootherstep interpolation.
pub fn smootherstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    if edge0 == edge1 {
        return if x < edge0 { 0.0 } else { 1.0 };
    }
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// A nannou plugin that attaches an [`Sdf`] component to each window and registers the SDF
/// shader-model renderer.
pub struct NannouSdfPlugin;

impl Plugin for NannouSdfPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            render::SDF_SHADER_HANDLE,
            "sdf_shader.wgsl",
            Shader::from_wgsl
        );

        app.add_plugins((
            NannouShaderModelPlugin::<SdfShaderModel>::default(),
            render::SdfComputePlugin,
        ))
        .add_systems(First, spawn_sdf)
        .add_systems(PostUpdate, sync_sdf_gpu_buffers);
    }
}

fn spawn_sdf(
    mut commands: Commands,
    windows: Query<Entity, (With<Window>, Without<Sdf>)>,
    mut buffers: ResMut<Assets<ShaderBuffer>>,
) {
    for window in windows.iter() {
        let gpu = SdfGpuHandles::new(&mut buffers);
        commands.entity(window).insert(Sdf::with_gpu(window, gpu));
    }
}

fn sync_sdf_gpu_buffers(sdfs: Query<&Sdf>, mut buffers: ResMut<Assets<ShaderBuffer>>) {
    for sdf in sdfs.iter() {
        let mut scene = sdf.scene.write().expect("Sdf scene lock poisoned");
        let mut gpu = sdf.gpu.write().expect("Sdf gpu lock poisoned");

        if gpu.compute_pending() {
            continue;
        }

        let packed = scene.pack_for_gpu();
        let gpu_update = scene.prepare_gpu_update(&packed);
        let requires_dispatch = gpu_update.compute.requires_dispatch();

        if gpu.packed_version != scene.version {
            if let Some(mut buffer) = buffers.get_mut(&gpu.edits) {
                buffer.set_data(packed.edits);
            }
            if let Some(mut buffer) = buffers.get_mut(&gpu.nodes) {
                buffer.set_data(packed.nodes);
            }
            if let Some(mut buffer) = buffers.get_mut(&gpu.stages) {
                buffer.set_data(packed.stages);
            }
            gpu.packed_version = scene.version;
        }

        if requires_dispatch {
            if let Some(mut buffer) = buffers.get_mut(&gpu.dirty_bricks) {
                buffer.set_data(gpu_update.dirty_bricks.clone());
            }
        }

        if gpu.cache_version != scene.cache_version {
            if gpu_update.reset_brick_map {
                if let Some(mut buffer) = buffers.get_mut(&gpu.brick_map) {
                    buffer.set_data(gpu_update.brick_map);
                }
                if let Some(mut buffer) = buffers.get_mut(&gpu.brick_meta) {
                    buffer.set_data(gpu_update.brick_meta);
                }
            }
            resize_gpu_buffer(&mut buffers, &gpu.distance_atlas, gpu_update.distance_bytes);
            resize_gpu_buffer(&mut buffers, &gpu.color_atlas, gpu_update.color_bytes);
            resize_gpu_buffer(&mut buffers, &gpu.material_atlas, gpu_update.material_bytes);
            gpu.cache_version = scene.cache_version;
        }

        gpu.compute = gpu_update.compute;
        if requires_dispatch {
            gpu.pending_cache_version = gpu.compute.cache_version;
        } else {
            gpu.completed_cache_version = gpu.compute.cache_version;
            gpu.pending_cache_version = 0;
        }
    }
}

fn resize_gpu_buffer(buffers: &mut Assets<ShaderBuffer>, handle: &Handle<ShaderBuffer>, size: u64) {
    let Some(mut buffer) = buffers.get_mut(handle) else {
        return;
    };
    if buffer.buffer_description.size == size && buffer.data.is_none() {
        return;
    }
    buffer.data = None;
    buffer.resize_in_place(size);
}

/// A persistent SDF scene handle attached to a nannou window.
#[derive(Component, Clone)]
pub struct Sdf {
    scene: Arc<RwLock<SdfScene>>,
    gpu: Arc<RwLock<SdfGpuHandles>>,
    window: Entity,
}

impl Sdf {
    /// Create a standalone SDF scene handle for the given window entity.
    pub fn new(window: Entity) -> Self {
        Self {
            scene: Arc::new(RwLock::new(SdfScene::default())),
            gpu: Arc::new(RwLock::new(SdfGpuHandles::default())),
            window,
        }
    }

    fn with_gpu(window: Entity, gpu: SdfGpuHandles) -> Self {
        Self {
            scene: Arc::new(RwLock::new(SdfScene::default())),
            gpu: Arc::new(RwLock::new(gpu)),
            window,
        }
    }

    /// The window entity this SDF scene belongs to.
    pub fn window(&self) -> Entity {
        self.window
    }

    /// Configure the scene bounds, voxel size, brick size, update budget, and cache format.
    pub fn configure(&self) -> SdfConfigBuilder<'_> {
        SdfConfigBuilder { sdf: self }
    }

    /// Record a transaction graph and replace the previous transaction layer with it.
    pub fn transaction(&self, f: impl FnOnce(&SdfTransaction)) {
        let transaction = SdfTransaction::new();
        f(&transaction);
        let graph = transaction.finish();
        self.scene
            .write()
            .expect("Sdf scene lock poisoned")
            .replace_transaction_graph(graph);
    }

    /// Begin a persistent handle-layer sphere edit.
    pub fn sphere(&self) -> SdfBuilder<'_> {
        SdfBuilder::direct(self, SdfShape::sphere())
    }

    /// Begin a persistent handle-layer cuboid edit.
    pub fn cuboid(&self) -> SdfBuilder<'_> {
        SdfBuilder::direct(self, SdfShape::cuboid())
    }

    /// Begin a persistent handle-layer rounded cuboid edit.
    pub fn rounded_cuboid(&self) -> SdfBuilder<'_> {
        SdfBuilder::direct(self, SdfShape::rounded_cuboid())
    }

    /// Begin a persistent handle-layer capsule edit.
    pub fn capsule(&self) -> SdfBuilder<'_> {
        SdfBuilder::direct(self, SdfShape::capsule())
    }

    /// Begin a persistent handle-layer cylinder edit.
    pub fn cylinder(&self) -> SdfBuilder<'_> {
        SdfBuilder::direct(self, SdfShape::cylinder())
    }

    /// Begin a persistent handle-layer cone edit.
    pub fn cone(&self) -> SdfBuilder<'_> {
        SdfBuilder::direct(self, SdfShape::cone())
    }

    /// Begin a persistent handle-layer torus edit.
    pub fn torus(&self) -> SdfBuilder<'_> {
        SdfBuilder::direct(self, SdfShape::torus())
    }

    /// Begin a persistent handle-layer ellipsoid edit.
    pub fn ellipsoid(&self) -> SdfBuilder<'_> {
        SdfBuilder::direct(self, SdfShape::ellipsoid())
    }

    /// Begin a persistent handle-layer plane edit.
    pub fn plane(&self) -> SdfBuilder<'_> {
        SdfBuilder::direct(self, SdfShape::plane())
    }

    /// Mutate an existing handle-layer edit. Invalid handles become no-ops.
    pub fn edit(&self, handle: SdfHandle) -> SdfHandleEdit<'_> {
        SdfHandleEdit { sdf: self, handle }
    }

    /// Remove a handle-layer edit.
    pub fn remove(&self, handle: SdfHandle) -> bool {
        self.scene
            .write()
            .expect("Sdf scene lock poisoned")
            .remove_handle(handle)
    }

    /// Evaluate the current combined SDF scene on the CPU.
    pub fn sample(&self, point: Vec3) -> Option<SdfSample> {
        self.scene
            .read()
            .expect("Sdf scene lock poisoned")
            .sample(point)
    }

    /// Run a read-only closure against the current scene state.
    pub fn with_scene<R>(&self, f: impl FnOnce(&SdfScene) -> R) -> R {
        let scene = self.scene.read().expect("Sdf scene lock poisoned");
        f(&scene)
    }

    /// The number of dirty logical bricks currently tracked.
    pub fn dirty_brick_count(&self) -> usize {
        self.scene
            .read()
            .expect("Sdf scene lock poisoned")
            .dirty_brick_count()
    }

    /// Drain the dirty brick set, expanding a full invalidation into explicit brick coordinates.
    pub fn take_dirty_bricks(&self) -> Vec<SdfBrick> {
        self.scene
            .write()
            .expect("Sdf scene lock poisoned")
            .take_dirty_bricks()
    }

    /// Current brick-cache status for debugging and adaptive update budgets.
    pub fn status(&self) -> SdfStatus {
        self.scene.read().expect("Sdf scene lock poisoned").status()
    }

    pub(crate) fn shader_model(&self, settings: SdfRenderSettings) -> SdfShaderModel {
        let scene = self.scene.read().expect("Sdf scene lock poisoned");
        let gpu = self.gpu.read().expect("Sdf gpu lock poisoned");
        render::shader_model(&scene, &gpu, settings)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SdfStatus {
    pub dirty_bricks: usize,
    pub resident_bricks: usize,
    pub atlas_capacity: u32,
    pub atlas_full: bool,
}

/// GPU buffer handles derived from the CPU scene.
#[derive(Clone, Debug, Default)]
pub struct SdfGpuHandles {
    pub edits: Handle<ShaderBuffer>,
    pub nodes: Handle<ShaderBuffer>,
    pub stages: Handle<ShaderBuffer>,
    pub dirty_bricks: Handle<ShaderBuffer>,
    pub brick_map: Handle<ShaderBuffer>,
    pub brick_meta: Handle<ShaderBuffer>,
    pub distance_atlas: Handle<ShaderBuffer>,
    pub color_atlas: Handle<ShaderBuffer>,
    pub material_atlas: Handle<ShaderBuffer>,
    pub compute: SdfGpuComputeState,
    packed_version: u64,
    cache_version: u64,
    pending_cache_version: u64,
    completed_cache_version: u64,
}

impl SdfGpuHandles {
    fn new(buffers: &mut Assets<ShaderBuffer>) -> Self {
        let mut edits = ShaderBuffer::from(vec![PackedSdfEdit::default()]);
        edits.buffer_description.label = Some("nannou_sdf_edits");
        edits.buffer_description.usage |= BufferUsages::STORAGE | BufferUsages::COPY_DST;
        edits.asset_usage = RenderAssetUsages::RENDER_WORLD;

        let mut nodes = ShaderBuffer::from(vec![PackedSdfNode::default()]);
        nodes.buffer_description.label = Some("nannou_sdf_nodes");
        nodes.buffer_description.usage |= BufferUsages::STORAGE | BufferUsages::COPY_DST;
        nodes.asset_usage = RenderAssetUsages::RENDER_WORLD;

        let mut stages = ShaderBuffer::from(vec![PackedSdfStage::default()]);
        stages.buffer_description.label = Some("nannou_sdf_stages");
        stages.buffer_description.usage |= BufferUsages::STORAGE | BufferUsages::COPY_DST;
        stages.asset_usage = RenderAssetUsages::RENDER_WORLD;

        let mut dirty_bricks = ShaderBuffer::from(vec![PackedDirtyBrick::default()]);
        dirty_bricks.buffer_description.label = Some("nannou_sdf_dirty_bricks");
        dirty_bricks.buffer_description.usage |= BufferUsages::STORAGE | BufferUsages::COPY_DST;
        dirty_bricks.asset_usage = RenderAssetUsages::RENDER_WORLD;

        let mut brick_map = ShaderBuffer::from(vec![INVALID_ATLAS_SLOT]);
        brick_map.buffer_description.label = Some("nannou_sdf_brick_map");
        brick_map.buffer_description.usage |= BufferUsages::STORAGE | BufferUsages::COPY_DST;
        brick_map.asset_usage = RenderAssetUsages::RENDER_WORLD;

        let mut brick_meta = ShaderBuffer::from(vec![PackedBrickMeta::default()]);
        brick_meta.buffer_description.label = Some("nannou_sdf_brick_meta");
        brick_meta.buffer_description.usage |= BufferUsages::STORAGE | BufferUsages::COPY_DST;
        brick_meta.asset_usage = RenderAssetUsages::RENDER_WORLD;

        let mut distance_atlas = ShaderBuffer::with_size(4, RenderAssetUsages::RENDER_WORLD);
        distance_atlas.buffer_description.label = Some("nannou_sdf_distance_atlas");
        distance_atlas.buffer_description.usage |= BufferUsages::STORAGE | BufferUsages::COPY_DST;

        let mut color_atlas = ShaderBuffer::with_size(16, RenderAssetUsages::RENDER_WORLD);
        color_atlas.buffer_description.label = Some("nannou_sdf_color_atlas");
        color_atlas.buffer_description.usage |= BufferUsages::STORAGE | BufferUsages::COPY_DST;

        let mut material_atlas = ShaderBuffer::with_size(4, RenderAssetUsages::RENDER_WORLD);
        material_atlas.buffer_description.label = Some("nannou_sdf_material_atlas");
        material_atlas.buffer_description.usage |= BufferUsages::STORAGE | BufferUsages::COPY_DST;

        Self {
            edits: buffers.add(edits),
            nodes: buffers.add(nodes),
            stages: buffers.add(stages),
            dirty_bricks: buffers.add(dirty_bricks),
            brick_map: buffers.add(brick_map),
            brick_meta: buffers.add(brick_meta),
            distance_atlas: buffers.add(distance_atlas),
            color_atlas: buffers.add(color_atlas),
            material_atlas: buffers.add(material_atlas),
            compute: SdfGpuComputeState::default(),
            packed_version: 0,
            cache_version: 0,
            pending_cache_version: 0,
            completed_cache_version: 0,
        }
    }

    fn compute_pending(&self) -> bool {
        self.pending_cache_version != 0 && self.completed_cache_version < self.pending_cache_version
    }
}

#[derive(Clone, Debug, Default)]
pub struct SdfGpuComputeState {
    pub cache_version: u64,
    pub config: PackedSdfCacheConfig,
    pub stage_shape_kinds: Vec<u32>,
    pub dirty_count: u32,
    pub stage_count: u32,
    pub has_content: bool,
    pub resident_count: u32,
    pub atlas_full: bool,
}

impl SdfGpuComputeState {
    fn requires_dispatch(&self) -> bool {
        self.has_content && self.dirty_count > 0 && self.stage_count > 0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, ShaderType)]
pub struct PackedSdfCacheConfig {
    pub bounds_min: Vec4,
    pub bounds_max: Vec4,
    pub brick_dims: UVec4,
    pub atlas: UVec4,
    pub params: Vec4,
}

#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct PackedSdfStage {
    pub data: UVec4,
    pub params: Vec4,
}

impl PackedSdfStage {
    fn from_op(op: SdfOperation, edit_index: u32, shape_kind: u32) -> Self {
        Self {
            data: UVec4::new(op.id(), edit_index, shape_kind, 0),
            params: Vec4::new(op.weight(), 0.0, 0.0, 0.0),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct PackedDirtyBrick {
    pub coord: UVec4,
    pub data: UVec4,
}

impl PackedDirtyBrick {
    fn new(brick: SdfBrick, map_index: u32, atlas_slot: u32) -> Self {
        Self {
            coord: UVec4::new(brick.x as u32, brick.y as u32, brick.z as u32, 0),
            data: UVec4::new(atlas_slot, map_index, 0, 0),
        }
    }

    fn clear(brick: SdfBrick, map_index: u32) -> Self {
        Self {
            coord: UVec4::new(brick.x as u32, brick.y as u32, brick.z as u32, 0),
            data: UVec4::new(INVALID_ATLAS_SLOT, map_index, 0, 0),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct PackedBrickMeta {
    pub data: UVec4,
    pub distances: Vec4,
}

impl PackedBrickMeta {
    fn empty() -> Self {
        Self {
            data: UVec4::new(INVALID_ATLAS_SLOT, 0, 0, 0),
            distances: Vec4::new(f32::INFINITY, f32::NEG_INFINITY, 0.0, 0.0),
        }
    }

    fn from_meta(meta: &BrickMeta) -> Self {
        let mut flags = 0;
        if meta.resident {
            flags |= 1;
        }
        if meta.stale {
            flags |= 2;
        }
        if meta.initialized {
            flags |= 4;
        }
        Self {
            data: UVec4::new(
                meta.atlas_slot.unwrap_or(INVALID_ATLAS_SLOT),
                flags,
                meta.epoch as u32,
                meta.candidate_count,
            ),
            distances: Vec4::new(meta.min_distance, meta.max_distance, 0.0, 0.0),
        }
    }
}

/// Builder returned by [`Sdf::configure`].
pub struct SdfConfigBuilder<'a> {
    sdf: &'a Sdf,
}

impl<'a> SdfConfigBuilder<'a> {
    pub fn bounds(self, bounds: SdfBounds) -> Self {
        self.update(|config| config.bounds = bounds);
        self
    }

    pub fn voxel_size(self, voxel_size: f32) -> Self {
        self.update(|config| config.voxel_size = voxel_size.max(f32::EPSILON));
        self
    }

    pub fn brick_size(self, brick_size: u32) -> Self {
        self.update(|config| config.brick_size = brick_size.max(1));
        self
    }

    pub fn distance_format(self, distance_format: SdfDistanceFormat) -> Self {
        self.update(|config| config.distance_format = distance_format);
        self
    }

    pub fn update_budget(self, update_budget: SdfUpdateBudget) -> Self {
        self.update(|config| config.update_budget = update_budget);
        self
    }

    pub fn narrow_band(self, narrow_band: f32) -> Self {
        self.update(|config| config.narrow_band = narrow_band.max(0.0));
        self
    }

    pub fn atlas_capacity(self, atlas_capacity: u32) -> Self {
        self.update(|config| config.atlas_capacity = atlas_capacity.max(1));
        self
    }

    fn update(&self, f: impl FnOnce(&mut SdfConfig)) {
        let mut scene = self.sdf.scene.write().expect("Sdf scene lock poisoned");
        let old = scene.config.clone();
        f(&mut scene.config);
        if scene.config != old {
            scene.invalidate_all_bricks();
        }
    }
}

/// A transaction recorder. Values built from it commit to the transaction on drop.
pub struct SdfTransaction {
    graph: RefCell<SdfGraph>,
    next_call_order: Arc<Cell<u64>>,
}

impl SdfTransaction {
    fn new() -> Self {
        Self {
            graph: RefCell::new(SdfGraph::default()),
            next_call_order: Arc::new(Cell::new(0)),
        }
    }

    fn child(&self) -> Self {
        Self {
            graph: RefCell::new(SdfGraph::default()),
            next_call_order: self.next_call_order.clone(),
        }
    }

    fn finish(self) -> SdfGraph {
        self.graph.into_inner()
    }

    fn record_edit(&self, op: SdfOperation, mut edit: SdfEdit) {
        if edit.key.is_none() {
            let order = self.next_call_order.get();
            self.next_call_order.set(order + 1);
            edit.identity = SdfIdentity::CallOrder(order);
        } else if let Some(key) = &edit.key {
            edit.identity = SdfIdentity::Key(key.clone());
        }
        self.graph.borrow_mut().items.push(SdfGraphItem {
            op,
            node: SdfNode::Primitive(edit),
        });
    }

    fn record_node(&self, op: SdfOperation, node: SdfNode) {
        self.graph
            .borrow_mut()
            .items
            .push(SdfGraphItem { op, node });
    }

    pub fn sphere(&self) -> SdfBuilder<'_> {
        SdfBuilder::transaction(self, SdfShape::sphere())
    }

    pub fn cuboid(&self) -> SdfBuilder<'_> {
        SdfBuilder::transaction(self, SdfShape::cuboid())
    }

    pub fn rounded_cuboid(&self) -> SdfBuilder<'_> {
        SdfBuilder::transaction(self, SdfShape::rounded_cuboid())
    }

    pub fn capsule(&self) -> SdfBuilder<'_> {
        SdfBuilder::transaction(self, SdfShape::capsule())
    }

    pub fn cylinder(&self) -> SdfBuilder<'_> {
        SdfBuilder::transaction(self, SdfShape::cylinder())
    }

    pub fn cone(&self) -> SdfBuilder<'_> {
        SdfBuilder::transaction(self, SdfShape::cone())
    }

    pub fn torus(&self) -> SdfBuilder<'_> {
        SdfBuilder::transaction(self, SdfShape::torus())
    }

    pub fn ellipsoid(&self) -> SdfBuilder<'_> {
        SdfBuilder::transaction(self, SdfShape::ellipsoid())
    }

    pub fn plane(&self) -> SdfBuilder<'_> {
        SdfBuilder::transaction(self, SdfShape::plane())
    }

    pub fn union(&self, f: impl FnOnce(&SdfTransaction)) {
        self.scoped(SdfOperation::Union, f);
    }

    pub fn subtract(&self, f: impl FnOnce(&SdfTransaction)) {
        self.scoped(SdfOperation::Subtract, f);
    }

    pub fn intersect(&self, f: impl FnOnce(&SdfTransaction)) {
        self.scoped(SdfOperation::Intersect, f);
    }

    pub fn smooth_union(&self, k: f32, f: impl FnOnce(&SdfTransaction)) {
        self.scoped(SdfOperation::SmoothUnion(k), f);
    }

    pub fn smooth_subtract(&self, k: f32, f: impl FnOnce(&SdfTransaction)) {
        self.scoped(SdfOperation::SmoothSubtract(k), f);
    }

    pub fn smooth_intersect(&self, k: f32, f: impl FnOnce(&SdfTransaction)) {
        self.scoped(SdfOperation::SmoothIntersect(k), f);
    }

    pub fn blend(&self, weight: f32, f: impl FnOnce(&SdfTransaction)) {
        self.scoped(SdfOperation::Blend(weight), f);
    }

    pub fn interpolate(&self, weight: f32, f: impl FnOnce(&SdfTransaction, &SdfTransaction)) {
        let from = self.child();
        let to = self.child();
        f(&from, &to);
        self.record_node(
            SdfOperation::Union,
            SdfNode::Interpolate {
                weight,
                from: Box::new(from.finish()),
                to: Box::new(to.finish()),
            },
        );
    }

    fn scoped(&self, op: SdfOperation, f: impl FnOnce(&SdfTransaction)) {
        let child = self.child();
        f(&child);
        self.record_node(op, SdfNode::Group(child.finish()));
    }
}

/// Shape construction builder.
pub struct SdfBuilder<'a> {
    target: SdfBuilderTarget<'a>,
    edit: Option<SdfEdit>,
    op: SdfOperation,
}

enum SdfBuilderTarget<'a> {
    Direct(&'a Sdf),
    Transaction(&'a SdfTransaction),
}

impl<'a> SdfBuilder<'a> {
    fn direct(sdf: &'a Sdf, shape: SdfShape) -> Self {
        Self {
            target: SdfBuilderTarget::Direct(sdf),
            edit: Some(SdfEdit::new(shape)),
            op: SdfOperation::Union,
        }
    }

    fn transaction(transaction: &'a SdfTransaction, shape: SdfShape) -> Self {
        Self {
            target: SdfBuilderTarget::Transaction(transaction),
            edit: Some(SdfEdit::new(shape)),
            op: SdfOperation::Union,
        }
    }

    /// Commit this builder into the persistent handle layer and return the stable handle.
    pub fn finish_handle(mut self) -> SdfHandle {
        let Some(mut edit) = self.edit.take() else {
            return SdfHandle::INVALID;
        };
        match self.target {
            SdfBuilderTarget::Direct(sdf) => sdf
                .scene
                .write()
                .expect("Sdf scene lock poisoned")
                .insert_handle_edit(edit),
            SdfBuilderTarget::Transaction(_) => {
                edit.identity = SdfIdentity::Detached;
                SdfHandle::INVALID
            }
        }
    }

    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.update(|edit| {
            let key = key.into();
            edit.identity = SdfIdentity::Key(key.clone());
            edit.key = Some(key);
        });
        self
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.update_shape(|shape| match shape {
            SdfShape::Sphere { radius: r } => *r = radius,
            SdfShape::Capsule { radius: r, .. } => *r = radius,
            SdfShape::Cylinder { radius: r, .. } => *r = radius,
            _ => {}
        });
        self
    }

    pub fn w_h_d(mut self, w: f32, h: f32, d: f32) -> Self {
        self.update_shape(|shape| match shape {
            SdfShape::Cuboid { size, .. } => *size = Vec3::new(w, h, d),
            _ => {}
        });
        self
    }

    pub fn roundness(mut self, roundness: f32) -> Self {
        self.update_shape(|shape| match shape {
            SdfShape::Cuboid { roundness: r, .. } => *r = roundness.max(0.0),
            _ => {}
        });
        self.update(|edit| edit.modifiers.roundness = roundness.max(0.0));
        self
    }

    pub fn from_to(mut self, from: Vec3, to: Vec3) -> Self {
        self.update_shape(|shape| match shape {
            SdfShape::Capsule { from: f, to: t, .. } => {
                *f = from;
                *t = to;
            }
            _ => {}
        });
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.update_shape(|shape| match shape {
            SdfShape::Cylinder { height: h, .. } | SdfShape::Cone { height: h, .. } => *h = height,
            _ => {}
        });
        self
    }

    pub fn radius_top(mut self, radius: f32) -> Self {
        self.update_shape(|shape| {
            if let SdfShape::Cone { radius_top, .. } = shape {
                *radius_top = radius;
            }
        });
        self
    }

    pub fn radius_bottom(mut self, radius: f32) -> Self {
        self.update_shape(|shape| {
            if let SdfShape::Cone { radius_bottom, .. } = shape {
                *radius_bottom = radius;
            }
        });
        self
    }

    pub fn major_radius(mut self, radius: f32) -> Self {
        self.update_shape(|shape| {
            if let SdfShape::Torus { major_radius, .. } = shape {
                *major_radius = radius;
            }
        });
        self
    }

    pub fn minor_radius(mut self, radius: f32) -> Self {
        self.update_shape(|shape| {
            if let SdfShape::Torus { minor_radius, .. } = shape {
                *minor_radius = radius;
            }
        });
        self
    }

    pub fn radii(mut self, radii: Vec3) -> Self {
        self.update_shape(|shape| {
            if let SdfShape::Ellipsoid { radii: r } = shape {
                *r = radii.max(Vec3::splat(f32::EPSILON));
            }
        });
        self
    }

    pub fn normal(mut self, normal: Vec3) -> Self {
        self.update_shape(|shape| {
            if let SdfShape::Plane { normal: n, .. } = shape {
                *n = normal.normalize_or_zero();
            }
        });
        self
    }

    pub fn offset(mut self, offset: f32) -> Self {
        self.update_shape(|shape| {
            if let SdfShape::Plane { offset: d, .. } = shape {
                *d = offset;
            }
        });
        self
    }

    pub fn translate(mut self, v: Vec3) -> Self {
        self.update(|edit| edit.transform.translation += v);
        self
    }

    pub fn xyz(mut self, v: Vec3) -> Self {
        self.update(|edit| edit.transform.translation = v);
        self
    }

    pub fn x_y_z(self, x: f32, y: f32, z: f32) -> Self {
        self.xyz(Vec3::new(x, y, z))
    }

    pub fn x(mut self, x: f32) -> Self {
        self.update(|edit| edit.transform.translation.x = x);
        self
    }

    pub fn y(mut self, y: f32) -> Self {
        self.update(|edit| edit.transform.translation.y = y);
        self
    }

    pub fn z(mut self, z: f32) -> Self {
        self.update(|edit| edit.transform.translation.z = z);
        self
    }

    pub fn scale(mut self, s: f32) -> Self {
        self.update(|edit| edit.transform.scale = Vec3::splat(s));
        self
    }

    pub fn scale_axes(mut self, v: Vec3) -> Self {
        self.update(|edit| edit.transform.scale = v);
        self
    }

    pub fn scale_x(mut self, s: f32) -> Self {
        self.update(|edit| edit.transform.scale.x = s);
        self
    }

    pub fn scale_y(mut self, s: f32) -> Self {
        self.update(|edit| edit.transform.scale.y = s);
        self
    }

    pub fn scale_z(mut self, s: f32) -> Self {
        self.update(|edit| edit.transform.scale.z = s);
        self
    }

    pub fn quaternion(mut self, q: Quat) -> Self {
        self.update(|edit| edit.transform.rotation = q);
        self
    }

    pub fn euler(mut self, euler: Vec3) -> Self {
        self.update(|edit| {
            edit.transform.rotation = Quat::from_euler(EulerRot::XYZ, euler.x, euler.y, euler.z);
        });
        self
    }

    pub fn pitch(mut self, pitch: f32) -> Self {
        self.update(|edit| edit.transform.rotation *= Quat::from_rotation_x(pitch));
        self
    }

    pub fn yaw(mut self, yaw: f32) -> Self {
        self.update(|edit| edit.transform.rotation *= Quat::from_rotation_y(yaw));
        self
    }

    pub fn roll(mut self, roll: f32) -> Self {
        self.update(|edit| edit.transform.rotation *= Quat::from_rotation_z(roll));
        self
    }

    pub fn shell(mut self, thickness: f32) -> Self {
        self.update(|edit| edit.modifiers.shell = Some(thickness.abs()));
        self
    }

    pub fn onion(mut self, thickness: f32) -> Self {
        self.update(|edit| edit.modifiers.onion = Some(thickness.abs()));
        self
    }

    pub fn elongate(mut self, amount: Vec3) -> Self {
        self.update(|edit| edit.modifiers.elongate = amount.max(Vec3::ZERO));
        self
    }

    pub fn repeat(mut self, period: Vec3) -> Self {
        self.update(|edit| {
            edit.modifiers.repeat = Some(SdfRepeat {
                period: period.max(Vec3::splat(f32::EPSILON)),
                bounds: None,
            });
        });
        self
    }

    pub fn repeat_bounds(mut self, bounds: SdfBounds) -> Self {
        self.update(|edit| {
            let repeat = edit.modifiers.repeat.get_or_insert(SdfRepeat {
                period: Vec3::ONE,
                bounds: None,
            });
            repeat.bounds = Some(bounds);
        });
        self
    }

    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.update(|edit| edit.color = color.into());
        self
    }

    pub fn material(mut self, material: impl Into<MaterialId>) -> Self {
        self.update(|edit| edit.material = material.into());
        self
    }

    pub fn union(mut self) -> Self {
        self.op = SdfOperation::Union;
        self
    }

    pub fn subtract(mut self) -> Self {
        self.op = SdfOperation::Subtract;
        self
    }

    pub fn intersect(mut self) -> Self {
        self.op = SdfOperation::Intersect;
        self
    }

    pub fn smooth_union(mut self, k: f32) -> Self {
        self.op = SdfOperation::SmoothUnion(k);
        self
    }

    pub fn smooth_subtract(mut self, k: f32) -> Self {
        self.op = SdfOperation::SmoothSubtract(k);
        self
    }

    pub fn smooth_intersect(mut self, k: f32) -> Self {
        self.op = SdfOperation::SmoothIntersect(k);
        self
    }

    pub fn blend(mut self, weight: f32) -> Self {
        self.op = SdfOperation::Blend(weight);
        self
    }

    pub fn interpolate(mut self, weight: f32) -> Self {
        self.op = SdfOperation::Interpolate(weight);
        self
    }

    fn update(&mut self, f: impl FnOnce(&mut SdfEdit)) {
        if let Some(edit) = &mut self.edit {
            f(edit);
        }
    }

    fn update_shape(&mut self, f: impl FnOnce(&mut SdfShape)) {
        self.update(|edit| f(&mut edit.shape));
    }

    fn commit(&mut self) {
        let Some(edit) = self.edit.take() else {
            return;
        };
        match self.target {
            SdfBuilderTarget::Direct(sdf) => {
                sdf.scene
                    .write()
                    .expect("Sdf scene lock poisoned")
                    .insert_handle_edit(edit);
            }
            SdfBuilderTarget::Transaction(transaction) => {
                transaction.record_edit(self.op, edit);
            }
        }
    }
}

impl Drop for SdfBuilder<'_> {
    fn drop(&mut self) {
        self.commit();
    }
}

/// Direct mutator for a handle-layer edit.
pub struct SdfHandleEdit<'a> {
    sdf: &'a Sdf,
    handle: SdfHandle,
}

impl<'a> SdfHandleEdit<'a> {
    pub fn xyz(self, v: Vec3) -> Self {
        self.update(|edit| edit.transform.translation = v);
        self
    }

    pub fn x_y_z(self, x: f32, y: f32, z: f32) -> Self {
        self.xyz(Vec3::new(x, y, z))
    }

    pub fn x(self, x: f32) -> Self {
        self.update(|edit| edit.transform.translation.x = x);
        self
    }

    pub fn y(self, y: f32) -> Self {
        self.update(|edit| edit.transform.translation.y = y);
        self
    }

    pub fn z(self, z: f32) -> Self {
        self.update(|edit| edit.transform.translation.z = z);
        self
    }

    pub fn radius(self, radius: f32) -> Self {
        self.update(|edit| match &mut edit.shape {
            SdfShape::Sphere { radius: r }
            | SdfShape::Capsule { radius: r, .. }
            | SdfShape::Cylinder { radius: r, .. } => *r = radius,
            _ => {}
        });
        self
    }

    pub fn w_h_d(self, w: f32, h: f32, d: f32) -> Self {
        self.update(|edit| {
            if let SdfShape::Cuboid { size, .. } = &mut edit.shape {
                *size = Vec3::new(w, h, d);
            }
        });
        self
    }

    pub fn color(self, color: impl Into<Color>) -> Self {
        let color = color.into();
        self.update(|edit| edit.color = color);
        self
    }

    pub fn material(self, material: impl Into<MaterialId>) -> Self {
        let material = material.into();
        self.update(|edit| edit.material = material);
        self
    }

    fn update(&self, f: impl FnOnce(&mut SdfEdit)) {
        let mut scene = self.sdf.scene.write().expect("Sdf scene lock poisoned");
        scene.update_handle(self.handle, f);
    }
}

/// Finite SDF scene bounds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SdfBounds {
    pub min: Vec3,
    pub max: Vec3,
}

impl SdfBounds {
    pub fn from_min_max(min: Vec3, max: Vec3) -> Self {
        Self {
            min: min.min(max),
            max: min.max(max),
        }
    }

    pub fn from_center_size(center: Vec3, size: Vec3) -> Self {
        let half = size.abs() * 0.5;
        Self::from_min_max(center - half, center + half)
    }

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    pub fn union(self, other: Self) -> Self {
        Self::from_min_max(self.min.min(other.min), self.max.max(other.max))
    }

    pub fn inflate(self, amount: f32) -> Self {
        let amount = Vec3::splat(amount.max(0.0));
        Self::from_min_max(self.min - amount, self.max + amount)
    }

    pub fn intersect(self, other: Self) -> Option<Self> {
        let min = self.min.max(other.min);
        let max = self.max.min(other.max);
        if min.cmpgt(max).any() {
            None
        } else {
            Some(Self { min, max })
        }
    }

    pub fn transform(self, transform: Mat4) -> Self {
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        for x in [self.min.x, self.max.x] {
            for y in [self.min.y, self.max.y] {
                for z in [self.min.z, self.max.z] {
                    let p = transform.transform_point3(Vec3::new(x, y, z));
                    min = min.min(p);
                    max = max.max(p);
                }
            }
        }
        Self::from_min_max(min, max)
    }
}

impl Default for SdfBounds {
    fn default() -> Self {
        Self::from_min_max(Vec3::splat(-256.0), Vec3::splat(256.0))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SdfConfig {
    pub bounds: SdfBounds,
    pub voxel_size: f32,
    pub brick_size: u32,
    pub distance_format: SdfDistanceFormat,
    pub update_budget: SdfUpdateBudget,
    pub narrow_band: f32,
    pub atlas_capacity: u32,
}

impl Default for SdfConfig {
    fn default() -> Self {
        Self {
            bounds: SdfBounds::default(),
            voxel_size: 1.0,
            brick_size: 8,
            distance_format: SdfDistanceFormat::R32Float,
            update_budget: SdfUpdateBudget::Unlimited,
            narrow_band: 4.0,
            atlas_capacity: DEFAULT_ATLAS_CAPACITY,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum SdfDistanceFormat {
    R32Float,
    R16Float,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum SdfUpdateBudget {
    Unlimited,
    MaxBricksPerFrame(u32),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct MaterialId(pub u32);

impl From<u32> for MaterialId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl Default for MaterialId {
    fn default() -> Self {
        Self(0)
    }
}

/// Stable identity for handle-layer edits.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct SdfHandle {
    index: u32,
    generation: u32,
}

impl SdfHandle {
    pub const INVALID: Self = Self {
        index: u32::MAX,
        generation: 0,
    };

    pub fn index(self) -> u32 {
        self.index
    }

    pub fn generation(self) -> u32 {
        self.generation
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SdfShape {
    Sphere {
        radius: f32,
    },
    Cuboid {
        size: Vec3,
        roundness: f32,
    },
    Capsule {
        from: Vec3,
        to: Vec3,
        radius: f32,
    },
    Cylinder {
        radius: f32,
        height: f32,
    },
    Cone {
        radius_top: f32,
        radius_bottom: f32,
        height: f32,
    },
    Torus {
        major_radius: f32,
        minor_radius: f32,
    },
    Ellipsoid {
        radii: Vec3,
    },
    Plane {
        normal: Vec3,
        offset: f32,
    },
}

impl SdfShape {
    fn sphere() -> Self {
        Self::Sphere { radius: 1.0 }
    }

    fn cuboid() -> Self {
        Self::Cuboid {
            size: Vec3::ONE,
            roundness: 0.0,
        }
    }

    fn rounded_cuboid() -> Self {
        Self::Cuboid {
            size: Vec3::ONE,
            roundness: 0.1,
        }
    }

    fn capsule() -> Self {
        Self::Capsule {
            from: Vec3::new(0.0, -0.5, 0.0),
            to: Vec3::new(0.0, 0.5, 0.0),
            radius: 0.5,
        }
    }

    fn cylinder() -> Self {
        Self::Cylinder {
            radius: 0.5,
            height: 1.0,
        }
    }

    fn cone() -> Self {
        Self::Cone {
            radius_top: 0.0,
            radius_bottom: 0.5,
            height: 1.0,
        }
    }

    fn torus() -> Self {
        Self::Torus {
            major_radius: 0.75,
            minor_radius: 0.25,
        }
    }

    fn ellipsoid() -> Self {
        Self::Ellipsoid { radii: Vec3::ONE }
    }

    fn plane() -> Self {
        Self::Plane {
            normal: Vec3::Y,
            offset: 0.0,
        }
    }

    fn kind_id(&self) -> u32 {
        match self {
            Self::Sphere { .. } => 0,
            Self::Cuboid { .. } => 1,
            Self::Capsule { .. } => 2,
            Self::Cylinder { .. } => 3,
            Self::Cone { .. } => 4,
            Self::Torus { .. } => 5,
            Self::Ellipsoid { .. } => 6,
            Self::Plane { .. } => 7,
        }
    }

    fn distance(&self, p: Vec3) -> f32 {
        match *self {
            Self::Sphere { radius } => p.length() - radius,
            Self::Cuboid { size, roundness } => {
                let half = (size.abs() * 0.5 - Vec3::splat(roundness)).max(Vec3::ZERO);
                let q = p.abs() - half;
                q.max(Vec3::ZERO).length() + q.max_element().min(0.0) - roundness
            }
            Self::Capsule { from, to, radius } => {
                let pa = p - from;
                let ba = to - from;
                let h = if ba.length_squared() > 0.0 {
                    (pa.dot(ba) / ba.length_squared()).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                (pa - ba * h).length() - radius
            }
            Self::Cylinder { radius, height } => {
                let d = Vec2::new(Vec2::new(p.x, p.z).length(), p.y).abs()
                    - Vec2::new(radius, height * 0.5);
                d.max(Vec2::ZERO).length() + d.max_element().min(0.0)
            }
            Self::Cone {
                radius_top,
                radius_bottom,
                height,
            } => {
                let half_h = height.abs() * 0.5;
                let y = (p.y + half_h).clamp(0.0, height.abs());
                let t = if height.abs() > 0.0 {
                    y / height.abs()
                } else {
                    0.0
                };
                let radius = radius_bottom + (radius_top - radius_bottom) * t;
                let side = Vec2::new(p.x, p.z).length() - radius;
                let cap = p.y.abs() - half_h;
                Vec2::new(side.max(0.0), cap.max(0.0)).length() + side.max(cap).min(0.0)
            }
            Self::Torus {
                major_radius,
                minor_radius,
            } => {
                Vec2::new(Vec2::new(p.x, p.z).length() - major_radius, p.y).length() - minor_radius
            }
            Self::Ellipsoid { radii } => {
                let r = radii.max(Vec3::splat(f32::EPSILON));
                (p / r).length() - 1.0
            }
            Self::Plane { normal, offset } => p.dot(normal.normalize_or_zero()) - offset,
        }
    }

    fn local_bounds(&self, scene_bounds: SdfBounds) -> SdfBounds {
        match *self {
            Self::Sphere { radius } => {
                SdfBounds::from_min_max(Vec3::splat(-radius), Vec3::splat(radius))
            }
            Self::Cuboid { size, roundness } => {
                let half = size.abs() * 0.5 + Vec3::splat(roundness.max(0.0));
                SdfBounds::from_min_max(-half, half)
            }
            Self::Capsule { from, to, radius } => SdfBounds::from_min_max(
                from.min(to) - Vec3::splat(radius),
                from.max(to) + Vec3::splat(radius),
            ),
            Self::Cylinder { radius, height } => SdfBounds::from_min_max(
                Vec3::new(-radius, -height.abs() * 0.5, -radius),
                Vec3::new(radius, height.abs() * 0.5, radius),
            ),
            Self::Cone {
                radius_top,
                radius_bottom,
                height,
            } => {
                let r = radius_top.abs().max(radius_bottom.abs());
                SdfBounds::from_min_max(
                    Vec3::new(-r, -height.abs() * 0.5, -r),
                    Vec3::new(r, height.abs() * 0.5, r),
                )
            }
            Self::Torus {
                major_radius,
                minor_radius,
            } => {
                let r = major_radius.abs() + minor_radius.abs();
                SdfBounds::from_min_max(
                    Vec3::new(-r, -minor_radius.abs(), -r),
                    Vec3::new(r, minor_radius.abs(), r),
                )
            }
            Self::Ellipsoid { radii } => SdfBounds::from_min_max(-radii.abs(), radii.abs()),
            Self::Plane { .. } => scene_bounds,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SdfTransform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for SdfTransform {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl SdfTransform {
    fn matrix(self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    fn distance_scale(self) -> f32 {
        self.scale.abs().min_element().max(f32::EPSILON)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SdfModifiers {
    pub roundness: f32,
    pub shell: Option<f32>,
    pub onion: Option<f32>,
    pub elongate: Vec3,
    pub repeat: Option<SdfRepeat>,
}

impl Default for SdfModifiers {
    fn default() -> Self {
        Self {
            roundness: 0.0,
            shell: None,
            onion: None,
            elongate: Vec3::ZERO,
            repeat: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SdfRepeat {
    pub period: Vec3,
    pub bounds: Option<SdfBounds>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SdfEdit {
    pub handle: Option<SdfHandle>,
    pub key: Option<String>,
    pub shape: SdfShape,
    pub transform: SdfTransform,
    pub inverse_transform: Mat4,
    pub material: MaterialId,
    pub color: Color,
    pub modifiers: SdfModifiers,
    pub local_aabb: SdfBounds,
    pub world_aabb: SdfBounds,
    pub version: u64,
    identity: SdfIdentity,
}

impl SdfEdit {
    fn new(shape: SdfShape) -> Self {
        let mut edit = Self {
            handle: None,
            key: None,
            shape,
            transform: SdfTransform::default(),
            inverse_transform: Mat4::IDENTITY,
            material: MaterialId::default(),
            color: Color::srgb(1.0, 1.0, 1.0),
            modifiers: SdfModifiers::default(),
            local_aabb: SdfBounds::default(),
            world_aabb: SdfBounds::default(),
            version: 0,
            identity: SdfIdentity::Detached,
        };
        edit.refresh_derived(SdfBounds::default());
        edit
    }

    fn refresh_derived(&mut self, scene_bounds: SdfBounds) {
        let matrix = self.transform.matrix();
        self.inverse_transform = matrix.inverse();
        self.local_aabb = self.shape.local_bounds(scene_bounds);
        let expansion = self.modifier_expansion();
        self.world_aabb = self.local_aabb.transform(matrix).inflate(expansion);
    }

    fn modifier_expansion(&self) -> f32 {
        self.modifiers.roundness
            + self.modifiers.shell.unwrap_or(0.0)
            + self.modifiers.onion.unwrap_or(0.0)
            + self.modifiers.elongate.max_element()
    }

    fn sample(&self, point: Vec3) -> SdfSample {
        let mut local = self.inverse_transform.transform_point3(point);
        if let Some(repeat) = &self.modifiers.repeat {
            local = repeat_point(local, repeat.period);
        }
        if self.modifiers.elongate != Vec3::ZERO {
            let q = local.abs() - self.modifiers.elongate;
            local = q.max(Vec3::ZERO) * local.signum();
        }
        let mut distance = self.shape.distance(local) * self.transform.distance_scale();
        if let Some(thickness) = self.modifiers.onion {
            distance = distance.abs() - thickness;
        }
        if let Some(thickness) = self.modifiers.shell {
            distance = distance.abs() - thickness * 0.5;
        }
        SdfSample {
            distance,
            material: self.material,
            color: self.color,
        }
    }
}

fn repeat_point(p: Vec3, period: Vec3) -> Vec3 {
    p - period * (p / period).round()
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
enum SdfIdentity {
    Key(String),
    CallOrder(u64),
    Handle(SdfHandle),
    Detached,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SdfOperation {
    Union,
    Subtract,
    Intersect,
    SmoothUnion(f32),
    SmoothSubtract(f32),
    SmoothIntersect(f32),
    Blend(f32),
    Interpolate(f32),
}

impl SdfOperation {
    fn id(self) -> u32 {
        match self {
            Self::Union => 0,
            Self::Subtract => 1,
            Self::Intersect => 2,
            Self::SmoothUnion(_) => 3,
            Self::SmoothSubtract(_) => 4,
            Self::SmoothIntersect(_) => 5,
            Self::Blend(_) => 6,
            Self::Interpolate(_) => 7,
        }
    }

    fn weight(self) -> f32 {
        match self {
            Self::SmoothUnion(k) | Self::SmoothSubtract(k) | Self::SmoothIntersect(k) => k,
            Self::Blend(w) | Self::Interpolate(w) => w,
            _ => 0.0,
        }
    }

    fn smooth_radius(self) -> f32 {
        match self {
            Self::SmoothUnion(k) | Self::SmoothSubtract(k) | Self::SmoothIntersect(k) => k.max(0.0),
            _ => 0.0,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SdfGraph {
    pub items: Vec<SdfGraphItem>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SdfGraphItem {
    pub op: SdfOperation,
    pub node: SdfNode,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SdfNode {
    Primitive(SdfEdit),
    Group(SdfGraph),
    Interpolate {
        weight: f32,
        from: Box<SdfGraph>,
        to: Box<SdfGraph>,
    },
}

impl SdfGraph {
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn bounds(&self) -> Option<SdfBounds> {
        self.items
            .iter()
            .filter_map(|item| item.node.bounds())
            .reduce(SdfBounds::union)
    }

    fn sample(&self, point: Vec3) -> Option<SdfSample> {
        let mut acc = None;
        for item in &self.items {
            let Some(rhs) = item.node.sample(point) else {
                continue;
            };
            acc = Some(match acc {
                Some(lhs) => combine_samples(lhs, rhs, item.op),
                None => rhs,
            });
        }
        acc
    }

    fn flatten_edits<'a>(&'a self, out: &mut Vec<&'a SdfEdit>) {
        for item in &self.items {
            item.node.flatten_edits(out);
        }
    }

    fn graph_signature(&self, out: &mut Vec<(SdfIdentity, SdfOperation)>) {
        for item in &self.items {
            item.node.graph_signature(item.op, out);
        }
    }

    fn max_smooth_radius(&self) -> f32 {
        self.items
            .iter()
            .map(|item| item.op.smooth_radius().max(item.node.max_smooth_radius()))
            .fold(0.0, f32::max)
    }
}

impl SdfNode {
    fn bounds(&self) -> Option<SdfBounds> {
        match self {
            Self::Primitive(edit) => Some(edit.world_aabb),
            Self::Group(graph) => graph.bounds(),
            Self::Interpolate { from, to, .. } => match (from.bounds(), to.bounds()) {
                (Some(a), Some(b)) => Some(a.union(b)),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            },
        }
    }

    fn sample(&self, point: Vec3) -> Option<SdfSample> {
        match self {
            Self::Primitive(edit) => Some(edit.sample(point)),
            Self::Group(graph) => graph.sample(point),
            Self::Interpolate { weight, from, to } => {
                let a = from.sample(point)?;
                let b = to.sample(point)?;
                Some(lerp_sample(a, b, *weight))
            }
        }
    }

    fn flatten_edits<'a>(&'a self, out: &mut Vec<&'a SdfEdit>) {
        match self {
            Self::Primitive(edit) => out.push(edit),
            Self::Group(graph) => graph.flatten_edits(out),
            Self::Interpolate { from, to, .. } => {
                from.flatten_edits(out);
                to.flatten_edits(out);
            }
        }
    }

    fn graph_signature(&self, op: SdfOperation, out: &mut Vec<(SdfIdentity, SdfOperation)>) {
        match self {
            Self::Primitive(edit) => out.push((edit.identity.clone(), op)),
            Self::Group(graph) => graph.graph_signature(out),
            Self::Interpolate { from, to, .. } => {
                from.graph_signature(out);
                to.graph_signature(out);
            }
        }
    }

    fn max_smooth_radius(&self) -> f32 {
        match self {
            Self::Primitive(_) => 0.0,
            Self::Group(graph) => graph.max_smooth_radius(),
            Self::Interpolate { from, to, .. } => {
                from.max_smooth_radius().max(to.max_smooth_radius())
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SdfSample {
    pub distance: f32,
    pub material: MaterialId,
    pub color: Color,
}

fn combine_samples(lhs: SdfSample, rhs: SdfSample, op: SdfOperation) -> SdfSample {
    match op {
        SdfOperation::Union => {
            if rhs.distance < lhs.distance {
                rhs
            } else {
                lhs
            }
        }
        SdfOperation::Subtract => {
            let distance = lhs.distance.max(-rhs.distance);
            SdfSample { distance, ..lhs }
        }
        SdfOperation::Intersect => {
            if rhs.distance > lhs.distance {
                rhs
            } else {
                lhs
            }
        }
        SdfOperation::SmoothUnion(k) => smooth_union_sample(lhs, rhs, k),
        SdfOperation::SmoothSubtract(k) => smooth_subtract_sample(lhs, rhs, k),
        SdfOperation::SmoothIntersect(k) => smooth_intersect_sample(lhs, rhs, k),
        SdfOperation::Blend(weight) | SdfOperation::Interpolate(weight) => {
            lerp_sample(lhs, rhs, weight)
        }
    }
}

fn lerp_sample(lhs: SdfSample, rhs: SdfSample, weight: f32) -> SdfSample {
    let t = weight.clamp(0.0, 1.0);
    let distance = lhs.distance + (rhs.distance - lhs.distance) * t;
    if t < 0.5 {
        SdfSample { distance, ..lhs }
    } else {
        SdfSample { distance, ..rhs }
    }
}

fn smooth_union_sample(lhs: SdfSample, rhs: SdfSample, k: f32) -> SdfSample {
    if k <= 0.0 {
        return combine_samples(lhs, rhs, SdfOperation::Union);
    }
    let h = (0.5 + 0.5 * (rhs.distance - lhs.distance) / k).clamp(0.0, 1.0);
    let distance = rhs.distance + (lhs.distance - rhs.distance) * h - k * h * (1.0 - h);
    if h > 0.5 {
        SdfSample { distance, ..lhs }
    } else {
        SdfSample { distance, ..rhs }
    }
}

fn smooth_subtract_sample(lhs: SdfSample, rhs: SdfSample, k: f32) -> SdfSample {
    if k <= 0.0 {
        return combine_samples(lhs, rhs, SdfOperation::Subtract);
    }
    let h = (0.5 - 0.5 * (rhs.distance + lhs.distance) / k).clamp(0.0, 1.0);
    let distance = lhs.distance + (-rhs.distance - lhs.distance) * h + k * h * (1.0 - h);
    SdfSample { distance, ..lhs }
}

fn smooth_intersect_sample(lhs: SdfSample, rhs: SdfSample, k: f32) -> SdfSample {
    if k <= 0.0 {
        return combine_samples(lhs, rhs, SdfOperation::Intersect);
    }
    let h = (0.5 - 0.5 * (rhs.distance - lhs.distance) / k).clamp(0.0, 1.0);
    let distance = rhs.distance + (lhs.distance - rhs.distance) * h + k * h * (1.0 - h);
    if h > 0.5 {
        SdfSample { distance, ..lhs }
    } else {
        SdfSample { distance, ..rhs }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct SdfBrick {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl SdfBrick {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Clone, Debug)]
pub struct BrickMeta {
    pub coord: SdfBrick,
    pub resident: bool,
    pub stale: bool,
    pub initialized: bool,
    pub atlas_slot: Option<u32>,
    pub epoch: u64,
    pub min_distance: f32,
    pub max_distance: f32,
    pub candidate_count: u32,
}

#[derive(Clone, Copy, Debug)]
struct BrickResidencyEstimate {
    brick: SdfBrick,
    min_distance: f32,
    max_distance: f32,
    candidate_count: u32,
}

#[derive(Clone, Debug, Default)]
pub struct BrickCache {
    pub bricks: HashMap<SdfBrick, BrickMeta>,
    pub epoch: u64,
    pub atlas_capacity: u32,
    slot_bricks: Vec<Option<SdfBrick>>,
    free_slots: Vec<u32>,
    atlas_full: bool,
}

impl BrickCache {
    fn ensure_capacity(&mut self, atlas_capacity: u32) -> bool {
        if self.atlas_capacity == atlas_capacity
            && self.slot_bricks.len() == atlas_capacity as usize
        {
            return false;
        }
        if atlas_capacity > self.atlas_capacity
            && self.slot_bricks.len() == self.atlas_capacity as usize
        {
            let old_capacity = self.atlas_capacity;
            self.slot_bricks
                .extend((old_capacity..atlas_capacity).map(|_| None));
            self.free_slots.extend((old_capacity..atlas_capacity).rev());
            self.atlas_capacity = atlas_capacity;
            self.atlas_full = false;
            self.epoch += 1;
            return true;
        }
        self.bricks.clear();
        self.slot_bricks = vec![None; atlas_capacity as usize];
        self.free_slots = (0..atlas_capacity).rev().collect();
        self.atlas_capacity = atlas_capacity;
        self.atlas_full = false;
        self.epoch += 1;
        true
    }

    fn clear(&mut self) {
        self.bricks.clear();
        for slot in &mut self.slot_bricks {
            *slot = None;
        }
        self.free_slots = (0..self.atlas_capacity).rev().collect();
        self.atlas_full = false;
        self.epoch += 1;
    }

    fn allocate(&mut self, brick: SdfBrick) -> Option<u32> {
        if let Some(slot) = self.bricks.get(&brick).and_then(|meta| meta.atlas_slot) {
            return Some(slot);
        }
        let Some(slot) = self.free_slots.pop() else {
            self.atlas_full = true;
            return None;
        };
        if let Some(slot_brick) = self.slot_bricks.get_mut(slot as usize) {
            *slot_brick = Some(brick);
        }
        self.bricks.insert(
            brick,
            BrickMeta {
                coord: brick,
                resident: true,
                stale: true,
                initialized: false,
                atlas_slot: Some(slot),
                epoch: self.epoch,
                min_distance: f32::INFINITY,
                max_distance: f32::NEG_INFINITY,
                candidate_count: 0,
            },
        );
        Some(slot)
    }

    fn free(&mut self, brick: SdfBrick) -> bool {
        let Some(meta) = self.bricks.remove(&brick) else {
            return false;
        };
        let Some(slot) = meta.atlas_slot else {
            return true;
        };
        if let Some(slot_brick) = self.slot_bricks.get_mut(slot as usize) {
            *slot_brick = None;
        }
        self.free_slots.push(slot);
        self.atlas_full = false;
        true
    }

    fn resident_count(&self) -> usize {
        self.bricks
            .values()
            .filter(|meta| meta.resident && meta.atlas_slot.is_some())
            .count()
    }
}

#[derive(Clone, Debug, Default)]
pub struct DirtyBrickSet {
    bricks: HashSet<SdfBrick>,
    all_dirty: bool,
}

impl DirtyBrickSet {
    fn insert(&mut self, brick: SdfBrick) {
        if !self.all_dirty {
            self.bricks.insert(brick);
        }
    }

    fn invalidate_all(&mut self) {
        self.bricks.clear();
        self.all_dirty = true;
    }

    fn count(&self, config: &SdfConfig) -> usize {
        if self.all_dirty {
            brick_dimensions(config)
                .map(|dims| dims.x as usize * dims.y as usize * dims.z as usize)
                .unwrap_or(0)
        } else {
            self.bricks.len()
        }
    }

    fn drain(&mut self, config: &SdfConfig) -> Vec<SdfBrick> {
        if self.all_dirty {
            self.all_dirty = false;
            self.bricks.clear();
            return all_bricks(config);
        }
        self.bricks.drain().collect()
    }

    fn drain_budgeted(&mut self, config: &SdfConfig) -> Vec<SdfBrick> {
        let mut bricks = self.drain(config);
        let budget = match config.update_budget {
            SdfUpdateBudget::Unlimited => return bricks,
            SdfUpdateBudget::MaxBricksPerFrame(max) => max as usize,
        };
        if bricks.len() <= budget {
            return bricks;
        }
        let remaining = bricks.split_off(budget);
        for brick in remaining {
            self.insert(brick);
        }
        bricks
    }
}

#[derive(Clone, Debug)]
pub struct SdfScene {
    pub config: SdfConfig,
    pub transaction_graph: SdfGraph,
    pub previous_transaction_graph: SdfGraph,
    handle_slots: Vec<HandleSlot>,
    handle_order: Vec<SdfHandle>,
    pub dirty_bricks: DirtyBrickSet,
    pub brick_cache: BrickCache,
    brick_map_reset_pending: bool,
    version: u64,
    cache_version: u64,
}

impl Default for SdfScene {
    fn default() -> Self {
        let mut scene = Self {
            config: SdfConfig::default(),
            transaction_graph: SdfGraph::default(),
            previous_transaction_graph: SdfGraph::default(),
            handle_slots: Vec::new(),
            handle_order: Vec::new(),
            dirty_bricks: DirtyBrickSet::default(),
            brick_cache: BrickCache::default(),
            brick_map_reset_pending: true,
            version: 1,
            cache_version: 1,
        };
        scene.invalidate_all_bricks();
        scene
    }
}

#[derive(Clone, Debug)]
struct HandleSlot {
    generation: u32,
    edit: Option<SdfEdit>,
}

impl SdfScene {
    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn config(&self) -> &SdfConfig {
        &self.config
    }

    pub fn transaction_graph(&self) -> &SdfGraph {
        &self.transaction_graph
    }

    pub fn handle_graph(&self) -> SdfGraph {
        let mut graph = SdfGraph::default();
        for handle in &self.handle_order {
            if let Some(edit) = self.handle_edit(*handle) {
                graph.items.push(SdfGraphItem {
                    op: SdfOperation::Union,
                    node: SdfNode::Primitive(edit.clone()),
                });
            }
        }
        graph
    }

    pub fn sample(&self, point: Vec3) -> Option<SdfSample> {
        let tx = self.transaction_graph.sample(point);
        let handles = self.handle_graph().sample(point);
        match (tx, handles) {
            (Some(a), Some(b)) => Some(combine_samples(a, b, SdfOperation::Union)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        }
    }

    pub fn status(&self) -> SdfStatus {
        SdfStatus {
            dirty_bricks: self.dirty_brick_count(),
            resident_bricks: self.brick_cache.resident_count(),
            atlas_capacity: self.brick_cache.atlas_capacity,
            atlas_full: self.brick_cache.atlas_full,
        }
    }

    fn replace_transaction_graph(&mut self, mut graph: SdfGraph) {
        refresh_graph(&mut graph, self.config.bounds);
        let old = self.transaction_graph.clone();
        self.diff_transaction_graphs(&old, &graph);
        self.previous_transaction_graph = old;
        self.transaction_graph = graph;
        self.version += 1;
    }

    fn diff_transaction_graphs(&mut self, old: &SdfGraph, new: &SdfGraph) {
        let old_sig = graph_signature(old);
        let new_sig = graph_signature(new);
        if old_sig != new_sig {
            if let Some(bounds) = old
                .bounds()
                .into_iter()
                .chain(new.bounds())
                .reduce(SdfBounds::union)
            {
                self.mark_aabb_dirty(bounds);
            }
            return;
        }

        let old_edits = edit_map(old);
        let new_edits = edit_map(new);

        for (identity, old_edit) in &old_edits {
            match new_edits.get(identity) {
                Some(new_edit) if *new_edit == *old_edit => {}
                Some(new_edit) => {
                    self.mark_aabb_dirty(old_edit.world_aabb.union(new_edit.world_aabb))
                }
                None => self.mark_aabb_dirty(old_edit.world_aabb),
            }
        }

        for (identity, new_edit) in &new_edits {
            if !old_edits.contains_key(identity) {
                self.mark_aabb_dirty(new_edit.world_aabb);
            }
        }
    }

    fn insert_handle_edit(&mut self, mut edit: SdfEdit) -> SdfHandle {
        let handle = SdfHandle {
            index: self.handle_slots.len() as u32,
            generation: 1,
        };
        edit.handle = Some(handle);
        edit.identity = SdfIdentity::Handle(handle);
        edit.refresh_derived(self.config.bounds);
        self.mark_aabb_dirty(edit.world_aabb);
        self.handle_slots.push(HandleSlot {
            generation: handle.generation,
            edit: Some(edit),
        });
        self.handle_order.push(handle);
        self.version += 1;
        handle
    }

    fn update_handle(&mut self, handle: SdfHandle, f: impl FnOnce(&mut SdfEdit)) -> bool {
        let Some(slot) = self.handle_slots.get_mut(handle.index as usize) else {
            return false;
        };
        if slot.generation != handle.generation {
            return false;
        }
        let Some(edit) = &mut slot.edit else {
            return false;
        };
        let old_bounds = edit.world_aabb;
        f(edit);
        edit.version += 1;
        edit.refresh_derived(self.config.bounds);
        let new_bounds = edit.world_aabb;
        self.mark_aabb_dirty(old_bounds.union(new_bounds));
        self.version += 1;
        true
    }

    fn remove_handle(&mut self, handle: SdfHandle) -> bool {
        let Some(slot) = self.handle_slots.get_mut(handle.index as usize) else {
            return false;
        };
        if slot.generation != handle.generation {
            return false;
        }
        let Some(edit) = slot.edit.take() else {
            return false;
        };
        let dirty_bounds = edit.world_aabb;
        slot.generation = slot.generation.saturating_add(1);
        self.mark_aabb_dirty(dirty_bounds);
        self.version += 1;
        true
    }

    fn handle_edit(&self, handle: SdfHandle) -> Option<&SdfEdit> {
        let slot = self.handle_slots.get(handle.index as usize)?;
        if slot.generation == handle.generation {
            slot.edit.as_ref()
        } else {
            None
        }
    }

    fn mark_aabb_dirty(&mut self, bounds: SdfBounds) {
        let inflated = bounds.inflate(self.config.narrow_band + self.config.voxel_size * 2.0);
        let Some(bounds) = inflated.intersect(self.config.bounds) else {
            return;
        };
        let brick_world = self.config.voxel_size * self.config.brick_size as f32;
        if brick_world <= 0.0 {
            return;
        }
        let rel_min = (bounds.min - self.config.bounds.min) / brick_world;
        let rel_max = (bounds.max - self.config.bounds.min) / brick_world;
        let Some(dims) = brick_dimensions(&self.config) else {
            return;
        };
        let dims = dims.as_ivec3();
        let min = rel_min
            .floor()
            .as_ivec3()
            .clamp(IVec3::ZERO, dims - IVec3::ONE);
        let max = rel_max
            .ceil()
            .as_ivec3()
            .clamp(IVec3::ZERO, dims - IVec3::ONE);
        let count = (max.x - min.x + 1).max(0) as i64
            * (max.y - min.y + 1).max(0) as i64
            * (max.z - min.z + 1).max(0) as i64;
        if count > 1_000_000 {
            self.dirty_bricks.invalidate_all();
            return;
        }
        for z in min.z..=max.z {
            for y in min.y..=max.y {
                for x in min.x..=max.x {
                    self.dirty_bricks.insert(SdfBrick::new(x, y, z));
                }
            }
        }
    }

    fn invalidate_all_bricks(&mut self) {
        self.dirty_bricks.invalidate_all();
        self.brick_cache.clear();
        self.brick_cache.epoch += 1;
        self.brick_map_reset_pending = true;
        self.cache_version += 1;
        self.version += 1;
    }

    fn dirty_brick_count(&self) -> usize {
        self.dirty_bricks.count(&self.config)
    }

    fn take_dirty_bricks(&mut self) -> Vec<SdfBrick> {
        self.dirty_bricks.drain(&self.config)
    }

    fn prepare_gpu_update(&mut self, packed: &PackedSdfScene) -> SdfGpuUpdate {
        let mut cache_changed = self
            .brick_cache
            .ensure_capacity(self.config.atlas_capacity.max(1));
        let brick_dims = brick_dimensions(&self.config).unwrap_or(UVec3::ONE);
        let map_len = brick_dims.x as usize * brick_dims.y as usize * brick_dims.z as usize;
        let has_content = !packed.stages.is_empty();
        let mut reset_brick_map = self.brick_map_reset_pending;
        self.brick_map_reset_pending = false;

        if !has_content {
            if self.brick_cache.resident_count() > 0 {
                self.brick_cache.clear();
                cache_changed = true;
                reset_brick_map = true;
            }
            self.dirty_bricks.bricks.clear();
            self.dirty_bricks.all_dirty = false;
        }

        let mut dirty_bricks = Vec::new();
        let mut queued_dirty_bricks = HashSet::new();
        let mut atlas_full = false;
        if has_content {
            let candidate_bounds = self.candidate_bounds();
            let mut candidate_bricks = Vec::new();
            for brick in self.dirty_bricks.drain_budgeted(&self.config) {
                let Some(map_index) = brick_map_index(&self.config, brick) else {
                    continue;
                };
                match self.brick_candidate_estimate(brick, &candidate_bounds) {
                    Some(candidate) => candidate_bricks.push((candidate, map_index as u32)),
                    None => {
                        if self.brick_cache.free(brick) {
                            cache_changed = true;
                            if !reset_brick_map {
                                dirty_bricks.push(PackedDirtyBrick::clear(brick, map_index as u32));
                            }
                        }
                    }
                }
            }

            let new_candidate_count = candidate_bricks
                .iter()
                .filter(|(candidate, _)| {
                    self.brick_cache
                        .bricks
                        .get(&candidate.brick)
                        .and_then(|meta| meta.atlas_slot)
                        .is_none()
                })
                .count();
            let required_capacity = self
                .brick_cache
                .bricks
                .len()
                .saturating_add(new_candidate_count);
            if required_capacity > self.config.atlas_capacity as usize {
                let target = required_capacity
                    .min(MAX_AUTO_ATLAS_CAPACITY as usize)
                    .max(1) as u32;
                if target > self.config.atlas_capacity {
                    self.config.atlas_capacity = target;
                    cache_changed |= self.brick_cache.ensure_capacity(target);
                }
            }

            for (candidate, map_index) in candidate_bricks {
                let was_allocated = self
                    .brick_cache
                    .bricks
                    .get(&candidate.brick)
                    .and_then(|meta| meta.atlas_slot)
                    .is_some();
                let Some(slot) = self.brick_cache.allocate(candidate.brick) else {
                    atlas_full = true;
                    self.dirty_bricks.insert(candidate.brick);
                    continue;
                };
                let epoch = self.brick_cache.epoch;
                if let Some(meta) = self.brick_cache.bricks.get_mut(&candidate.brick) {
                    meta.resident = true;
                    meta.stale = true;
                    meta.epoch = epoch;
                    meta.min_distance = candidate.min_distance;
                    meta.max_distance = candidate.max_distance;
                    meta.candidate_count = candidate.candidate_count;
                }
                dirty_bricks.push(PackedDirtyBrick::new(candidate.brick, map_index, slot));
                queued_dirty_bricks.insert(candidate.brick);
                cache_changed |= !was_allocated;
            }
        }

        if cache_changed {
            self.cache_version += 1;
        }

        let (brick_map, brick_meta) = self.pack_brick_cache(map_len);
        for brick in &queued_dirty_bricks {
            if let Some(meta) = self.brick_cache.bricks.get_mut(brick) {
                meta.initialized = true;
                meta.stale = false;
            }
        }
        let samples_per_brick = samples_per_brick(&self.config);
        let atlas_samples = self.config.atlas_capacity as u64 * samples_per_brick as u64;
        let compute = SdfGpuComputeState {
            cache_version: self.version,
            config: PackedSdfCacheConfig::from_config(&self.config, brick_dims),
            stage_shape_kinds: packed
                .stages
                .iter()
                .map(|stage| stage.data.z)
                .collect::<Vec<_>>(),
            dirty_count: dirty_bricks.len() as u32,
            stage_count: packed.stages.len() as u32,
            has_content,
            resident_count: self.brick_cache.resident_count() as u32,
            atlas_full: atlas_full || self.brick_cache.atlas_full,
        };

        SdfGpuUpdate {
            dirty_bricks: non_empty_dirty(dirty_bricks),
            brick_map,
            brick_meta,
            distance_bytes: atlas_samples.max(1) * 4,
            color_bytes: atlas_samples.max(1) * 16,
            material_bytes: atlas_samples.max(1) * 4,
            reset_brick_map,
            compute,
        }
    }

    fn brick_candidate_estimate(
        &self,
        brick: SdfBrick,
        candidate_bounds: &[SdfBounds],
    ) -> Option<BrickResidencyEstimate> {
        let brick_bounds = brick_bounds(&self.config, brick)?;
        if !candidate_bounds
            .iter()
            .any(|bounds| bounds.intersect(brick_bounds).is_some())
        {
            return None;
        }
        Some(BrickResidencyEstimate {
            brick,
            min_distance: -self.config.narrow_band,
            max_distance: self.config.narrow_band,
            candidate_count: candidate_bounds.len() as u32,
        })
    }

    fn candidate_bounds(&self) -> Vec<SdfBounds> {
        let expansion =
            self.config.narrow_band + self.config.voxel_size * 2.0 + self.max_smooth_radius();
        let mut bounds = Vec::new();
        collect_candidate_bounds(&self.transaction_graph, expansion, &mut bounds);
        let handle_graph = self.handle_graph();
        collect_candidate_bounds(&handle_graph, expansion, &mut bounds);
        bounds
    }

    fn max_smooth_radius(&self) -> f32 {
        let handle_graph = self.handle_graph();
        self.transaction_graph
            .max_smooth_radius()
            .max(handle_graph.max_smooth_radius())
    }

    fn pack_brick_cache(&self, map_len: usize) -> (Vec<u32>, Vec<PackedBrickMeta>) {
        let mut brick_map = vec![INVALID_ATLAS_SLOT; map_len.max(1)];
        let mut brick_meta = vec![PackedBrickMeta::empty(); map_len.max(1)];
        for (brick, meta) in &self.brick_cache.bricks {
            let Some(index) = brick_map_index(&self.config, *brick) else {
                continue;
            };
            if index >= brick_map.len() {
                continue;
            }
            if meta.resident && meta.initialized {
                brick_map[index] = meta.atlas_slot.unwrap_or(INVALID_ATLAS_SLOT);
            }
            brick_meta[index] = PackedBrickMeta::from_meta(meta);
        }
        (brick_map, brick_meta)
    }

    fn pack_for_gpu(&self) -> PackedSdfScene {
        let mut edits = Vec::new();
        let mut nodes = Vec::new();
        let mut stages = Vec::new();
        pack_graph(&self.transaction_graph, &mut edits, &mut nodes, &mut stages);
        pack_graph(&self.handle_graph(), &mut edits, &mut nodes, &mut stages);

        if edits.is_empty() {
            edits.push(PackedSdfEdit::default());
        }
        if nodes.is_empty() {
            nodes.push(PackedSdfNode::default());
        }
        PackedSdfScene {
            edits,
            nodes,
            stages,
        }
    }
}

fn refresh_graph(graph: &mut SdfGraph, scene_bounds: SdfBounds) {
    for item in &mut graph.items {
        refresh_node(&mut item.node, scene_bounds);
    }
}

fn refresh_node(node: &mut SdfNode, scene_bounds: SdfBounds) {
    match node {
        SdfNode::Primitive(edit) => edit.refresh_derived(scene_bounds),
        SdfNode::Group(graph) => refresh_graph(graph, scene_bounds),
        SdfNode::Interpolate { from, to, .. } => {
            refresh_graph(from, scene_bounds);
            refresh_graph(to, scene_bounds);
        }
    }
}

fn collect_candidate_bounds(graph: &SdfGraph, expansion: f32, out: &mut Vec<SdfBounds>) {
    for item in &graph.items {
        collect_node_candidate_bounds(&item.node, expansion, out);
    }
}

fn collect_node_candidate_bounds(node: &SdfNode, expansion: f32, out: &mut Vec<SdfBounds>) {
    match node {
        SdfNode::Primitive(edit) => out.push(edit.world_aabb.inflate(expansion)),
        SdfNode::Group(graph) => collect_candidate_bounds(graph, expansion, out),
        SdfNode::Interpolate { from, to, .. } => {
            collect_candidate_bounds(from, expansion, out);
            collect_candidate_bounds(to, expansion, out);
        }
    }
}

fn edit_map(graph: &SdfGraph) -> HashMap<SdfIdentity, &SdfEdit> {
    let mut edits = Vec::new();
    graph.flatten_edits(&mut edits);
    edits
        .into_iter()
        .map(|edit| (edit.identity.clone(), edit))
        .collect()
}

fn graph_signature(graph: &SdfGraph) -> u64 {
    let mut signature = Vec::new();
    graph.graph_signature(&mut signature);
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for (identity, op) in signature {
        identity.hash(&mut hasher);
        op.id().hash(&mut hasher);
        op.weight().to_bits().hash(&mut hasher);
    }
    hasher.finish()
}

fn brick_dimensions(config: &SdfConfig) -> Option<UVec3> {
    let brick_world = config.voxel_size * config.brick_size as f32;
    if brick_world <= 0.0 {
        return None;
    }
    let size = config.bounds.size() / brick_world;
    Some(size.ceil().max(Vec3::ONE).as_uvec3())
}

fn all_bricks(config: &SdfConfig) -> Vec<SdfBrick> {
    let Some(dims) = brick_dimensions(config) else {
        return Vec::new();
    };
    let mut bricks = Vec::with_capacity(dims.x as usize * dims.y as usize * dims.z as usize);
    for z in 0..dims.z as i32 {
        for y in 0..dims.y as i32 {
            for x in 0..dims.x as i32 {
                bricks.push(SdfBrick::new(x, y, z));
            }
        }
    }
    bricks
}

fn brick_map_index(config: &SdfConfig, brick: SdfBrick) -> Option<usize> {
    let dims = brick_dimensions(config)?;
    if brick.x < 0
        || brick.y < 0
        || brick.z < 0
        || brick.x >= dims.x as i32
        || brick.y >= dims.y as i32
        || brick.z >= dims.z as i32
    {
        return None;
    }
    Some(
        brick.x as usize
            + brick.y as usize * dims.x as usize
            + brick.z as usize * dims.x as usize * dims.y as usize,
    )
}

fn brick_bounds(config: &SdfConfig, brick: SdfBrick) -> Option<SdfBounds> {
    brick_map_index(config, brick)?;
    let brick_world = config.voxel_size * config.brick_size as f32;
    let min =
        config.bounds.min + Vec3::new(brick.x as f32, brick.y as f32, brick.z as f32) * brick_world;
    Some(SdfBounds::from_min_max(
        min,
        (min + Vec3::splat(brick_world)).min(config.bounds.max),
    ))
}

fn samples_per_axis(config: &SdfConfig) -> u32 {
    config.brick_size.saturating_add(1).max(2)
}

fn samples_per_brick(config: &SdfConfig) -> u32 {
    let axis = samples_per_axis(config);
    axis.saturating_mul(axis).saturating_mul(axis)
}

fn non_empty_dirty(mut dirty: Vec<PackedDirtyBrick>) -> Vec<PackedDirtyBrick> {
    if dirty.is_empty() {
        dirty.push(PackedDirtyBrick::default());
    }
    dirty
}

impl PackedSdfCacheConfig {
    fn from_config(config: &SdfConfig, brick_dims: UVec3) -> Self {
        let sample_axis = samples_per_axis(config);
        let samples = samples_per_brick(config);
        Self {
            bounds_min: config.bounds.min.extend(0.0),
            bounds_max: config.bounds.max.extend(0.0),
            brick_dims: brick_dims.extend(config.brick_size),
            atlas: UVec4::new(config.atlas_capacity, samples, sample_axis, 0),
            params: Vec4::new(config.voxel_size, config.narrow_band, 0.0, 0.0),
        }
    }
}

struct SdfGpuUpdate {
    dirty_bricks: Vec<PackedDirtyBrick>,
    brick_map: Vec<u32>,
    brick_meta: Vec<PackedBrickMeta>,
    distance_bytes: u64,
    color_bytes: u64,
    material_bytes: u64,
    reset_brick_map: bool,
    compute: SdfGpuComputeState,
}

fn pack_graph(
    graph: &SdfGraph,
    edits: &mut Vec<PackedSdfEdit>,
    nodes: &mut Vec<PackedSdfNode>,
    stages: &mut Vec<PackedSdfStage>,
) {
    let mut op_override = None;
    pack_graph_with_override(graph, &mut op_override, edits, nodes, stages);
}

fn pack_graph_with_override(
    graph: &SdfGraph,
    op_override: &mut Option<SdfOperation>,
    edits: &mut Vec<PackedSdfEdit>,
    nodes: &mut Vec<PackedSdfNode>,
    stages: &mut Vec<PackedSdfStage>,
) {
    for item in &graph.items {
        let op = op_override.take().unwrap_or(item.op);
        pack_node(&item.node, op, edits, nodes, stages);
    }
}

fn pack_node(
    node: &SdfNode,
    op: SdfOperation,
    edits: &mut Vec<PackedSdfEdit>,
    nodes: &mut Vec<PackedSdfNode>,
    stages: &mut Vec<PackedSdfStage>,
) {
    match node {
        SdfNode::Primitive(edit) => {
            let edit_index = edits.len() as u32;
            edits.push(PackedSdfEdit::from_edit(edit));
            nodes.push(PackedSdfNode::from_op(op, edit_index));
            stages.push(PackedSdfStage::from_op(
                op,
                edit_index,
                edit.shape.kind_id(),
            ));
        }
        SdfNode::Group(graph) => {
            let mut op_override = Some(op);
            pack_graph_with_override(graph, &mut op_override, edits, nodes, stages);
        }
        SdfNode::Interpolate { weight, from, to } => {
            let start = nodes.len() as u32;
            let mut from_override = Some(op);
            pack_graph_with_override(from, &mut from_override, edits, nodes, stages);
            let mut to_override = Some(SdfOperation::Interpolate(*weight));
            pack_graph_with_override(to, &mut to_override, edits, nodes, stages);
            if let Some(node) = nodes.get_mut(start as usize) {
                node.data0.z = 2;
                node.data1.x = *weight;
            }
        }
    }
}

struct PackedSdfScene {
    edits: Vec<PackedSdfEdit>,
    nodes: Vec<PackedSdfNode>,
    stages: Vec<PackedSdfStage>,
}

#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct PackedSdfEdit {
    pub inv_x: Vec4,
    pub inv_y: Vec4,
    pub inv_z: Vec4,
    pub inv_w: Vec4,
    pub params0: Vec4,
    pub params1: Vec4,
    pub params2: Vec4,
    pub color: Vec4,
    pub data: UVec4,
}

impl PackedSdfEdit {
    fn from_edit(edit: &SdfEdit) -> Self {
        let inv = edit.inverse_transform;
        let color = LinearRgba::from(edit.color).to_vec4();
        let (params0, params1, params2) = match edit.shape {
            SdfShape::Sphere { radius } => {
                (Vec4::new(radius, 0.0, 0.0, 0.0), Vec4::ZERO, Vec4::ZERO)
            }
            SdfShape::Cuboid { size, roundness } => {
                (size.extend(roundness), Vec4::ZERO, Vec4::ZERO)
            }
            SdfShape::Capsule { from, to, radius } => {
                (from.extend(radius), to.extend(0.0), Vec4::ZERO)
            }
            SdfShape::Cylinder { radius, height } => {
                (Vec4::new(radius, height, 0.0, 0.0), Vec4::ZERO, Vec4::ZERO)
            }
            SdfShape::Cone {
                radius_top,
                radius_bottom,
                height,
            } => (
                Vec4::new(radius_top, radius_bottom, height, 0.0),
                Vec4::ZERO,
                Vec4::ZERO,
            ),
            SdfShape::Torus {
                major_radius,
                minor_radius,
            } => (
                Vec4::new(major_radius, minor_radius, 0.0, 0.0),
                Vec4::ZERO,
                Vec4::ZERO,
            ),
            SdfShape::Ellipsoid { radii } => (radii.extend(0.0), Vec4::ZERO, Vec4::ZERO),
            SdfShape::Plane { normal, offset } => (normal.extend(offset), Vec4::ZERO, Vec4::ZERO),
        };
        Self {
            inv_x: inv.x_axis,
            inv_y: inv.y_axis,
            inv_z: inv.z_axis,
            inv_w: inv.w_axis,
            params0,
            params1,
            params2: Vec4::new(
                edit.transform.distance_scale(),
                params2.y,
                params2.z,
                params2.w,
            ),
            color,
            data: UVec4::new(edit.shape.kind_id(), edit.material.0, 0, 0),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct PackedSdfNode {
    pub data0: UVec4,
    pub data1: Vec4,
}

impl PackedSdfNode {
    fn from_op(op: SdfOperation, edit_index: u32) -> Self {
        Self {
            data0: UVec4::new(op.id(), edit_index, 1, 0),
            data1: Vec4::new(op.weight(), 0.0, 0.0, 0.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sdf() -> Sdf {
        Sdf::new(Entity::PLACEHOLDER)
    }

    #[test]
    fn smoothstep_helpers_match_edges() {
        assert_eq!(smoothstep(0.0, 1.0, -1.0), 0.0);
        assert_eq!(smoothstep(0.0, 1.0, 2.0), 1.0);
        assert_eq!(smootherstep(0.0, 1.0, -1.0), 0.0);
        assert_eq!(smootherstep(0.0, 1.0, 2.0), 1.0);
    }

    #[test]
    fn transactions_replace_keyed_edits() {
        let sdf = sdf();
        sdf.take_dirty_bricks();
        sdf.transaction(|s| {
            s.sphere().key("ball").radius(10.0);
        });
        let first_dirty = sdf.dirty_brick_count();
        assert!(first_dirty > 0);
        sdf.take_dirty_bricks();

        sdf.transaction(|s| {
            s.sphere().key("ball").radius(10.0);
        });
        assert_eq!(sdf.dirty_brick_count(), 0);

        sdf.transaction(|s| {
            s.sphere().key("ball").radius(20.0);
        });
        assert!(sdf.dirty_brick_count() > 0);
    }

    #[test]
    fn handle_edits_persist_outside_transactions() {
        let sdf = sdf();
        let handle = sdf.sphere().radius(5.0).finish_handle();
        assert!(sdf.sample(Vec3::ZERO).is_some());

        sdf.transaction(|s| {
            s.cuboid().key("box").w_h_d(2.0, 2.0, 2.0).x(20.0);
        });
        assert!(sdf.sample(Vec3::ZERO).unwrap().distance < 0.0);

        sdf.remove(handle);
        assert!(sdf.sample(Vec3::ZERO).unwrap().distance > 0.0);
    }

    #[test]
    fn scoped_subtract_changes_field() {
        let sdf = sdf();
        sdf.transaction(|s| {
            s.sphere().key("ball").radius(10.0);
            s.subtract(|s| {
                s.sphere().key("cut").radius(4.0);
            });
        });
        let sample = sdf.sample(Vec3::ZERO).unwrap();
        assert!(sample.distance > 0.0);
    }

    #[test]
    fn gpu_pack_emits_ordered_shape_stages() {
        let sdf = sdf();
        sdf.transaction(|s| {
            s.sphere().key("ball").radius(10.0);
            s.smooth_subtract(4.0, |s| {
                s.capsule()
                    .key("cut")
                    .from_to(Vec3::new(-8.0, 0.0, 0.0), Vec3::new(8.0, 0.0, 0.0))
                    .radius(2.0);
            });
        });

        let stages = sdf.with_scene(|scene| scene.pack_for_gpu().stages);
        assert_eq!(stages.len(), 2);
        assert_eq!(stages[0].data.x, SdfOperation::Union.id());
        assert_eq!(stages[0].data.z, SdfShape::sphere().kind_id());
        assert_eq!(stages[1].data.x, SdfOperation::SmoothSubtract(4.0).id());
        assert_eq!(stages[1].data.z, SdfShape::capsule().kind_id());
    }

    #[test]
    fn brick_candidates_use_inflated_edit_bounds() {
        let sdf = sdf();
        sdf.configure()
            .bounds(SdfBounds::from_min_max(
                Vec3::splat(-16.0),
                Vec3::splat(16.0),
            ))
            .voxel_size(1.0)
            .brick_size(8)
            .narrow_band(1.0);
        sdf.transaction(|s| {
            s.sphere().key("ball").radius(4.0);
        });

        sdf.with_scene(|scene| {
            let candidate_bounds = scene.candidate_bounds();
            assert!(
                scene
                    .brick_candidate_estimate(SdfBrick::new(2, 1, 1), &candidate_bounds)
                    .is_some()
            );
            assert!(
                scene
                    .brick_candidate_estimate(SdfBrick::new(3, 1, 1), &candidate_bounds)
                    .is_none()
            );
        });
    }

    #[test]
    fn gpu_prepare_grows_atlas_for_resident_bricks() {
        let sdf = sdf();
        sdf.configure()
            .bounds(SdfBounds::from_min_max(
                Vec3::splat(-24.0),
                Vec3::splat(24.0),
            ))
            .voxel_size(1.0)
            .brick_size(8)
            .atlas_capacity(1);
        sdf.transaction(|s| {
            s.sphere().key("ball").radius(18.0);
        });

        let mut scene = sdf.scene.write().expect("Sdf scene lock poisoned");
        let packed = scene.pack_for_gpu();
        let update = scene.prepare_gpu_update(&packed);

        assert!(scene.config.atlas_capacity > 1);
        assert!(!update.compute.atlas_full);
        assert!(update.compute.dirty_count > 1);
        assert!(scene.brick_cache.atlas_capacity >= update.compute.resident_count);
    }

    #[test]
    fn brick_map_keeps_stale_resident_slots_visible() {
        let mut scene = SdfScene::default();
        scene.config.bounds = SdfBounds::from_min_max(Vec3::ZERO, Vec3::splat(16.0));
        scene.config.voxel_size = 1.0;
        scene.config.brick_size = 8;
        scene.brick_cache.ensure_capacity(2);

        let new_brick = SdfBrick::new(0, 0, 0);
        let stale_brick = SdfBrick::new(1, 0, 0);
        let new_slot = scene.brick_cache.allocate(new_brick).unwrap();
        let stale_slot = scene.brick_cache.allocate(stale_brick).unwrap();
        scene
            .brick_cache
            .bricks
            .get_mut(&stale_brick)
            .unwrap()
            .initialized = true;

        let (brick_map, _) = scene.pack_brick_cache(8);
        assert_eq!(
            brick_map[brick_map_index(&scene.config, new_brick).unwrap()],
            INVALID_ATLAS_SLOT
        );
        assert_eq!(
            brick_map[brick_map_index(&scene.config, stale_brick).unwrap()],
            stale_slot
        );
        assert_ne!(new_slot, stale_slot);
    }
}

impl fmt::Debug for Sdf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sdf").field("window", &self.window).finish()
    }
}
