//! Logic related to the positioning of the cursor within text.

use crate::text::{self, FontSize, Point, Scalar};
use nannou_core::geom::{Range, Rect};

/// An index representing the position of a cursor within some text.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Index {
    /// The byte index of the line upon which the cursor is situated.
    pub line: usize,
    /// The index within all possible cursor positions for the line.
    ///
    /// For example, for the line `foo`, a `char` of `1` would indicate the cursor's position
    /// as `f|oo` where `|` is the cursor.
    pub char: usize,
}

/// Every possible cursor position within each line of text yielded by the given iterator.
///
/// Yields `(xs, y_range)`, where `y_range` is the `Range` occupied by the line across the *y*
/// axis and `xs` is every possible cursor position along the *x* axis
#[derive(Clone)]
pub struct XysPerLine<'a, I> {
    lines_with_rects: I,
    font: &'a text::Font,
    text: &'a str,
    font_size: FontSize,
}

/// Similarly to `XysPerLine`, yields every possible cursor position within each line of text
/// yielded by the given iterator.
///
/// Rather than taking an iterator type yielding lines and positioning data, this method
/// constructs its own iterator to do so internally, saving some boilerplate involved in common
/// `XysPerLine` use cases.
///
/// Yields `(xs, y_range)`, where `y_range` is the `Range` occupied by the line across the *y*
/// axis and `xs` is every possible cursor position along the *x* axis.
#[derive(Clone)]
pub struct XysPerLineFromText<'a> {
    xys_per_line: XysPerLine<
        'a,
        std::iter::Zip<
            std::iter::Cloned<std::slice::Iter<'a, text::line::Info>>,
            text::line::Rects<std::iter::Cloned<std::slice::Iter<'a, text::line::Info>>>,
        >,
    >,
}

/// Each possible cursor position along the *x* axis within a line of text.
///
/// `Xs` iterators are produced by the `XysPerLine` iterator.
pub struct Xs<'a, 'b> {
    next_x: Option<Scalar>,
    layout: text::LayoutIter<'a, 'b>,
}

impl Index {
    /// The cursor index of the beginning of the word (block of non-whitespace) before `self`.
    ///
    /// If `self` is at the beginning of the line, call previous, which returns the last
    /// index position of the previous line, or None if it's the first line
    ///
    /// If `self` points to whitespace, skip past that whitespace, then return the index of
    /// the start of the word that precedes the whitespace
    ///
    /// If `self` is in the middle or end of a word, return the index of the start of that word
    pub fn previous_word_start<I>(self, text: &str, mut line_infos: I) -> Option<Self>
    where
        I: Iterator<Item = text::line::Info>,
    {
        let Index { line, char } = self;
        if char > 0 {
            line_infos.nth(line).and_then(|line_info| {
                let line_count = line_info.char_range().count();
                let mut chars_rev = (&text[line_info.byte_range()]).chars().rev();
                if char != line_count {
                    chars_rev.nth(line_count - char - 1);
                }
                let mut new_char = 0;
                let mut hit_non_whitespace = false;
                for (i, char_) in chars_rev.enumerate() {
                    // loop until word starts, then continue until the word ends
                    if !char_.is_whitespace() {
                        hit_non_whitespace = true;
                    }
                    if char_.is_whitespace() && hit_non_whitespace {
                        new_char = char - i;
                        break;
                    }
                }
                Some(Index {
                    line: line,
                    char: new_char,
                })
            })
        } else {
            self.previous(line_infos)
        }
    }

    /// The cursor index of the end of the first word (block of non-whitespace) after `self`.
    ///
    /// If `self` is at the end of the text, this returns `None`.
    ///
    /// If `self` is at the end of a line other than the last, this returns the first index of
    /// the next line.
    ///
    /// If `self` points to whitespace, skip past that whitespace, then return the index of
    /// the end of the word after the whitespace
    ///
    /// If `self` is in the middle or start of a word, return the index of the end of that word
    pub fn next_word_end<I>(self, text: &str, mut line_infos: I) -> Option<Self>
    where
        I: Iterator<Item = text::line::Info>,
    {
        let Index { line, char } = self;
        line_infos.nth(line).and_then(|line_info| {
            let line_count = line_info.char_range().count();
            if char < line_count {
                let mut chars = (&text[line_info.byte_range()]).chars();
                let mut new_char = line_count;
                let mut hit_non_whitespace = false;
                if char != 0 {
                    chars.nth(char - 1);
                }
                for (i, char_) in chars.enumerate() {
                    // loop until word starts, then continue until the word ends
                    if !char_.is_whitespace() {
                        hit_non_whitespace = true;
                    }
                    if char_.is_whitespace() && hit_non_whitespace {
                        new_char = char + i;
                        break;
                    }
                }
                Some(Index {
                    line: line,
                    char: new_char,
                })
            } else {
                line_infos.next().map(|_| Index {
                    line: line + 1,
                    char: 0,
                })
            }
        })
    }

