//! A simple tool for creating a new nannou project.
//!
//! 1. Determines whether the user is just sketching or wants an App (with model and event `fn`s).
//! 2. Asks for a sketch/app name.

extern crate names;
extern crate rand;

use std::env;
use std::io::{self, BufRead, Write};
use std::fs::{self, File};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;

enum Goal { Sketch, App }

const TEMPLATE_SKETCH: &[u8] = include_bytes!("../../examples/template_sketch.rs");
const TEMPLATE_APP: &[u8] = include_bytes!("../../examples/template_app.rs");

impl Goal {
    fn template_bytes(&self) -> &[u8] {
        match *self {
            Goal::Sketch => TEMPLATE_SKETCH,
            Goal::App => TEMPLATE_APP,
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

enum Response { Yes, No }

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

fn main() {
    // Retrieve the name of the exe that the user wishes to package.
    let response = ask_user("Are you sketching? (Y/n): ").expect("failed to get user input");
    let goal = match yes_or_no(&response) {
        Some(Response::Yes) => Goal::Sketch,
        Some(Response::No) => Goal::App,
        _ => {
            println!("I don't understand \"{}\", I was expecting \"y\" or \"n\". Exiting", response);
            return;
        }
    };

    // Get a name for the sketch.
    let name = loop {
        let response = ask_user("Name your sketch: ").expect("failed to get user input");
        if !response.is_empty() {
            break response;
        }
        let random_name = names::Generator::default().next().expect("failed to generate name");
        let question = format!("Hmmm... How about the name, \"{}\"? (Y/n): ", random_name);
        let response = ask_user(&question).expect("failed to get user input");
        if let Some(Response::Yes) = yes_or_no(&response) {
            break random_name;
        }
    };

    // TODO: Ask the user for additional cargo args.

    // TODO: Find the current version of nannou.
    let nannou_version = "0.5";
    let nannou_dependency = format!("nannou = \"{}\"", nannou_version);

    // TODO: Load the template example or fallback to compiled template.
    let template_bytes = goal.template_bytes();

    // Get the current directory.
    let current_directory = env::current_dir()
        .expect("could not retrieve current directory");

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
        file.write_all(template_bytes).expect("failed to write to new main file");
    }

    // Append the nannou dependency to the "Cargo.toml" file.
    {
        println!("Adding nannou dependency \"{}\"", nannou_dependency);
        let cargo_toml_path = project_path.join("Cargo").with_extension("toml");
        let mut file = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(&cargo_toml_path)
            .expect("failed to open \"Cargo.toml\" to add nannou dependency");
        writeln!(file, "\n{}", nannou_dependency).expect("failed to append nannou dependency");
    }

    // Change the directory to the newly created path.
    println!("Changing the current directory to \"{}\"", project_path.display());
    env::set_current_dir(&project_path).expect("failed to change directories");

    // Building and running.
    let bev = random_beverage();
    println!("Building `{}`. This might take a while for the first time. Grab a {}?", name, bev);
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
