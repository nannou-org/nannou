//! Text layout logic.
//!
//! Currently, this crate is used primarily by the `draw.text()` API but will also play an
//! important role in future GUI work.

pub mod cursor;
pub mod font;
pub mod glyph;
pub mod layout;
pub mod line;
pub mod rt {
    //! Re-exported RustType geometric types.
    pub use rusttype::{gpu_cache, point, vector, Point, Rect, Vector};
}

// Re-export all relevant rusttype types here.
pub use rusttype::gpu_cache::Cache as GlyphCache;
pub use rusttype::{Glyph, GlyphId, GlyphIter, LayoutIter, Scale, ScaledGlyph};
pub use self::layout::Layout;

use crate::geom;
use std::borrow::Cow;

/// The RustType `FontCollection` type used by nannou.
pub type FontCollection = rusttype::FontCollection<'static>;
/// The RustType `Font` type used by nannou.
pub type Font = rusttype::Font<'static>;
/// The RustType `PositionedGlyph` type used by nannou.
pub type PositionedGlyph = rusttype::PositionedGlyph<'static>;

/// The type used for scalar values.
pub type Scalar = crate::geom::scalar::Default;

/// The point type used when working with text.
pub type Point<S = Scalar> = crate::geom::Point2<S>;

/// The type used to specify `FontSize` in font points.
pub type FontSize = u32;

/// A context for building some **Text**.
pub struct Builder<'a> {
    text: Cow<'a, str>,
    layout_builder: layout::Builder,
}

/// An instance of some multi-line text and its layout.
#[derive(Clone)]
pub struct Text<'a> {
    text: Cow<'a, str>,
    font: Font,
    layout: Layout,
    line_infos: Vec<line::Info>,
    rect: geom::Rect,
}

/// An iterator yielding each line within the given `text` as a new `&str`, where the start and end
/// indices into each line are provided by the given iterator.
#[derive(Clone)]
pub struct Lines<'a, I> {
    text: &'a str,
    ranges: I,
}

/// An alias for the line info iterator yielded by `Text::line_infos`.
pub type TextLineInfos<'a> = line::Infos<'a, line::NextBreakFnPtr>;

/// An alias for the line iterator yielded by `Text::lines`.
pub type TextLines<'a> =
    Lines<'a, std::iter::Map<std::slice::Iter<'a, line::Info>, fn(&line::Info) -> std::ops::Range<usize>>>;

/// An alias for the line rect iterator used internally within the `Text::line_rects` iterator.
type LineRects<'a> = line::Rects<std::iter::Cloned<std::slice::Iter<'a, line::Info>>>;

/// An alias for the line rect iterator yielded by `Text::line_rects`.
#[derive(Clone)]
pub struct TextLineRects<'a> {
    line_rects: LineRects<'a>,
    offset: geom::Vector2,
}

/// An alias for the iterator yielded by `Text::lines_with_rects`.
pub type TextLinesWithRects<'a> = std::iter::Zip<TextLines<'a>, TextLineRects<'a>>;

/// An alias for the iterator yielded by `Text::glyphs_per_line`.
pub type TextGlyphsPerLine<'a> = glyph::RectsPerLine<'a, TextLinesWithRects<'a>>;

/// An alias for the iterator yielded by `Text::glyphs`.
pub type TextGlyphs<'a> = std::iter::FlatMap<
    TextGlyphsPerLine<'a>,
    glyph::Rects<'a, 'a>,
    fn(glyph::Rects<'a, 'a>) -> glyph::Rects<'a, 'a>
>;

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

/// The way in which text should wrap around the width.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Wrap {
    /// Wrap at the first character that exceeds the width.
    Character,
    /// Wrap at the first word that exceeds the width.
    Whitespace,
}

impl<'a> From<Cow<'a, str>> for Builder<'a> {
    fn from(text: Cow<'a, str>) -> Self {
        let layout_builder = Default::default();
        Builder { text, layout_builder }
    }
}

impl<'a> From<&'a str> for Builder<'a> {
    fn from(s: &'a str) -> Self {
        let text = Cow::Borrowed(s);
        Self::from(text)
    }
}

impl From<String> for Builder<'static> {
    fn from(s: String) -> Self {
        let text = Cow::Owned(s);
        Self::from(text)
    }
}

impl<'a> Builder<'a> {
    /// Apply the given function to the inner text layout.
    fn map_layout<F>(mut self, map: F) -> Self
    where
        F: FnOnce(layout::Builder) -> layout::Builder,
    {
        self.layout_builder = map(self.layout_builder);
        self
    }

