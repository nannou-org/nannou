// A reference of what the ISF generated GLSL code should look like.

#version 450

//----- GENERATION BEGINS -----//

// The normalized coordinates of the current fragment.
// (0, 0) is bottom left. (1, 1) is top right.
layout(location = 0) in vec2 isf_FragNormCoord;

// Uniform data updated once per frame. These are accessible in all ISF shaders.
layout(set = 0, binding = 0) uniform IsfData {
    int PASSINDEX;
    vec2 RENDERSIZE;
    float TIME;
    float TIMEDELTA;
    vec4 DATE;
    int FRAMEINDEX;
};

// Uniform data for a set of the ISF `INPUTS`.
// Only generated in the case that one or more inputs exist.
// Only generated for the types contained below.
layout(set = 1, binding = 0) uniform IsfDataInputs {
    bool my_event;
    int my_long;
    float my_float;
    bool my_bool;
    vec2 my_point2d;
    vec4 my_color;
};

// TODO image, audio, audioFFT input uniforms?

// The sampler shared between all images.
layout(set = 2, binding = 0) uniform sampler img_sampler;

// The following are not to be used directly, but rather via the generated macros.
// All `INPUT`s of type image, audio or audioFFT (not necessarily in that order).
layout(set = 2, binding = 1) uniform texture2D img_a;
layout(set = 2, binding = 2) uniform texture2D img_b;
layout(set = 2, binding = 3) uniform texture2D audio_a;
layout(set = 2, binding = 4) uniform texture2D audio_b;
layout(set = 2, binding = 5) uniform texture2D audio_fft_a;
layout(set = 2, binding = 6) uniform texture2D audio_fft_b;

// Textures for the `PASSES`. Names derived from `TARGET` field.
layout(set = 2, binding = 7) uniform texture2D img_a;
layout(set = 2, binding = 8) uniform texture2D img_b;

// ISF provided short-hand for retrieving image size.
ivec2 IMG_SIZE(texture2D img) {
    return textureSize(sampler2D(img, img_sampler), 0);
}

// ISF provided short-hand for retrieving image color.
vec4 IMG_NORM_PIXEL(texture2D img, vec2 norm_px_coord) {
    return texture(sampler2D(img, img_sampler), norm_px_coord);
}

// ISF provided short-hand for retrieving image color.
vec4 IMG_PIXEL(texture2D img, vec2 px_coord) {
    ivec2 s = IMG_SIZE(img);
    vec2 norm_px_coord = vec2(px_coord.x / float(s.x), px_coord.y / float(s.y));
    return IMG_NORM_PIXEL(img, px_coord);
}

// The following two functions don't appear in the spec, but appear in a lot of the tests...

// ISF provided short-hand for retrieving image color.
vec4 IMG_THIS_NORM_PIXEL(texture2D img) {
    return IMG_NORM_PIXEL(img, isf_FragNormCoord);
}

// ISF provided short-hand for retrieving image color.
vec4 IMG_THIS_PIXEL(texture2D img) {
    return IMG_THIS_NORM_PIXEL(img);
}

//----- GENERATION COMPLETE -----//

void main() {
    ivec2 s = IMG_SIZE(img_a);
    vec4 a = IMG_NORM_PIXEL(img_b, vec2(0.25, 0.3));
    vec4 b = IMG_PIXEL(img_a, vec2(float(s.x) * 0.5, float(s.y) * 0.5));
    vec4 c = IMG_THIS_PIXEL(img_b);
}
