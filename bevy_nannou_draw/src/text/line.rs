//! Text handling logic related to individual lines of text.
//!
//! This module is the core of multi-line text handling.

use nannou_core::geom::{Range, Rect};

use crate::text::{self, FontSize, Scalar, Wrap};

/// The two types of **Break** indices returned by the **WrapIndicesBy** iterators.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Break {
    /// A break caused by the text exceeding some maximum width.
    Wrap {
        /// The byte index at which the break occurs.
        byte: usize,
        /// The char index at which the string should wrap due to exceeding a maximum width.
        char: usize,
        /// The byte length which should be skipped in order to reach the first non-whitespace
        /// character to use as the beginning of the next line.
        len_bytes: usize,
        /// The number of chars which should be skipped in order to reach the first non-whitespace
        /// character to use as the beginning of the next line.
        len_chars: usize,
    },
    /// A break caused by a newline character.
    Newline {
        /// The byte index at which the string should wrap due to exceeding a maximum width.
        byte: usize,
        /// The char index at which the string should wrap due to exceeding a maximum width.
        char: usize,
        /// The width of the "newline" token in bytes.
        len_bytes: usize,
        /// The width of the "newline" token in chars.
        len_chars: usize,
    },
    /// The end of the string has been reached, with the given length.
    End {
        /// The ending byte index.
        byte: usize,
        /// The ending char index.
        char: usize,
    },
}

/// The type yielded by functions dedicated to finding the next line break.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct NextBreak {
    /// The reason for the break.
    pub break_: Break,
    /// The total width of the line.
    pub width: Scalar,
    /// The maximum heigh of the line.
    pub height: Scalar,
}

/// Information about a single line of text within a `&str`.
///
/// `Info` is a minimal amount of information that can be stored for efficient reasoning about
/// blocks of text given some `&str`. The `start` and `end_break` can be used for indexing into
/// the `&str`, and the `width` can be used for calculating line `Rect`s, alignment, etc.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Info {
    /// The index into the `&str` that represents the first character within the line.
    pub start_byte: usize,
    /// The character index into the `&str` of the first character in the line.
    pub start_char: usize,
    /// The index within the `&str` at which this line breaks into a new line, along with the
    /// index at which the following line begins. The variant describes whether the break is
    /// caused by a `Newline` character or a `Wrap` by the given wrap function.
    pub end_break: Break,
    /// The total width of all characters within the line.
    pub width: Scalar,
    /// The greatest height of all characters yielded.
    pub height: Scalar,
}

/// An iterator yielding an `Info` struct for each line in the given `text` wrapped by the
/// given `next_break_fn`.
///
/// `Infos` is a fundamental part of performing lazy reasoning about text within nannou.
///
/// Construct an `Infos` iterator via the [infos function](./fn.infos.html) and its two builder
/// methods, [wrap_by_character](./struct.Infos.html#method.wrap_by_character) and
/// [wrap_by_whitespace](./struct.Infos.html#method.wrap_by_whitespace).
pub struct Infos<'a, F> {
    text: &'a str,
    font: &'a text::Font,
    font_size: FontSize,
    max_width: Scalar,
    next_break_fn: F,
    /// The index that indicates the start of the next line to be yielded.
    start_byte: usize,
    /// The character index that indicates the start of the next line to be yielded.
    start_char: usize,
    /// The break type of the previously yielded line
    last_break: Option<Break>,
}

/// An iterator yielding a `Rect` for each line in
#[derive(Clone)]
pub struct Rects<I> {
    infos: I,
    x_align: text::Justify,
    line_spacing: Scalar,
    last_line_top: Scalar,
    font_size: FontSize,
    next: Option<Rect>,
}

/// An iterator yielding a `Rect` for each selected line in a block of text.
///
/// The yielded `Rect`s represent the selected range within each line of text.
///
/// Lines that do not contain any selected text will be skipped.
pub struct SelectedRects<'a, I> {
    selected_char_rects_per_line: text::glyph::SelectedRectsPerLine<'a, I>,
}

/// An alias for function pointers that are compatible with the `Block`'s required text
/// wrapping function.
pub type NextBreakFnPtr = fn(&str, &text::Font, FontSize, Scalar) -> NextBreak;

impl Break {
    /// Return the index at which the break occurs.
    pub fn byte_index(self) -> usize {
        match self {
            Break::Wrap { byte, .. } | Break::Newline { byte, .. } | Break::End { byte, .. } => {
                byte
            }
        }
    }

