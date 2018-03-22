use color::{self, Alpha, IntoColor, Rgb, Rgba};
use math::num_traits::{Float, One};

/// The default scalar value for working with color channels, hues, etc.
pub type DefaultScalar = f32;

/// An **Rgba** type with the default Scalar.
///
/// Used by the **draw::properties::Common** type.
pub type DefaultRgba = Rgba<DefaultScalar>;

/// Types that may be converted directly into an RGBA color.
pub trait IntoRgba<S>
where
    S: Float,
{
    /// Convert self into RGBA.
    fn into_rgba(self) -> Rgba<S>;
}

/// Nodes that support setting colors.
pub trait SetColor<S>: Sized
where
    S: Float,
{
    /// Provide a mutable reference to the RGBA field which can be used for setting colors.
    fn rgba_mut(&mut self) -> &mut Option<Rgba<S>>;

    /// Specify a color.
    ///
    /// This method supports any color type that can be converted into RGBA.
    ///
    /// Colors that have no alpha channel will be given an opaque alpha channel value `1.0`.
    fn color<C>(mut self, color: C) -> Self
    where
        C: IntoRgba<S>,
    {
        *self.rgba_mut() = Some(color.into_rgba());
        self
    }

    /// Specify the color via red, green and blue channels.
    fn rgb(self, r: S, g: S, b: S) -> Self {
        self.color(Rgb::new(r, g, b))
    }

    /// Specify the color via red, green, blue and alpha channels.
    fn rgba(self, r: S, g: S, b: S, a: S) -> Self {
        self.color(Rgba::new(r, g, b, a))
    }

    /// Specify the color via hue, saturation and luminance.
    fn hsl<H>(self, h: H, s: S, l: S) -> Self
    where
        H: Into<color::RgbHue<S>>,
    {
        self.color(color::Hsl::new(h.into(), s, l))
    }

    /// Specify the color via hue, saturation, luminance and an alpha channel.
    fn hsla<H>(self, h: H, s: S, l: S, a: S) -> Self
    where
        H: Into<color::RgbHue<S>>,
    {
        self.color(color::Hsla::new(h.into(), s, l, a))
    }
}

impl<S> SetColor<S> for Option<Rgba<S>>
where
    S: Float,
{
    fn rgba_mut(&mut self) -> &mut Option<Rgba<S>> {
        self
    }
}

fn into_rgb_with_alpha<C, S>(color: C) -> Rgba<S>
where
    C: IntoColor<S>,
    S: Float + One,
{
    let color = color.into_rgb();
    let alpha = One::one();
    Alpha { color, alpha }
}

impl<S> IntoRgba<S> for color::Xyz<S>
where
    S: Float + One,
{
    fn into_rgba(self) -> Rgba<S> {
        into_rgb_with_alpha(self)
    }
}

impl<S> IntoRgba<S> for color::Yxy<S>
where
    S: Float + One,
{
    fn into_rgba(self) -> Rgba<S> {
        into_rgb_with_alpha(self)
    }
}

impl<S> IntoRgba<S> for color::Lab<S>
where
    S: Float + One,
{
    fn into_rgba(self) -> Rgba<S> {
        into_rgb_with_alpha(self)
    }
}

impl<S> IntoRgba<S> for color::Lch<S>
where
    S: Float + One,
{
    fn into_rgba(self) -> Rgba<S> {
        into_rgb_with_alpha(self)
    }
}

impl<S> IntoRgba<S> for color::Rgb<S>
where
    S: Float + One,
{
    fn into_rgba(self) -> Rgba<S> {
        into_rgb_with_alpha(self)
    }
}

impl<S> IntoRgba<S> for color::Hsl<S>
where
    S: Float + One,
{
    fn into_rgba(self) -> Rgba<S> {
        into_rgb_with_alpha(self)
    }
}

impl<S> IntoRgba<S> for color::Hsv<S>
where
    S: Float + One,
{
    fn into_rgba(self) -> Rgba<S> {
        into_rgb_with_alpha(self)
    }
}

impl<S> IntoRgba<S> for color::Hwb<S>
where
    S: Float + One,
{
    fn into_rgba(self) -> Rgba<S> {
        into_rgb_with_alpha(self)
    }
}

impl<S> IntoRgba<S> for color::Luma<S>
where
    S: Float + One,
{
    fn into_rgba(self) -> Rgba<S> {
        into_rgb_with_alpha(self)
    }
}

impl<C, S> IntoRgba<S> for Alpha<C, S>
where
    C: IntoColor<S>,
    S: Float,
{
    fn into_rgba(self) -> Rgba<S> {
        let Alpha { color, alpha } = self;
        let color = color.into_rgb();
        Alpha { color, alpha }
    }
}
