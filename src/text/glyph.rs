//! Logic and types specific to individual glyph layout.

use crate::geom::{Range, Rect};
use crate::text::{self, FontSize, Scalar};

/// Some position along the X axis (used within `CharXs`).
pub type X = Scalar;

/// The half of the width of some character (used within `CharXs`).
pub type HalfW = Scalar;

/// An iterator yielding the `Rect` for each `char`'s `Glyph` in the given `text`.
pub struct Rects<'a, 'b> {
    /// The *y* axis `Range` of the `Line` for which character `Rect`s are being yielded.
    ///
    /// Every yielded `Rect` will use this as its `y` `Range`.
    y: Range,
    /// The position of the next `Rect`'s left edge along the *x* axis.
    next_left: Scalar,
    /// `PositionedGlyphs` yielded by the RustType `LayoutIter`.
    layout: text::LayoutIter<'a, 'b>,
}

/// An iterator that, for every `(line, line_rect)` pair yielded by the given iterator,
/// produces an iterator that yields a `Rect` for every character in that line.
pub struct RectsPerLine<'a, I> {
    lines_with_rects: I,
    font: &'a text::Font,
    font_size: FontSize,
}

/// Yields a `Rect` for each selected character in a single line of text.
///
/// This iterator can only be produced by the `SelectedCharRectsPerLine` iterator.
pub struct SelectedRects<'a, 'b> {
    enumerated_rects: std::iter::Enumerate<Rects<'a, 'b>>,
    end_char_idx: usize,
}

/// Yields an iteraor yielding `Rect`s for each selected character in each line of text within
/// the given iterator yielding char `Rect`s.
///
/// Given some `start` and `end` indices, only `Rect`s for `char`s between these two indices
/// will be produced.
///
/// All lines that have no selected `Rect`s will be skipped.
pub struct SelectedRectsPerLine<'a, I> {
    enumerated_rects_per_line: std::iter::Enumerate<RectsPerLine<'a, I>>,
    start_cursor_idx: text::cursor::Index,
    end_cursor_idx: text::cursor::Index,
}

impl<'a, 'b> Iterator for Rects<'a, 'b> {
    type Item = Rect;
    fn next(&mut self) -> Option<Self::Item> {
        let Rects {
            ref mut next_left,
            ref mut layout,
            y,
        } = *self;
        layout.next().map(|g| {
            let left = *next_left;
            let right = g
                .pixel_bounding_box()
                .map(|bb| bb.max.x as Scalar)
                .unwrap_or_else(|| left + g.unpositioned().h_metrics().advance_width as Scalar);
            *next_left = right;
            let x = Range::new(left, right);
            Rect { x: x, y: y }
        })
    }
}

impl<'a, I> Iterator for RectsPerLine<'a, I>
where
    I: Iterator<Item = (&'a str, Rect)>,
{
    type Item = Rects<'a, 'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let RectsPerLine {
            ref mut lines_with_rects,
            font,
            font_size,
        } = *self;
        let scale = text::pt_to_scale(font_size);
        lines_with_rects.next().map(|(line, line_rect)| {
            let (x, y) = (line_rect.left() as f32, line_rect.top() as f32);
            let point = text::rt::Point { x: x, y: y };
            Rects {
                next_left: line_rect.x.start,
                layout: font.layout(line, scale, point),
                y: line_rect.y,
            }
        })
    }
}

impl<'a, 'b> Iterator for SelectedRects<'a, 'b> {
    type Item = Rect;
    fn next(&mut self) -> Option<Self::Item> {
        let SelectedRects {
            ref mut enumerated_rects,
            end_char_idx,
        } = *self;
        enumerated_rects.next().and_then(
            |(i, rect)| {
                if i < end_char_idx {
                    Some(rect)
                } else {
                    None
                }
            },
        )
    }
}

impl<'a, I> Iterator for SelectedRectsPerLine<'a, I>
where
    I: Iterator<Item = (&'a str, Rect)>,
{
    type Item = SelectedRects<'a, 'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let SelectedRectsPerLine {
            ref mut enumerated_rects_per_line,
            start_cursor_idx,
            end_cursor_idx,
        } = *self;

        enumerated_rects_per_line.next().map(|(i, rects)| {
            let end_char_idx =
                // If this is the last line, the end is the char after the final selected char.
                if i == end_cursor_idx.line {
                    end_cursor_idx.char
                // Otherwise if in range, every char in the line is selected.
                } else if start_cursor_idx.line <= i && i < end_cursor_idx.line {
                    std::u32::MAX as usize
                // Otherwise if out of range, no chars are selected.
                } else {
                    0
                };

            let mut enumerated_rects = rects.enumerate();

            // If this is the first line, skip all non-selected chars.
            if i == start_cursor_idx.line {
                for _ in 0..start_cursor_idx.char {
                    enumerated_rects.next();
                }
            }

            SelectedRects {
                enumerated_rects: enumerated_rects,
                end_char_idx: end_char_idx,
            }
        })
    }
}

/// Produce an iterator that, for every `(line, line_rect)` pair yielded by the given iterator,
/// produces an iterator that yields a `Rect` for every character in that line.
///
/// This is useful when information about character positioning is needed when reasoning about
/// text layout.
pub fn rects_per_line<'a, I>(
    lines_with_rects: I,
    font: &'a text::Font,
    font_size: FontSize,
) -> RectsPerLine<'a, I>
where
    I: Iterator<Item = (&'a str, Rect)>,
{
    RectsPerLine {
        lines_with_rects: lines_with_rects,
        font: font,
        font_size: font_size,
    }
}

/// Find the index of the character that directly follows the cursor at the given `cursor_idx`.
///
/// Returns `None` if either the given `cursor::Index` `line` or `idx` fields are out of bounds
/// of the line information yielded by the `line_infos` iterator.
pub fn index_after_cursor<I>(mut line_infos: I, cursor_idx: text::cursor::Index) -> Option<usize>
where
    I: Iterator<Item = text::line::Info>,
{
    line_infos.nth(cursor_idx.line).and_then(|line_info| {
        let start_char = line_info.start_char;
        let end_char = line_info.end_char();
        let char_index = start_char + cursor_idx.char;
        if char_index <= end_char {
            Some(char_index)
        } else {
            None
        }
    })
}

/// Produces an iterator that yields iteraors yielding `Rect`s for each selected character in
/// each line of text within the given iterator yielding char `Rect`s.
///
/// Given some `start` and `end` indices, only `Rect`s for `char`s between these two indices
/// will be produced.
///
/// All lines that have no selected `Rect`s will be skipped.
pub fn selected_rects_per_line<'a, I>(
    lines_with_rects: I,
    font: &'a text::Font,
    font_size: FontSize,
    start: text::cursor::Index,
    end: text::cursor::Index,
) -> SelectedRectsPerLine<'a, I>
where
    I: Iterator<Item = (&'a str, Rect)>,
{
    SelectedRectsPerLine {
        enumerated_rects_per_line: rects_per_line(lines_with_rects, font, font_size).enumerate(),
        start_cursor_idx: start,
        end_cursor_idx: end,
    }
}
