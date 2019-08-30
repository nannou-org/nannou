//! Items related to the styling of text.

use crate::text::{font, Font, FontSize, Justify, Scalar, Wrap};

/// A context for building a text layout.
#[derive(Clone, Debug, Default)]
pub struct Builder {
    pub line_spacing: Option<Scalar>,
    pub line_wrap: Option<Option<Wrap>>,
    pub font_size: Option<FontSize>,
    pub justify: Option<Justify>,
    pub font: Option<Option<Font>>,
}

/// Properties related to the layout of multi-line text for a single font and font size.
#[derive(Clone, Debug)]
pub struct Layout {
    pub line_spacing: Scalar,
    pub line_wrap: Option<Wrap>,
    pub justify: Justify,
    pub font_size: FontSize,
    pub font: Option<Font>,
}

pub const DEFAULT_LINE_WRAP: Option<Wrap> = Some(Wrap::Whitespace);
pub const DEFAULT_FONT_SIZE: u32 = 12;
pub const DEFAULT_LINE_SPACING: f32 = 0.0;
pub const DEFAULT_JUSTIFY: Justify = Justify::Left;

impl Builder {
    /// The font size to use for the text.
    pub fn font_size(mut self, size: FontSize) -> Self {
        self.font_size = Some(size);
        self
    }

    /// Specify whether or not text should be wrapped around some width and how to do so.
    ///
    /// The default value is `DEFAULT_LINE_WRAP`.
    pub fn line_wrap(mut self, line_wrap: Option<Wrap>) -> Self {
        self.line_wrap = Some(line_wrap);
        self
    }

    /// Specify that the **Text** should not wrap lines around the width.
    ///
    /// Shorthand for `builder.line_wrap(None)`.
    pub fn no_line_wrap(self) -> Self {
        self.line_wrap(None)
    }

    /// Line wrap the **Text** at the beginning of the first word that exceeds the width.
    ///
    /// Shorthand for `builder.line_wrap(Some(Wrap::Whitespace))`.
    pub fn wrap_by_word(self) -> Self {
        self.line_wrap(Some(Wrap::Whitespace))
    }

    /// Line wrap the **Text** at the beginning of the first character that exceeds the width.
    ///
    /// Shorthand for `builder.line_wrap(Some(Wrap::Character))`.
    pub fn wrap_by_character(self) -> Self {
        self.line_wrap(Some(Wrap::Character))
    }

    /// A method for specifying the `Font` used for displaying the `Text`.
    pub fn font(mut self, font: Font) -> Self {
        self.font = Some(Some(font));
        self
    }

    /// Describe the end along the *x* axis to which the text should be aligned.
    pub fn justify(mut self, justify: Justify) -> Self {
        self.justify = Some(justify);
        self
    }

    /// Align the text to the left of its bounding **Rect**'s *x* axis range.
    pub fn left_justify(self) -> Self {
        self.justify(Justify::Left)
    }

    /// Align the text to the middle of its bounding **Rect**'s *x* axis range.
    pub fn center_justify(self) -> Self {
        self.justify(Justify::Center)
    }

    /// Align the text to the right of its bounding **Rect**'s *x* axis range.
    pub fn right_justify(self) -> Self {
        self.justify(Justify::Right)
    }

    /// Specify how much vertical space should separate each line of text.
    pub fn line_spacing(mut self, spacing: Scalar) -> Self {
        self.line_spacing = Some(spacing);
        self
    }

    /// Set all the parameters via an existing `Layout`
    pub fn layout(mut self, layout: &Layout) -> Self {
        self.font = Some(layout.font.clone());
        self.line_spacing(layout.line_spacing)
            .line_wrap(layout.line_wrap)
            .justify(layout.justify)
            .font_size(layout.font_size)
    }

    /// Build the text layout.
    pub fn build(self) -> Layout {
        Layout {
            line_spacing: self.line_spacing.unwrap_or(DEFAULT_LINE_SPACING),
            line_wrap: self.line_wrap.unwrap_or(DEFAULT_LINE_WRAP),
            justify: self.justify.unwrap_or(DEFAULT_JUSTIFY),
            font_size: self.font_size.unwrap_or(DEFAULT_FONT_SIZE),
            font: self.font.unwrap_or(None),
        }
    }
}

impl Default for Layout {
    fn default() -> Self {
        Layout {
            line_spacing: DEFAULT_LINE_SPACING,
            line_wrap: DEFAULT_LINE_WRAP,
            justify: DEFAULT_JUSTIFY,
            font_size: DEFAULT_FONT_SIZE,
            font: None,
        }
    }
}
