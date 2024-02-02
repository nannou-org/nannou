use num_traits::Float;
use nannou_core::color::{self, Component, IntoLinSrgba, LinSrgba};

/// A **Srgba** type with the default Scalar.
pub type DefaultSrgba = color::Srgba<color::DefaultScalar>;

/// A **LinSrgba** type with the default Scalar.
pub type DefaultLinSrgba = color::LinSrgba<color::DefaultScalar>;

/// Nodes that support setting colors.
pub trait SetColor<S>: Sized
where
    S: Component,
{
    /// Provide a mutable reference to the RGBA field which can be used for setting colors.
    fn rgba_mut(&mut self) -> &mut Option<LinSrgba<S>>;

    /// Specify a color.
    ///
    /// This method supports any color type that can be converted into RGBA.
    ///
    /// Colors that have no alpha channel will be given an opaque alpha channel value `1.0`.
    fn color<C>(mut self, color: C) -> Self
    where
        C: IntoLinSrgba<S>,
    {
        *self.rgba_mut() = Some(color.into_lin_srgba());
        self
    }

    /// Specify the color via red, green and blue channels.
    fn rgb<T>(self, r: T, g: T, b: T) -> Self
    where
        T: Component,
        S: Float,
    {
        self.color(color::Srgb::new(r, g, b))
    }

    /// Specify the color via red, green and blue channels as bytes
    fn rgb8(self, r: u8, g: u8, b: u8) -> Self
    where
        S: Float,
    {
        self.color(color::Srgb::<u8>::new(r, g, b))
    }

    /// Specify the color via red, green, blue and alpha channels.
    fn rgba<T>(self, r: T, g: T, b: T, a: T) -> Self
    where
        T: Component,
        S: Float,
    {
        self.color(color::Srgba::new(r, g, b, a))
    }

    /// Specify the color via red, green, blue and alpha channels as bytes
    fn rgba8(self, r: u8, g: u8, b: u8, a: u8) -> Self
    where
        S: Float,
    {
        self.color(color::Srgba::<u8>::new(r, g, b, a))
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
    fn hsl(self, h: S, s: S, l: S) -> Self
    where
        S: Float + Into<color::RgbHue<S>>,
    {
        let hue = color::RgbHue::from_degrees(h * S::from(360.0).unwrap());
        self.color(color::Hsl::new(hue, s, l))
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
    fn hsla(self, h: S, s: S, l: S, a: S) -> Self
    where
        S: Float + Into<color::RgbHue<S>>,
    {
        let hue = color::RgbHue::from_degrees(h * S::from(360.0).unwrap());
        self.color(color::Hsla::new(hue, s, l, a))
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
    fn hsv(self, h: S, s: S, v: S) -> Self
    where
        S: Float,
    {
        let hue = color::RgbHue::from_degrees(h * S::from(360.0).unwrap());
        self.color(color::Hsv::new(hue, s, v))
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
    fn hsva(self, h: S, s: S, v: S, a: S) -> Self
    where
        S: Float,
    {
        let hue = color::RgbHue::from_degrees(h * S::from(360.0).unwrap());
        self.color(color::Hsva::new(hue, s, v, a))
    }

    /// Specify the color as gray scale
    ///
    /// The given g expects a value between `0.0` and `1.0` where `0.0` is black and `1.0` is white
    fn gray<T>(self, g: T) -> Self
    where
        T: Component,
        S: Float,
    {
        self.color(color::Srgb::new(g, g, g))
    }
}

impl<S> SetColor<S> for Option<LinSrgba<S>>
where
    S: Component,
{
    fn rgba_mut(&mut self) -> &mut Option<LinSrgba<S>> {
        self
    }
}
