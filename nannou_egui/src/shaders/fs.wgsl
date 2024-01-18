@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texture_color = textureSample(image_texture, image_sampler, in.uv);
    // This assumes that texture images are not premultiplied.
    let color = in.color * vec4<f32>(texture_color.rgb * texture_color.a, texture_color.a);

    return color;
}