    /// Return the index of the `char` at which the break occurs.
    ///
    /// To clarify, this index is to be used in relation to the `Chars` iterator.
    pub fn char_index(self) -> usize {
        match self {
            Break::Wrap { char, .. } | Break::Newline { char, .. } | Break::End { char, .. } => {
                char
            }
        }
    }
}

impl<'a, F> Clone for Infos<'a, F>
where
    F: Clone,
{
    fn clone(&self) -> Self {
        Infos {
            text: self.text,
            font: self.font,
            font_size: self.font_size,
            max_width: self.max_width,
            next_break_fn: self.next_break_fn.clone(),
            start_byte: self.start_byte,
            start_char: self.start_char,
            last_break: None,
        }
    }
}

impl Info {
    /// The end of the byte index range for indexing into the slice.
    pub fn end_byte(&self) -> usize {
        self.end_break.byte_index()
    }

    /// The end of the index range for indexing into the slice.
    pub fn end_char(&self) -> usize {
        self.end_break.char_index()
    }

    /// The index range for indexing (via bytes) into the original str slice.
    pub fn byte_range(self) -> std::ops::Range<usize> {
        self.start_byte..self.end_byte()
    }

    /// The index range for indexing into a `char` iterator over the original str slice.
    pub fn char_range(self) -> std::ops::Range<usize> {
        self.start_char..self.end_char()
    }
}

impl<'a> Infos<'a, NextBreakFnPtr> {
    /// Converts `Self` into an `Infos` whose lines are wrapped at the character that first
    /// causes the line width to exceed the given `max_width`.
    pub fn wrap_by_character(mut self, max_width: Scalar) -> Self {
        self.next_break_fn = next_break_by_character;
        self.max_width = max_width;
        self
    }

    /// Converts `Self` into an `Infos` whose lines are wrapped at the whitespace prior to the
    /// character that causes the line width to exceed the given `max_width`.
    pub fn wrap_by_whitespace(mut self, max_width: Scalar) -> Self {
        self.next_break_fn = next_break_by_whitespace;
        self.max_width = max_width;
        self
    }
}

/// A function for finding the advance width between the given character that also considers
/// the kerning for some previous glyph.
///
/// This also updates the `last_glyph` with the glyph produced for the given `char`.
///
/// This is primarily for use within the `next_break` functions below.
///
/// The following code is adapted from the rusttype::LayoutIter::next src.
fn advance_width_and_height(
    ch: char,
    font: &text::Font,
    scale: text::Scale,
    last_glyph: &mut Option<text::GlyphId>,
) -> (Scalar, Scalar) {
    let g = font.glyph(ch).scaled(scale);
    let kern = last_glyph
        .map(|last| font.pair_kerning(scale, last, g.id()))
        .unwrap_or(0.0);
    let advance_width = g.h_metrics().advance_width;
    let height = g
        .exact_bounding_box()
        .map(|bb| bb.min.y.abs() as Scalar)
        .unwrap_or(0.0);
    *last_glyph = Some(g.id());
    let adv_w = (kern + advance_width) as Scalar;
    (adv_w, height)
}

/// Returns the next index at which the text naturally breaks via a newline character,
/// along with the width of the line.
fn next_break(text: &str, font: &text::Font, font_size: FontSize) -> NextBreak {
    let scale = text::pt_to_scale(font_size);
    let mut width = 0.0;
    let mut height = 0.0;
    let mut char_i = 0;
    let mut char_indices = text.char_indices().peekable();
    let mut last_glyph = None;
    while let Some((byte_i, ch)) = char_indices.next() {
        // Check for a newline.
        if ch == '\r' {
            if let Some(&(_, '\n')) = char_indices.peek() {
                let break_ = Break::Newline {
                    byte: byte_i,
                    char: char_i,
                    len_bytes: 2, // skip "\r\n"
                    len_chars: 2,
                };
                return NextBreak {
                    break_,
                    width,
                    height,
                };
            }
        } else if ch == '\n' {
            let break_ = Break::Newline {
                byte: byte_i,
                char: char_i,
                len_bytes: 1, // skip "\n"
                len_chars: 1,
            };
            return NextBreak {
                break_,
                width,
                height,
            };
        }

        // Update the width.
        let (adv_w, h) = advance_width_and_height(ch, font, scale, &mut last_glyph);
        width += adv_w;
        height = height.max(h);
        char_i += 1;
    }
    let break_ = Break::End {
        byte: text.len(),
        char: char_i,
    };
    NextBreak {
        break_,
        width,
        height,
    }
}

