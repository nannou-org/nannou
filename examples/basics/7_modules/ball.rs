use nannou::prelude::*;

pub struct Ball {
    pub position: Point2,
    color: Srgb<u8>,
}

impl Ball {
    pub fn new(color: Srgb<u8>) -> Self {
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
