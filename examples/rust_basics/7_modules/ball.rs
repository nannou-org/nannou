use nannou::prelude::*;

pub struct Ball {
    pub position: Point2,
    color: Srgba,
}

impl Ball {
    pub fn new(color: Srgba) -> Self {
        let position = pt2(0.0, 0.0);
        Ball { position, color }
    }

    pub fn display(&self, draw: &Draw) {
        draw.ellipse()
            .xy(self.position)
            .radius(100.0)
            .color(self.color);
    }
}
