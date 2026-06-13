//! A small tool that bumps every published nannou crate to a new version.
//!
//! All `nannou*` crates share a single version and are released in lockstep, so
//! this only needs to edit the workspace manifest:
//!
//! - Set `[workspace.package].version`, which every crate inherits via
//!   `version.workspace = true`.
//! - Set the `version` of each internal `nannou*` entry in
//!   `[workspace.dependencies]`, i.e. the requirement dependent crates publish
//!   with.
//!
//! Usage: `cargo run --bin set_version -- "0.42.0"`

fn main() {
    // Retrieve the specified version from the CLI args.
    let desired_version_string = std::env::args()
        .nth(1)
        .expect("expected one argument with the desired version, e.g. \"0.42.0\"");
    let desired_version = semver::Version::parse(&desired_version_string)
        .expect("failed to parse the specified version as valid semver, e.g. \"0.42.0\"");

    // Locate and read the workspace manifest (two directories up from this crate).
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_manifest_path = std::path::Path::new(manifest_dir)
        .parent()
        .unwrap() // nannou/scripts
        .parent()
        .unwrap() // nannou
        .join("Cargo.toml");
    let workspace_manifest_string = std::fs::read_to_string(&workspace_manifest_path)
        .expect("failed to read the workspace manifest");
    let mut workspace_toml = workspace_manifest_string
        .parse::<toml_edit::DocumentMut>()
        .expect("failed to parse workspace manifest as toml");

    let workspace = workspace_toml["workspace"]
        .as_table_mut()
        .expect("no table 'workspace'");

    // Set the shared package version that all `nannou*` crates inherit.
    let version = workspace["package"]
        .as_table_mut()
        .expect("no table 'workspace.package'")["version"]
        .as_value_mut()
        .expect("no value 'workspace.package.version'");
    set_version(version, &desired_version);

    // Set the version requirement of each internal `nannou*` dependency.
    let deps = workspace["dependencies"]
        .as_table_mut()
        .expect("no table 'workspace.dependencies'");
    let nannou_deps: Vec<_> = deps
        .iter()
        .map(|(name, _)| name.to_string())
        .filter(|name| is_nannou_crate(name))
        .collect();
    for dep in nannou_deps {
        let version = deps[&dep]
            .as_inline_table_mut()
            .expect("internal nannou dependency should be an inline table")
            .get_mut("version")
            .expect("internal nannou dependency should have a 'version'");
        set_version(version, &desired_version);
    }

    safe_file_save(
        &workspace_manifest_path,
        workspace_toml.to_string().as_bytes(),
    )
    .expect("failed to write the edited workspace manifest");

    println!(
        "Successfully set all nannou crates to version {}!",
        desired_version
    );
}

/// Overwrite a toml version value with `desired_version`.
fn set_version(value: &mut toml_edit::Value, desired_version: &semver::Version) {
    *value = format!("{}", desired_version).into();
}

/// Whether the given crate name is one of the nannou crates sharing the version.
fn is_nannou_crate(name: &str) -> bool {
    name.contains("nannou")
}

/// Saves the file to a temporary file before removing the original to reduce the chance of losing
/// data in the case that something goes wrong during saving.
pub fn safe_file_save(path: &std::path::Path, content: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let temp_path = path.with_extension("tmp");

    // If the temp file exists, remove it.
    if temp_path.exists() {
        std::fs::remove_file(&temp_path)?;
    }

    // Write the temp file.
    let file = std::fs::File::create(&temp_path)?;
    let mut buffered = std::io::BufWriter::new(file);
    buffered.write_all(content)?;

    // If there's already a file at `path`, remove it.
    if path.exists() {
        std::fs::remove_file(path)?;
    }

    // Rename the temp file to the original path name.
    std::fs::rename(temp_path, path)?;

    Ok(())
}
