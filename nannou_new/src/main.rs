//! A simple tool for creating a new nannou project.
//!
//! 1. Asks if the user is just sketching.
//! 2. If so, generates a project from `template_sketch`.
//! 3. Otherwise generates a project fro `template_app`.
//! 4. Adds the latest nannou version as a dep to the `Cargo.toml`.
//! 5. Builds the project with optimisations. Suggests getting a beverage.

use std::env;
use std::fs::{self, File};
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;

use cargo::CargoResult;

enum Project {
    Sketch,
    App,
}

impl Project {
    fn lowercase(&self) -> &str {
        match *self {
            Project::Sketch => "sketch",
            Project::App => "app",
        }
    }

    fn template_file_name(&self) -> &str {
        match *self {
            Project::Sketch => "template_sketch.rs",
            Project::App => "template_app.rs",
        }
    }
}

// Ask the user the given question, get a trimmed response.
fn ask_user(question: &str) -> io::Result<String> {
    print!("{}", question);
    io::stdout().flush()?;
    let mut response = String::new();
    io::stdin().read_line(&mut response)?;
    Ok(response.trim().to_string())
}

enum Response {
    Yes,
    No,
}

fn yes_or_no(s: &str) -> Option<Response> {
    let s = s.to_ascii_lowercase();
    match &s[..] {
        "" | "y" | "yes" | "yeah" | "yep" | "flip yes" => Some(Response::Yes),
        "n" | "no" | "nah" | "nope" | "flip no" => Some(Response::No),
        _ => None,
    }
}

fn random_beverage() -> &'static str {
    if rand::random() {
        "cup of coffee"
    } else {
        "cup of tea"
    }
}

// Retrieve the latest version of the crates.io package with the given name using cargo.
fn crates_io_package_latest_version(name: &str) -> CargoResult<cargo::core::Package> {
    use cargo::core::source::Source;

    // Setup the cargo config.
    let cargo_config = cargo::Config::default()?;

    // The crates.io source ID.
    let src_id = cargo::core::SourceId::crates_io(&cargo_config)?;

    // The crates.io registry source.
    let yanked = std::collections::HashSet::new();
    let mut registry_source =
        cargo::sources::registry::RegistrySource::remote(src_id, &yanked, &cargo_config);

    // The nannou dependency (don't really understand why we need "Dependency").
    let vers = None;
    let dep = cargo::core::dependency::Dependency::parse_no_deprecated(name, vers, src_id)?;

    // Hold the package lock until the end of the function, query and download requires it.
    let _lock = cargo_config.acquire_package_cache_lock()?;

    // Retrieve the PackageId by querying the source and finding the most recent one.
    let versions = registry_source.query_vec(&dep)?;
    let most_recent_pkg_id = match versions.iter().map(|v| v.package_id()).max() {
        Some(pkg_id) => pkg_id,
        None => panic!("could not find `{}` package in crates.io registry", name),
    };

    // Finally download the Package
    Source::download_now(Box::new(registry_source), most_recent_pkg_id, &cargo_config)
}

