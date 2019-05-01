use nannou::prelude::*;

pub struct Ball {
    pub position: Point2,
    color: Rgba,
}

impl Ball {
    pub fn new(color: Rgba) -> Self {
        let position = pt2(0.0, 0.0);
        Ball { position, color }
    }

    pub fn display(&self, draw: &app::Draw) {
        draw.ellipse()
            .xy(self.position)
            .radius(100.0)
            .color(self.color);
    }
}
