#version 450

layout(location = 0) out vec4 f_color;

layout(push_constant) uniform PushConstantData {
    float time;
} pc;

layout(binding = 0) uniform UniformBufferObject {
    vec3 resolution;
} ubo;

void main() {
   f_color = vec4(1.0, 0.0, abs(cos(pc.time)), 1.0);
}