/// Returns the next index at which the text will break by either:
/// - A newline character.
/// - A line wrap at the beginning of the first character exceeding the `max_width`.
///
/// Also returns the width of each line alongside the Break.
fn next_break_by_character(
    text: &str,
    font: &text::Font,
    font_size: FontSize,
    max_width: Scalar,
) -> NextBreak {
    let scale = text::pt_to_scale(font_size);
    let mut width = 0.0;
    let mut height = 0.0;
    let mut char_i = 0;
    let mut char_indices = text.char_indices().peekable();
    let mut last_glyph = None;
    while let Some((byte_i, ch)) = char_indices.next() {
        // Check for a newline.
        if ch == '\r' {
            if let Some(&(_, '\n')) = char_indices.peek() {
                let break_ = Break::Newline {
                    byte: byte_i,
                    char: char_i,
                    len_bytes: 2, // skip "\r\n"
                    len_chars: 2,
                };
                return NextBreak {
                    break_,
                    width,
                    height,
                };
            }
        } else if ch == '\n' {
            let break_ = Break::Newline {
                byte: byte_i,
                char: char_i,
                len_bytes: 1, // skip "\n"
                len_chars: 1,
            };
            return NextBreak {
                break_,
                width,
                height,
            };
        }

        // Add the character's width to the width so far.
        let (adv_w, h) = advance_width_and_height(ch, font, scale, &mut last_glyph);
        let new_width = width + adv_w;

        // Check for a line wrap.
        if new_width > max_width {
            let break_ = Break::Wrap {
                byte: byte_i,
                char: char_i,
                len_bytes: 0, // skip nothing in middle of word
                len_chars: 0,
            };
            return NextBreak {
                break_,
                width,
                height,
            };
        }

        height = height.max(h);
        width = new_width;
        char_i += 1;
    }

    let break_ = Break::End {
        byte: text.len(),
        char: char_i,
    };
    NextBreak {
        break_,
        width,
        height,
    }
}

/// Returns the next index at which the text will break by either:
/// - A newline character.
/// - A line wrap at the beginning of the whitespace that preceeds the first word
/// exceeding the `max_width`.
/// - A line wrap at the beginning of the first character exceeding the `max_width`,
/// if no whitespace appears for `max_width` characters.
///
/// Also returns the width the line alongside the Break.
fn next_break_by_whitespace(
    text: &str,
    font: &text::Font,
    font_size: FontSize,
    max_width: Scalar,
) -> NextBreak {
    struct Last {
        byte: usize,
        char: usize,
        width_before: Scalar,
    }
    let scale = text::pt_to_scale(font_size);
    let mut last_whitespace_start = None;
    let mut width = 0.0;
    let mut height = 0.0;
    let mut char_i = 0;
    let mut char_indices = text.char_indices().peekable();
    let mut last_glyph = None;
    while let Some((byte_i, ch)) = char_indices.next() {
        // Check for a newline.
        if ch == '\r' {
            if let Some(&(_, '\n')) = char_indices.peek() {
                let break_ = Break::Newline {
                    byte: byte_i,
                    char: char_i,
                    len_bytes: 2, // skip "\r\n"
                    len_chars: 2,
                };
                return NextBreak {
                    break_,
                    width,
                    height,
                };
            }
        } else if ch == '\n' {
            let break_ = Break::Newline {
                byte: byte_i,
                char: char_i,
                len_bytes: 1, // skip "\n"
                len_chars: 1,
            };
            return NextBreak {
                break_,
                width,
                height,
            };
        }

        // Add the character's width to the width so far.
        let (adv_w, h) = advance_width_and_height(ch, font, scale, &mut last_glyph);
        let new_width = width + adv_w;

        // Check for a line wrap.
        if width > max_width {
            match last_whitespace_start {
                Some(Last {
                    byte,
                    char,
                    width_before,
                }) => {
                    let break_ = Break::Wrap {
                        byte: byte,
                        char: char,
                        len_bytes: 1, // skip one whitespace character
                        len_chars: 1,
                    };
                    let width = width_before;
                    return NextBreak {
                        break_,
                        width,
                        height,
                    };
                }
                None => {
                    let break_ = Break::Wrap {
                        byte: byte_i,
                        char: char_i,
                        len_bytes: 0, // skip nothing in middle of word
                        len_chars: 0,
                    };
                    return NextBreak {
                        break_,
                        width,
                        height,
                    };
                }
            }
        }

        // Check for a new whitespace.
        if ch.is_whitespace() {
            last_whitespace_start = Some(Last {
                byte: byte_i,
                char: char_i,
                width_before: width,
            });
        }

        width = new_width;
        height = height.max(h);
        char_i += 1;
    }

    let break_ = Break::End {
        byte: text.len(),
        char: char_i,
    };
    NextBreak {
        break_,
        width,
        height,
    }
}