    /// The font size to use for the text.
    pub fn font_size(self, size: FontSize) -> Self {
        self.map_layout(|l| l.font_size(size))
    }

    /// Specify whether or not text should be wrapped around some width and how to do so.
    ///
    /// The default value is `DEFAULT_LINE_WRAP`.
    pub fn line_wrap(self, line_wrap: Option<Wrap>) -> Self {
        self.map_layout(|l| l.line_wrap(line_wrap))
    }

    /// Specify that the **Text** should not wrap lines around the width.
    ///
    /// Shorthand for `builder.line_wrap(None)`.
    pub fn no_line_wrap(self) -> Self {
        self.map_layout(|l| l.no_line_wrap())
    }

    /// Line wrap the **Text** at the beginning of the first word that exceeds the width.
    ///
    /// Shorthand for `builder.line_wrap(Some(Wrap::Whitespace))`.
    pub fn wrap_by_word(self) -> Self {
        self.map_layout(|l| l.wrap_by_word())
    }

    /// Line wrap the **Text** at the beginning of the first character that exceeds the width.
    ///
    /// Shorthand for `builder.line_wrap(Some(Wrap::Character))`.
    pub fn wrap_by_character(self) -> Self {
        self.map_layout(|l| l.wrap_by_character())
    }

    /// A method for specifying the `Font` used for displaying the `Text`.
    pub fn font(self, font: Font) -> Self {
        self.map_layout(|l| l.font(font))
    }

    /// Describe the end along the *x* axis to which the text should be aligned.
    pub fn justify(self, justify: Justify) -> Self {
        self.map_layout(|l| l.justify(justify))
    }

    /// Align the text to the left of its bounding **Rect**'s *x* axis range.
    pub fn left_justify(self) -> Self {
        self.map_layout(|l| l.left_justify())
    }

    /// Align the text to the middle of its bounding **Rect**'s *x* axis range.
    pub fn center_justify(self) -> Self {
        self.map_layout(|l| l.center_justify())
    }

    /// Align the text to the right of its bounding **Rect**'s *x* axis range.
    pub fn right_justify(self) -> Self {
        self.map_layout(|l| l.right_justify())
    }

    /// Specify how much vertical space should separate each line of text.
    pub fn line_spacing(self, spacing: Scalar) -> Self {
        self.map_layout(|l| l.line_spacing(spacing))
    }

    /// Specify how the whole text should be aligned along the y axis of its bounding rectangle
    pub fn y_align(self, align: Align) -> Self {
        self.map_layout(|l| l.y_align(align))
    }

    /// Align the top edge of the text with the top edge of its bounding rectangle.
    pub fn align_top(self) -> Self {
        self.map_layout(|l| l.align_top())
    }

    /// Align the middle of the text with the middle of the bounding rect along the y axis..
    ///
    /// This is the default behaviour.
    pub fn align_middle_y(self) -> Self {
        self.map_layout(|l| l.align_middle_y())
    }

    /// Align the bottom edge of the text with the bottom edge of its bounding rectangle.
    pub fn align_bottom(self) -> Self {
        self.map_layout(|l| l.align_bottom())
    }

    /// Set all the parameters via an existing `Layout`
    pub fn layout(self, layout: &Layout) -> Self {
        self.map_layout(|l| l.layout(layout))
    }

    /// Build the text.
    ///
    /// This iterates over the text in order to pre-calculates the text's multi-line information
    /// using the `line::infos` function.
    ///
    /// The given `rect` will be used for applying the layout including text alignment, positioning
    /// of text, multi-line wrapping, etc,
    pub fn build(self, rect: geom::Rect) -> Text<'a> {
        let text = self.text;
        let layout = self.layout_builder.build();
        let font = layout.font.clone().unwrap_or_else(|| {
            if cfg!(feature = "notosans") {
                font::default_notosans()
            } else {
                let assets = crate::app::find_assets_path()
                    .expect("failed to detect the assets directory when searching for a default font");;
                font::default(&assets).expect("failed to detect a default font")
            }
        });
        let max_width = rect.w();
        let line_infos = line::infos_maybe_wrapped(
            &text,
            &font,
            layout.font_size,
            layout.line_wrap,
            max_width,
        ).collect();
        Text { text, font, layout, line_infos, rect }
    }
}

