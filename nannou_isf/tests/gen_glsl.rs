/// Tests the following for each test file:
///
/// - Reads file to string.
/// - Parse file for `Isf`.
/// - Generate GLSL for `Isf`.
/// - Merge generated GLSL with original GLSL.
/// - Attempt to compile merged GLSL to SPIR-V.
///
/// Run with `cargo test -- --nocapture` to print result to command line.
#[test]
fn test_gen_glsl() {
    let test_files_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files");
    assert!(test_files_path.exists());
    assert!(test_files_path.is_dir());
    let mut successes = vec![];
    let mut failures = vec![];
    for entry in std::fs::read_dir(test_files_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        let shader_ty = match hotglsl::ext_to_shader_ty(ext) {
            None => continue,
            Some(ty) => ty,
        };
        if should_ignore(&path) {
            continue;
        }
        let old_glsl_string = std::fs::read_to_string(&path).unwrap();
        println!("Parsing {} for `Isf`...", path.display());
        let isf = match isf::parse(&old_glsl_string) {
            // Ignore non-ISF vertex shaders.
            Err(isf::ParseError::MissingTopComment) if ext == "vs" => continue,
            Err(err) => panic!("err while parsing {}: {}", path.display(), err),
            Ok(isf) => isf,
        };

        println!("  Generating GLSL string...");
        let isf_glsl_string = nannou_isf::glsl_string_from_isf(&isf);
        println!("  Merging GLSL strings...");
        let new_glsl_string = nannou_isf::prefix_isf_glsl_str(&isf_glsl_string, old_glsl_string);

        println!("  Compiling GLSL to SPIR-V...");
        match hotglsl::compile_str(&new_glsl_string, shader_ty) {
            Ok(_) => successes.push(path),
            Err(err) => {
                eprintln!("{}", err);
                failures.push((path, err, new_glsl_string));
            }
        }
    }

    successes.sort();
    failures.sort_by_key(|elem| elem.0.clone());

    if !successes.is_empty() {
        println!("Successes:");
        for path in &successes {
            println!("  {}", path.display());
        }
    }

    if !failures.is_empty() {
        println!("Failures:");
        for (path, err, _src) in &failures {
            println!("  {}\n{}\n", path.display(), err);
        }
    }

    println!("{} succeeded, {} failed", successes.len(), failures.len());
    assert!(failures.is_empty());
}

// Shaders known to trigger bugs in `glsl` parser crate.
const GLSL_PARSER_BUGS: &[&str] = &["Film Burn.fs"];

// We only support GLSL 450 shaders.
const GLSL_450_INCOMPATIBLE: &[&str] = &[
    // The following don't define the `location` for inputs from vertex shader.
    "Lens Flare.fs",
    "Color Blowout.fs",
    "FastMosh.fs",
    "Optical Flow Distort.fs",
    "Circular Screen.fs",
    "Edge Trace.fs",
    "Rotate.fs",
    "Neon.fs",
    "Smudged Lens.fs",
    "Glow-Fast.fs",
    "Line Screen.fs",
    "Fast Blur.fs",
    "Bloom.fs",
    "Life.fs",
    "Dot Screen.fs",
    "Sketch.fs",
    "Gloom.fs",
    "Mirror Edge.fs",
    "Saturation Bleed.fs",
    "v002 Dilate.fs",
    "Sharpen Luminance.fs",
    "Unsharp Mask.fs",
    "Edge Blur.fs",
    "Sorting Smear.fs",
    "Edges.fs",
    "Emboss.fs",
    "Auto Levels.fs",
    "v002 Erode.fs",
    "v002 Light Leak.fs",
    "Sharpen RGB.fs",
    "Glow.fs",
    "Edge Distort.fs",
    "City Lights.fs",
    "Multi Pass Gaussian Blur.fs",
    "Soft Blur.fs",
    "Thermal Camera.fs",
    // Unable to parse the `#ENDIF` on these.
    "Multi-Pixellate.fs",
    "Circle Splash Distortion.fs",
    "Triangles.fs",
    "Cubic Warp.fs",
    "Pixellate.fs",
    "Grid Warp.fs",
    "Ripples.fs",
    "Noise Pixellate.fs",
    "Bump Distortion.fs",
    // Forward declarations. These don't seem to be supported in glsl 450?
    "Chroma Desaturation Mask.fs",
    "Chroma Mask.fs",
    "Color Replacement.fs",
    // These unnecessarily re-declare the `round` function.
    "VU Meter.fs",
    "Etch-a-Sketch.fs",
    // These have what seems to be invalid for loop syntax?
    // e.g. `for (i = 0(i)<(3); ++(i)) {`
    // Error: `function call, method, or subroutine call expected`
    "RGB Halftone-lookaround.fs",
    "CMYK Halftone-Lookaround.fs",
];

// Tests that are known to fail but there's no clear solution explained by the spec.
const KNOWN_BUGS: &[&str] = &[
    // NOTE: Seems like the following two are using some implicitly declared uniforms associated
    // with their input images? Is this meant to be a more efficient alternative to `IMG_SIZE`?
    // This uses undeclared `_inputImage_imgRect`.
    "Radial Replicate.fs",
    // This uses undeclared `_maskImage_imgRect`.
    "Trail Mask.fs",
];

fn should_ignore(path: &std::path::Path) -> bool {
    let file_name = match path.file_name().and_then(|s| s.to_str()) {
        Some(name) => name,
        None => return true,
    };
    GLSL_PARSER_BUGS.contains(&file_name)
        || GLSL_450_INCOMPATIBLE.contains(&file_name)
        || KNOWN_BUGS.contains(&file_name)
}
