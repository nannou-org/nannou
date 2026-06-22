#import bevy_pbr::forward_io::Vertex

const INVALID_ATLAS_SLOT: u32 = 0xffffffffu;
const FAR_DISTANCE: f32 = 1000000.0;

struct PackedSdfCacheConfig {
    bounds_min: vec4<f32>,
    bounds_max: vec4<f32>,
    brick_dims: vec4<u32>,
    atlas: vec4<u32>,
    params: vec4<f32>,
};

struct SdfComputeUniform {
    cache: PackedSdfCacheConfig,
    counts: vec4<u32>,
};

struct SdfRenderUniform {
    bounds_min: vec4<f32>,
    bounds_max: vec4<f32>,
    camera_position: vec4<f32>,
    camera_forward: vec4<f32>,
    camera_right: vec4<f32>,
    camera_up: vec4<f32>,
    lighting_direction: vec4<f32>,
    lighting_color: vec4<f32>,
    render_params: vec4<f32>,
    grid: vec4<u32>,
    atlas: vec4<u32>,
    cache_params: vec4<f32>,
    counts: vec4<u32>,
};

struct PackedSdfEdit {
    inv_x: vec4<f32>,
    inv_y: vec4<f32>,
    inv_z: vec4<f32>,
    inv_w: vec4<f32>,
    params0: vec4<f32>,
    params1: vec4<f32>,
    params2: vec4<f32>,
    color: vec4<f32>,
    data: vec4<u32>,
};

struct PackedSdfStage {
    data: vec4<u32>,
    params: vec4<f32>,
};

struct PackedDirtyBrick {
    coord: vec4<u32>,
    data: vec4<u32>,
};

