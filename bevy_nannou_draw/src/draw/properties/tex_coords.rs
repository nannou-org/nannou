use nannou_core::geom;

pub trait SetTexCoords : Sized {
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