/// Produce the width of the given line of text including spaces (i.e. ' ').
pub fn width(text: &str, font: &text::Font, font_size: FontSize) -> Scalar {
    let scale = text::Scale::uniform(text::pt_to_px(font_size));
    let point = text::rt::Point { x: 0.0, y: 0.0 };

    let mut total_w = 0.0;
    for g in font.layout(text, scale, point) {
        match g.pixel_bounding_box() {
            Some(bb) => total_w = bb.max.x as f32,
            None => total_w += g.unpositioned().h_metrics().advance_width,
        }
    }

    total_w as Scalar
}

/// Produce an `Infos` iterator wrapped by the given `next_break_fn`.
pub fn infos_wrapped_by<'a, F>(
    text: &'a str,
    font: &'a text::Font,
    font_size: FontSize,
    max_width: Scalar,
    next_break_fn: F,
) -> Infos<'a, F>
where
    F: for<'b> FnMut(&'b str, &'b text::Font, FontSize, Scalar) -> NextBreak,
{
    Infos {
        text: text,
        font: font,
        font_size: font_size,
        max_width: max_width,
        next_break_fn: next_break_fn,
        start_byte: 0,
        start_char: 0,
        last_break: None,
    }
}

/// Produce an `Infos` iterator that yields an `Info` for every line in the given text.
///
/// The produced `Infos` iterator will not wrap the text, and only break each line via newline
/// characters within the text (either `\n` or `\r\n`).
pub fn infos<'a>(
    text: &'a str,
    font: &'a text::Font,
    font_size: FontSize,
) -> Infos<'a, NextBreakFnPtr> {
    fn no_wrap(
        text: &str,
        font: &text::Font,
        font_size: FontSize,
        _max_width: Scalar,
    ) -> NextBreak {
        next_break(text, font, font_size)
    }

    infos_wrapped_by(text, font, font_size, std::f32::MAX, no_wrap)
}

/// Simplify the retrieval of line information for text that may or may not be wrapped.
pub fn infos_maybe_wrapped<'a>(
    text: &'a str,
    font: &'a text::Font,
    font_size: FontSize,
    maybe_wrap: Option<Wrap>,
    max_width: Scalar,
) -> Infos<'a, NextBreakFnPtr> {
    match maybe_wrap {
        None => infos(text, font, font_size),
        Some(Wrap::Character) => infos(text, font, font_size).wrap_by_character(max_width),
        Some(Wrap::Whitespace) => infos(text, font, font_size).wrap_by_whitespace(max_width),
    }
}

/// Produce an iterator yielding the bounding `Rect` for each line in the text.
///
/// Yielded `Rect`s will begin with the top-left of the first line at a [0.0, 0.0].
///
/// This function assumes that `font_size` and `max_width` are the same as those used to produce
/// the `Info`s yielded by the `infos` Iterator.
pub fn rects<I>(
    mut infos: I,
    font_size: FontSize,
    max_width: Scalar,
    x_align: text::Justify,
    line_spacing: Scalar,
) -> Rects<I>
where
    I: Iterator<Item = Info>,
{
    let first_rect = infos.next().map(|first_info| {
        // Calculate the `x` `Range` of the first line `Rect`.
        let x_bounds = Range::new(0.0, max_width);
        let range = Range::new(0.0, first_info.width);
        let x = match x_align {
            text::Justify::Left => range.align_start_of(x_bounds),
            text::Justify::Center => range.align_middle_of(x_bounds),
            text::Justify::Right => range.align_end_of(x_bounds),
        };
        let y_start = -(font_size as Scalar);
        //let y_end = y_start + first_info.height;
        let y_end = y_start + font_size as Scalar;
        let y = Range::new(y_start, y_end);
        Rect { x: x, y: y }
    });
    Rects {
        infos: infos,
        next: first_rect,
        x_align: x_align,
        last_line_top: 0.0,
        font_size: font_size,
        line_spacing: line_spacing,
    }
}

