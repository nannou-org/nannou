use skeptic::*;
use std::path::{Path, PathBuf};
use std::{env, fs, io};

// Check the given path and parent directories for a path with the given name.
fn check_parents(name: &str, path: &Path) -> io::Result<Option<PathBuf>> {
    match check_dir(name, path) {
        Ok(None) => match path.parent() {
            None => Ok(None),
            Some(parent) => check_parents(name, parent),
        },
        other_result => other_result,
    }
}

// Check the given directory for a path with the matching name.
fn check_dir(name: &str, path: &Path) -> io::Result<Option<PathBuf>> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        if entry_path.ends_with(name) {
            return Ok(Some(entry_path));
        }
    }
    Ok(None)
}

fn main() {
    let out_dir = env::var("OUT_DIR").expect("failed to retrieve OUT_DIR");
    let workspace_cargo_toml_path = check_parents("Cargo.toml", Path::new(&out_dir))
        .expect("an error occurred while searching for Cargo.toml")
        .expect("no Cargo.toml found");
    let book_root_path = workspace_cargo_toml_path.parent().unwrap().join("guide");
    let book_src_path = book_root_path.join("src");
    let book_src_path_str = format!("{}", book_src_path.display());
    let mdbook_files = markdown_files_of_directory(&book_src_path_str);
    generate_doc_tests(&mdbook_files);
}
