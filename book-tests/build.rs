use skeptic::*;

fn main() {
    let cargo_toml_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let book_root_path = cargo_toml_path.parent().unwrap().parent().unwrap();
    let book_src_path = book_root_path.join("src");
    let book_src_path_str = format!("{}", book_src_path.display());
    let mdbook_files = markdown_files_of_directory(&book_src_path_str);
    generate_doc_tests(&mdbook_files);
}
