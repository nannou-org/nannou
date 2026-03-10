use bevy::prelude::*;

use crate::draw::drawing::DrawingContext;
use crate::draw::primitive::Primitive;
use crate::draw::properties::spatial::{self, dimension, orientation, position};
use crate::draw::properties::{SetColor, SetDimensions, SetOrientation, SetPosition};
use crate::draw::{self, Drawing};
use crate::render::ShaderModel;
use crate::text::{self, Align, FontSize, Justify, Layout, Scalar, Wrap};

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

    /// A method for specifying the font family used for displaying the `Text`.
    pub fn font(self, family: impl Into<String>) -> Self {
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

    /// Specify how the whole text should be aligned along the y axis of its bounding rectangle
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

    /// A method for specifying the font family used for displaying the `Text`.
    pub fn font(self, family: impl Into<String>) -> Self {
        self.map_ty(|ty| ty.font(family))
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

    /// Set a color for each glyph.
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
    fn render_primitive(self, ctxt: draw::render::RenderContext, mesh: &mut Mesh) {
        let draw::render::RenderContext {
            text_buffer,
            theme,
            transform,
            fill_tessellator,
            output_attachment_size,
            text_cx,
            ..
        } = ctxt;

        // Extract the text string from the shared text buffer.
        let s = &text_buffer[self.text.clone()];
        if s.is_empty() {
            return;
        }

        // Build the layout using the shared text context.
        let layout_params = self.style.layout.build();

        // Compute a bounding rect from the spatial properties.
        let w = self.spatial.dimensions.x.unwrap_or(output_attachment_size.x);
        let h = self.spatial.dimensions.y.unwrap_or(output_attachment_size.y);
        let x = self.spatial.position.point.x;
        let y = self.spatial.position.point.y;
        let rect = nannou_core::geom::Rect::from_x_y_w_h(x, y, w, h);

        // Lock the text context and build the parley layout.
        let mut inner = text_cx.0.lock().unwrap();
        let text_obj = text::Text::layout_with_inner(&mut inner, s, &layout_params, rect);
        drop(inner);

        // Get the path events for every glyph.
        let path_events = text_obj.path_events();
        if path_events.is_empty() {
            return;
        }

        // Default color.
        let default_color = self
            .style
            .color
            .unwrap_or_else(|| theme.fill(&draw::theme::Primitive::Text));

        let color_arr: [f32; 4] = LinearRgba::from(default_color).to_f32_array();

        // Tessellate the glyph outlines into the mesh.
        use lyon::tessellation::{BuffersBuilder, FillOptions, VertexBuffers};

        let mut buffers: VertexBuffers<[f32; 3], u32> = VertexBuffers::new();
        {
            let mut builder =
                BuffersBuilder::new(&mut buffers, |vertex: lyon::tessellation::FillVertex| {
                    let p = vertex.position();
                    let transformed = *transform * Vec4::new(p.x, p.y, 0.0, 1.0);
                    [transformed.x, transformed.y, transformed.z]
                });

            let _ = fill_tessellator.tessellate(
                path_events.iter().copied(),
                &FillOptions::default(),
                &mut builder,
            );
        }

        if buffers.vertices.is_empty() {
            return;
        }

        // Append the tessellated vertices and indices to the mesh.
        use crate::draw::mesh::MeshExt;
        let base_idx = mesh.points().len() as u32;

        mesh.points_mut().extend(buffers.vertices.iter());
        mesh.normals_mut()
            .extend(std::iter::repeat_n([0.0, 0.0, 1.0], buffers.vertices.len()));
        mesh.colors_mut()
            .extend(std::iter::repeat_n(color_arr, buffers.vertices.len()));
        mesh.tex_coords_mut()
            .extend(std::iter::repeat_n([0.0, 0.0], buffers.vertices.len()));
        for idx in &buffers.indices {
            mesh.push_index(idx + base_idx);
        }
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
