//! Re-exports the `ilda-idtf` crate and extends it with a **FrameReader** API, simplifying the
//! process of reading the ILDA IDTF format into frames of points that are compatible with the
//! `nannou_laser` API.
//!
//! See the extensive, top-level `ilda-idtf` API docs [here](https://docs.rs/ilda-idtf).

use std::io;
use std::path::Path;

#[doc(inline)]
pub use ilda_idtf::*;

use crate::{Point, point};

/// A type that simplifies the process of reading laser frames from the ILDA IDTF format in a
/// manner that is compatible with the `nannou_laser` stream APIs.
pub struct FrameReader<R> {
    reader: SectionReader<R>,
    points: Vec<Point>,
    palette: Vec<layout::Color>,
}

/// A `FrameReader` that reads from a buffered file.
pub type BufFileFrameReader = FrameReader<io::BufReader<std::fs::File>>;

impl<R> FrameReader<R>
where
    R: io::Read,
{
    /// Create a new `FrameReader` from the given ILDA IDTF format reader.
    pub fn new(reader: R) -> Self {
        Self::from_section_reader(SectionReader::new(reader))
    }

    /// Create a new `FrameReader` from the given `SectionReader`.
    pub fn from_section_reader(reader: SectionReader<R>) -> Self {
        let points = vec![];
        let palette = DEFAULT_PALETTE.to_vec();
        Self {
            reader,
            points,
            palette,
        }
    }

    /// Consumes the `FrameReader` and returns the inner `SectionReader`.
    pub fn into_section_reader(self) -> SectionReader<R> {
        self.reader
    }

    /// Produce the next frame as a list of points representing consecutive lines.
    pub fn next(&mut self) -> io::Result<Option<&[Point]>> {
        self.points.clear();
        loop {
            let FrameReader {
                ref mut reader,
                ref mut points,
                ref mut palette,
            } = *self;

            // Read the next section.
            let section = match reader.read_next()? {
                None => return Ok(None),
                Some(s) => s,
            };

            match section.reader {
                // Update the color palette.
                SubsectionReaderKind::ColorPalette(mut r) => {
                    palette.clear();
                    while let Some(new_palette) = r.read_next()? {
                        palette.push(new_palette.color);
                    }
                    continue;
                }

                // Update the points.
                SubsectionReaderKind::Coords3dIndexedColor(mut r) => {
                    while let Some(p) = r.read_next()? {
                        points.push(point_from_coords_3d_indexed_color(*p, palette));
                    }
                }
                SubsectionReaderKind::Coords2dIndexedColor(mut r) => {
                    while let Some(p) = r.read_next()? {
                        points.push(point_from_coords_2d_indexed_color(*p, palette));
                    }
                }
                SubsectionReaderKind::Coords3dTrueColor(mut r) => {
                    while let Some(p) = r.read_next()? {
                        points.push(point_from_coords_3d_true_color(*p));
                    }
                }
                SubsectionReaderKind::Coords2dTrueColor(mut r) => {
                    while let Some(p) = r.read_next()? {
                        points.push(point_from_coords_2d_true_color(*p));
                    }
                }
            }

            return Ok(Some(&self.points[..]));
        }
    }
}

impl BufFileFrameReader {
    /// Creates a new `FrameReader` from the file at the given path.
    ///
    /// Returns a `FrameReader` that performs buffered reads on the file at the given path.
    pub fn open<P>(path: P) -> io::Result<Self>
    where
        P: AsRef<Path>,
    {
        Ok(Self::from_section_reader(open(path)?))
    }
}

impl<R> From<SectionReader<R>> for FrameReader<R>
where
    R: io::Read,
{
    fn from(r: SectionReader<R>) -> Self {
        Self::from_section_reader(r)
    }
}

impl<R> From<FrameReader<R>> for SectionReader<R>
where
    R: io::Read,
{
    fn from(val: FrameReader<R>) -> Self {
        val.into_section_reader()
    }
}

fn normalise_coord(c: i16) -> f32 {
    c as f32 / std::i16::MAX as f32
}

fn normalise_color(c: u8) -> f32 {
    c as f32 / std::u8::MAX as f32
}

fn coords3d_to_position(coords: layout::Coords3d) -> point::Position {
    [
        normalise_coord(coords.x.get()),
        normalise_coord(coords.y.get()),
    ]
}

fn coords2d_to_position(coords: layout::Coords2d) -> point::Position {
    [
        normalise_coord(coords.x.get()),
        normalise_coord(coords.y.get()),
    ]
}

fn rgb_from_ilda(c: layout::Color) -> point::Rgb {
    [
        normalise_color(c.red),
        normalise_color(c.green),
        normalise_color(c.blue),
    ]
}

const BLACK: [f32; 3] = [0.0; 3];

fn point_from_coords_3d_indexed_color(
    p: layout::Coords3dIndexedColor,
    palette: &[layout::Color],
) -> Point {
    let position = coords3d_to_position(p.coords);
    let color = match p.status.is_blanking() {
        true => BLACK,
        false => rgb_from_ilda(palette[p.color_index as usize]),
    };
    Point::new(position, color)
}

fn point_from_coords_2d_indexed_color(
    p: layout::Coords2dIndexedColor,
    palette: &[layout::Color],
) -> Point {
    let position = coords2d_to_position(p.coords);
    let color = match p.status.is_blanking() {
        true => BLACK,
        false => rgb_from_ilda(palette[p.color_index as usize]),
    };
    Point::new(position, color)
}

fn point_from_coords_3d_true_color(p: layout::Coords3dTrueColor) -> Point {
    let position = coords3d_to_position(p.coords);
    let color = match p.status.is_blanking() {
        true => BLACK,
        false => rgb_from_ilda(p.color),
    };
    Point::new(position, color)
}

fn point_from_coords_2d_true_color(p: layout::Coords2dTrueColor) -> Point {
    let position = coords2d_to_position(p.coords);
    let color = match p.status.is_blanking() {
        true => BLACK,
        false => rgb_from_ilda(p.color),
    };
    Point::new(position, color)
}
