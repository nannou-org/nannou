#version 450

layout(location = 0) out vec4 f_color;
layout(location = 0) in vec2 tex_coords;

layout(push_constant) uniform PushConstantData {
    float time;
    float time_delta;
    uint frame;
    uint frame_rate;
} pc;

layout(set = 0, binding = 0) uniform UniformBufferObject {
    vec3 resolution;
} ubo;

layout(set = 0, binding = 1) uniform sampler2D channel1;
layout(set = 0, binding = 2) uniform sampler2D channel2;
layout(set = 0, binding = 3) uniform sampler2D channel3;
layout(set = 0, binding = 4) uniform sampler2D channel4;

void main() {
  vec4 t = texture(channel4, tex_coords);
  f_color = vec4(t.xy, abs(cos(pc.time)), 1.0);
}
