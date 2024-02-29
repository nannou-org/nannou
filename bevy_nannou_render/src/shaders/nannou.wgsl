#import bevy_render::view::View

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ VERTEX ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ //

struct DrawMeshUniform {
    mode: u32,
};

@group(0) @binding(0) var<uniform> view: View;
@group(1) @binding(0) var<uniform> mesh: DrawMeshUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @location(0) color: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) mode: u32,
    @builtin(position) pos: vec4<f32>,
};

@vertex
fn vertex(
    input: VertexInput,
) -> VertexOutput {
    let out_pos: vec4<f32> = view.view_proj * vec4<f32>(input.position, 1.0);
    return VertexOutput(input.color, input.tex_coords, mesh.mode, out_pos);
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ FRAGMENT ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ //

struct FragmentInput {
    @location(0) color: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) mode: u32,
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
};

@group(2) @binding(0) var text_sampler: sampler;
@group(2) @binding(1) var text: texture_2d<f32>;
@group(3) @binding(0) var texture_sampler: sampler;
@group(3) @binding(1) var texture: texture_2d<f32>;

@fragment
fn fragment(
    input: FragmentInput,
) -> FragmentOutput {
    let texture_color: vec4<f32> = textureSample(texture, texture_sampler, input.tex_coords);
    let text_color: vec4<f32> = textureSample(text, text_sampler, input.tex_coords);
    let text_alpha: f32 = text_color.x;
    var out_color: vec4<f32>;
    if (input.mode == u32(0)) {
        out_color = input.color;
    } else {
        if (input.mode == u32(1)) {
            out_color = texture_color;
        } else {
            if (input.mode == u32(2)) {
                out_color = vec4<f32>(input.color.xyz, input.color.w * text_alpha);
            } else {
                out_color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
            }
        }
    }
    return FragmentOutput(out_color);
}
