use bevy::prelude::Color;

/// Nodes that support setting colors.
pub trait SetColor: Sized {
    /// Provide a mutable reference to the RGBA field which can be used for setting colors.
    fn color_mut(&mut self) -> &mut Option<Color>;

    /// Specify a color.
    ///
    /// This method supports any color type that can be converted into RGBA.
    ///
    /// Colors that have no alpha channel will be given an opaque alpha channel value `1.0`.
    fn color<C>(mut self, color: C) -> Self
    where
        C: Into<Color>,
    {
        *self.color_mut() = Some(color.into());
        self
    }

    /// Specify the color via red, green and blue channels.
    fn rgb(self, r: f32, g: f32, b: f32) -> Self {
        self.color(Color::rgb(r, g, b))
    }

    /// Specify the color via red, green and blue channels as bytes
    fn rgb8(self, r: u8, g: u8, b: u8) -> Self {
        self.color(Color::rgb_u8(r, g, b))
    }

    /// Specify the color via red, green, blue and alpha channels.
    fn rgba(self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.color(Color::rgba(r, g, b, a))
    }

    /// Specify the color via red, green, blue and alpha channels as bytes
    fn rgba8(self, r: u8, g: u8, b: u8, a: u8) -> Self {
        self.color(Color::rgba_u8(r, g, b, a))
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
    fn hsl(self, h: f32, s: f32, l: f32) -> Self {
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
    fn hsla(self, h: f32, s: f32, l: f32, a: f32) -> Self {
        let hue = h * 360.0;
        self.color(Color::hsla(hue, s, l, a))
    }

    /// Specify the color via hue, saturation and *value* (brightness).
    ///
    /// This is sometimes also known as "hsb".
    ///
    /// The given hue expects a value between `0.0` and `1.0` where `0.0` is 0 degress and `1.0` is
    /// 360 degrees (or 2 PI radians).
    ///
    /// See the [wikipedia entry](https://en.wikipedia.org/wiki/HSL_and_HSV) for more details on
    /// this color space.
    fn hsv(self, h: f32, s: f32, v: f32) -> Self {
        let hue = h * 360.0;
        self.color(Color::hsv(hue, s, v))
    }

    /// Specify the color via hue, saturation, *value* (brightness) and an alpha channel.
    ///
    /// This is sometimes also known as "hsba".
    ///
    /// The given hue expects a value between `0.0` and `1.0` where `0.0` is 0 degress and `1.0` is
    /// 360 degrees (or 2 PI radians).
    ///
    /// See the [wikipedia entry](https://en.wikipedia.org/wiki/HSL_and_HSV) for more details on
    /// this color space.
    fn hsva(self, h: f32, s: f32, v: f32, a: f32) -> Self {
        let hue = h * 360.0;
        self.color(Color::hsva(hue, s, v, a))
    }

    /// Specify the color as gray scale
    ///
    /// The given g expects a value between `0.0` and `1.0` where `0.0` is black and `1.0` is white
    fn gray(self, g: f32) -> Self {
        self.color(Color::rgb(g, g, g))
    }
}

impl SetColor for Option<Color> {
    fn color_mut(&mut self) -> &mut Option<Color> {
        self
    }
}