struct PackedBrickMeta {
    data: vec4<u32>,
    distances: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct CachedSample {
    distance: f32,
    color: vec4<f32>,
    material: u32,
    resident: bool,
    slot: u32,
    brick: vec3<i32>,
};

@group(0) @binding(0) var<uniform> sdf_compute: SdfComputeUniform;
@group(0) @binding(1) var<storage, read> compute_edits: array<PackedSdfEdit>;
@group(0) @binding(2) var<storage, read> compute_stages: array<PackedSdfStage>;
@group(0) @binding(3) var<storage, read> compute_dirty_bricks: array<PackedDirtyBrick>;
@group(0) @binding(4) var<storage, read_write> compute_brick_map: array<u32>;
@group(0) @binding(5) var<storage, read_write> compute_brick_meta: array<PackedBrickMeta>;
@group(0) @binding(6) var<storage, read_write> compute_distance_atlas: array<f32>;
@group(0) @binding(7) var<storage, read_write> compute_color_atlas: array<vec4<f32>>;
@group(0) @binding(8) var<storage, read_write> compute_material_atlas: array<u32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> sdf: SdfRenderUniform;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<storage, read> brick_map: array<u32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var<storage, read> brick_meta: array<PackedBrickMeta>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var<storage, read> distance_atlas: array<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var<storage, read> color_atlas: array<vec4<f32>>;
@group(#{MATERIAL_BIND_GROUP}) @binding(5) var<storage, read> material_atlas: array<u32>;

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(vertex.position.xy, 0.0, 1.0);
    out.uv = vertex.position.xy * 0.5 + vec2<f32>(0.5, 0.5);
    return out;
}

fn transform_point(edit: PackedSdfEdit, p: vec3<f32>) -> vec3<f32> {
    let local = mat4x4<f32>(edit.inv_x, edit.inv_y, edit.inv_z, edit.inv_w) * vec4<f32>(p, 1.0);
    return local.xyz;
}

fn sd_sphere(p: vec3<f32>, radius: f32) -> f32 {
    return length(p) - radius;
}

fn sd_box(p: vec3<f32>, size: vec3<f32>, roundness: f32) -> f32 {
    let half_size = max(abs(size) * 0.5 - vec3<f32>(roundness), vec3<f32>(0.0));
    let q = abs(p) - half_size;
    return length(max(q, vec3<f32>(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0) - roundness;
}

fn sd_capsule(p: vec3<f32>, a: vec3<f32>, b: vec3<f32>, radius: f32) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let denom = max(dot(ba, ba), 0.000001);
    let h = clamp(dot(pa, ba) / denom, 0.0, 1.0);
    return length(pa - ba * h) - radius;
}

fn sd_cylinder(p: vec3<f32>, radius: f32, height: f32) -> f32 {
    let d = abs(vec2<f32>(length(p.xz), p.y)) - vec2<f32>(radius, abs(height) * 0.5);
    return length(max(d, vec2<f32>(0.0))) + min(max(d.x, d.y), 0.0);
}

fn sd_cone(p: vec3<f32>, top: f32, bottom: f32, height: f32) -> f32 {
    let h = max(abs(height), 0.000001);
    let half_h = h * 0.5;
    let y = clamp(p.y + half_h, 0.0, h);
    let t = y / h;
    let radius = bottom + (top - bottom) * t;
    let side = length(p.xz) - radius;
    let cap = abs(p.y) - half_h;
    return length(max(vec2<f32>(side, cap), vec2<f32>(0.0))) + min(max(side, cap), 0.0);
}

fn sd_torus(p: vec3<f32>, major: f32, minor: f32) -> f32 {
    return length(vec2<f32>(length(p.xz) - major, p.y)) - minor;
}

fn sd_ellipsoid(p: vec3<f32>, radii: vec3<f32>) -> f32 {
    let r = max(abs(radii), vec3<f32>(0.000001));
    return (length(p / r) - 1.0) * min(r.x, min(r.y, r.z));
}

fn combine_distance(lhs: f32, lhs_color: vec4<f32>, rhs: f32, rhs_color: vec4<f32>, op: u32, weight: f32) -> CachedSample {
    if (lhs > FAR_DISTANCE * 0.5) {
        return CachedSample(rhs, rhs_color, 0u, true, 0u, vec3<i32>(0));
    }
    if (op == 1u) {
        return CachedSample(max(lhs, -rhs), lhs_color, 0u, true, 0u, vec3<i32>(0));
    }
    if (op == 2u) {
        if (rhs > lhs) {
            return CachedSample(rhs, rhs_color, 0u, true, 0u, vec3<i32>(0));
        }
        return CachedSample(lhs, lhs_color, 0u, true, 0u, vec3<i32>(0));
    }
    if (op == 3u && weight > 0.0) {
        let h = clamp(0.5 + 0.5 * (rhs - lhs) / weight, 0.0, 1.0);
        let d = mix(rhs, lhs, h) - weight * h * (1.0 - h);
        return CachedSample(d, mix(rhs_color, lhs_color, h), 0u, true, 0u, vec3<i32>(0));
    }
    if (op == 4u && weight > 0.0) {
        let h = clamp(0.5 - 0.5 * (rhs + lhs) / weight, 0.0, 1.0);
        let d = mix(lhs, -rhs, h) + weight * h * (1.0 - h);
        return CachedSample(d, lhs_color, 0u, true, 0u, vec3<i32>(0));
    }
    if (op == 5u && weight > 0.0) {
        let h = clamp(0.5 - 0.5 * (rhs - lhs) / weight, 0.0, 1.0);
        let d = mix(rhs, lhs, h) + weight * h * (1.0 - h);
        return CachedSample(d, mix(rhs_color, lhs_color, h), 0u, true, 0u, vec3<i32>(0));
    }
    if (op == 6u || op == 7u) {
        let t = clamp(weight, 0.0, 1.0);
        return CachedSample(mix(lhs, rhs, t), mix(lhs_color, rhs_color, t), 0u, true, 0u, vec3<i32>(0));
    }
    if (rhs < lhs) {
        return CachedSample(rhs, rhs_color, 0u, true, 0u, vec3<i32>(0));
    }
    return CachedSample(lhs, lhs_color, 0u, true, 0u, vec3<i32>(0));
}

fn compute_local_coord(sample_index: u32) -> vec3<u32> {
    let axis = sdf_compute.cache.atlas.z;
    let z = sample_index / (axis * axis);
    let y = (sample_index / axis) % axis;
    let x = sample_index % axis;
    return vec3<u32>(x, y, z);
}

fn compute_atlas_index(slot: u32, local: vec3<u32>) -> u32 {
    let axis = sdf_compute.cache.atlas.z;
    return slot * sdf_compute.cache.atlas.y + local.z * axis * axis + local.y * axis + local.x;
}

fn compute_world_position(brick: vec3<u32>, local: vec3<u32>) -> vec3<f32> {
    let brick_size = sdf_compute.cache.brick_dims.w;
    let grid = vec3<f32>(brick * vec3<u32>(brick_size) + local);
    let p = sdf_compute.cache.bounds_min.xyz + grid * sdf_compute.cache.params.x;
    return clamp(p, sdf_compute.cache.bounds_min.xyz, sdf_compute.cache.bounds_max.xyz);
}

fn apply_stage(sample_index: u32, dirty_index: u32, rhs_distance: f32, edit: PackedSdfEdit, stage: PackedSdfStage) {
    let dirty = compute_dirty_bricks[dirty_index];
    let slot = dirty.data.x;
    if (slot == INVALID_ATLAS_SLOT) {
        return;
    }
    let atlas_index = compute_atlas_index(slot, compute_local_coord(sample_index));
    let lhs = compute_distance_atlas[atlas_index];
    let lhs_color = compute_color_atlas[atlas_index];
    let combined = combine_distance(lhs, lhs_color, rhs_distance, edit.color, stage.data.x, stage.params.x);
    compute_distance_atlas[atlas_index] = combined.distance;
    compute_color_atlas[atlas_index] = combined.color;
    if (combined.distance == rhs_distance || lhs > FAR_DISTANCE * 0.5) {
        compute_material_atlas[atlas_index] = edit.data.y;
    }
}

@compute @workgroup_size(64)
fn sdf_init_bricks(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= sdf_compute.cache.atlas.y || id.y >= sdf_compute.counts.y) {
        return;
    }
    let dirty = compute_dirty_bricks[id.y];
    let slot = dirty.data.x;
    if (slot == INVALID_ATLAS_SLOT) {
        return;
    }
    let atlas_index = compute_atlas_index(slot, compute_local_coord(id.x));
    compute_distance_atlas[atlas_index] = FAR_DISTANCE;
    compute_color_atlas[atlas_index] = vec4<f32>(0.0);
    compute_material_atlas[atlas_index] = 0u;
}

@compute @workgroup_size(64)
fn sdf_eval_sphere(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= sdf_compute.cache.atlas.y || id.y >= sdf_compute.counts.y) { return; }
    let stage = compute_stages[sdf_compute.counts.x];
    let edit = compute_edits[stage.data.y];
    let dirty = compute_dirty_bricks[id.y];
    if (dirty.data.x == INVALID_ATLAS_SLOT) { return; }
    let p = compute_world_position(dirty.coord.xyz, compute_local_coord(id.x));
    let local = transform_point(edit, p);
    apply_stage(id.x, id.y, sd_sphere(local, edit.params0.x) * max(edit.params2.x, 0.000001), edit, stage);
}