fn main() {
    // Retrieve the name of the exe that the user wishes to package.
    let response = ask_user("Are you sketching? (Y/n): ").expect("failed to get user input");
    let project = match yes_or_no(&response) {
        Some(Response::Yes) => Project::Sketch,
        Some(Response::No) => Project::App,
        _ => {
            println!(
                "I don't understand \"{}\", I was expecting \"y\" or \"n\". Exiting",
                response
            );
            return;
        }
    };

    // Get a name for the sketch.
    let name = loop {
        let name_your = format!("Name your {}: ", project.lowercase());
        let response = ask_user(&name_your).expect("failed to get user input");
        if !response.is_empty() {
            break response;
        }
        let random_name = names::Generator::default()
            .next()
            .expect("failed to generate name");
        let question = format!("Hmmm... How about the name, \"{}\"? (Y/n): ", random_name);
        let response = ask_user(&question).expect("failed to get user input");
        if let Some(Response::Yes) = yes_or_no(&response) {
            break random_name;
        }
    };

    // Retrieve the nannou package from crates.io.
    let nannou_package = crates_io_package_latest_version("nannou")
        .expect("failed to retrieve `nannou` package from crates.io");

    // Get the latest nannou version.
    let nannou_version = format!("{}", nannou_package.version());
    let nannou_dependency = format!("nannou = \"{}\"", nannou_version);

    // Find the template file within the nannou package.
    let template_bytes = {
        let template_path = nannou_package
            .root()
            .join("examples")
            .join(project.template_file_name());
        std::fs::read(&template_path).unwrap_or_else(|_| {
            panic!(
                "failed to read template bytes from {}",
                template_path.display()
            )
        })
    };

    // Get the current directory.
    let current_directory = env::current_dir().expect("could not retrieve current directory");

    // Create the project path.
    let project_path = current_directory.join(&name);
    println!("Creating cargo project: \"{}\"", project_path.display());
    Command::new("cargo")
        .arg("new")
        .arg("--bin")
        .arg(&name)
        .output()
        .expect("failed to create cargo project");

    // Assert the path now exists.
    assert!(project_path.exists());
    assert!(project_path.is_dir());

    // Remove the existing main file.
    let src_path = project_path.join("src");
    let main_path = src_path.join(Path::new("main").with_extension("rs"));
    fs::remove_file(&main_path).expect("failed to remove existing \"main.rs\" file");

    // Create the file from the template string.
    {
        println!("Writing template file \"{}\"", main_path.display());
        let mut file = File::create(main_path).expect("failed to create new main file");
        file.write_all(&template_bytes)
            .expect("failed to write to new main file");
    }

    // Append the nannou dependency to the "Cargo.toml" file.
    {
        println!("Adding nannou dependency `{}`", nannou_dependency);
        let cargo_toml_path = project_path.join("Cargo").with_extension("toml");
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&cargo_toml_path)
            .expect("failed to open \"Cargo.toml\" to add nannou dependency");
        writeln!(file, "{}", nannou_dependency).expect("failed to append nannou dependency");
    }

    // Create the assets directory.
    let assets_path = project_path.join("assets");
    fs::create_dir(assets_path).expect("failed to create assets directory");

    // Change the directory to the newly created path.
    println!(
        "Changing the current directory to \"{}\"",
        project_path.display()
    );
    env::set_current_dir(&project_path).expect("failed to change directories");

    // Building and running.
    let bev = random_beverage();
    println!(
        "Building `{}`. This might take a while for the first time. Grab a {}?",
        name, bev
    );
    let mut child = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn cargo build and run process");

    // Read from cargo's stdout and stderr.
    let err = io::BufReader::new(child.stderr.take().unwrap());
    let out = io::BufReader::new(child.stdout.take().unwrap());
    let (tx, rx) = mpsc::channel();
    let tx_err = tx.clone();

    let err_handle = thread::spawn(move || {
        for line in err.lines().filter_map(|l| l.ok()) {
            tx_err.send(Err(line)).expect("failed to send piped stderr");
        }
    });

    let out_handle = thread::spawn(move || {
        for line in out.lines().filter_map(|l| l.ok()) {
            tx.send(Ok(line)).expect("failed to send piped stdout");
        }
    });

    // Print the stdout and stderr from cargo.
    for result in rx {
        match result {
            Ok(line) => println!("[cargo] {}", line),
            Err(line) => eprintln!("[cargo] {}", line),
        }
    }

    err_handle.join().unwrap();
    out_handle.join().unwrap();

    // If the exe is where it should be, assume the build succeeded.
    let exe_path = project_path.join("target").join("release").join(&name);
    if exe_path.exists() {
        println!("Successfully built \"{}\"! Go forth and create!", name);
    }
}