impl<'a> Text<'a> {
    /// Produce an iterator yielding information about each line.
    pub fn line_infos(&self) -> &[line::Info] {
        &self.line_infos
    }

    /// The full string of text as a slice.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The layout parameters for this text instance.
    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    /// The number of lines in the text.
    pub fn num_lines(&self) -> usize {
        self.line_infos.len()
    }

    /// The width around which the text was wrapped.
    pub fn rect(&self) -> geom::Rect {
        self.rect
    }

    /// The width of the widest line of text.
    pub fn width(&self) -> Scalar {
        self.line_infos.iter().fold(0.0, |max, info| max.max(info.width))
    }

    /// The total height of the text.
    pub fn height(&self) -> Scalar {
        height(self.num_lines(), self.layout.font_size, self.layout.line_spacing)
    }

    /// Produce an iterator yielding each wrapped line within the **Text**.
    pub fn lines(&self) -> TextLines {
        fn info_byte_range(info: &line::Info) -> std::ops::Range<usize> {
            info.byte_range()
        }
        lines(&self.text, self.line_infos.iter().map(info_byte_range))
    }

    /// The bounding rectangle for each line.
    pub fn line_rects(&self) -> TextLineRects {
        let offset = position_offset(
            self.num_lines(),
            self.layout.font_size,
            self.layout.line_spacing,
            self.rect,
            self.layout.y_align,
        );
        let line_rects = line::rects(
            self.line_infos.iter().cloned(),
            self.layout.font_size,
            self.rect.w(),
            self.layout.justify,
            self.layout.line_spacing,
        );
        TextLineRects { line_rects, offset }
    }

    /// Produce an iterator yielding all lines of text alongside their bounding rects.
    pub fn lines_with_rects(&self) -> TextLinesWithRects {
        self.lines().zip(self.line_rects())
    }

    /// Produce an iterator yielding iterators yielding every glyph alongside its bounding rect for
    /// each line.
    pub fn glyphs_per_line(&self) -> TextGlyphsPerLine {
        glyph::rects_per_line(self.lines_with_rects(), &self.font, self.layout.font_size)
    }

    /// Produce an iterator yielding every glyph alongside its bounding rect.
    ///
    /// This is the "flattened" version of the `glyphs_per_line` method.
    pub fn glyphs(&self) -> TextGlyphs {
        self.glyphs_per_line().flat_map(std::convert::identity)
    }

    /// Produce an iterator yielding the path events for every glyph in every line.
    pub fn path_events<'b>(&'b self) -> impl 'b + Iterator<Item = lyon::path::PathEvent> {
        use lyon::geom::{CubicBezierSegment, LineSegment, QuadraticBezierSegment};
        use lyon::path::PathEvent;

        // Translate the given lyon point by the given vector.
        fn trans_lyon_point(p: &lyon::math::Point, v: geom::Vector2) -> lyon::math::Point {
            lyon::math::point(p.x + v.x, p.y + v.y)
        }

        // Translate the given path event in 2D space.
        fn trans_path_event(e: &PathEvent, v: geom::Vector2) -> PathEvent {
            match *e {
                PathEvent::MoveTo(ref p) => PathEvent::MoveTo(trans_lyon_point(p, v)),
                PathEvent::Line(ref e) => PathEvent::Line(LineSegment {
                    from: trans_lyon_point(&e.from, v),
                    to: trans_lyon_point(&e.to, v),
                }),
                PathEvent::Quadratic(ref e) => PathEvent::Quadratic(QuadraticBezierSegment {
                    from: trans_lyon_point(&e.from, v),
                    ctrl: trans_lyon_point(&e.ctrl, v),
                    to: trans_lyon_point(&e.to, v),
                }),
                PathEvent::Cubic(ref e) => PathEvent::Cubic(CubicBezierSegment {
                    from: trans_lyon_point(&e.from, v),
                    ctrl1: trans_lyon_point(&e.ctrl1, v),
                    ctrl2: trans_lyon_point(&e.ctrl2, v),
                    to: trans_lyon_point(&e.to, v),
                }),
                PathEvent::Close(ref e) => PathEvent::Close(LineSegment {
                    from: trans_lyon_point(&e.from, v),
                    to: trans_lyon_point(&e.to, v),
                }),
            }
        }

        self.glyphs()
            .flat_map(|(g, r)| {
                glyph::path_events(g)
                    .into_iter()
                    .flat_map(|es| es)
                    .map(move |e| trans_path_event(&e, r.bottom_left()))
            })
    }

    /// Produce an iterator yielding positioned rusttype glyphs ready for caching.
    ///
    /// The window dimensions and DPI are required to transform glyph positions into rusttype's
    /// pixel-space, ready for caching into the rusttype glyph cache pixel buffer.
    ///
    /// TODO: Method may still require applying text.rect.xy offset.
    pub fn rt_glyphs<'b: 'a>(
        &'b self,
        window_dims: geom::Vector2,
        dpi_factor: Scalar,
        left: Scalar,
        y: geom::Range,
        y_align: Align,
    ) -> impl 'a + 'b + Iterator<Item = PositionedGlyph> {
        let x = geom::Range::new(left, left + self.rect.w());
        let bounding_rect = geom::Rect { x, y };
        positioned_glyphs(
            &self.text,
            &self.line_infos,
            bounding_rect,
            y_align,
            self.layout.line_spacing,
            self.layout.font_size,
            self.layout.justify,
            &self.font,
            window_dims,
            dpi_factor,
        )
    }

    /// Converts this `Text` instance into an instance that owns the inner text string.
    pub fn into_owned(self) -> Text<'static> {
        let Text { text, font, layout, line_infos, rect } = self;
        let text = Cow::Owned(text.into_owned());
        Text { text, font, layout, line_infos, rect }
    }
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

