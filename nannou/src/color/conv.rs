//! This module provides some more flexible conversions and aims to fill the gaps within the `From`
//! and `Into` implementations provided by the `palette` library.
//!
//! If a desired conversion is missing, feel free to open an issue or pull request!

use crate::color::white_point::D65;
use crate::color::{self, encoding, Alpha, Component, IntoColor, LinSrgba};
use crate::math::num_traits::Float;

/// Types that may be converted directly into a linear sRGBA color representation.
///
/// This is more flexible than `Into<LinSrgba<S>>` as it also supports converting from different
/// sRGBA encodings that also have different component types.
///
/// This trait is important for nannou as the `Draw` API works with the `LinSrgba` type as a target
/// for its generic colour type API.
pub trait IntoLinSrgba<S>
where
    S: Component,
{
    /// Convert self into RGBA.
    fn into_lin_srgba(self) -> LinSrgba<S>;
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

fn into_lin_srgb_with_alpha<C, S>(color: C) -> LinSrgba<S>
where
    C: IntoColor<D65, S>,
    S: Component + Float,
{
    let color: color::LinSrgb<S> = color.into_rgb::<encoding::Srgb>();
    let alpha = S::max_intensity();
    Alpha { color, alpha }
}
