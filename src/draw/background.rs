use color::{self, Rgb, Rgba};
use draw::Draw;
use draw::properties::{ColorScalar, IntoRgba};
use geom;
use math::BaseFloat;

/// A type used to update the background colour.
pub struct Background<'a, S = geom::DefaultScalar>
where
    S: 'a + BaseFloat,
{
    draw: &'a Draw<S>
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
    pub fn hsl<H>(self, h: H, s: ColorScalar, l: ColorScalar) -> Self
    where
        H: Into<color::RgbHue<ColorScalar>>,
    {
        self.color(color::Hsl::new(h.into(), s, l))
    }

    /// Specify the color via hue, saturation, luminance and an alpha channel.
    pub fn hsla<H>(self, h: H, s: ColorScalar, l: ColorScalar, a: ColorScalar) -> Self
    where
        H: Into<color::RgbHue<ColorScalar>>,
    {
        self.color(color::Hsla::new(h.into(), s, l, a))
    }
}