impl<'a> Iterator for TextLineRects<'a> {
    type Item = geom::Rect;
    fn next(&mut self) -> Option<Self::Item> {
        self.line_rects.next().map(|r| r.shift(self.offset))
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

/// The position offset required to shift the associated text into the given bounding rectangle.
///
/// This function assumes the `max_width` used to produce the `line_infos` is equal to the given
/// `bounding_rect` max width.
pub fn position_offset(
    num_lines: usize,
    font_size: FontSize,
    line_spacing: f32,
    bounding_rect: geom::Rect,
    y_align: Align,
) -> geom::Vector2 {
    let x_offset = bounding_rect.x.start;
    let y_offset = {
        // Calculate the `y` `Range` of the first line `Rect`.
        let total_text_height = height(num_lines, font_size, line_spacing);
        let total_text_y_range = geom::Range::new(0.0, total_text_height);
        let total_text_y = match y_align {
            Align::Start => total_text_y_range.align_start_of(bounding_rect.y),
            Align::Middle => total_text_y_range.align_middle_of(bounding_rect.y),
            Align::End => total_text_y_range.align_end_of(bounding_rect.y),
        };
        total_text_y.end
    };
    geom::vec2(x_offset, y_offset)
}

/// Produce the position of each glyph.
///
/// The top-left of the top line's rectangle will be positioned at [0.0, 0.0].
pub fn positioned_glyphs<'a>(
    text: &'a str,
    line_infos: &'a [line::Info],
    bounding_rect: geom::Rect,
    y_align: Align,
    line_spacing: Scalar,
    font_size: u32,
    justify: Justify,
    font: &'a Font,
    window_dim: geom::Vector2,
    dpi_factor: Scalar,
) -> impl 'a + Iterator<Item = PositionedGlyph> {
    // Produce the text layout iterators.
    let n_lines = line_infos.len();
    let line_infos = line_infos.iter().cloned();
    let lines = line_infos.clone().map(move |info| &text[info.byte_range()]);
    let bounding_w = bounding_rect.x.len();
    let offset = position_offset(n_lines, font_size, line_spacing, bounding_rect, y_align);
    let line_rects = line::rects(line_infos, font_size, bounding_w, justify, line_spacing)
        .map(move |r| r.shift(offset));

    // Functions for converting nannou coordinates to rusttype pixel coordinates.
    let trans_x = move |x: Scalar| (x + window_dim[0] / 2.0) * dpi_factor as Scalar;
    let trans_y = move |y: Scalar| ((-y) + window_dim[1] / 2.0) * dpi_factor as Scalar;

    // Clear the existing glyphs and fill the buffer with glyphs for this Text.
    let scale = f32_pt_to_scale(font_size as f32 * dpi_factor);
    lines
        .zip(line_rects)
        .flat_map(move |(line, line_rect)| {
            let (x, y) = (trans_x(line_rect.left()) as f32, trans_y(line_rect.bottom()) as f32);
            let point = rt::Point { x: x, y: y };
            font.layout(line, scale, point).map(|g| g.standalone())
        })
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

/// Begin building a **Text** instance.
pub fn text(s: &str) -> Builder {
    Builder::from(s)
}
