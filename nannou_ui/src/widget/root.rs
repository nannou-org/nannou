use crate::widget::{LayoutCtx, Widget};
use nannou::geom::Vector2;

pub struct Root;

impl Widget for Root {
    fn layout(&mut self, _ctx: LayoutCtx) -> Vector2 {
        Vector2::new(0.0, 0.0)
    }
}