    /// The cursor index that comes before `self`.
    ///
    /// If `self` is at the beginning of the text, this returns `None`.
    ///
    /// If `self` is at the beginning of a line other than the first, this returns the last
    /// index position of the previous line.
    ///
    /// If `self` is a position other than the start of a line, it will return the position
    /// that is immediately to the left.
    pub fn previous<I>(self, mut line_infos: I) -> Option<Self>
    where
        I: Iterator<Item = text::line::Info>,
    {
        let Index { line, char } = self;
        if char > 0 {
            let new_char = char - 1;
            line_infos.nth(line).and_then(|info| {
                if new_char <= info.char_range().count() {
                    Some(Index {
                        line: line,
                        char: new_char,
                    })
                } else {
                    None
                }
            })
        } else if line > 0 {
            let new_line = line - 1;
            line_infos.nth(new_line).map(|info| {
                let new_char = info.end_char() - info.start_char;
                Index {
                    line: new_line,
                    char: new_char,
                }
            })
        } else {
            None
        }
    }

    /// The cursor index that follows `self`.
    ///
    /// If `self` is at the end of the text, this returns `None`.
    ///
    /// If `self` is at the end of a line other than the last, this returns the first index of
    /// the next line.
    ///
    /// If `self` is a position other than the end of a line, it will return the position that
    /// is immediately to the right.
    pub fn next<I>(self, mut line_infos: I) -> Option<Self>
    where
        I: Iterator<Item = text::line::Info>,
    {
        let Index { line, char } = self;
        line_infos.nth(line).and_then(|info| {
            if char >= info.char_range().count() {
                line_infos.next().map(|_| Index {
                    line: line + 1,
                    char: 0,
                })
            } else {
                Some(Index {
                    line: line,
                    char: char + 1,
                })
            }
        })
    }

    /// Clamps `self` to the given lines.
    ///
    /// If `self` would lie after the end of the last line, return the index at the end of the
    /// last line.
    ///
    /// If `line_infos` is empty, returns cursor at line=0 char=0.
    pub fn clamp_to_lines<I>(self, line_infos: I) -> Self
    where
        I: Iterator<Item = text::line::Info>,
    {
        let mut last = None;
        for (i, info) in line_infos.enumerate() {
            if i == self.line {
                let num_chars = info.char_range().len();
                let char = std::cmp::min(self.char, num_chars);
                return Index {
                    line: i,
                    char: char,
                };
            }
            last = Some((i, info));
        }
        match last {
            Some((i, info)) => Index {
                line: i,
                char: info.char_range().len(),
            },
            None => Index { line: 0, char: 0 },
        }
    }
}

/// Every possible cursor position within each line of text yielded by the given iterator.
///
/// Yields `(xs, y_range)`, where `y_range` is the `Range` occupied by the line across the *y*
/// axis and `xs` is every possible cursor position along the *x* axis
pub fn xys_per_line<'a, I>(
    lines_with_rects: I,
    font: &'a text::Font,
    text: &'a str,
    font_size: FontSize,
) -> XysPerLine<'a, I> {
    XysPerLine {
        lines_with_rects: lines_with_rects,
        font: font,
        text: text,
        font_size: font_size,
    }
}

/// Similarly to `xys_per_line`, this produces an iterator yielding every possible cursor
/// position within each line of text yielded by the given iterator.
///
/// Rather than taking an iterator yielding lines and their positioning data, this method
/// constructs its own iterator to do so internally, saving some boilerplate involved in common
/// `xys_per_line` use cases.
///
/// Yields `(xs, y_range)`, where `y_range` is the `Range` occupied by the line across the *y*
/// axis and `xs` is every possible cursor position along the *x* axis.
pub fn xys_per_line_from_text<'a>(
    text: &'a str,
    line_infos: &'a [text::line::Info],
    font: &'a text::Font,
    font_size: FontSize,
    max_width: Scalar,
    x_align: text::Justify,
    line_spacing: Scalar,
) -> XysPerLineFromText<'a> {
    let line_infos = line_infos.iter().cloned();
    let line_rects = text::line::rects(
        line_infos.clone(),
        font_size,
        max_width,
        x_align,
        line_spacing,
    );
    let lines = line_infos.clone();
    let lines_with_rects = lines.zip(line_rects.clone());
    XysPerLineFromText {
        xys_per_line: text::cursor::xys_per_line(lines_with_rects, font, text, font_size),
    }
}

/// Convert the given character index into a cursor `Index`.
pub fn index_before_char<I>(line_infos: I, char_index: usize) -> Option<Index>
where
    I: Iterator<Item = text::line::Info>,
{
    for (i, line_info) in line_infos.enumerate() {
        let start_char = line_info.start_char;
        let end_char = line_info.end_char();
        if start_char <= char_index && char_index <= end_char {
            return Some(Index {
                line: i,
                char: char_index - start_char,
            });
        }
    }
    None
}

