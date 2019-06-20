use crate::color::white_point::D65;
use crate::color::{self, encoding, Alpha, Component, IntoColor, RgbHue, Srgb, Srgba};
use crate::math::num_traits::Float;

/// An **Srgba** type with the default Scalar.
///
/// Used by the **draw::properties::Common** type.
pub type DefaultSrgba = Srgba<color::DefaultScalar>;

/// Types that may be converted directly into an RGBA color.
pub trait IntoSrgba<S>
where
    S: Component,
{
    /// Convert self into RGBA.
    fn into_srgba(self) -> Srgba<S>;
}

/// Nodes that support setting colors.
pub trait SetColor<S>: Sized
where
    S: Component,
{
    /// Provide a mutable reference to the RGBA field which can be used for setting colors.
    fn rgba_mut(&mut self) -> &mut Option<Srgba<S>>;

    /// Specify a color.
    ///
    /// This method supports any color type that can be converted into RGBA.
    ///
    /// Colors that have no alpha channel will be given an opaque alpha channel value `1.0`.
    fn color<C>(mut self, color: C) -> Self
    where
        C: IntoSrgba<S>,
    {
        *self.rgba_mut() = Some(color.into_srgba());
        self
    }

    /// Specify the color via red, green and blue channels.
    fn rgb(self, r: S, g: S, b: S) -> Self {
        self.color(Srgb::new(r, g, b))
    }

    /// Specify the color via red, green, blue and alpha channels.
    fn rgba(self, r: S, g: S, b: S, a: S) -> Self {
        self.color(Srgba::new(r, g, b, a))
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
        let hue = RgbHue::from_degrees(h * S::from(360.0).unwrap());
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
        let hue = RgbHue::from_degrees(h * S::from(360.0).unwrap());
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
        let hue = RgbHue::from_degrees(h * S::from(360.0).unwrap());
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
        let hue = RgbHue::from_degrees(h * S::from(360.0).unwrap());
        self.color(color::Hsva::new(hue, s, v, a))
    }
}

impl<S> SetColor<S> for Option<Srgba<S>>
where
    S: Component,
{
    fn rgba_mut(&mut self) -> &mut Option<Srgba<S>> {
        self
    }
}

fn into_rgb_with_alpha<C, S>(color: C) -> Srgba<S>
where
    C: IntoColor<D65, S>,
    S: Component + Float,
{
    let linsrgb: color::LinSrgb<S> = color.into_rgb::<encoding::Srgb>();
    let color: Srgb<S> = linsrgb.into_encoding();
    let alpha = S::max_intensity();
    Alpha { color, alpha }
}

impl<S> IntoSrgba<S> for color::Xyz<D65, S>
where
    S: Component + Float,
{
    fn into_srgba(self) -> Srgba<S> {
        into_rgb_with_alpha(self)
    }
}

impl<S> IntoSrgba<S> for color::Yxy<D65, S>
where
    S: Component + Float,
{
    fn into_srgba(self) -> Srgba<S> {
        into_rgb_with_alpha(self)
    }
}

impl<S> IntoSrgba<S> for color::Lab<D65, S>
where
    S: Component + Float,
{
    fn into_srgba(self) -> Srgba<S> {
        into_rgb_with_alpha(self)
    }
}

impl<S> IntoSrgba<S> for color::Lch<D65, S>
where
    S: Component + Float,
{
    fn into_srgba(self) -> Srgba<S> {
        into_rgb_with_alpha(self)
    }
}

impl<T, S> IntoSrgba<S> for color::Srgb<T>
where
    T: Component,
    S: Component,
{
    fn into_srgba(self) -> Srgba<S> {
        let color = self.into_format();
        let alpha = S::max_intensity();
        Alpha { color, alpha }
    }
}

impl<S> IntoSrgba<S> for color::Hsl<encoding::Srgb, S>
where
    S: Component + Float,
{
    fn into_srgba(self) -> Srgba<S> {
        into_rgb_with_alpha(self)
    }
}

impl<S> IntoSrgba<S> for color::Hsv<encoding::Srgb, S>
where
    S: Component + Float,
{
    fn into_srgba(self) -> Srgba<S> {
        into_rgb_with_alpha(self)
    }
}

impl<S> IntoSrgba<S> for color::Hwb<encoding::Srgb, S>
where
    S: Component + Float,
{
    fn into_srgba(self) -> Srgba<S> {
        into_rgb_with_alpha(self)
    }
}

impl<S> IntoSrgba<S> for color::SrgbLuma<S>
where
    S: Component + Float,
{
    fn into_srgba(self) -> Srgba<S> {
        into_rgb_with_alpha(self)
    }
}

impl<C, S> IntoSrgba<S> for Alpha<C, S>
where
    C: IntoSrgba<S>,
    S: Component,
{
    fn into_srgba(self) -> Srgba<S> {
        let Alpha { color, alpha } = self;
        let mut srgba = color.into_srgba();
        srgba.alpha = alpha;
        srgba
    }
}
