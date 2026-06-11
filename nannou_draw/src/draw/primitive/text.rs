use core::hash::BuildHasher;

use crate::draw::drawing::DrawingContext;
use crate::draw::mesh::MeshExt;
use crate::draw::primitive::Primitive;
use crate::draw::properties::spatial::{self, dimension, orientation, position};
use crate::draw::properties::{SetColor, SetDimensions, SetOrientation, SetPosition};
use crate::draw::{self, Drawing};
use crate::text::{self, Align, FontSize, Justify, Layout, Scalar, Wrap};
use bevy::platform::hash::FixedHasher;
use bevy::prelude::*;
use bevy::text::{
    FontAtlasKey, FontAtlasSet, FontHinting, FontSmoothing, GlyphCacheKey, ScaleCx,
    add_glyph_to_atlas, get_glyph_atlas_info,
};
use swash::FontRef;

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
pub type DrawingText<'a> = Drawing<'a, Text>;

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

impl<'a> DrawingText<'a> {
    /// The font size to use for the text.
    pub fn font_size(self, size: text::FontSize) -> Self {
        update_text_layout(&self.draw, self.index, |l| l.font_size(size));
        self
    }

    /// Specify that the **Text** should not wrap lines around the width.
    pub fn no_line_wrap(self) -> Self {
        update_text_layout(&self.draw, self.index, |l| l.no_line_wrap());
        self
    }

    /// Line wrap the **Text** at the beginning of the first word that exceeds the width.
    pub fn wrap_by_word(self) -> Self {
        update_text_layout(&self.draw, self.index, |l| l.wrap_by_word());
        self
    }

    /// Line wrap the **Text** at the beginning of the first character that exceeds the width.
    pub fn wrap_by_character(self) -> Self {
        update_text_layout(&self.draw, self.index, |l| l.wrap_by_character());
        self
    }

    /// A method for specifying the font family used for displaying the `Text`.
    pub fn font(self, family: impl Into<String>) -> Self {
        let family = family.into();
        update_text_layout(&self.draw, self.index, |l| l.font_family(family));
        self
    }

    /// Build the **Text** with the given **Style**.
    pub fn with_style(self, style: Style) -> Self {
        update_text(&self.draw, self.index, |text| text.style = style);
        self
    }

    /// Describe the end along the *x* axis to which the text should be aligned.
    pub fn justify(self, justify: text::Justify) -> Self {
        update_text_layout(&self.draw, self.index, |l| l.justify(justify));
        self
    }

    /// Align the text to the left of its bounding **Rect**'s *x* axis range.
    pub fn left_justify(self) -> Self {
        update_text_layout(&self.draw, self.index, |l| l.left_justify());
        self
    }

    /// Align the text to the middle of its bounding **Rect**'s *x* axis range.
    pub fn center_justify(self) -> Self {
        update_text_layout(&self.draw, self.index, |l| l.center_justify());
        self
    }

    /// Align the text to the right of its bounding **Rect**'s *x* axis range.
    pub fn right_justify(self) -> Self {
        update_text_layout(&self.draw, self.index, |l| l.right_justify());
        self
    }

    /// Specify how much vertical space should separate each line of text.
    pub fn line_spacing(self, spacing: text::Scalar) -> Self {
        update_text_layout(&self.draw, self.index, |l| l.line_spacing(spacing));
        self
    }

    /// Specify how the whole text should be aligned along the y axis of its bounding rectangle
    pub fn y_align_text(self, align: Align) -> Self {
        update_text_layout(&self.draw, self.index, |l| l.y_align(align));
        self
    }

    /// Align the top edge of the text with the top edge of its bounding rectangle.
    pub fn align_text_top(self) -> Self {
        update_text_layout(&self.draw, self.index, |l| l.align_top());
        self
    }

    /// Align the middle of the text with the middle of the bounding rect along the y axis.
    pub fn align_text_middle_y(self) -> Self {
        update_text_layout(&self.draw, self.index, |l| l.align_middle_y());
        self
    }

    /// Align the bottom edge of the text with the bottom edge of its bounding rectangle.
    pub fn align_text_bottom(self) -> Self {
        update_text_layout(&self.draw, self.index, |l| l.align_bottom());
        self
    }

