use bevy::prelude::Color;
use crate::draw::Draw;
use nannou_core::color::{self, IntoColor, Srgb, Srgba};

/// A type used to update the background colour.
pub struct Background<'a> {
    draw: &'a Draw,
}

/// Begin coloring the background.
pub fn new<'a>(draw: &'a Draw) -> Background<'a> {
    Background { draw }
}

impl<'a> Background<'a> {
    /// Clear the background with the given color.
    ///
    /// This method supports any color type that can be converted into RGBA.
    ///
    /// Colors that have no alpha channel will be given an opaque alpha channel value `1.0`.
    pub fn color<C>(self, color: C) -> Self
    where
        C: Into<Color>,
    {
        if let Ok(mut state) = self.draw.state.try_write() {
            state.background_color = Some(color.into());
        }
        self
    }

    /// Specify the color via red, green and blue channels.
    pub fn rgb(self, r: f32, g: f32, b: f32) -> Self {
        self.color(Color::rgb(r, g, b))
    }

    /// Specify the color via red, green, blue and alpha channels.
    pub fn rgba(self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.color(Color::rgba(r, g, b, a))
    }

    /// Specify the color via hue, saturation and luminance.
    ///
    /// If you're looking for HSV or HSB, use the `hsv` method instead.
    ///
    /// The given hue expects a value between `0.0` and `1.0` where `0.0` is 0 degress and `1.0` is
    /// 360 degrees (or 2 PI radians).
    ///
    /// See the [wikipedia entry](https://en.wikipedia.org/wiki/HSL_and_HSV) for more details on
    /// this color space.
    pub fn hsl(self, h: f32, s: f32, l: f32) -> Self {
        let hue = h * 360.0;
        self.color(Color::hsl(hue, s, l))
    }

    /// Specify the color via hue, saturation, luminance and an alpha channel.
    ///
    /// If you're looking for HSVA or HSBA, use the `hsva` method instead.
    ///
    /// The given hue expects a value between `0.0` and `1.0` where `0.0` is 0 degress and `1.0` is
    /// 360 degrees (or 2 PI radians).
    ///
    /// See the [wikipedia entry](https://en.wikipedia.org/wiki/HSL_and_HSV) for more details on
    /// this color space.
    pub fn hsla(self, h: f32, s: f32, l: f32, a: f32) -> Self {
        let hue = h * 360.0;
        self.color(Color::hsla(hue, s, l, a))
    }

    // /// Specify the color via hue, saturation and *value* (brightness).
    // ///
    // /// This is sometimes also known as "hsb".
    // ///
    // /// The given hue expects a value between `0.0` and `1.0` where `0.0` is 0 degress and `1.0` is
    // /// 360 degrees (or 2 PI radians).
    // ///
    // /// See the [wikipedia entry](https://en.wikipedia.org/wiki/HSL_and_HSV) for more details on
    // /// this color space.
    // pub fn hsv(self, h: f32, s: f32, v: f32) -> Self {
    //     let hue = h * 360.0;
    //     self.color(color::Hsv::new(hue, s, v))
    // }
    //
    // /// Specify the color via hue, saturation, *value* (brightness) and an alpha channel.
    // ///
    // /// This is sometimes also known as "hsba".
    // ///
    // /// The given hue expects a value between `0.0` and `1.0` where `0.0` is 0 degress and `1.0` is
    // /// 360 degrees (or 2 PI radians).
    // ///
    // /// See the [wikipedia entry](https://en.wikipedia.org/wiki/HSL_and_HSV) for more details on
    // /// this color space.
    // pub fn hsva(self, h: f32, s: f32, v: f32, a: f32) -> Self {
    //     let hue = h * 360.0;
    //     self.color(color::Hsva::new(hue, s, v, a))
    // }
}