@compute @workgroup_size(64)
fn sdf_eval_cuboid(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= sdf_compute.cache.atlas.y || id.y >= sdf_compute.counts.y) { return; }
    let stage = compute_stages[sdf_compute.counts.x];
    let edit = compute_edits[stage.data.y];
    let dirty = compute_dirty_bricks[id.y];
    if (dirty.data.x == INVALID_ATLAS_SLOT) { return; }
    let p = compute_world_position(dirty.coord.xyz, compute_local_coord(id.x));
    let local = transform_point(edit, p);
    apply_stage(id.x, id.y, sd_box(local, edit.params0.xyz, edit.params0.w) * max(edit.params2.x, 0.000001), edit, stage);
}

@compute @workgroup_size(64)
fn sdf_eval_capsule(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= sdf_compute.cache.atlas.y || id.y >= sdf_compute.counts.y) { return; }
    let stage = compute_stages[sdf_compute.counts.x];
    let edit = compute_edits[stage.data.y];
    let dirty = compute_dirty_bricks[id.y];
    if (dirty.data.x == INVALID_ATLAS_SLOT) { return; }
    let p = compute_world_position(dirty.coord.xyz, compute_local_coord(id.x));
    let local = transform_point(edit, p);
    apply_stage(id.x, id.y, sd_capsule(local, edit.params0.xyz, edit.params1.xyz, edit.params0.w) * max(edit.params2.x, 0.000001), edit, stage);
}

