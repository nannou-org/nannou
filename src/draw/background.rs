use color::{self, Rgb, Rgba};
use draw::properties::{ColorScalar, IntoRgba};
use draw::Draw;
use geom;
use math::BaseFloat;

/// A type used to update the background colour.
pub struct Background<'a, S = geom::DefaultScalar>
where
    S: 'a + BaseFloat,
{
    draw: &'a Draw<S>,
}

/// Begin coloring the background.
pub fn new<'a, S>(draw: &'a Draw<S>) -> Background<'a, S>
where
    S: BaseFloat,
{
    Background { draw }
}

impl<'a, S> Background<'a, S>
where
    S: BaseFloat,
{
    /// Clear the background with the given color.
    ///
    /// This method supports any color type that can be converted into RGBA.
    ///
    /// Colors that have no alpha channel will be given an opaque alpha channel value `1.0`.
    pub fn color<C>(self, color: C) -> Self
    where
        C: IntoRgba<ColorScalar>,
    {
        if let Ok(mut state) = self.draw.state.try_borrow_mut() {
            state.background_color = Some(color.into_rgba());
        }
        self
    }

    /// Specify the color via red, green and blue channels.
    pub fn rgb(self, r: ColorScalar, g: ColorScalar, b: ColorScalar) -> Self {
        self.color(Rgb::new(r, g, b))
    }

    /// Specify the color via red, green, blue and alpha channels.
    pub fn rgba(self, r: ColorScalar, g: ColorScalar, b: ColorScalar, a: ColorScalar) -> Self {
        self.color(Rgba::new(r, g, b, a))
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
    pub fn hsl(self, h: ColorScalar, s: ColorScalar, l: ColorScalar) -> Self {
        let hue = h * 360.0;
        self.color(color::Hsl::new(hue.into(), s, l))
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
    pub fn hsla(self, h: ColorScalar, s: ColorScalar, l: ColorScalar, a: ColorScalar) -> Self {
        let hue = h * 360.0;
        self.color(color::Hsla::new(hue.into(), s, l, a))
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
    pub fn hsv(self, h: ColorScalar, s: ColorScalar, v: ColorScalar) -> Self {
        let hue = h * 360.0;
        self.color(color::Hsv::new(hue.into(), s, v))
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
    pub fn hsva(self, h: ColorScalar, s: ColorScalar, v: ColorScalar, a: ColorScalar) -> Self
    where
        S: Into<color::RgbHue<S>>,
    {
        let hue = h * 360.0;
        self.color(color::Hsva::new(hue.into(), s, v, a))
    }
}
