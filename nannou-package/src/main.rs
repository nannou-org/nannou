//! A small rust script for packaging a nannou project.

use copy_dir::copy_dir;
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// Check the given path and `depth` number of parent directories for a folder with the given name.
fn check_parents(directory: &Path, name: &str) -> io::Result<Option<PathBuf>> {
    match check_dir(directory, name) {
        Ok(None) => match directory.parent() {
            None => Ok(None),
            Some(parent) => check_parents(parent, name),
        },
        other_result => other_result,
    }
}

// Check the given directory for a folder with the matching name.
fn check_dir(directory: &Path, name: &str) -> io::Result<Option<PathBuf>> {
    let path = fs::read_dir(directory)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|entry_path| entry_path.ends_with(name));
    Ok(path)
}

// Zip the given directory to the given `writer`.
fn zip_dir<T>(dir: &Path, writer: T) -> zip::result::ZipResult<()>
where
    T: io::Write + io::Seek,
{
    let mut zip = zip::ZipWriter::new(writer);
    let options = zip::write::FileOptions::default().unix_permissions(0o755);
    let mut buffer = Vec::new();
    let entries = WalkDir::new(dir).into_iter().filter_map(Result::ok);
    for entry in entries {
        let path = entry.path();
        let name = path
            .strip_prefix(Path::new(dir))
            .ok()
            .and_then(|s| s.to_str())
            .expect("could not get path name while zipping directory");
        if path.is_file() {
            println!("\tzipping \"{}\" as {} ...", path.display(), name);
            zip.start_file(name, options)?;
            let mut f = fs::File::open(path)?;
            f.read_to_end(&mut buffer)?;
            zip.write_all(&*buffer)?;
            buffer.clear();
        }
    }
    zip.finish()?;
    Result::Ok(())
}

// Get the string for the `target_arch`.
//
// TODO: Surely there's a better way to do this?
fn target_arch() -> Option<&'static str> {
    let target_arch = if cfg!(target_arch = "x86") {
        "x86"
    } else if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "mips") {
        "mips"
    } else if cfg!(target_arch = "powerpc") {
        "powerpc"
    } else if cfg!(target_arch = "powerpc64") {
        "powerpc64"
    } else if cfg!(target_arch = "arm") {
        "arm"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        return None;
    };
    Some(target_arch)
}

// Get the string for the `target_os`.
//
// TODO: Surely there's a better way to do this?
fn target_os() -> Option<&'static str> {
    let target_os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "ios") {
        "ios"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "android") {
        "android"
    } else if cfg!(target_os = "freebsd") {
        "freebsd"
    } else if cfg!(target_os = "dragonfly") {
        "dragonfly"
    } else if cfg!(target_os = "bitrig") {
        "bitrig"
    } else if cfg!(target_os = "openbsd") {
        "openbsd"
    } else if cfg!(target_os = "netbsd") {
        "netbsd"
    } else {
        return None;
    };
    Some(target_os)
}

fn main() {
    // Retrieve the name of the exe that the user wishes to package.
    print!("Please write the name of the `bin` target that you'd like to package: ");
    io::stdout().flush().expect("failed to flush stdout");
    let mut name = String::new();
    io::stdin().read_line(&mut name).unwrap();
    let name = name.trim();

    // Find the Cargo.toml directory.
    let current_directory = env::current_dir().expect("could not retrieve current directory");
    let cargo_toml_path = check_parents(&current_directory, "Cargo.toml")
        .expect("error finding `Cargo.toml` root directory")
        .expect("could not find `Cargo.toml` root directory");
    let cargo_directory = cargo_toml_path
        .parent()
        .expect("could not find cargo root directory");

    // Find the path to the `exe` that we're going to package.
    let target_path = cargo_directory.join("target");
    if !target_path.exists() || !target_path.is_dir() {
        panic!(
            "The directory \"{}\" does not exist.",
            target_path.display()
        );
    }
    let release_path = target_path.join("release");
    if !release_path.exists() || !release_path.is_dir() {
        panic!(
            "The directory \"{}\" does not exist.",
            release_path.display()
        );
    }
    let exe_path = release_path.join(&name);
    if !exe_path.exists() || !exe_path.is_file() {
        panic!("The file \"{}\" does not exist.", release_path.display());
    }

    // Create a `builds` directory in the project root if not already added.
    let builds_path = cargo_directory.join("builds");
    if !builds_path.exists() || !builds_path.is_dir() {
        println!("Creating \"{}\"", builds_path.display());
        fs::create_dir_all(&builds_path).expect("could not create `builds` directory");
    }

    // Create a build name.
    //
    // TODO: The following `arch` and `os` assumes that the pre-built target exe was built on the
    // same architecture with which this `nannou-package` exe was built. This should be fixed to
    // use the *actual* target that we're packaging properly somehow. This would be especially
    // useful if someone was cross-compiling for multiple platforms on one machine.
    let arch =
        target_arch().expect("unknown `target_arch` - please let us know at the nannou repo!");
    let os = target_os().expect("unknown `target_os` - please let us know at the nannou repo!");
    let now = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let build_name = format!("{}-{}-{}-{}", name, arch, os, now);
    let build_path = builds_path.join(build_name);
    println!("Creating \"{}\"", build_path.display());
    fs::create_dir(&build_path).expect("could not create build directory");

    // Copy the exe to the build directory.
    let exe_name = exe_path
        .file_name()
        .and_then(|s| s.to_str())
        .expect("could not get exe name");
    let build_exe_path = build_path.join(&exe_name);
    println!(
        "Copying \"{}\" to \"{}\"",
        exe_path.display(),
        build_exe_path.display()
    );
    fs::copy(&exe_path, build_exe_path).expect("could not copy exe to build directory");

    // If there's an `assets` directory, copy it to the build directory.
    let assets_path = cargo_directory.join("assets");
    if assets_path.exists() && assets_path.is_dir() {
        let build_assets_path = build_path.join("assets");
        println!(
            "Copying \"{}\" to \"{}\"",
            assets_path.display(),
            build_assets_path.display()
        );
        copy_dir(assets_path, build_assets_path).expect("could not copy assets directory");
    }

    // Create a `.zip` of the build.
    let zip_path = build_path.with_extension("zip");
    let zip_file_name = zip_path
        .file_name()
        .and_then(|s| s.to_str())
        .expect("could not create zip file name");
    println!("Creating \"{}\"", zip_path.display());
    let file = fs::File::create(&zip_path).expect("could not create zip file");
    let buffered = io::BufWriter::new(file);
    zip_dir(&build_path, buffered).expect("could not zip build directory");

    // Remove the old build directory as we no longer need it.
    println!("Removing \"{}\"", build_path.display());
    fs::remove_dir_all(build_path).expect("could not remove build directory after zipping");

    // Declare that we're done!
    println!("Done! Build \"{}\" successfully created.", zip_file_name);
}
