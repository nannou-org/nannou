//! Text layout logic.
//!
//! Currently, this crate is used primarily by the `draw.text()` API but will also play an
//! important role in future GUI work.

pub mod cursor;
pub mod font;
pub mod glyph;
pub mod line;
pub mod rt {
    //! Re-exported RustType geometric types.
    pub use rusttype::{gpu_cache, point, vector, Point, Rect, Vector};
}

// Re-export all relevant rusttype types here.
pub use rusttype::gpu_cache::Cache as GlyphCache;
pub use rusttype::{Glyph, GlyphId, GlyphIter, LayoutIter, Scale};

/// The RustType `FontCollection` type used by conrod.
pub type FontCollection = rusttype::FontCollection<'static>;
/// The RustType `Font` type used by conrod.
pub type Font = rusttype::Font<'static>;
/// The RustType `PositionedGlyph` type used by conrod.
pub type PositionedGlyph = rusttype::PositionedGlyph<'static>;

/// The type used for scalar values.
pub type Scalar = crate::geom::scalar::Default;

/// The point type used when working with text.
pub type Point<S = Scalar> = crate::geom::Point2<S>;

/// The type used to specify `FontSize` in font points.
pub type FontSize = u32;

/// An iterator yielding each line within the given `text` as a new `&str`, where the start and end
/// indices into each line are provided by the given iterator.
#[derive(Clone)]
pub struct Lines<'a, I> {
    text: &'a str,
    ranges: I,
}

/// Alignment along an axis.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum Align {
    Start,
    Middle,
    End,
}

/// A type used for referring to typographic alignment of `Text`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Justify {
    /// Align text to the start of the bounding `Rect`'s *x* axis.
    Left,
    /// Symmetrically align text along the *y* axis.
    Center,
    /// Align text to the end of the bounding `Rect`'s *x* axis.
    Right,
    // /// Align wrapped text to both the start and end of the bounding `Rect`s *x* axis.
    // ///
    // /// Extra space is added between words in order to achieve this alignment.
    // TODO: Full,
}

impl<'a, I> Iterator for Lines<'a, I>
where
    I: Iterator<Item = std::ops::Range<usize>>,
{
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        let Lines {
            text,
            ref mut ranges,
        } = *self;
        ranges.next().map(|range| &text[range])
    }
}

/// Determine the total height of a block of text with the given number of lines, font size and
/// `line_spacing` (the space that separates each line of text).
pub fn height(num_lines: usize, font_size: FontSize, line_spacing: Scalar) -> Scalar {
    if num_lines > 0 {
        num_lines as Scalar * font_size as Scalar + (num_lines - 1) as Scalar * line_spacing
    } else {
        0.0
    }
}

/// Produce an iterator yielding each line within the given `text` as a new `&str`, where the
/// start and end indices into each line are provided by the given iterator.
pub fn lines<I>(text: &str, ranges: I) -> Lines<I>
where
    I: Iterator<Item = std::ops::Range<usize>>,
{
    Lines {
        text: text,
        ranges: ranges,
    }
}

/// Converts the given font size in "points" to its font size in pixels.
/// This is useful for when the font size is not an integer.
pub fn f32_pt_to_px(font_size_in_points: f32) -> f32 {
    font_size_in_points * 4.0 / 3.0
}

/// Converts the given font size in "points" to a uniform `rusttype::Scale`.
/// This is useful for when the font size is not an integer.
pub fn f32_pt_to_scale(font_size_in_points: f32) -> Scale {
    Scale::uniform(f32_pt_to_px(font_size_in_points))
}

/// Converts the given font size in "points" to its font size in pixels.
pub fn pt_to_px(font_size_in_points: FontSize) -> f32 {
    f32_pt_to_px(font_size_in_points as f32)
}

/// Converts the given font size in "points" to a uniform `rusttype::Scale`.
pub fn pt_to_scale(font_size_in_points: FontSize) -> Scale {
    Scale::uniform(pt_to_px(font_size_in_points))
}
