//! Text layout logic.
//!
//! Currently, this crate is used primarily by the `draw.text()` API and
//! the immediate-mode `draw.text_layout()` / `app.text()` APIs for
//! text measurement and glyph outline extraction.

use std::borrow::Cow;

use bevy::prelude::*;
use nannou_core::geom;
use parley::style::{FontStack, StyleProperty};
use parley::Alignment;

pub use self::layout::Layout;

pub mod font;
pub mod glyph;
pub mod layout;

/// The type used for scalar values.
pub type Scalar = nannou_core::geom::scalar::Default;

/// The point type used when working with text.
pub type Point = nannou_core::geom::Point2;

/// The type used to specify `FontSize` in font points.
pub type FontSize = u32;

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
}

/// The way in which text should wrap around the width.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Wrap {
    /// Wrap at the first character that exceeds the width.
    Character,
    /// Wrap at the first word that exceeds the width.
    Whitespace,
}

/// A context for building some **Text** with immediate layout.
pub struct Builder<'a> {
    text: Cow<'a, str>,
    layout_builder: layout::Builder,
    text_cx: font::SharedTextCx,
}

/// An instance of laid-out text backed by parley.
pub struct Text {
    text_string: String,
    parley_layout: parley::Layout<Color>,
    layout: Layout,
    rect: geom::Rect,
}

impl<'a> Builder<'a> {
    /// Create a new text builder with the given string and shared text context.
    pub fn new(s: &'a str, text_cx: font::SharedTextCx) -> Self {
        Builder {
            text: Cow::Borrowed(s),
            layout_builder: Default::default(),
            text_cx,
        }
    }

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
    pub fn line_wrap(self, line_wrap: Option<Wrap>) -> Self {
        self.map_layout(|l| l.line_wrap(line_wrap))
    }

    /// Specify that the **Text** should not wrap lines around the width.
    pub fn no_line_wrap(self) -> Self {
        self.map_layout(|l| l.no_line_wrap())
    }

    /// Line wrap the **Text** at the beginning of the first word that exceeds the width.
    pub fn wrap_by_word(self) -> Self {
        self.map_layout(|l| l.wrap_by_word())
    }

    /// Line wrap the **Text** at the beginning of the first character that exceeds the width.
    pub fn wrap_by_character(self) -> Self {
        self.map_layout(|l| l.wrap_by_character())
    }

    /// A method for specifying the font family used for displaying the text.
    pub fn font_family(self, family: impl Into<String>) -> Self {
        self.map_layout(|l| l.font_family(family))
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

    /// Specify how the whole text should be aligned along the y axis of its bounding rectangle.
    pub fn y_align(self, align: Align) -> Self {
        self.map_layout(|l| l.y_align(align))
    }

    /// Align the top edge of the text with the top edge of its bounding rectangle.
    pub fn align_top(self) -> Self {
        self.map_layout(|l| l.align_top())
    }

    /// Align the middle of the text with the middle of the bounding rect along the y axis.
    pub fn align_middle_y(self) -> Self {
        self.map_layout(|l| l.align_middle_y())
    }

    /// Align the bottom edge of the text with the bottom edge of its bounding rectangle.
    pub fn align_bottom(self) -> Self {
        self.map_layout(|l| l.align_bottom())
    }

    /// Set all the parameters via an existing `Layout`.
    pub fn layout(self, layout: &Layout) -> Self {
        self.map_layout(|l| l.layout(layout))
    }

    /// Build the text layout.
    ///
    /// This locks the shared text context, computes the parley layout within the
    /// given `rect`, and returns a `Text` with measurement and glyph methods.
    pub fn build(self, rect: geom::Rect) -> Text {
        let layout = self.layout_builder.build();
        let mut inner = self.text_cx.0.lock().unwrap();
        Text::layout_with_inner(&mut inner, &self.text, &layout, rect)
    }
}

impl Text {
    /// Internal: compute a parley layout using an already-locked inner context.
    pub(crate) fn layout_with_inner(
        inner: &mut font::NannouTextCxInner,
        text: &str,
        layout: &Layout,
        rect: geom::Rect,
    ) -> Self {
        let font_size = layout.font_size as f32;
        let scale = 1.0;

        let mut builder = inner
            .layout
            .ranged_builder(&mut inner.font, text, scale, true);

        builder.push_default(StyleProperty::FontSize(font_size));

        if let Some(ref family) = layout.font_family {
            builder.push_default(StyleProperty::FontStack(FontStack::Single(
                parley::style::FontFamily::Named(family.into()),
            )));
        }

        if let Some(spacing) = (layout.line_spacing != 0.0).then_some(layout.line_spacing) {
            builder.push_default(StyleProperty::LineHeight(
                parley::style::LineHeight::Absolute(font_size + spacing),
            ));
        }

        let mut parley_layout = builder.build(text);

        // Break lines according to wrap mode.
        let max_width = rect.w();
        match layout.line_wrap {
            None => {
                // No wrapping: use infinite width.
                parley_layout.break_all_lines(None);
            }
            Some(Wrap::Whitespace) => {
                parley_layout.break_all_lines(Some(max_width));
            }
            Some(Wrap::Character) => {
                // Parley doesn't have a separate character-wrap mode in break_all_lines.
                // We use the same width-based breaking.
                parley_layout.break_all_lines(Some(max_width));
            }
        }

        // Align horizontally.
        let alignment = match layout.justify {
            Justify::Left => Alignment::Start,
            Justify::Center => Alignment::Center,
            Justify::Right => Alignment::End,
        };
        parley_layout.align(
            Some(max_width),
            alignment,
            parley::AlignmentOptions::default(),
        );

        Text {
            text_string: text.to_string(),
            parley_layout,
            layout: layout.clone(),
            rect,
        }
    }

