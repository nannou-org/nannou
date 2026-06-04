use bevy::color::Color;

pub trait ColorExt {
    fn gray(value: f32) -> Color {
        Color::srgb(value, value, value)
    }
}

impl ColorExt for Color {}