/// Produces an iterator yielding a `Rect` for the selected range in each selected line in a block
/// of text.
///
/// The yielded `Rect`s represent the selected range within each line of text.
///
/// Lines that do not contain any selected text will be skipped.
pub fn selected_rects<'a, I>(
    lines_with_rects: I,
    font: &'a text::Font,
    font_size: FontSize,
    start: text::cursor::Index,
    end: text::cursor::Index,
) -> SelectedRects<'a, I>
where
    I: Iterator<Item = (&'a str, Rect)>,
{
    SelectedRects {
        selected_char_rects_per_line: text::glyph::selected_rects_per_line(
            lines_with_rects,
            font,
            font_size,
            start,
            end,
        ),
    }
}

impl<'a, F> Iterator for Infos<'a, F>
where
    F: for<'b> FnMut(&'b str, &'b text::Font, FontSize, Scalar) -> NextBreak,
{
    type Item = Info;
    fn next(&mut self) -> Option<Self::Item> {
        let Infos {
            text,
            font,
            font_size,
            max_width,
            ref mut next_break_fn,
            ref mut start_byte,
            ref mut start_char,
            ref mut last_break,
        } = *self;

        let next = next_break_fn(&text[*start_byte..], font, font_size, max_width);
        match next.break_ {
            Break::Newline { .. } | Break::Wrap { .. } => {
                let next_break = match next.break_ {
                    Break::Newline {
                        byte,
                        char,
                        len_bytes,
                        len_chars,
                    } => Break::Newline {
                        byte: *start_byte + byte,
                        char: *start_char + char,
                        len_bytes: len_bytes,
                        len_chars: len_chars,
                    },
                    Break::Wrap {
                        byte,
                        char,
                        len_bytes,
                        len_chars,
                    } => Break::Wrap {
                        byte: *start_byte + byte,
                        char: *start_char + char,
                        len_bytes: len_bytes,
                        len_chars: len_chars,
                    },
                    _ => unreachable!(),
                };

                let info = Info {
                    start_byte: *start_byte,
                    start_char: *start_char,
                    end_break: next_break,
                    width: next.width,
                    height: next.height,
                };

                match next.break_ {
                    Break::Newline {
                        byte,
                        char,
                        len_bytes,
                        len_chars,
                    }
                    | Break::Wrap {
                        byte,
                        char,
                        len_bytes,
                        len_chars,
                    } => {
                        // skip over end-of-break " ", "\n", or "\r\n"
                        *start_byte = info.start_byte + byte + len_bytes;
                        *start_char = info.start_char + char + len_chars;
                    }
                    _ => unreachable!(),
                };
                *last_break = Some(next_break);
                Some(info)
            }

            Break::End { char, .. } => {
                // if the last line ends in a new line, or the entire text is empty, return an
                // empty line Info.
                let empty_line = {
                    match *last_break {
                        Some(last_break_) => match last_break_ {
                            Break::Newline { .. } => true,
                            _ => false,
                        },
                        None => true,
                    }
                };
                if *start_byte < text.len() || empty_line {
                    let total_bytes = text.len();
                    let total_chars = *start_char + char;
                    let end_break = Break::End {
                        byte: total_bytes,
                        char: total_chars,
                    };
                    let info = Info {
                        start_byte: *start_byte,
                        start_char: *start_char,
                        end_break: end_break,
                        width: next.width,
                        height: next.height,
                    };
                    *start_byte = total_bytes;
                    *start_char = total_chars;
                    *last_break = Some(end_break);
                    Some(info)
                } else {
                    None
                }
            }
        }
    }
}

impl<I> Iterator for Rects<I>
where
    I: Iterator<Item = Info>,
{
    type Item = Rect;
    fn next(&mut self) -> Option<Self::Item> {
        let Rects {
            ref mut next,
            ref mut infos,
            x_align,
            ref mut last_line_top,
            font_size,
            line_spacing,
        } = *self;
        next.map(|line_rect| {
            *next = infos.next().map(|info| {
                let y = {
                    let line_top = *last_line_top - font_size as Scalar - line_spacing;
                    *last_line_top = line_top;
                    let y_start = line_top - font_size as Scalar;
                    //let y_end = y_start + info.height;
                    let y_end = y_start + font_size as Scalar;
                    Range::new(y_start, y_end)
                };

                let x = {
                    let range = Range::new(0.0, info.width);
                    match x_align {
                        text::Justify::Left => range.align_start_of(line_rect.x),
                        text::Justify::Center => range.align_middle_of(line_rect.x),
                        text::Justify::Right => range.align_end_of(line_rect.x),
                    }
                };

                Rect { x: x, y: y }
            });

            line_rect
        })
    }
}

impl<'a, I> Iterator for SelectedRects<'a, I>
where
    I: Iterator<Item = (&'a str, Rect)>,
{
    type Item = Rect;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(mut rects) = self.selected_char_rects_per_line.next() {
            if let Some((_, first_rect)) = rects.next() {
                let total_selected_rect = rects.fold(first_rect, |mut total, (_, next)| {
                    total.x.end = next.x.end;
                    total
                });
                return Some(total_selected_rect);
            }
        }
        None
    }
}
