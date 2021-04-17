//! A crate aimed at making it easy to set up an ISF hot-loading environment with nannou.

pub use crate::pipeline::{IsfPipeline, IsfTime};
use std::path::Path;

mod pipeline;

/// Read the ISF from the shader file at the given path.
///
/// Assumes the given path contains a shader.
pub fn isf_from_shader_path(path: &Path) -> Result<isf::Isf, isf::ParseError> {
    let glsl_string = std::fs::read_to_string(path).expect("failed to read shader to string");
    isf::parse(&glsl_string)
}

/// The associated uniform type for the given input type.
///
/// Returns `None` for variants that are not stored within the uniform buffer.
pub fn input_type_uniform_type(ty: &isf::InputType) -> Option<&'static str> {
    let s = match *ty {
        isf::InputType::Event | isf::InputType::Bool(_) => "bool",
        isf::InputType::Long(_) => "int",
        isf::InputType::Float(_) => "float",
        isf::InputType::Point2d(_) => "vec2",
        isf::InputType::Color(_) => "vec4",
        isf::InputType::Image | isf::InputType::Audio(_) | isf::InputType::AudioFft(_) => {
            return None
        }
    };
    Some(s)
}

/// The size in bytes of the input type when laid out within the uniform struct.
///
/// Returns `None` for variants that are not stored within the uniform buffer.
pub fn input_type_uniform_size_bytes(ty: &isf::InputType) -> Option<usize> {
    let size = match *ty {
        isf::InputType::Color(_) => 4 * 4,
        isf::InputType::Point2d(_) => 2 * 2,
        isf::InputType::Long(_)
        | isf::InputType::Float(_)
        | isf::InputType::Bool(_)
        | isf::InputType::Event { .. } => 1 * 4,
        isf::InputType::Image | isf::InputType::Audio(_) | isf::InputType::AudioFft(_) => {
            return None
        }
    };
    Some(size)
}

