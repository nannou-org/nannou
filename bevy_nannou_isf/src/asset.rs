use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, AsyncReadExt, LoadContext, LoadedAsset};
use bevy::ecs::system::SystemParamItem;
use bevy::prelude::*;
use bevy::render::render_asset::{
    PrepareAssetError, RenderAsset, RenderAssetPlugin, RenderAssetUsages,
};
use bevy::render::render_resource::{AsBindGroup, ShaderRef, ShaderStage};
use bevy::utils::ConditionalSendFuture;
use std::collections::BTreeMap;
use std::io::BufReader;
use thiserror::Error;

// 1. ISF Asset
#[derive(Asset, TypePath, Debug, Clone)]
pub struct Isf {
    pub isf: isf::Isf,
    pub shader: Handle<Shader>,
    pub imported_images: BTreeMap<String, Handle<Image>>,
}

impl Isf {
    pub fn num_images(&self) -> usize {
        let mut image_count = self.imported_images.len();
        let inputs = &self.isf.inputs;
        for input in inputs {
            match input.ty {
                isf::InputType::Image { .. }
                | isf::InputType::Audio(_)
                | isf::InputType::AudioFft(_) => image_count += 1,
                _ => {}
            }
        }
        for pass in &self.isf.passes {
            if let Some(ref target) = pass.target {
                image_count += 1;
            }
        }
        image_count
    }
}

// 2. ISF Asset Loader
#[derive(Default)]
pub struct IsfLoader;

#[derive(Debug, Error)]
pub enum IsfAssetLoaderError {
    #[error("Failed to load ISF file")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse ISF")]
    Parse(#[from] isf::ParseError),
}

impl AssetLoader for IsfLoader {
    type Asset = Isf;
    type Settings = ();
    type Error = IsfAssetLoaderError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut dyn Reader,
        settings: &'a Self::Settings,
        load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let glsl_src = std::str::from_utf8(&bytes).map_err(|_| {
            IsfAssetLoaderError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid UTF-8",
            ))
        })?;

        let isf = isf::parse(glsl_src).map_err(|e| IsfAssetLoaderError::Parse(e))?;
        let glsl = glsl_string_from_isf(&isf);
        let glsl = prefix_isf_glsl_str(&glsl, glsl_src.to_string());
        let shader = Shader::from_glsl(glsl, ShaderStage::Fragment, file!());
        let shader = load_context.add_labeled_asset(String::from("shader"), shader);

        let mut imported_images = BTreeMap::new();
        for (name, import) in &isf.imported {
            let image = load_context.load::<Image>(import.path.clone());
            imported_images.insert(name.clone(), image);
            
        }
        Ok(Isf {
            isf,
            shader,
            imported_images,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["fs", "isf"]
    }
}

// 4. Plugin to register the asset loader and material
pub struct IsfAssetPlugin;

impl Plugin for IsfAssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RenderAssetPlugin::<GpuIsf>::default())
            .init_asset::<Isf>()
            .init_asset_loader::<IsfLoader>();
    }
}

/// Generate the necessary GLSL declarations from the given ISF to be prefixed to the GLSL string
/// from which the ISF was parsed.
///
/// This string should be inserted directly after the version preprocessor.
pub fn glsl_string_from_isf(isf: &isf::Isf) -> String {
    // The normalised coords passed through from the vertex shader.
    let frag_norm_coord_str = "
        layout(location = 0) in vec2 isf_FragNormCoord;
    ";

    // Create the `IsfData` uniform buffer with time, date, etc.
    let isf_data_str = "
        layout(set = 0, binding = 0) uniform IsfData {
            int PASSINDEX;
            int _pad0;
            int _pad1;
            int _pad2;
            vec2 RENDERSIZE;
            int _pad3;
            int _pad4;
            float TIME;
            int _pad5;
            int _pad6;
            int _pad7;
            float TIMEDELTA;
            int _pad8;
            int _pad9;
            int _pad10;
            vec4 DATE;
            int FRAMEINDEX;
            int _pad11;
            int _pad12;
            int _pad13;
        };
    ";

    // Create the `IsfDataInputs` uniform buffer with a field for each event, float, long, bool,
    // point2d and color.
    let isf_data_input_str = match inputs_require_isf_data_input(&isf.inputs) {
        false => None,
        true => {
            let mut isf_data_input_string = "
                layout(set = 1, binding = 0) uniform IsfDataInputs {\n
            "
            .to_string();

            for input in &isf.inputs {
                let (ty_str, padding) = match input.ty {
                    isf::InputType::Event | isf::InputType::Bool(_) => ("bool", 3),
                    isf::InputType::Long(_) => ("int", 3),
                    isf::InputType::Float(_) => ("float", 3),
                    isf::InputType::Point2d(_) => ("vec2", 2),
                    isf::InputType::Color(_) => ("vec4", 0),
                    isf::InputType::Image
                    | isf::InputType::Audio(_)
                    | isf::InputType::AudioFft(_) => continue,
                };
                isf_data_input_string.push_str(&format!("{} {};\n", ty_str, input.name));
                if padding > 0 {
                    for i in 0..padding {
                        isf_data_input_string.push_str(&format!("float _pad{};\n", i));
                    }
                }
            }
            isf_data_input_string.push_str("};\n");
            Some(isf_data_input_string)
        }
    };

    // Create the `img_sampler` binding, used for sampling all input images.
    let img_sampler_str = "
        layout(set = 2, binding = 0) uniform sampler img_sampler;
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
pub const FRAGCOLOR_OUT_DECL_STR: &str = "layout(location = 0) out vec4 FragColor;";

/// Check to see if the `gl_FragColor` variable from old GLSL versions exist and if there's no out
/// variable for it. If so, create a variable for it.
///
/// TODO: This should check that `gl_FragColor` doesn't just exist in a comment or behind a
/// pre-existing macro or something. This was originally just added to makes the tests past.
pub fn glfragcolor_exists_and_no_out(glsl_str: &str) -> bool {
    glsl_str.contains("gl_FragColor") && !glsl_str.contains("out vec4 gl_FragColor")
}

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

pub struct GpuIsf {
    pub isf: Isf,
}

impl RenderAsset for GpuIsf {
    type SourceAsset = Isf;
    type Param = ();

    fn prepare_asset(
        isf: Self::SourceAsset,
        param: &mut SystemParamItem<Self::Param>,
    ) -> Result<Self, PrepareAssetError<Self::SourceAsset>> {
        Ok(GpuIsf { isf })
    }
}
