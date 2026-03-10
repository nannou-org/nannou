//! Glyph outline extraction via skrifa's OutlinePen.

use bevy::prelude::*;
use lyon::path::PathEvent;
use skrifa::instance::{LocationRef, Size};
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::raw::TableProvider;
use skrifa::{FontRef, GlyphId, MetadataProvider};

/// A pen that converts skrifa outline commands into lyon `PathEvent` values.
///
/// Skrifa outlines are in font units with y-up. We apply an origin offset
/// and a scale factor (font_size / units_per_em) to convert to layout space.
struct LyonOutlinePen {
    events: Vec<PathEvent>,
    /// The offset applied to every point (glyph position in layout space).
    origin: lyon::math::Point,
    /// Scale factor: font_size / units_per_em.
    scale: f32,
    /// Whether to negate y for conversion from parley top-down to nannou y-up.
    flip_y: bool,
    /// Tracks the current point for building path events.
    current: lyon::math::Point,
    /// The start of the current sub-path.
    start: lyon::math::Point,
}

impl LyonOutlinePen {
    fn new(origin: lyon::math::Point, scale: f32, flip_y: bool) -> Self {
        Self {
            events: Vec::new(),
            origin,
            scale,
            flip_y,
            current: lyon::math::point(0.0, 0.0),
            start: lyon::math::point(0.0, 0.0),
        }
    }

    /// Transform a point from font units to layout space.
    fn point(&self, x: f32, y: f32) -> lyon::math::Point {
        let y_sign = if self.flip_y { -1.0 } else { 1.0 };
        lyon::math::point(
            self.origin.x + x * self.scale,
            self.origin.y + y * self.scale * y_sign,
        )
    }
}

impl OutlinePen for LyonOutlinePen {
    fn move_to(&mut self, x: f32, y: f32) {
        let p = self.point(x, y);
        self.start = p;
        self.current = p;
        self.events.push(PathEvent::Begin { at: p });
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let from = self.current;
        let to = self.point(x, y);
        self.current = to;
        self.events.push(PathEvent::Line { from, to });
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        let from = self.current;
        let ctrl = self.point(cx0, cy0);
        let to = self.point(x, y);
        self.current = to;
        self.events.push(PathEvent::Quadratic { from, ctrl, to });
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        let from = self.current;
        let ctrl1 = self.point(cx0, cy0);
        let ctrl2 = self.point(cx1, cy1);
        let to = self.point(x, y);
        self.current = to;
        self.events.push(PathEvent::Cubic {
            from,
            ctrl1,
            ctrl2,
            to,
        });
    }

    fn close(&mut self) {
        self.events.push(PathEvent::End {
            last: self.current,
            first: self.start,
            close: true,
        });
    }
}

/// Extract lyon path events for all glyphs in a parley layout.
///
/// `pos_offset` is the Vec2 that maps parley's top-down coordinate origin
/// into nannou's y-up coordinate space. It is typically computed by
/// `Text::position_offset()`.
pub fn text_path_events(
    parley_layout: &parley::Layout<Color>,
    pos_offset: Vec2,
) -> Vec<PathEvent> {
    let mut all_events = Vec::new();

    for line in parley_layout.lines() {
        let baseline = line.metrics().baseline;

        for item in line.items() {
            let parley::PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                continue;
            };

            let run = glyph_run.run();
            let font_data = run.font();
            let font_size = run.font_size();

            // Build a skrifa FontRef from the font data blob.
            let font_blob = &font_data.data;
            let font_index = font_data.index;
            let Ok(font_ref) = FontRef::from_index(font_blob.as_ref(), font_index) else {
                continue;
            };

            let outlines = font_ref.outline_glyphs();
            let upem = font_ref
                .head()
                .map(|h| h.units_per_em() as f32)
                .unwrap_or(1000.0);
            let scale = font_size / upem;

            for glyph in glyph_run.positioned_glyphs() {
                let glyph_id = GlyphId::new(glyph.id as u32);
                let Some(outline_glyph) = outlines.get(glyph_id) else {
                    continue;
                };

                // Glyph position in parley space (top-down).
                // Convert to nannou y-up space using pos_offset.
                let gx = pos_offset.x + glyph.x;
                // pos_offset.y is the top of the text block in nannou coords.
                // Subtract baseline to get the baseline y in nannou coords.
                let gy = pos_offset.y - baseline;

                let origin = lyon::math::point(gx, gy);

                // Skrifa outlines are in font units, y-up from baseline.
                // We need to keep y-up (flip_y = false) because nannou is y-up.
                let mut pen = LyonOutlinePen::new(origin, scale, false);

                // Draw at identity size (1 upem) — we handle scaling in the pen.
                let settings =
                    DrawSettings::unhinted(Size::unscaled(), LocationRef::default());
                let _ = outline_glyph.draw(settings, &mut pen);

                all_events.extend(pen.events);
            }
        }
    }

    all_events
}