@compute @workgroup_size(64)
fn sdf_eval_cylinder(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= sdf_compute.cache.atlas.y || id.y >= sdf_compute.counts.y) { return; }
    let stage = compute_stages[sdf_compute.counts.x];
    let edit = compute_edits[stage.data.y];
    let dirty = compute_dirty_bricks[id.y];
    if (dirty.data.x == INVALID_ATLAS_SLOT) { return; }
    let p = compute_world_position(dirty.coord.xyz, compute_local_coord(id.x));
    let local = transform_point(edit, p);
    apply_stage(id.x, id.y, sd_cylinder(local, edit.params0.x, edit.params0.y) * max(edit.params2.x, 0.000001), edit, stage);
}

@compute @workgroup_size(64)
fn sdf_eval_cone(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= sdf_compute.cache.atlas.y || id.y >= sdf_compute.counts.y) { return; }
    let stage = compute_stages[sdf_compute.counts.x];
    let edit = compute_edits[stage.data.y];
    let dirty = compute_dirty_bricks[id.y];
    if (dirty.data.x == INVALID_ATLAS_SLOT) { return; }
    let p = compute_world_position(dirty.coord.xyz, compute_local_coord(id.x));
    let local = transform_point(edit, p);
    apply_stage(id.x, id.y, sd_cone(local, edit.params0.x, edit.params0.y, edit.params0.z) * max(edit.params2.x, 0.000001), edit, stage);
}

@compute @workgroup_size(64)
fn sdf_eval_torus(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= sdf_compute.cache.atlas.y || id.y >= sdf_compute.counts.y) { return; }
    let stage = compute_stages[sdf_compute.counts.x];
    let edit = compute_edits[stage.data.y];
    let dirty = compute_dirty_bricks[id.y];
    if (dirty.data.x == INVALID_ATLAS_SLOT) { return; }
    let p = compute_world_position(dirty.coord.xyz, compute_local_coord(id.x));
    let local = transform_point(edit, p);
    apply_stage(id.x, id.y, sd_torus(local, edit.params0.x, edit.params0.y) * max(edit.params2.x, 0.000001), edit, stage);
}

@compute @workgroup_size(64)
fn sdf_eval_ellipsoid(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= sdf_compute.cache.atlas.y || id.y >= sdf_compute.counts.y) { return; }
    let stage = compute_stages[sdf_compute.counts.x];
    let edit = compute_edits[stage.data.y];
    let dirty = compute_dirty_bricks[id.y];
    if (dirty.data.x == INVALID_ATLAS_SLOT) { return; }
    let p = compute_world_position(dirty.coord.xyz, compute_local_coord(id.x));
    let local = transform_point(edit, p);
    apply_stage(id.x, id.y, sd_ellipsoid(local, edit.params0.xyz) * max(edit.params2.x, 0.000001), edit, stage);
}

@compute @workgroup_size(64)
fn sdf_eval_plane(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= sdf_compute.cache.atlas.y || id.y >= sdf_compute.counts.y) { return; }
    let stage = compute_stages[sdf_compute.counts.x];
    let edit = compute_edits[stage.data.y];
    let dirty = compute_dirty_bricks[id.y];
    if (dirty.data.x == INVALID_ATLAS_SLOT) { return; }
    let p = compute_world_position(dirty.coord.xyz, compute_local_coord(id.x));
    let local = transform_point(edit, p);
    let n = normalize(edit.params0.xyz);
    apply_stage(id.x, id.y, (dot(local, n) - edit.params0.w) * max(edit.params2.x, 0.000001), edit, stage);
}

