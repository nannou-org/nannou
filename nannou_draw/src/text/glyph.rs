//! Glyph outline extraction via skrifa.

use bevy::prelude::*;
use lyon::path::PathEvent;
use skrifa::instance::{LocationRef, Size};
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::raw::TableProvider;
use skrifa::{FontRef, GlyphId, MetadataProvider};

/// Converts skrifa outline commands into lyon `PathEvent` values.
struct LyonOutlinePen {
    events: Vec<PathEvent>,
    origin: lyon::math::Point,
    scale: f32,
    flip_y: bool,
    current: lyon::math::Point,
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

    /// Transform a font-unit point to layout space.
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
pub fn text_path_events(parley_layout: &parley::Layout<Color>, pos_offset: Vec2) -> Vec<PathEvent> {
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

                let gx = pos_offset.x + glyph.x;
                let gy = pos_offset.y - baseline;
                let origin = lyon::math::point(gx, gy);
                let mut pen = LyonOutlinePen::new(origin, scale, false);
                let settings = DrawSettings::unhinted(Size::unscaled(), LocationRef::default());
                let _ = outline_glyph.draw(settings, &mut pen);

                all_events.extend(pen.events);
            }
        }
    }

    all_events
}

/// Extract lyon path events for each glyph separately, enabling per-glyph coloring.
pub fn per_glyph_path_events(
    parley_layout: &parley::Layout<Color>,
    pos_offset: Vec2,
) -> Vec<Vec<PathEvent>> {
    let mut per_glyph = Vec::new();

    for line in parley_layout.lines() {
        let baseline = line.metrics().baseline;

        for item in line.items() {
            let parley::PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                continue;
            };

            let run = glyph_run.run();
            let font_data = run.font();
            let font_size = run.font_size();

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
                    per_glyph.push(Vec::new());
                    continue;
                };

                let gx = pos_offset.x + glyph.x;
                let gy = pos_offset.y - baseline;
                let origin = lyon::math::point(gx, gy);

                let mut pen = LyonOutlinePen::new(origin, scale, false);
                let settings = DrawSettings::unhinted(Size::unscaled(), LocationRef::default());
                let _ = outline_glyph.draw(settings, &mut pen);

                per_glyph.push(pen.events);
            }
        }
    }

    per_glyph
}
