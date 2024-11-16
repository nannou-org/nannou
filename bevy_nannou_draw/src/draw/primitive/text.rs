use bevy::prelude::*;

use crate::draw::drawing::DrawingContext;
use crate::draw::primitive::Primitive;
use crate::draw::properties::spatial::{self, dimension, orientation, position};
use crate::draw::properties::{SetColor, SetDimensions, SetOrientation, SetPosition};
use crate::draw::{self, Drawing};
use crate::render::ShaderModel;
use crate::text::{self, Align, Font, FontSize, Justify, Layout, Scalar, Wrap};

/// Properties related to drawing the **Text** primitive.
#[derive(Clone, Debug)]
pub struct Text {
    spatial: spatial::Properties,
    style: Style,
    // The byte range into the `Draw` context's text buffer.
    text: std::ops::Range<usize>,
}

/// Styling properties for the **Text** primitive.
#[derive(Clone, Debug, Default)]
pub struct Style {
    pub color: Option<Color>,
    pub glyph_colors: Vec<Color>, // Overrides `color` if non-empty.
    pub layout: text::layout::Builder,
}

/// The drawing context for the **Text** primitive.
pub type DrawingText<'a, SM> = Drawing<'a, Text, SM>;

impl Text {
    /// Begin drawing some text.
    pub fn new(ctxt: DrawingContext, text: &str) -> Self {
        let start = ctxt.text_buffer.len();
        ctxt.text_buffer.push_str(text);
        let end = ctxt.text_buffer.len();
        let text = start..end;
        let spatial = Default::default();
        let style = Default::default();
        Text {
            spatial,
            style,
            text,
        }
    }

    // Apply the given function to the inner text layout.
    fn map_layout<F>(mut self, map: F) -> Self
    where
        F: FnOnce(text::layout::Builder) -> text::layout::Builder,
    {
        self.style.layout = map(self.style.layout);
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

    /// Align the middle of the text with the middle of the bounding rect along the y axis.
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

    /// Specify the entire styling for the **Text**.
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set a color for each glyph.
    /// Colors unspecified glyphs using the drawing color.
    pub fn glyph_colors(mut self, colors: Vec<Color>) -> Self {
        self.style.glyph_colors = colors;
        self
    }
}

impl<'a, SM> DrawingText<'a, SM>
where
    SM: ShaderModel + Default,
{
    /// The font size to use for the text.
    pub fn font_size(self, size: text::FontSize) -> Self {
        self.map_ty(|ty| ty.font_size(size))
    }

    /// Specify that the **Text** should not wrap lines around the width.
    pub fn no_line_wrap(self) -> Self {
        self.map_ty(|ty| ty.no_line_wrap())
    }

    /// Line wrap the **Text** at the beginning of the first word that exceeds the width.
    pub fn wrap_by_word(self) -> Self {
        self.map_ty(|ty| ty.wrap_by_word())
    }

    /// Line wrap the **Text** at the beginning of the first character that exceeds the width.
    pub fn wrap_by_character(self) -> Self {
        self.map_ty(|ty| ty.wrap_by_character())
    }

    /// A method for specifying the `Font` used for displaying the `Text`.
    pub fn font(self, font: text::Font) -> Self {
        self.map_ty(|ty| ty.font(font))
    }

    /// Build the **Text** with the given **Style**.
    pub fn with_style(self, style: Style) -> Self {
        self.map_ty(|ty| ty.with_style(style))
    }

    /// Describe the end along the *x* axis to which the text should be aligned.
    pub fn justify(self, justify: text::Justify) -> Self {
        self.map_ty(|ty| ty.justify(justify))
    }

    /// Align the text to the left of its bounding **Rect**'s *x* axis range.
    pub fn left_justify(self) -> Self {
        self.map_ty(|ty| ty.left_justify())
    }

    /// Align the text to the middle of its bounding **Rect**'s *x* axis range.
    pub fn center_justify(self) -> Self {
        self.map_ty(|ty| ty.center_justify())
    }

    /// Align the text to the right of its bounding **Rect**'s *x* axis range.
    pub fn right_justify(self) -> Self {
        self.map_ty(|ty| ty.right_justify())
    }

    /// Specify how much vertical space should separate each line of text.
    pub fn line_spacing(self, spacing: text::Scalar) -> Self {
        self.map_ty(|ty| ty.line_spacing(spacing))
    }

    /// Specify how the whole text should be aligned along the y axis of its bounding rectangle
    pub fn y_align_text(self, align: Align) -> Self {
        self.map_ty(|ty| ty.y_align(align))
    }

    /// Align the top edge of the text with the top edge of its bounding rectangle.
    pub fn align_text_top(self) -> Self {
        self.map_ty(|ty| ty.align_top())
    }

    /// Align the middle of the text with the middle of the bounding rect along the y axis.
    ///
    /// This is the default behaviour.
    pub fn align_text_middle_y(self) -> Self {
        self.map_ty(|ty| ty.align_middle_y())
    }

    /// Align the bottom edge of the text with the bottom edge of its bounding rectangle.
    pub fn align_text_bottom(self) -> Self {
        self.map_ty(|ty| ty.align_bottom())
    }

    /// Set all the parameters via an existing `Layout`
    pub fn layout(self, layout: &Layout) -> Self {
        self.map_ty(|ty| ty.layout(layout))
    }

    /// Set a color for each glyph, which is typically one character.
    /// Colors unspecified glyphs using the drawing color.
    /// NOTE: Sometimes, a glyph can represent multiple characters,
    ///       or be a part in other glyphs.
    pub fn glyph_colors<I, C>(self, glyph_colors: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: Into<Color>,
    {
        let glyph_colors = glyph_colors.into_iter().map(|c| c.into()).collect();

        self.map_ty(|ty| ty.glyph_colors(glyph_colors))
    }
}

impl draw::render::RenderPrimitive for Text {
    fn render_primitive(self, _ctxt: draw::render::RenderContext, _mesh: &mut Mesh) {
        todo!()
    }
}

impl SetOrientation for Text {
    fn properties(&mut self) -> &mut orientation::Properties {
        SetOrientation::properties(&mut self.spatial)
    }
}

impl SetPosition for Text {
    fn properties(&mut self) -> &mut position::Properties {
        SetPosition::properties(&mut self.spatial)
    }
}

impl SetDimensions for Text {
    fn properties(&mut self) -> &mut dimension::Properties {
        SetDimensions::properties(&mut self.spatial)
    }
}

impl SetColor for Text {
    fn color_mut(&mut self) -> &mut Option<Color> {
        SetColor::color_mut(&mut self.style.color)
    }
}

// Primitive conversions.

impl From<Text> for Primitive {
    fn from(prim: Text) -> Self {
        Primitive::Text(prim)
    }
}

impl Into<Option<Text>> for Primitive {
    fn into(self) -> Option<Text> {
        match self {
            Primitive::Text(prim) => Some(prim),
            _ => None,
        }
    }
}
