#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view
#import bevy_pbr::mesh_view_bindings::globals

struct ShaderModel {
    mouse: vec2<f32>,
};

fn hue(a: f32) -> vec3<f32> {
    let t = vec3<f32>(0.0, 1.0, 2.0) + a * 6.3;
    return cos(t) * 0.5 + vec3<f32>(0.5, 0.5, 0.5);
}

// fixes negative mod
fn safe_mod(x: f32, y: f32) -> f32 {
    return x - y * floor(x/y);
}

fn safe_mod_vec2(v: vec2<f32>, y: f32) -> vec2<f32> {
    return vec2<f32>(
        safe_mod(v.x, y),
        safe_mod(v.y, y)
    );
}

@group(2) @binding(0) var<uniform> shader_model: ShaderModel;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let resolution = view.viewport.zw;  // (width, height)
    let time = globals.time;

    let fragCoord = vec2<f32>(mesh.uv.x * resolution.x, (1.0 - mesh.uv.y) * resolution.y);

    let s = 15.0;
    let t = safe_mod(time/60.0, 1.0);

    // Zoom
    var z = shader_model.mouse.y / resolution.y;
    if (z < 0.1) {
        z = 0.0;
    }

    let u = (2.0 * fragCoord - resolution) / resolution.y * s / (1.0 - z);

    let x = round(u.x);
    let y = round(u.y);

    var m = 0.0;
    if (t < 1.0 / 3.0) {
        m = x * x + y * y;
    } else if (t < 2.0 / 3.0) {
        m = x * x - y * y;
    } else {
        m = x * y;
    }

    let g = abs(safe_mod_vec2(u, 1.0) - vec2<f32>(0.5));
    var c = hue(m * t) * min(g.x, g.y);

    let uv_centered = fragCoord / resolution;
    let v = abs(fract(uv_centered + 0.5) - 0.5) * 16.0;
    c *= sqrt(min(v.x, v.y));

    return vec4<f32>(c + c * c, 1.0);
}