@compute @workgroup_size(1)
fn sdf_finalize_bricks(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= sdf_compute.counts.y) {
        return;
    }
    let dirty = compute_dirty_bricks[id.x];
    let slot = dirty.data.x;
    let map_index = dirty.data.y;
    if (slot == INVALID_ATLAS_SLOT) {
        compute_brick_map[map_index] = INVALID_ATLAS_SLOT;
        compute_brick_meta[map_index] = PackedBrickMeta(vec4<u32>(INVALID_ATLAS_SLOT, 0u, 0u, 0u), vec4<f32>(FAR_DISTANCE, -FAR_DISTANCE, 0.0, 0.0));
        return;
    }
    var min_distance = FAR_DISTANCE;
    var max_distance = -FAR_DISTANCE;
    var i = 0u;
    loop {
        if (i >= sdf_compute.cache.atlas.y) {
            break;
        }
        let local = compute_local_coord(i);
        let atlas_index = compute_atlas_index(slot, local);
        let d = compute_distance_atlas[atlas_index];
        min_distance = min(min_distance, d);
        max_distance = max(max_distance, d);
        i = i + 1u;
    }

    let resident = min_distance <= sdf_compute.cache.params.y && max_distance >= -sdf_compute.cache.params.y;
    if (resident) {
        compute_brick_map[map_index] = slot;
        compute_brick_meta[map_index] = PackedBrickMeta(vec4<u32>(slot, 1u, 0u, 0u), vec4<f32>(min_distance, max_distance, 0.0, 0.0));
    } else {
        compute_brick_map[map_index] = INVALID_ATLAS_SLOT;
        compute_brick_meta[map_index] = PackedBrickMeta(vec4<u32>(INVALID_ATLAS_SLOT, 0u, 0u, 0u), vec4<f32>(min_distance, max_distance, 0.0, 0.0));
    }
}

fn render_brick_map_index(brick: vec3<i32>) -> u32 {
    return u32(brick.x) + u32(brick.y) * sdf.grid.x + u32(brick.z) * sdf.grid.x * sdf.grid.y;
}

fn render_brick_valid(brick: vec3<i32>) -> bool {
    return all(brick >= vec3<i32>(0)) &&
        brick.x < i32(sdf.grid.x) &&
        brick.y < i32(sdf.grid.y) &&
        brick.z < i32(sdf.grid.z);
}

fn render_local_atlas_index(slot: u32, local: vec3<u32>) -> u32 {
    let axis = sdf.atlas.z;
    return slot * sdf.atlas.y + local.z * axis * axis + local.y * axis + local.x;
}

fn resident_slot(brick: vec3<i32>) -> u32 {
    if (!render_brick_valid(brick)) {
        return INVALID_ATLAS_SLOT;
    }
    return brick_map[render_brick_map_index(brick)];
}

fn sample_cached(p: vec3<f32>) -> CachedSample {
    let rel = (p - sdf.bounds_min.xyz) / max(sdf.cache_params.x, 0.000001);
    let brick_size = f32(sdf.grid.w);
    let brick = vec3<i32>(floor(rel / brick_size));
    let slot = resident_slot(brick);
    if (slot == INVALID_ATLAS_SLOT) {
        return CachedSample(FAR_DISTANCE, vec4<f32>(0.0), 0u, false, INVALID_ATLAS_SLOT, brick);
    }

    let local = rel - vec3<f32>(brick) * brick_size;
    let base = vec3<u32>(clamp(floor(local), vec3<f32>(0.0), vec3<f32>(brick_size - 1.0)));
    let frac = clamp(local - vec3<f32>(base), vec3<f32>(0.0), vec3<f32>(1.0));

    let c000 = render_local_atlas_index(slot, base + vec3<u32>(0u, 0u, 0u));
    let c100 = render_local_atlas_index(slot, base + vec3<u32>(1u, 0u, 0u));
    let c010 = render_local_atlas_index(slot, base + vec3<u32>(0u, 1u, 0u));
    let c110 = render_local_atlas_index(slot, base + vec3<u32>(1u, 1u, 0u));
    let c001 = render_local_atlas_index(slot, base + vec3<u32>(0u, 0u, 1u));
    let c101 = render_local_atlas_index(slot, base + vec3<u32>(1u, 0u, 1u));
    let c011 = render_local_atlas_index(slot, base + vec3<u32>(0u, 1u, 1u));
    let c111 = render_local_atlas_index(slot, base + vec3<u32>(1u, 1u, 1u));

    let d00 = mix(distance_atlas[c000], distance_atlas[c100], frac.x);
    let d10 = mix(distance_atlas[c010], distance_atlas[c110], frac.x);
    let d01 = mix(distance_atlas[c001], distance_atlas[c101], frac.x);
    let d11 = mix(distance_atlas[c011], distance_atlas[c111], frac.x);
    let d0 = mix(d00, d10, frac.y);
    let d1 = mix(d01, d11, frac.y);

    let col00 = mix(color_atlas[c000], color_atlas[c100], frac.x);
    let col10 = mix(color_atlas[c010], color_atlas[c110], frac.x);
    let col01 = mix(color_atlas[c001], color_atlas[c101], frac.x);
    let col11 = mix(color_atlas[c011], color_atlas[c111], frac.x);
    let col0 = mix(col00, col10, frac.y);
    let col1 = mix(col01, col11, frac.y);

    return CachedSample(mix(d0, d1, frac.z), mix(col0, col1, frac.z), material_atlas[c000], true, slot, brick);
}