/// Determine the *xy* location of the cursor at the given cursor `Index`.
pub fn xy_at<'a, I>(xys_per_line: I, idx: Index) -> Option<(Scalar, Range)>
where
    I: Iterator<Item = (Xs<'a, 'a>, Range)>,
{
    for (i, (xs, y)) in xys_per_line.enumerate() {
        if i == idx.line {
            for (j, x) in xs.enumerate() {
                if j == idx.char {
                    return Some((x, y));
                }
            }
        }
    }
    None
}

/// Find the closest line for the given `y` position, and return the line index, Xs iterator, and y-range of that line
///
/// Returns `None` if there are no lines
pub fn closest_line<'a, I>(y_pos: Scalar, xys_per_line: I) -> Option<(usize, Xs<'a, 'a>, Range)>
where
    I: Iterator<Item = (Xs<'a, 'a>, Range)>,
{
    let mut xys_per_line_enumerated = xys_per_line.enumerate();
    xys_per_line_enumerated
        .next()
        .and_then(|(first_line_idx, (first_line_xs, first_line_y))| {
            let mut closest_line = (first_line_idx, first_line_xs, first_line_y);
            let mut closest_diff = (y_pos - first_line_y.middle()).abs();
            for (line_idx, (line_xs, line_y)) in xys_per_line_enumerated {
                if line_y.contains(y_pos) {
                    closest_line = (line_idx, line_xs, line_y);
                    break;
                } else {
                    let diff = (y_pos - line_y.middle()).abs();
                    if diff < closest_diff {
                        closest_line = (line_idx, line_xs, line_y);
                        closest_diff = diff;
                    } else {
                        break;
                    }
                }
            }
            Some(closest_line)
        })
}

/// Find the closest cursor index to the given `xy` position, and the center `Point` of that
/// cursor.
///
/// Returns `None` if the given `text` is empty.
pub fn closest_cursor_index_and_xy<'a, I>(xy: Point, xys_per_line: I) -> Option<(Index, Point)>
where
    I: Iterator<Item = (Xs<'a, 'a>, Range)>,
{
    closest_line(xy[1], xys_per_line).and_then(
        |(closest_line_idx, closest_line_xs, closest_line_y)| {
            let (closest_char_idx, closest_x) =
                closest_cursor_index_on_line(xy[0], closest_line_xs);
            let index = Index {
                line: closest_line_idx,
                char: closest_char_idx,
            };
            let point = [closest_x, closest_line_y.middle()].into();
            Some((index, point))
        },
    )
}

/// Find the closest cursor index to the given `x` position on the given line along with the
/// `x` position of that cursor.
pub fn closest_cursor_index_on_line<'a>(x_pos: Scalar, line_xs: Xs<'a, 'a>) -> (usize, Scalar) {
    let mut xs_enumerated = line_xs.enumerate();
    // `xs` always yields at least one `x` (the start of the line).
    let (first_idx, first_x) = xs_enumerated.next().unwrap();
    let first_diff = (x_pos - first_x).abs();
    let mut closest = (first_idx, first_x);
    let mut closest_diff = first_diff;
    for (i, x) in xs_enumerated {
        let diff = (x_pos - x).abs();
        if diff < closest_diff {
            closest = (i, x);
            closest_diff = diff;
        } else {
            break;
        }
    }
    closest
}

impl<'a, I> Iterator for XysPerLine<'a, I>
where
    I: Iterator<Item = (text::line::Info, Rect)>,
{
    // The `Range` occupied by the line across the *y* axis, along with an iterator yielding
    // each possible cursor position along the *x* axis.
    type Item = (Xs<'a, 'a>, Range);
    fn next(&mut self) -> Option<Self::Item> {
        let XysPerLine {
            ref mut lines_with_rects,
            font,
            text,
            font_size,
        } = *self;
        let scale = text::pt_to_scale(font_size);
        lines_with_rects.next().map(|(line_info, line_rect)| {
            let line = &text[line_info.byte_range()];
            let (x, y) = (line_rect.left() as f32, line_rect.top() as f32);
            let point = text::rt::Point { x: x, y: y };
            let y = line_rect.y;
            let layout = font.layout(line, scale, point);
            let xs = Xs {
                next_x: Some(line_rect.x.start),
                layout: layout,
            };
            (xs, y)
        })
    }
}

impl<'a> Iterator for XysPerLineFromText<'a> {
    type Item = (Xs<'a, 'a>, Range);
    fn next(&mut self) -> Option<Self::Item> {
        self.xys_per_line.next()
    }
}

impl<'a, 'b> Iterator for Xs<'a, 'b> {
    // Each possible cursor position along the *x* axis.
    type Item = Scalar;
    fn next(&mut self) -> Option<Self::Item> {
        self.next_x.map(|x| {
            self.next_x = self.layout.next().map(|g| {
                g.pixel_bounding_box()
                    .map(|r| r.max.x as Scalar)
                    .unwrap_or_else(|| x + g.unpositioned().h_metrics().advance_width as Scalar)
            });
            x
        })
    }
}