/// Generate the necessary GLSL declarations from the given ISF to be prefixed to the GLSL string
/// from which the ISF was parsed.
///
/// This string should be inserted directly after the version preprocessor.
pub fn glsl_string_from_isf(isf: &isf::Isf) -> String {
    // The normalised coords passed through from the vertex shader.
    let frag_norm_coord_str = "
layout(location = 0) in vec2 isf_FragNormCoord;\n\
    ";

    // Create the `IsfData` uniform buffer with time, date, etc.
    let isf_data_str = "
layout(set = 0, binding = 0) uniform IsfData {
    vec4 DATE;
    vec2 RENDERSIZE;
    float TIME;
    float TIMEDELTA;
    int PASSINDEX;
    int FRAMEINDEX;
};\n\
    ";

    // Create the `IsfDataInputs` uniform buffer with a field for each event, float, long, bool,
    // point2d and color.
    let isf_data_input_str = match inputs_require_isf_data_input(&isf.inputs) {
        false => None,
        true => {
            let mut isf_data_input_string = "
layout(set = 1, binding = 0) uniform IsfDataInputs {\n\
            "
            .to_string();

            // Input uniforms should be sorted by name and input type uniform size.

            // Must layout from largest to smallest types to avoid padding holes.
            let b16 = isf
                .inputs
                .iter()
                .filter(|i| input_type_uniform_size_bytes(&i.ty) == Some(16));
            let b8 = isf
                .inputs
                .iter()
                .filter(|i| input_type_uniform_size_bytes(&i.ty) == Some(8));
            let b4 = isf
                .inputs
                .iter()
                .filter(|i| input_type_uniform_size_bytes(&i.ty) == Some(4));
            for input in b16.chain(b8).chain(b4) {
                dbg!(&input.ty);
                let ty_str = match input_type_uniform_type(&input.ty) {
                    Some(s) => s,
                    None => continue,
                };
                isf_data_input_string.push_str(&format!("    {} {};\n", ty_str, input.name));
            }
            isf_data_input_string.push_str("};\n");
            Some(isf_data_input_string)
        }
    };

    // Create the `img_sampler` binding, used for sampling all input images.
    let img_sampler_str = "
layout(set = 2, binding = 0) uniform sampler img_sampler;\n\
    ";

    // Create the textures for the "IMPORTED" images.
    let mut binding = 1;
    let mut imported_textures = vec![];
    for (name, _) in &isf.imported {
        let s = format!(
            "layout(set = 2, binding = {}) uniform texture2D {};\n",
            binding, name
        );
        imported_textures.push(s);
        binding += 1;
    }

    // Create the `texture2D` bindings for image, audio and audioFFT inputs.
    let mut input_textures = vec![];
    for input in &isf.inputs {
        match input.ty {
            isf::InputType::Image | isf::InputType::Audio(_) | isf::InputType::AudioFft(_) => {}
            _ => continue,
        }
        let s = format!(
            "layout(set = 2, binding = {}) uniform texture2D {};\n",
            binding, input.name
        );
        input_textures.push(s);
        binding += 1;
    }

    // Now create textures for the `PASSES`.
    let mut pass_textures = vec![];
    for pass in &isf.passes {
        let target = match pass.target {
            None => continue,
            Some(ref t) => t,
        };
        let s = format!(
            "layout(set = 2, binding = {}) uniform texture2D {};\n",
            binding, target
        );
        pass_textures.push(s);
        binding += 1;
    }

    // Image functions.
    let img_fns_str = "
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

// ISF provided short-hand for retrieving image color.
vec4 IMG_THIS_NORM_PIXEL(texture2D img) {
    return IMG_NORM_PIXEL(img, isf_FragNormCoord);
}

// ISF provided short-hand for retrieving image color.
vec4 IMG_THIS_PIXEL(texture2D img) {
    return IMG_THIS_NORM_PIXEL(img);
}
    ";

    // Combine all the declarations together.
    let mut s = String::new();
    s.push_str(&frag_norm_coord_str);
    s.push_str(&isf_data_str);
    s.extend(isf_data_input_str);
    s.push_str(&img_sampler_str);
    s.extend(imported_textures);
    s.extend(input_textures);
    s.extend(pass_textures);
    s.push_str(&img_fns_str);
    s
}

/// Check to see if the `gl_FragColor` variable from old GLSL versions exist and if there's no out
/// variable for it. If so, create a variable for it.
///
/// TODO: This should check that `gl_FragColor` doesn't just exist in a comment or behind a
/// pre-existing macro or something. This was originally just added to makes the tests past.
pub fn glfragcolor_exists_and_no_out(glsl_str: &str) -> bool {
    glsl_str.contains("gl_FragColor") && !glsl_str.contains("out vec4 gl_FragColor")
}

/// We can't create allow a `gl_FragColor` out, so in the case we have to rename it we create the
/// out decl for it here.
pub const FRAGCOLOR_OUT_DECL_STR: &str = "layout(location = 0) out vec4 FragColor;";

/// Inserts the ISF into the beginning of the shader, returning the resulting glsl source.
pub fn prefix_isf_glsl_str(isf_glsl_str: &str, mut shader_string: String) -> String {
    // Check to see if we need to declare the `gl_FragCoord` output.
    // While we're at it, replace `vv_FragNormCoord` with `isf_FragNormCoord` if necessary.
    let glfragcolor_decl_str = {
        shader_string = shader_string.replace("vv_FragNormCoord", "isf_FragNormCoord");
        if glfragcolor_exists_and_no_out(&shader_string) {
            shader_string = shader_string.replace("gl_FragColor", "FragColor");
            Some(FRAGCOLOR_OUT_DECL_STR.to_string())
        } else {
            None
        }
    };

    // See if the version exists or if it needs to be added.
    enum Version {
        // Where the version currently exists.
        Exists(std::ops::Range<usize>),
        // The version string that needs to be added.
        NeedsToBeAdded(&'static str),
    }
    // TODO: This will break if there's a commented line like `//#version` before the actual
    // version. This caveat is possibly worth the massive speedup we gain by not parsing with
    // `glsl` crate.
    let version = if let Some(start) = shader_string.find("#version ") {
        let version_line = shader_string[start..]
            .lines()
            .next()
            .expect("failed to retrieve verison line");
        let end = start + version_line.len();
        Version::Exists(start..end)
    } else {
        Version::NeedsToBeAdded("#version 450\n")
    };

    // The output string we will fill and return.
    let mut output = String::new();

    // Add the version to the top. Grab the remaining part of the shader string yet to be added.
    let remaining_shader_str = match version {
        Version::NeedsToBeAdded(s) => {
            output.push_str(s);
            &shader_string
        }
        Version::Exists(range) => {
            output.push_str(&format!("{}\n", &shader_string[range.clone()]));
            &shader_string[range.end..]
        }
    };

    output.extend(glfragcolor_decl_str);
    output.push_str(isf_glsl_str);
    output.push_str(remaining_shader_str);
    output
}

// Check whether or not any of the given list of isf inputs would require the `IsfDataInputs`
// uniform.
fn inputs_require_isf_data_input(inputs: &[isf::Input]) -> bool {
    for input in inputs {
        match input.ty {
            isf::InputType::Image | isf::InputType::Audio(_) | isf::InputType::AudioFft(_) => (),
            _ => return true,
        }
    }
    false
}
