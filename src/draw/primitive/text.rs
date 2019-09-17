use crate::draw::drawing::DrawingContext;
use crate::draw::primitive::Primitive;
use crate::draw::properties::spatial::{self, dimension, orientation, position};
use crate::draw::properties::{
    ColorScalar, Draw, Drawn, LinSrgba, SetColor, SetDimensions, SetOrientation, SetPosition,
};
use crate::draw::{self, theme, Drawing, IntoDrawn};
use crate::geom::{self, Vector2};
use crate::math::BaseFloat;
use crate::text::{self, Align, Font, FontSize, Justify, Layout, Scalar, Wrap};

/// Properties related to drawing the **Text** primitive.
#[derive(Clone, Debug)]
pub struct Text<S = geom::scalar::Default> {
    spatial: spatial::Properties<S>,
    style: Style,
    // The byte range into the `Draw` context's text buffer.
    text: std::ops::Range<usize>,
}

/// Styling properties for the **Text** primitive.
#[derive(Clone, Debug, Default)]
pub struct Style {
    pub color: Option<LinSrgba>,
    pub layout: text::layout::Builder,
}

/// The drawing context for the **Text** primitive.
pub type DrawingText<'a, S = geom::scalar::Default> = Drawing<'a, Text<S>, S>;

impl<S> Text<S> {
    /// Begin drawing some text.
    pub fn new(ctxt: DrawingContext<S>, text: &str) -> Self {
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

    /// Specify the entire styling for the **Text**.
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl<'a, S> DrawingText<'a, S>
where
    S: BaseFloat,
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

    /// Align the middle of the text with the middle of the bounding rect along the y axis..
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
}

impl<S> IntoDrawn<S> for Text<S>
where
    S: BaseFloat,
{
    type Vertices = draw::properties::VerticesFromRanges;
    type Indices = draw::properties::IndicesFromRange;
    fn into_drawn(self, mut draw: Draw<S>) -> Drawn<S, Self::Vertices, Self::Indices> {
        let Text {
            spatial,
            style,
            text,
        } = self;
        let Style { color, layout } = style;
        let layout = layout.build();
        let (maybe_x, maybe_y, maybe_z) = spatial.dimensions.to_scalars(&draw);
        assert!(
            maybe_z.is_none(),
            "z dimension support for text is unimplemented"
        );
        let w = maybe_x
            .map(|s| <f32 as crate::math::NumCast>::from(s).unwrap())
            .unwrap_or(200.0);
        let h = maybe_y
            .map(|s| <f32 as crate::math::NumCast>::from(s).unwrap())
            .unwrap_or(200.0);
        let rect: geom::Rect = geom::Rect::from_wh(Vector2 { x: w, y: h });
        let color = color.unwrap_or_else(|| draw.theme().fill_lin_srgba(&theme::Primitive::Text));
        let path = draw.drawing_context(|ctxt| {
            let DrawingContext {
                mesh,
                fill_tessellator,
                path_event_buffer,
                text_buffer,
                glyph_cache,
            } = ctxt;
            let text_str = &text_buffer[text.clone()];
            let text = text::text(text_str).layout(&layout).build(rect);

            // TODO:
            // - Using `Path` is very slow - we should be caching glyphs.
            // - Can't do the CPU raster of the text here due to not knowing the DPI.
            // - We don't know the DPI until we finally draw to the frame.
            // - We should switch `Draw` to yield "primitives" rather than directly yielding
            //   vertices and indices. This way we can handle the text raster later on and can
            //   continue to ignore information about the window and DPI until finally drawing to
            //   the frame.
            use draw::primitive::path::PathInit;
            let path: PathInit<S> = Default::default();
            let mut empty_text = String::new();
            let ctxt = DrawingContext {
                mesh,
                fill_tessellator,
                path_event_buffer,
                glyph_cache,
                text_buffer: &mut empty_text,
            };
            let path = path.fill().color(color).events(ctxt, text.path_events());
            path
        });
        path.into_drawn(draw)
    }
}

impl<S> SetOrientation<S> for Text<S> {
    fn properties(&mut self) -> &mut orientation::Properties<S> {
        SetOrientation::properties(&mut self.spatial)
    }
}

impl<S> SetPosition<S> for Text<S> {
    fn properties(&mut self) -> &mut position::Properties<S> {
        SetPosition::properties(&mut self.spatial)
    }
}

impl<S> SetDimensions<S> for Text<S> {
    fn properties(&mut self) -> &mut dimension::Properties<S> {
        SetDimensions::properties(&mut self.spatial)
    }
}

impl<S> SetColor<ColorScalar> for Text<S> {
    fn rgba_mut(&mut self) -> &mut Option<LinSrgba> {
        SetColor::rgba_mut(&mut self.style.color)
    }
}

// Primitive conversions.

impl<S> From<Text<S>> for Primitive<S> {
    fn from(prim: Text<S>) -> Self {
        Primitive::Text(prim)
    }
}

impl<S> Into<Option<Text<S>>> for Primitive<S> {
    fn into(self) -> Option<Text<S>> {
        match self {
            Primitive::Text(prim) => Some(prim),
            _ => None,
        }
    }
}
