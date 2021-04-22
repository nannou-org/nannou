use crate::widget::{LayoutCtx, Widget};
use nannou::geom::Vector2;

pub struct Container {
    size: Vector2,
}

impl Widget for Container {
    fn layout(&mut self, _ctx: LayoutCtx) -> Vector2 {
        unimplemented!();
        self.size
    }
}

impl From<Vector2> for Container {
    fn from(size: Vector2) -> Self {
        Self { size }
    }
}
