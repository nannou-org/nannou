//! A small rust program that sets all nannou crates within the workspace to the specified version.
//!
//! Does the following:
//!
//! - Find all crates via the cargo workspace toml.
//! - Sets the version number for each of the `nannou*` packages and updates each of their
//!   respective `nannou*` dependencies.
//! - Writes the resulting TOML files.

fn main() {
    // Retrieve the specified version from the CLI args.
    let desired_version_string = std::env::args()
        .nth(1)
        .expect("expected one argument with the desired version, e.g. \"0.42.0\"");
    let desired_version = semver::Version::parse(&desired_version_string)
        .expect("failed to parse the specified version as valid semver, e.g. \"0.42.0\"");

    // Read the packages from the workspace manifest.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_manifest_dir = std::path::Path::new(manifest_dir)
        .parent()
        .unwrap() // nannou/scripts
        .parent()
        .unwrap(); // nannou
    let workspace_manifest_path = workspace_manifest_dir.join("Cargo.toml");
    let workspace_manifest_string = std::fs::read_to_string(&workspace_manifest_path)
        .expect("failed to read the workspace manifest");
    let workspace_toml = workspace_manifest_string
        .parse::<toml_edit::DocumentMut>()
        .expect("failed to parse workspace manifest as toml");
    let workspace_table = workspace_toml
        .as_table()
        .get("workspace")
        .and_then(|item| item.as_table())
        .expect("no table 'workspace'");
    let members_array = workspace_table
        .get("members")
        .and_then(|m| m.as_value())
        .and_then(|v| v.as_array())
        .expect("no array 'members'");
    let package_relative_paths: Vec<_> = members_array.iter().filter_map(|v| v.as_str()).collect();

    // Set the versions and dependency versions where necessary.
    let mut manifest_updates = vec![];
    for relative_path in &package_relative_paths {
        // Read the manifest for each crate into a toml document.
        let dir_path = workspace_manifest_dir.join(relative_path);
        let manifest_path = dir_path.join("Cargo.toml");
        let manifest_string =
            std::fs::read_to_string(&manifest_path).expect("failed to read the manifest");
        let mut manifest_toml = manifest_string
            .parse::<toml_edit::DocumentMut>()
            .expect("failed to parse manifest as toml");
        let manifest_table = manifest_toml.as_table_mut();

        // Update the manifest table.
        if is_nannou_member(relative_path) {
            set_package_version(manifest_table, &desired_version);
        }
        if let Some(deps) = manifest_table["dependencies"].as_table_mut() {
            update_dependencies_table(deps, &desired_version);
        }
        if let Some(deps) = manifest_table["dev-dependencies"].as_table_mut() {
            update_dependencies_table(deps, &desired_version);
        }

        // Retrieve the updated string.
        let toml_string = manifest_toml.to_string();
        manifest_updates.push((manifest_path, toml_string));
    }

    // Only write the files once we've successfully created a new TOML string for each package.
    for (manifest_path, toml_string) in manifest_updates {
        safe_file_save(&manifest_path, toml_string.as_bytes())
            .expect("failed to write the edited manifest");
    }

    println!("Successfully set version to {}!", desired_version);
}

/// Set the version within the package entry of the given manifest table.
fn set_package_version(manifest_table: &mut toml_edit::Table, desired_version: &semver::Version) {
    let package_table = manifest_table["package"]
        .as_table_mut()
        .expect("failed to retrieve package table");
    let version_value = package_table["version"]
        .as_value_mut()
        .expect("failed to retrieve value for version key");
    *version_value = format!("{}", desired_version).into();
}

/// Shared between updating the `[dependencies]` and `[dev-dependencies]` tables.
fn update_dependencies_table(table: &mut toml_edit::Table, desired_version: &semver::Version) {
    let nannou_deps: Vec<_> = table
        .iter()
        .map(|(name, _)| name.to_string())
        .filter(|s| is_nannou_member(s))
        .collect();
    for dep in nannou_deps {
        let value = table[&dep]
            .as_value_mut()
            .expect("failed to retrieve toml value for dependency");
        let version_value = match *value {
            toml_edit::Value::String(_) => value,
            toml_edit::Value::InlineTable(ref mut inline_table) => {
                inline_table.get_or_insert("version", "")
            }
            ref v => panic!("unexpected dependency value: {:?}", v),
        };
        *version_value = format!("{}", desired_version).into();
    }
}

/// Check if the given crate name is one of the nannou crates whose version requires setting.
fn is_nannou_member(workspace_member_name: &str) -> bool {
    workspace_member_name.contains("nannou")
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
    buffered.write(content)?;

    // If there's already a file at `path`, remove it.
    if path.exists() {
        std::fs::remove_file(path)?;
    }

    // Rename the temp file to the original path name.
    std::fs::rename(temp_path, path)?;

    Ok(())
}