fn intersect_bounds(origin: vec3<f32>, dir: vec3<f32>) -> vec2<f32> {
    let inv_dir = 1.0 / dir;
    let t0 = (sdf.bounds_min.xyz - origin) * inv_dir;
    let t1 = (sdf.bounds_max.xyz - origin) * inv_dir;
    let tmin = min(t0, t1);
    let tmax = max(t0, t1);
    return vec2<f32>(max(max(tmin.x, tmin.y), tmin.z), min(min(tmax.x, tmax.y), tmax.z));
}

fn next_brick_exit(origin: vec3<f32>, dir: vec3<f32>, t: f32, brick: vec3<i32>) -> f32 {
    let brick_world = f32(sdf.grid.w) * max(sdf.cache_params.x, 0.000001);
    let brick_min = sdf.bounds_min.xyz + vec3<f32>(brick) * brick_world;
    let brick_max = min(brick_min + vec3<f32>(brick_world), sdf.bounds_max.xyz);
    let p = origin + dir * t;
    var tx = FAR_DISTANCE;
    var ty = FAR_DISTANCE;
    var tz = FAR_DISTANCE;
    if (dir.x > 0.0) { tx = (brick_max.x - p.x) / dir.x; }
    if (dir.x < 0.0) { tx = (brick_min.x - p.x) / dir.x; }
    if (dir.y > 0.0) { ty = (brick_max.y - p.y) / dir.y; }
    if (dir.y < 0.0) { ty = (brick_min.y - p.y) / dir.y; }
    if (dir.z > 0.0) { tz = (brick_max.z - p.z) / dir.z; }
    if (dir.z < 0.0) { tz = (brick_min.z - p.z) / dir.z; }
    return t + max(min(tx, min(ty, tz)), sdf.render_params.y);
}

fn normal_component(plus: CachedSample, minus: CachedSample, center_distance: f32) -> f32 {
    if (plus.resident && minus.resident) {
        return plus.distance - minus.distance;
    }
    if (plus.resident) {
        return plus.distance - center_distance;
    }
    if (minus.resident) {
        return center_distance - minus.distance;
    }
    return 0.0;
}

fn normal_at(p: vec3<f32>, eps: f32) -> vec3<f32> {
    let ex = vec3<f32>(eps, 0.0, 0.0);
    let ey = vec3<f32>(0.0, eps, 0.0);
    let ez = vec3<f32>(0.0, 0.0, eps);
    let center = sample_cached(p);
    let n = vec3<f32>(
        normal_component(sample_cached(p + ex), sample_cached(p - ex), center.distance),
        normal_component(sample_cached(p + ey), sample_cached(p - ey), center.distance),
        normal_component(sample_cached(p + ez), sample_cached(p - ez), center.distance)
    );
    let len = length(n);
    if (len > 0.000001) {
        return n / len;
    }
    return normalize(-sdf.camera_forward.xyz);
}

fn slot_color(slot: u32) -> vec4<f32> {
    let f = f32((slot * 1664525u + 1013904223u) & 255u) / 255.0;
    return vec4<f32>(fract(f + 0.17), fract(f + 0.47), fract(f + 0.73), 1.0);
}

fn viewport_aspect(uv: vec2<f32>) -> f32 {
    let dx = abs(dpdx(uv.x));
    let dy = abs(dpdy(uv.y));
    if (dx > 0.0000001 && dy > 0.0000001) {
        return dy / dx;
    }
    return 1.0;
}

