use crate::draw::drawing::DrawingContext;
use crate::draw::primitive::Primitive;
use crate::draw::properties::spatial::{self, dimension, orientation, position};
use crate::draw::properties::{
    ColorScalar, LinSrgba, SetColor, SetDimensions, SetOrientation, SetPosition,
};
use crate::draw::{theme, Drawing};
use crate::geom;
use crate::math::BaseFloat;
use crate::text;

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
    pub line_spacing: Option<text::Scalar>,
    pub maybe_wrap: Option<Option<Wrap>>,
    pub font_size: Option<text::FontSize>,
    pub justify: Option<text::Justify>,
    pub font_id: Option<Option<text::font::Id>>,
}

/// The way in which text should wrap around the width.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Wrap {
    /// Wrap at the first character that exceeds the width.
    Character,
    /// Wrap at the first word that exceeds the width.
    Whitespace,
}

/// The drawing context for the **Text** primitive.
pub type DrawingText<'a, S = geom::scalar::Default> = Drawing<'a, Text<S>, S>;

pub const DEFAULT_WRAP: Option<Wrap> = Some(Wrap::Whitespace);
pub const DEFAULT_FONT_SIZE: u32 = 12;
pub const DEFAULT_LINE_SPACING: f32 = 1.0;

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

    /// The font size to use for the text.
    pub fn font_size(mut self, size: text::FontSize) -> Self {
        self.style.font_size = Some(size);
        self
    }

    /// Specify that the **Text** should not wrap lines around the width.
    pub fn no_line_wrap(mut self) -> Self {
        self.style.maybe_wrap = Some(None);
        self
    }

    /// Line wrap the **Text** at the beginning of the first word that exceeds the width.
    pub fn wrap_by_word(mut self) -> Self {
        self.style.maybe_wrap = Some(Some(Wrap::Whitespace));
        self
    }

    /// Line wrap the **Text** at the beginning of the first character that exceeds the width.
    pub fn wrap_by_character(mut self) -> Self {
        self.style.maybe_wrap = Some(Some(Wrap::Character));
        self
    }

    /// A method for specifying the `Font` used for displaying the `Text`.
    pub fn font_id(mut self, font_id: text::font::Id) -> Self {
        self.style.font_id = Some(Some(font_id));
        self
    }

    /// Build the **Text** with the given **Style**.
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Describe the end along the *x* axis to which the text should be aligned.
    pub fn justify(mut self, justify: text::Justify) -> Self {
        self.style.justify = Some(justify);
        self
    }

    /// Align the text to the left of its bounding **Rect**'s *x* axis range.
    pub fn left_justify(self) -> Self {
        self.justify(text::Justify::Left)
    }

    /// Align the text to the middle of its bounding **Rect**'s *x* axis range.
    pub fn center_justify(self) -> Self {
        self.justify(text::Justify::Center)
    }

    /// Align the text to the right of its bounding **Rect**'s *x* axis range.
    pub fn right_justify(self) -> Self {
        self.justify(text::Justify::Right)
    }

    /// Specify how much vertical space should separate each line of text.
    pub fn line_spacing(mut self, spacing: text::Scalar) -> Self {
        self.style.line_spacing = Some(spacing);
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
    pub fn font_id(self, font_id: text::font::Id) -> Self {
        self.map_ty(|ty| ty.font_id(font_id))
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
}

// impl<S> IntoDrawn<S> for Text<S>
// where
//     S: BaseFloat,
// {
//     type Vertices = draw::properties::VerticesFromRanges;
//     type Indices = draw::properties::IndicesFromRange;
//     fn into_drawn(self, draw: Draw<S>) -> Drawn<S, Self::Vertices, Self::Indices> {
//         let Text {
//             spatial,
//             style,
//             text,
//         } = self;
//
//         let maybe_wrap = style.maybe_wrap.unwrap_or(DEFAULT_WRAP);
//         let font_size = style.font_size(DEFAULT_FONT_SIZE);
//
//         let font = match style.font_id(&ui.theme)
//             .or(ui.fonts.ids().next())
//             .and_then(|id| ui.fonts.get(id))
//         {
//             Some(font) => font,
//             None => return,
//         };
//
//         // Produces an iterator yielding info for each line within the `text`.
//         let line_infos = || match maybe_wrap {
//             None =>
//                 text::line::infos(text, font, font_size),
//             Some(Wrap::Character) =>
//                 text::line::infos(text, font, font_size).wrap_by_character(rect.w()),
//             Some(Wrap::Whitespace) =>
//                 text::line::infos(text, font, font_size).wrap_by_whitespace(rect.w()),
//         };
//
//         // If the string is different, we must update both the string and the line breaks.
//         if &state.string[..] != text {
//             state.update(|state| {
//                 state.string = text.to_owned();
//                 state.line_infos = new_line_infos().collect();
//             });
//
//         // Otherwise, we'll check to see if we have to update the line breaks.
//         } else {
//             use utils::write_if_different;
//             use std::borrow::Cow;
//
//             // Compare the line_infos and only collect the new ones if they are different.
//             let maybe_new_line_infos = {
//                 let line_infos = &state.line_infos[..];
//                 match write_if_different(line_infos, new_line_infos()) {
//                     Cow::Owned(new) => Some(new),
//                     _ => None,
//                 }
//             };
//
//             if let Some(new_line_infos) = maybe_new_line_infos {
//                 state.update(|state| state.line_infos = new_line_infos);
//             }
//         }
//
//
//
//         // 1. Retrieve text slice from intermediary text buffer.
//         // 2. Insert a rect for every glyph into the mesh while updating glyph cache pixel buffer.
//         // 3.
//
//         let dimensions = spatial::dimension::Properties::default();
//         let spatial = spatial::Properties {
//             dimensions,
//             orientation,
//             position,
//         };
//         let color = color.or_else(|| {
//             if vertex_data_ranges.colors.len() >= vertex_data_ranges.points.len() {
//                 return None;
//             }
//             Some(draw.theme().fill_lin_srgba(&draw::theme::Primitive::Path))
//         });
//         let vertices = draw::properties::VerticesFromRanges::new(vertex_data_ranges, color);
//         let indices = draw::properties::IndicesFromRange::new(index_range, min_index);
//         (spatial, vertices, indices)
//     }
// }

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
