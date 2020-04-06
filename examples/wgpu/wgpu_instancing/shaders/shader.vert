// NOTE: This shader requires being manually compiled to SPIR-V in order to
// avoid having downstream users require building shaderc and compiling the
// shader themselves. If you update this shader, be sure to also re-compile it
// and update `vert.spv`. You can do so using `glslangValidator` with the
// following command: `glslangValidator -V shader.vert`

#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
//locations 3,4,5, are implicitely taken by `instance_transform`
//since one location can contain at max vec4
layout(location = 2) in mat4 instance_transform;
layout(location = 6) in vec3 instance_color;

layout(location = 0) out vec3 v_normal;
layout(location = 1) out vec3 v_color;

layout(set = 0, binding = 0) uniform Data {
    mat4 world;
    mat4 view;
    mat4 proj;
} uniforms;

void main() {
    mat4 worldview = uniforms.view * uniforms.world * instance_transform;
    v_normal = transpose(inverse(mat3(worldview))) * normal;
    v_color = instance_color;
    gl_Position = uniforms.proj * worldview * vec4(position, 1.0);
}
