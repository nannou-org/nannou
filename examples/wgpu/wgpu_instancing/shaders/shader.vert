// NOTE: This shader requires being manually compiled to SPIR-V in order to
// avoid having downstream users require building shaderc and compiling the
// shader themselves. If you update this shader, be sure to also re-compile it
// and update `vert.spv`. You can do so using `glslangValidator` with the
// following command: `glslangValidator -V shader.vert`

#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;

// This is the instance data for a 4x4 matrix that we reconstruct in main
// Using a mat4 with location = 2 is not supported.
layout(location = 2) in vec4 model_matrix_0;
layout(location = 3) in vec4 model_matrix_1;
layout(location = 4) in vec4 model_matrix_2;
layout(location = 5) in vec4 model_matrix_3;
layout(location = 6) in vec3 instance_color;

layout(location = 0) out vec3 v_normal;
layout(location = 1) out vec3 v_color;

layout(set = 0, binding = 0) uniform Data {
    mat4 world;
    mat4 view;
    mat4 proj;
} uniforms;

// Temporary manual inverse hack until this is solved
// https://github.com/gfx-rs/naga/issues/893
mat3x3 custom_inverse(mat3x3 m) {
    float determinant = determinant(m);
    
    float invdet = 1.0 / determinant;
    
    mat3x3 minv;
    minv[0][0] = (m[1][1] * m[2][2] - m[2][1] * m[1][2]) * invdet;
    minv[0][1] = (m[0][2] * m[2][1] - m[0][1] * m[2][2]) * invdet;
    minv[0][2] = (m[0][1] * m[1][2] - m[0][2] * m[1][1]) * invdet;
    minv[1][0] = (m[1][2] * m[2][0] - m[1][0] * m[2][2]) * invdet;
    minv[1][1] = (m[0][0] * m[2][2] - m[0][2] * m[2][0]) * invdet;
    minv[1][2] = (m[1][0] * m[0][2] - m[0][0] * m[1][2]) * invdet;
    minv[2][0] = (m[1][0] * m[2][1] - m[2][0] * m[1][1]) * invdet;
    minv[2][1] = (m[2][0] * m[0][1] - m[0][0] * m[2][1]) * invdet;
    minv[2][2] = (m[0][0] * m[1][1] - m[1][0] * m[0][1]) * invdet;
    
    return minv;
}

void main() {
    mat4 instance_transform = mat4(
        model_matrix_0,
        model_matrix_1,
        model_matrix_2,
        model_matrix_3
    );
    
    mat4 worldview = uniforms.view * uniforms.world * instance_transform;
    v_normal = transpose(custom_inverse(mat3(worldview))) * normal;
    v_color = instance_color;
    gl_Position = uniforms.proj * worldview * vec4(position, 1.0);
}