    /// The layout parameters for this text instance.
    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    /// The rectangle used to layout and build the text instance.
    pub fn layout_rect(&self) -> geom::Rect {
        self.rect
    }

    /// The width of the laid-out text.
    pub fn width(&self) -> Scalar {
        self.parley_layout.width()
    }

    /// The height of the laid-out text.
    pub fn height(&self) -> Scalar {
        self.parley_layout.height()
    }

    /// The number of lines in the text.
    pub fn num_lines(&self) -> usize {
        self.parley_layout.len()
    }

    /// The rectangle that describes the bounding box of the text,
    /// positioned within the layout rect according to y_align.
    pub fn bounding_rect(&self) -> geom::Rect {
        let w = self.width();
        let h = self.height();
        if w == 0.0 && h == 0.0 {
            return geom::Rect::from_w_h(0.0, 0.0);
        }
        let offset = self.position_offset();
        // Parley layout is top-down; bounding rect in nannou coords.
        let x = geom::Range::new(offset.x, offset.x + w);
        let y = geom::Range::new(offset.y - h, offset.y);
        geom::Rect { x, y }
    }

    /// Iterate over line rects, each positioned in nannou coordinate space.
    pub fn line_rects(&self) -> Vec<geom::Rect> {
        let offset = self.position_offset();
        self.parley_layout
            .lines()
            .map(|line| {
                let metrics = line.metrics();
                // Parley: y increases downward. We flip to nannou y-up.
                let top = offset.y - metrics.baseline + metrics.ascent;
                let bottom = offset.y - metrics.baseline - metrics.descent;
                // metrics.offset accounts for alignment (e.g. center/right).
                // Exclude trailing whitespace from the line width.
                let line_x = offset.x + metrics.offset;
                let line_w = metrics.advance - metrics.trailing_whitespace;
                let x = geom::Range::new(line_x, line_x + line_w);
                let y = geom::Range::new(bottom, top);
                geom::Rect { x, y }
            })
            .collect()
    }

    /// Iterate over lines, yielding the text content of each line.
    pub fn lines(&self) -> Vec<&str> {
        self.parley_layout
            .lines()
            .map(|line| {
                let range = line.text_range();
                &self.text_string[range]
            })
            .collect()
    }

    /// Iterate over glyph rects (one per glyph cluster).
    pub fn glyphs(&self) -> Vec<geom::Rect> {
        let offset = self.position_offset();
        let mut rects = Vec::new();
        for line in self.parley_layout.lines() {
            let baseline = line.metrics().baseline;
            for item in line.items() {
                let parley::PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                    continue;
                };
                let run = glyph_run.run();
                let font_size = run.font_size();
                let run_metrics = run.metrics();
                let glyph_height = run_metrics.ascent + run_metrics.descent;
                for glyph in glyph_run.positioned_glyphs() {
                    let gx = offset.x + glyph.x;
                    let gy = offset.y - baseline;
                    let x = geom::Range::new(gx, gx + glyph.advance);
                    let y = geom::Range::new(gy - run_metrics.descent, gy + run_metrics.ascent);
                    let _ = (font_size, glyph_height); // suppress unused
                    rects.push(geom::Rect { x, y });
                }
            }
        }
        rects
    }

    /// Produce an iterator yielding the path events for every glyph in every line.
    pub fn path_events(&self) -> Vec<lyon::path::PathEvent> {
        glyph::text_path_events(&self.parley_layout, self.position_offset())
    }

    /// Compute the position offset to place parley's top-down layout
    /// into nannou's y-up coordinate system, accounting for y_align.
    fn position_offset(&self) -> Vec2 {
        let text_h = self.height();
        let y_offset = match self.layout.y_align {
            Align::End => {
                // Top-align: top of text at top of rect.
                self.rect.top()
            }
            Align::Middle => {
                // Middle-align: center text vertically in rect.
                self.rect.y.middle() + text_h / 2.0
            }
            Align::Start => {
                // Bottom-align: bottom of text at bottom of rect.
                self.rect.bottom() + text_h
            }
        };
        let x_offset = self.rect.left();
        Vec2::new(x_offset, y_offset)
    }
}

/// Determine the total height of a block of text with the given number of lines, font size and
/// `line_spacing` (the space that separates each line of text).
pub fn height_by_lines(num_lines: usize, font_size: FontSize, line_spacing: Scalar) -> Scalar {
    if num_lines > 0 {
        num_lines as Scalar * font_size as Scalar + (num_lines - 1) as Scalar * line_spacing
    } else {
        0.0
    }
}

/// The position offset required to shift the associated text into the given bounding rectangle.
pub fn position_offset(
    num_lines: usize,
    font_size: FontSize,
    line_spacing: f32,
    bounding_rect: geom::Rect,
    y_align: Align,
) -> Vec2 {
    let x_offset = bounding_rect.x.start;
    let y_offset = {
        let total_text_height = height_by_lines(num_lines, font_size, line_spacing);
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

