//! The `font::Id` and `font::Map` types.

use crate::text::{Font, FontCollection};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

/// A type-safe wrapper around the `FontId`.
///
/// This is used as both:
///
/// - The key for the `font::Map`'s inner `HashMap`.
/// - The `font_id` field for the rusttype::gpu_cache::Cache.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(usize);

/// A collection of mappings from `font::Id`s to `rusttype::Font`s.
#[derive(Debug)]
pub struct Map {
    next_index: usize,
    map: HashMap<Id, Font>,
}

/// An iterator yielding an `Id` for each new `rusttype::Font` inserted into the `Map` via the
/// `insert_collection` method.
pub struct NewIds {
    index_range: std::ops::Range<usize>,
}

/// Yields the `Id` for each `Font` within the `Map`.
#[derive(Clone)]
pub struct Ids<'a> {
    keys: std::collections::hash_map::Keys<'a, Id, Font>,
}

/// Returned when loading new fonts from file or bytes.
#[derive(Debug)]
pub enum Error {
    /// Some error occurred while loading a `FontCollection` from a file.
    Io(std::io::Error),
    /// No `Font`s could be yielded from the `FontCollection`.
    NoFont,
}

/// The name of the default directory that is searched for fonts.
pub const DEFAULT_DIRECTORY_NAME: &str = "fonts";

impl Id {
    /// Returns the inner `usize` from the `Id`.
    pub fn index(self) -> usize {
        self.0
    }
}

impl Map {
    /// Construct the new, empty `Map`.
    pub fn new() -> Self {
        Map {
            next_index: 0,
            map: HashMap::default(),
        }
    }

    /// Borrow the `rusttype::Font` associated with the given `font::Id`.
    pub fn get(&self, id: Id) -> Option<&Font> {
        self.map.get(&id)
    }

    /// Adds the given `rusttype::Font` to the `Map` and returns a unique `Id` for it.
    pub fn insert(&mut self, font: Font) -> Id {
        let index = self.next_index;
        self.next_index = index.wrapping_add(1);
        let id = Id(index);
        self.map.insert(id, font);
        id
    }

    /// Insert a single `Font` into the map by loading it from the given file path.
    pub fn insert_from_file<P>(&mut self, path: P) -> Result<Id, Error>
    where
        P: AsRef<std::path::Path>,
    {
        let font = from_file(path)?;
        Ok(self.insert(font))
    }

    // /// Adds each font in the given `rusttype::FontCollection` to the `Map` and returns an
    // /// iterator yielding a unique `Id` for each.
    // pub fn insert_collection(&mut self, collection: FontCollection) -> NewIds {
    //     let start_index = self.next_index;
    //     let mut end_index = start_index;
    //     for index in 0.. {
    //         match collection.font_at(index) {
    //             Some(font) => {
    //                 self.insert(font);
    //                 end_index += 1;
    //             }
    //             None => break,
    //         }
    //     }
    //     NewIds { index_range: start_index..end_index }
    // }

    /// Produces an iterator yielding the `Id` for each `Font` within the `Map`.
    pub fn ids(&self) -> Ids {
        Ids {
            keys: self.map.keys(),
        }
    }
}

/// Produce a unique ID for the given font.
pub fn id(font: &Font) -> Id {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for name in font.font_name_strings() {
        name.hash(&mut hasher);
    }
    Id((hasher.finish() % std::usize::MAX as u64) as usize)
}

/// Load a `FontCollection` from a file at a given path.
pub fn collection_from_file<P>(path: P) -> Result<FontCollection, std::io::Error>
where
    P: AsRef<std::path::Path>,
{
    use std::io::Read;
    let path = path.as_ref();
    let mut file = std::fs::File::open(path)?;
    let mut file_buffer = Vec::new();
    file.read_to_end(&mut file_buffer)?;
    Ok(FontCollection::from_bytes(file_buffer)?)
}

/// Load a single `Font` from a file at the given path.
pub fn from_file<P>(path: P) -> Result<Font, Error>
where
    P: AsRef<std::path::Path>,
{
    let collection = collection_from_file(path)?;
    collection.into_font().or(Err(Error::NoFont))
}

/// Load the default notosans font.
///
/// This function is only available if the `notosans` feature is enabled, which it is by default.
#[cfg(feature = "notosans")]
pub fn default_notosans() -> Font {
    let collection = FontCollection::from_bytes(notosans::REGULAR_TTF)
        .expect("failed to load the `notosans::REGULAR_TTF` font collection");
    collection
        .into_font()
        .expect("the `notosans::REGULAR_TTF` font collection contained no fonts")
}

/// The directory that is searched for default fonts.
pub fn default_directory(assets: &Path) -> PathBuf {
    assets.join(DEFAULT_DIRECTORY_NAME)
}

/// Load the default font.
///
/// If the `notosans` feature is enabled, this will return the font loaded from
/// `notosans::REGULAR_TTF`.
///
/// Otherwise this will attempt to locate the `assets/fonts` directory. If the directory exists,
/// the first font that is found will be loaded. If no fonts are found, an error is returned.
#[allow(unreachable_code, unused_variables)]
pub fn default(assets: &Path) -> Result<Font, Error> {
    #[cfg(feature = "notosans")]
    {
        return Ok(default_notosans());
    }

    // Find a font in `assets/fonts`.
    let fonts_dir = default_directory(assets);
    if fonts_dir.exists() && fonts_dir.is_dir() {
        for res in crate::io::walk_dir(&fonts_dir) {
            let entry = match res {
                Ok(e) => e,
                Err(_) => continue,
            };
            match from_file(entry.path()) {
                Err(_) => continue,
                Ok(font) => return Ok(font),
            }
        }
    }

    Err(Error::NoFont)
}

impl Iterator for NewIds {
    type Item = Id;
    fn next(&mut self) -> Option<Self::Item> {
        self.index_range.next().map(|i| Id(i))
    }
}

impl<'a> Iterator for Ids<'a> {
    type Item = Id;
    fn next(&mut self) -> Option<Self::Item> {
        self.keys.next().map(|&id| id)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref e) => std::error::Error::description(e),
            Error::NoFont => "No `Font` found in the loaded `FontCollection`.",
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            Error::Io(ref e) => std::fmt::Display::fmt(e, f),
            _ => write!(f, "{}", std::error::Error::description(self)),
        }
    }
}
