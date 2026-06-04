//! A simple little script that attempts to run every example listed within the `Cargo.toml` of
//! each specified package. E.g.
//!
//! ```ignore
//! cargo run -p run_all_examples -- nature_of_code
//! ```
//!
//! If no package is specified, all examples for the `examples`, `generative_design` and
//! `nature_of_code` packages will be run. E.g.
//!
//! ```ignore
//! cargo run -p run_all_examples
//! ```
//!
//! All build and example output is forwarded to the terminal so that progress can be observed.

use std::path::Path;
use std::process::Command;
use std::time::Duration;

/// Cap the number of parallel build jobs - nannou builds are currently quite RAM intensive.
const JOBS: &str = "8";

/// The packages whose examples are run when none are specified on the command line.
const ALL_PACKAGES: &[&str] = &["examples", "generative_design", "nature_of_code"];

/// How long each example is allowed to run before it is stopped.
const RUN_DURATION: Duration = Duration::from_secs(3);

fn main() {
    // Retrieve the specified packages if any, otherwise default to `ALL_PACKAGES`.
    let specified_packages: Vec<String> = std::env::args().skip(1).collect();
    let packages: Vec<String> = if specified_packages.is_empty() {
        ALL_PACKAGES.iter().map(|s| s.to_string()).collect()
    } else {
        specified_packages
    };

    // Resolve the workspace directory relative to this script's manifest (nannou/scripts/..).
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_dir = Path::new(manifest_dir)
        .parent()
        .unwrap() // nannou/scripts
        .parent()
        .unwrap(); // nannou

    // Examples that exited with a failure before they could be stopped, as "<package>/<example>".
    let mut failed: Vec<String> = vec![];

    for package in &packages {
        let manifest_path = workspace_dir.join(package).join("Cargo.toml");
        let bytes = std::fs::read(&manifest_path).unwrap();
        let toml: toml::Value = toml::from_str(std::str::from_utf8(&bytes).unwrap()).unwrap();

        // First, build all examples in the package, forwarding output to the terminal.
        println!("Building all examples within /nannou/{package}...");
        let status = Command::new("cargo")
            .args(["build", "-j", JOBS, "-p", package, "--examples"])
            .status()
            .expect("failed to run `cargo build -p package --examples`");
        if !status.success() {
            panic!("failed to build examples for package \"{package}\"");
        }

        // Find the `example` array within the manifest to retrieve all example names.
        let examples = toml["example"]
            .as_array()
            .expect("failed to retrieve example array");
        println!("Running all examples within /nannou/{package}...");
        for example in examples {
            let name = example["name"]
                .as_str()
                .expect("failed to retrieve example name");

            // Run the example, forwarding its output to the terminal.
            println!("Running example \"{name}\"...");
            let mut child = Command::new("cargo")
                .args(["run", "-j", JOBS, "-p", package, "--example", name])
                .spawn()
                .expect("failed to spawn `cargo run --example` process");

            // Allow the example to run for a moment.
            std::thread::sleep(RUN_DURATION);

            // If the example exited on its own with a failure before we stopped it, it must
            // have failed.
            if let Ok(Some(exit)) = child.try_wait() {
                if !exit.success() {
                    eprintln!("example \"{name}\" exited with {exit}");
                    failed.push(format!("{package}/{name}"));
                }
            }

            // Stop the example and wait for it to exit.
            child.kill().ok();
            child.wait().ok();
        }
    }

    if failed.is_empty() {
        println!("All examples ran successfully.");
    } else {
        eprintln!("{} example(s) failed:", failed.len());
        for entry in &failed {
            eprintln!("  - {entry}");
        }
        std::process::exit(1);
    }
}