    /// Set all the parameters via an existing `Layout`
    pub fn layout(self, layout: &Layout) -> Self {
        update_text_layout(&self.draw, self.index, |l| l.layout(layout));
        self
    }

    /// Set a color for each glyph.
    pub fn glyph_colors<I, C>(self, glyph_colors: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: Into<Color>,
    {
        let glyph_colors: Vec<Color> = glyph_colors.into_iter().map(|c| c.into()).collect();
        update_text(&self.draw, self.index, |text| {
            text.style.glyph_colors = glyph_colors
        });
        self
    }
}

// Update the inner `Text` of the primitive being drawn at `index`.
fn update_text(draw: &crate::draw::Draw, index: usize, f: impl FnOnce(&mut Text)) {
    crate::draw::drawing::with_primitive(draw, index, |prim| match prim {
        Primitive::Text(text) => f(text),
        _ => bevy::log::warn_once!("expected a `Text` primitive"),
    })
}

// Update the layout builder of the `Text` primitive being drawn at `index`.
fn update_text_layout(
    draw: &crate::draw::Draw,
    index: usize,
    f: impl FnOnce(text::layout::Builder) -> text::layout::Builder,
) {
    update_text(draw, index, |text| {
        let layout = std::mem::take(&mut text.style.layout);
        text.style.layout = f(layout);
    })
}

/// A run of glyph quads sampling a single font atlas texture.
pub(crate) struct TextQuadBatch {
    pub texture: Handle<Image>,
    pub mesh: Mesh,
}

impl Text {
    /// Lay out the text and emit one textured quad per glyph, rasterising glyphs into
    /// `bevy_text`'s cached font atlases as required.
    ///
    /// Glyph positions are computed at physical-pixel resolution (`scale_factor`) so that
    /// rasterised glyphs map 1:1 to screen pixels, then converted back to logical points.
    /// A new batch is started whenever consecutive glyphs sample different atlas textures.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_atlas_quads(
        self,
        text_buffer: &str,
        theme: &draw::Theme,
        transform: &Mat4,
        output_attachment_size: Vec2,
        scale_factor: f32,
        text_cx: &crate::text::font::SharedTextCx,
        font_atlas_set: &mut FontAtlasSet,
        images: &mut Assets<Image>,
        scale_cx: &mut ScaleCx,
    ) -> Vec<TextQuadBatch> {
        let s = &text_buffer[self.text.clone()];
        if s.is_empty() {
            return Vec::new();
        }

        let layout_params = self.style.layout.build();
        let w = self
            .spatial
            .dimensions
            .x
            .unwrap_or(output_attachment_size.x);
        let h = self
            .spatial
            .dimensions
            .y
            .unwrap_or(output_attachment_size.y);
        let x = self.spatial.position.point.x;
        let y = self.spatial.position.point.y;
        let rect = nannou_core::geom::Rect::from_x_y_w_h(x, y, w, h);

        let mut inner = text_cx.0.lock().unwrap();
        let text_obj =
            text::Text::layout_with_inner(&mut inner, s, &layout_params, rect, scale_factor);
        drop(inner);

        let default_color = self
            .style
            .color
            .unwrap_or_else(|| theme.fill(&draw::theme::Primitive::Text));
        let glyph_colors = &self.style.glyph_colors;

        let rect_center = Vec2::new(x, y);
        let pos_offset = text_obj.position_offset_value() + rect_center;

        // Rasterise with bevy's defaults so atlas entries are shared with bevy UI text.
        let font_smoothing = FontSmoothing::AntiAliased;
        let hinting = FontHinting::default();

        let mut batches: Vec<TextQuadBatch> = Vec::new();
        // Counts every positioned glyph (rendered or not) so `glyph_colors` indices
        // are stable regardless of which glyphs make it into an atlas.
        let mut glyph_index = 0;

        for line in text_obj.parley_layout().lines() {
            for item in line.items() {
                let parley::PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                    continue;
                };

                let run = glyph_run.run();
                let font = run.font();
                let font_size = run.font_size();
                let coords = run.normalized_coords();
                let font_atlas_key = FontAtlasKey {
                    id: font.data.id() as u32,
                    index: font.index,
                    font_size_bits: font_size.to_bits(),
                    variations_hash: FixedHasher.hash_one(coords),
                    hinting,
                    font_smoothing,
                };

                let Some(font_ref) = FontRef::from_index(font.data.as_ref(), font.index as usize)
                else {
                    glyph_index += glyph_run.positioned_glyphs().count();
                    continue;
                };

                let hint = hinting.is_enabled() && font_smoothing == FontSmoothing::AntiAliased;
                let mut scaler = scale_cx
                    .0
                    .builder(font_ref)
                    .size(font_size)
                    .hint(hint)
                    .normalized_coords(coords)
                    .build();

                for glyph in glyph_run.positioned_glyphs() {
                    let i = glyph_index;
                    glyph_index += 1;

                    let Ok(glyph_id) = u16::try_from(glyph.id) else {
                        continue;
                    };

                    let font_atlases = font_atlas_set.entry(font_atlas_key).or_default();
                    let atlas_info = get_glyph_atlas_info(font_atlases, GlyphCacheKey { glyph_id })
                        .map(Ok)
                        .unwrap_or_else(|| {
                            add_glyph_to_atlas(
                                font_atlases,
                                images,
                                &mut scaler,
                                font_smoothing,
                                glyph_id,
                            )
                        });
                    let Ok(atlas_info) = atlas_info else {
                        continue;
                    };

                    let size = atlas_info.rect.size();
                    if size.x <= 0.0 || size.y <= 0.0 {
                        continue;
                    }

                    let Some(atlas) = font_atlases
                        .iter()
                        .find(|atlas| atlas.texture.id() == atlas_info.texture)
                    else {
                        continue;
                    };

                    // Glyph centre in parley's y-down physical-pixel layout space,
                    // converted to nannou's y-up logical points.
                    let centre = size / 2.0 + Vec2::new(glyph.x, glyph.y) + atlas_info.offset;
                    let cx = pos_offset.x + centre.x / scale_factor;
                    let cy = pos_offset.y - centre.y / scale_factor;
                    let hw = size.x / (2.0 * scale_factor);
                    let hh = size.y / (2.0 * scale_factor);

                    // Monochrome glyphs are white-on-alpha in the atlas and tinted by
                    // vertex colour; colour glyphs (e.g. emoji) are sampled as-is.
                    let color = if atlas_info.is_alpha_mask {
                        glyph_colors.get(i).copied().unwrap_or(default_color)
                    } else {
                        Color::WHITE
                    };
                    let color_arr: [f32; 4] = LinearRgba::from(color).to_f32_array();

                    // UVs over the atlas; the image's y-down texel space performs the
                    // vertical flip relative to the y-up quad corners.
                    let atlas_size = atlas.texture_atlas.size.as_vec2();
                    let (uv_l, uv_r) = (
                        atlas_info.rect.min.x / atlas_size.x,
                        atlas_info.rect.max.x / atlas_size.x,
                    );
                    let (uv_t, uv_b) = (
                        atlas_info.rect.min.y / atlas_size.y,
                        atlas_info.rect.max.y / atlas_size.y,
                    );

                    let batch = match batches.last_mut() {
                        Some(batch) if batch.texture.id() == atlas_info.texture => batch,
                        _ => {
                            batches.push(TextQuadBatch {
                                texture: atlas.texture.clone(),
                                mesh: Mesh::init(),
                            });
                            batches.last_mut().unwrap()
                        }
                    };

                    let mesh = &mut batch.mesh;
                    let base = mesh.points().len() as u32;
                    // Corners ordered top-left, top-right, bottom-right, bottom-left.
                    let corners = [
                        (cx - hw, cy + hh, uv_l, uv_t),
                        (cx + hw, cy + hh, uv_r, uv_t),
                        (cx + hw, cy - hh, uv_r, uv_b),
                        (cx - hw, cy - hh, uv_l, uv_b),
                    ];
                    for (px, py, u, v) in corners {
                        let p = *transform * Vec4::new(px, py, 0.0, 1.0);
                        mesh.points_mut().push([p.x, p.y, p.z]);
                        mesh.colors_mut().push(color_arr);
                        mesh.tex_coords_mut().push([u, v]);
                        mesh.normals_mut().push([0.0, 0.0, 1.0]);
                    }
                    // Two triangles wound counter-clockwise in y-up space.
                    for idx in [0, 3, 2, 0, 2, 1] {
                        mesh.push_index(base + idx);
                    }
                }
            }
        }

        batches
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
