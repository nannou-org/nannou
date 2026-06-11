use nannou_core::geom;

use crate::draw::{Draw, drawing};

pub trait SetTexCoords: Sized {
    fn tex_coords_mut(&mut self) -> &mut Option<geom::Rect>;

    fn area(mut self, area: geom::Rect) -> Self {
        *self.tex_coords_mut() = Some(area);
        self
    }
}

impl SetTexCoords for Option<geom::Rect> {
    fn tex_coords_mut(&mut self) -> &mut Option<geom::Rect> {
        self
    }
}

// Set the texture coordinate area of the primitive being drawn at `index`.
pub(crate) fn set_tex_coords_area(draw: &Draw, index: usize, area: geom::Rect) {
    drawing::with_primitive(draw, index, |prim| match prim.tex_coords_mut() {
        Some(tex_coords) => *tex_coords = Some(area),
        None => bevy::log::warn_once!("drawing primitive does not support `tex_coords`"),
    })
}
