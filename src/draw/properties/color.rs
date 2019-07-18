use crate::color::white_point::D65;
use crate::color::{self, encoding, Alpha, Component, IntoColor, LinSrgba, RgbHue, Srgb, Srgba};
use crate::math::num_traits::Float;

/// A **Srgba** type with the default Scalar.
pub type DefaultSrgba = Srgba<color::DefaultScalar>;

/// A **LinSrgba** type with the default Scalar.
pub type DefaultLinSrgba = LinSrgba<color::DefaultScalar>;

/// Types that may be converted directly into an RGBA color.
pub trait IntoLinSrgba<S>
where
    S: Component,
{
    /// Convert self into RGBA.
    fn into_lin_srgba(self) -> LinSrgba<S>;
}

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
        self.color(Srgb::new(r, g, b))
    }

    /// Specify the color via red, green, blue and alpha channels.
    fn rgba<T>(self, r: T, g: T, b: T, a: T) -> Self
    where
        T: Component,
        S: Float,
    {
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

impl<S> SetColor<S> for Option<LinSrgba<S>>
where
    S: Component,
{
    fn rgba_mut(&mut self) -> &mut Option<LinSrgba<S>> {
        self
    }
}

fn into_lin_srgb_with_alpha<C, S>(color: C) -> LinSrgba<S>
where
    C: IntoColor<D65, S>,
    S: Component + Float,
{
    let color: color::LinSrgb<S> = color.into_rgb::<encoding::Srgb>();
    let alpha = S::max_intensity();
    Alpha { color, alpha }
}

impl<S> IntoLinSrgba<S> for color::Xyz<D65, S>
where
    S: Component + Float,
{
    fn into_lin_srgba(self) -> LinSrgba<S> {
        into_lin_srgb_with_alpha(self)
    }
}

impl<S> IntoLinSrgba<S> for color::Yxy<D65, S>
where
    S: Component + Float,
{
    fn into_lin_srgba(self) -> LinSrgba<S> {
        into_lin_srgb_with_alpha(self)
    }
}

impl<S> IntoLinSrgba<S> for color::Lab<D65, S>
where
    S: Component + Float,
{
    fn into_lin_srgba(self) -> LinSrgba<S> {
        into_lin_srgb_with_alpha(self)
    }
}

impl<S> IntoLinSrgba<S> for color::Lch<D65, S>
where
    S: Component + Float,
{
    fn into_lin_srgba(self) -> LinSrgba<S> {
        into_lin_srgb_with_alpha(self)
    }
}

impl<T, S> IntoLinSrgba<S> for color::LinSrgb<T>
where
    T: Component,
    S: Component,
{
    fn into_lin_srgba(self) -> LinSrgba<S> {
        let color = self.into_format();
        let alpha = S::max_intensity();
        Alpha { color, alpha }
    }
}

impl<T, S> IntoLinSrgba<S> for color::Srgb<T>
where
    T: Component,
    S: Component + Float,
{
    fn into_lin_srgba(self) -> LinSrgba<S> {
        let color = self.into_format().into_linear();
        let alpha = S::max_intensity();
        Alpha { color, alpha }
    }
}

impl<S> IntoLinSrgba<S> for color::Hsl<encoding::Srgb, S>
where
    S: Component + Float,
{
    fn into_lin_srgba(self) -> LinSrgba<S> {
        into_lin_srgb_with_alpha(self)
    }
}

impl<S> IntoLinSrgba<S> for color::Hsv<encoding::Srgb, S>
where
    S: Component + Float,
{
    fn into_lin_srgba(self) -> LinSrgba<S> {
        into_lin_srgb_with_alpha(self)
    }
}

impl<S> IntoLinSrgba<S> for color::Hwb<encoding::Srgb, S>
where
    S: Component + Float,
{
    fn into_lin_srgba(self) -> LinSrgba<S> {
        into_lin_srgb_with_alpha(self)
    }
}

impl<S> IntoLinSrgba<S> for color::SrgbLuma<S>
where
    S: Component + Float,
{
    fn into_lin_srgba(self) -> LinSrgba<S> {
        into_lin_srgb_with_alpha(self)
    }
}

impl<C, S, T> IntoLinSrgba<S> for Alpha<C, T>
where
    C: IntoLinSrgba<S>,
    S: Component,
    T: Component,
{
    fn into_lin_srgba(self) -> LinSrgba<S> {
        let Alpha { color, alpha } = self;
        let mut srgba = color.into_lin_srgba();
        srgba.alpha = alpha.convert();
        srgba
    }
}
