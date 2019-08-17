//! An extension of the `std::io` module. Includes functions for safely saving and loading files
//! from any serializable types, along with functions specifically for working with JSON and TOML.

use serde;
use serde_json;
use std::error::Error;
use std::io::{Read, Write};
use std::path::Path;
use std::{fmt, fs, io};
use toml;

/// Errors that might occur when saving a file.
#[derive(Debug)]
pub enum FileError<E> {
    Io(io::Error),
    Format(E),
}

pub type JsonFileError = FileError<serde_json::Error>;
pub type TomlFileSaveError = FileError<toml::ser::Error>;
pub type TomlFileLoadError = FileError<toml::de::Error>;

impl<E> From<io::Error> for FileError<E> {
    fn from(err: io::Error) -> Self {
        FileError::Io(err)
    }
}

impl From<serde_json::Error> for JsonFileError {
    fn from(err: serde_json::Error) -> Self {
        FileError::Format(err)
    }
}

impl From<toml::ser::Error> for TomlFileSaveError {
    fn from(err: toml::ser::Error) -> Self {
        FileError::Format(err)
    }
}

impl From<toml::de::Error> for TomlFileLoadError {
    fn from(err: toml::de::Error) -> Self {
        FileError::Format(err)
    }
}

impl<E> Error for FileError<E>
where
    E: Error,
{
    fn description(&self) -> &str {
        match *self {
            FileError::Io(ref err) => err.description(),
            FileError::Format(ref err) => err.description(),
        }
    }
}

impl<E> fmt::Display for FileError<E>
where
    E: Error,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// Saves the file to a temporary file before removing the original to reduce the chance of losing
/// data in the case that something goes wrong during saving.
///
/// This function also creates all necessary parent directories if they do not exist.
pub fn safe_file_save<P>(path: P, content: &[u8]) -> io::Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let temp_path = path.with_extension("tmp");

    // If the temp file exists, remove it.
    if temp_path.exists() {
        fs::remove_file(&temp_path)?;
    }

    // Create the directory if it doesn't exist.
    if let Some(directory) = path.parent() {
        if !directory.exists() {
            fs::create_dir_all(&temp_path)?;
        }
    }

    // Write the temp file.
    let file = fs::File::create(&temp_path)?;
    let mut buffered = io::BufWriter::new(file);
    buffered.write(content)?;

    // If there's already a file at `path`, remove it.
    if path.exists() {
        fs::remove_file(&path)?;
    }

    // Rename the temp file to the original path name.
    fs::rename(temp_path, path)?;

    Ok(())
}

/// A generic function for safely saving a serializable type to a JSON file.
pub fn save_to_json<P, T>(path: P, t: &T) -> Result<(), JsonFileError>
where
    P: AsRef<Path>,
    T: serde::Serialize,
{
    let string = serde_json::to_string_pretty(t)?;
    safe_file_save(path, string.as_bytes())?;
    Ok(())
}

/// A generic funtion for loading a type from a JSON file.
pub fn load_from_json<'a, P, T>(path: P) -> Result<T, JsonFileError>
where
    P: AsRef<Path>,
    T: for<'de> serde::Deserialize<'de>,
{
    let file = fs::File::open(path)?;
    let t = serde_json::from_reader(file)?;
    Ok(t)
}

/// A generic function for safely saving a serializable type to a TOML file.
pub fn save_to_toml<P, T>(path: P, t: &T) -> Result<(), TomlFileSaveError>
where
    P: AsRef<Path>,
    T: serde::Serialize,
{
    let string = toml::to_string_pretty(t)?;
    safe_file_save(path, string.as_bytes())?;
    Ok(())
}

/// A generic funtion for loading a type from a TOML file.
pub fn load_from_toml<'a, P, T>(path: P) -> Result<T, TomlFileLoadError>
where
    P: AsRef<Path>,
    T: for<'de> serde::Deserialize<'de>,
{
    let file = fs::File::open(path)?;
    let mut buffered = io::BufReader::new(file);
    let mut string = String::new();
    buffered.read_to_string(&mut string)?;
    let t = toml::from_str(&string)?;
    Ok(t)
}

/// Attempt to recursively walk the given directory and all its sub-directories.
///
/// This function is shorthand for the `walkdir` crate's `WalkDir::new` constructor.
pub fn walk_dir<P>(path: P) -> walkdir::WalkDir
where
    P: AsRef<Path>,
{
    walkdir::WalkDir::new(path)
}