fn configured_aspect(uv: vec2<f32>) -> f32 {
    if (sdf.camera_right.w > 0.0) {
        return sdf.camera_right.w;
    }
    return viewport_aspect(uv);
}

fn refine_hit_t(origin: vec3<f32>, dir: vec3<f32>, outside_t: f32, inside_t: f32) -> f32 {
    var lo_t = outside_t;
    var hi_t = inside_t;
    var i = 0u;
    loop {
        if (i >= 6u) {
            break;
        }
        let mid_t = (lo_t + hi_t) * 0.5;
        let mid_sample = sample_cached(origin + dir * mid_t);
        if (!mid_sample.resident || mid_sample.distance > 0.0) {
            lo_t = mid_t;
        } else {
            hi_t = mid_t;
        }
        i = i + 1u;
    }
    return hi_t;
}

fn shade_hit(p: vec3<f32>, sample: CachedSample, hit_epsilon: f32) -> vec4<f32> {
    if (sdf.counts.z == 3u) {
        return vec4<f32>(vec3<f32>(0.5 + sample.distance * 0.05), 1.0);
    }
    let n = normal_at(p, max(sdf.render_params.z, hit_epsilon));
    if (sdf.counts.z == 4u) {
        return vec4<f32>(n * 0.5 + vec3<f32>(0.5), 1.0);
    }
    let light_dir = normalize(-sdf.lighting_direction.xyz);
    let lambert = max(dot(n, light_dir), 0.0);
    let ambient = clamp(sdf.lighting_direction.w, 0.0, 1.0);
    let diffuse = max(sdf.lighting_color.w, 0.0);
    let light_color = max(sdf.lighting_color.rgb, vec3<f32>(0.0));
    let lit = sample.color.rgb * (ambient + lambert * diffuse) * light_color;
    return vec4<f32>(lit, sample.color.a);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    if (sdf.counts.x == 0u) {
        return vec4<f32>(0.0);
    }

    let screen = in.uv * 2.0 - vec2<f32>(1.0, 1.0);
    let origin = sdf.camera_position.xyz;
    let dir = normalize(sdf.camera_forward.xyz + sdf.camera_right.xyz * (screen.x * configured_aspect(in.uv)) + sdf.camera_up.xyz * screen.y);
    let bounds_hit = intersect_bounds(origin, dir);
    if (bounds_hit.y < max(bounds_hit.x, 0.0)) {
        return vec4<f32>(0.0);
    }

    var t = max(bounds_hit.x, 0.0);
    let max_t = min(bounds_hit.y, sdf.render_params.w);
    let max_steps = u32(sdf.render_params.x);
    let hit_epsilon = sdf.render_params.y;

    var i = 0u;
    var previous_t = t;
    var previous_distance = FAR_DISTANCE;
    var previous_resident = false;
    loop {
        if (i >= max_steps || t > max_t) {
            break;
        }
        let p = origin + dir * t;
        let sample = sample_cached(p);
        let exit_t = next_brick_exit(origin, dir, t, sample.brick);
        if (!sample.resident) {
            t = min(exit_t + hit_epsilon, max_t + hit_epsilon);
            previous_resident = false;
            i = i + 1u;
            continue;
        }
        if (sdf.counts.z == 2u) {
            return slot_color(sample.slot);
        }
        if (abs(sample.distance) <= hit_epsilon) {
            return shade_hit(p, sample, hit_epsilon);
        }
        if (previous_resident && previous_distance > 0.0 && sample.distance < 0.0) {
            let hit_t = refine_hit_t(origin, dir, previous_t, t);
            let hit_p = origin + dir * hit_t;
            let hit_sample = sample_cached(hit_p);
            if (hit_sample.resident) {
                return shade_hit(hit_p, hit_sample, hit_epsilon);
            }
            return shade_hit(p, sample, hit_epsilon);
        }
        let step = min(
            max(abs(sample.distance), hit_epsilon),
            max(exit_t - t, hit_epsilon)
        );
        previous_t = t;
        previous_distance = sample.distance;
        previous_resident = true;
        t = t + step;
        i = i + 1u;
    }

    if (sdf.counts.z == 1u) {
        return vec4<f32>(0.15, 0.05, 0.02, 0.35);
    }
    return vec4<f32>(0.0);
}
