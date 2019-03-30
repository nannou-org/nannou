// A simple little script that attempts to run every example listed within the `Cargo.toml`.
//
// This will fail if any of the examples `panic!`.
#[test]
#[ignore]
fn test_run_all_examples() {
    // Read the nannou cargo manifest to a `toml::Value`.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let manifest_path = std::path::Path::new(manifest_dir).join("Cargo").with_extension("toml");
    let bytes = std::fs::read(&manifest_path).unwrap();
    let toml: toml::Value = toml::from_slice(&bytes).unwrap();

    // Find the `examples` table within the `toml::Value` to find all example names.
    let examples = toml["example"].as_array().expect("failed to retrieve example array");
    for example in examples {
        let name = example["name"].as_str().expect("failed to retrieve example name");

        // For each example, invoke a cargo sub-process to run the example.
        let mut child = std::process::Command::new("cargo")
            .arg("run")
            .arg("--example")
            .arg(&name)
            .spawn()
            .expect("failed to spawn `cargo run --example` process");

        // Allow each example to run for 3 secs each.
        std::thread::sleep(std::time::Duration::from_secs(3));

        // Kill the example and retrieve any output.
        child.kill().ok();
        let output = child.wait_with_output().expect("failed to wait for child process");

        // If the example wrote to `stderr` it must have failed.
        if !output.stderr.is_empty() {
            panic!("example {} wrote to stderr: {:?}", name, output);
        }
    }
}
