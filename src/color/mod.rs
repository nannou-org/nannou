//! Color items, including everything from rgb, hsb/l/v, lap, alpha, luma and more, provided by the
//! palette crate. See [the palette docs](https://docs.rs/palette) for more details or see the
//! [**named**](./named/index.html) module for a set of provided color constants.

pub mod conv;

pub use self::conv::IntoLinSrgba;
pub use self::named::*;
#[doc(inline)]
pub use palette::*;

/// The default scalar value for working with color components, hues, etc.
pub type DefaultScalar = f32;

/// A color represented as red, green and blue intensities.
///
/// This type is an alias for the `Srgb` type, a type that represents the sRGB color space.
///
/// If you are looking for more advanced control over the RGB space and component type, please see
/// the `palette` crate's generic `Rgb` type.
pub type Rgb<S = DefaultScalar> = Srgb<S>;

/// The same as `Rgb`, but with an alpha value representing opacity.
///
/// This type is an alias for the `Srgba` type, a type that represents the sRGB color space
/// alongside an alpha value.
///
/// If you are looking for more advanced control over the RGB space and component type, please see
/// the `palette` crate's generic `Rgb` type.
pub type Rgba<S = DefaultScalar> = Srgba<S>;

/// A short-hand constructor for `Rgb::new`.
pub fn rgb(r: f32, g: f32, b: f32) -> Rgb {
    srgb(r, g, b)
}

/// A short-hand constructor for `Rgba::new`.
pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Rgba {
    srgba(r, g, b, a)
}

/// A short-hand constructor for `Srgb::new`.
pub fn srgb(r: f32, g: f32, b: f32) -> Srgb {
    Srgb::new(r, g, b)
}

/// A short-hand constructor for `Srgba::new`.
pub fn srgba(r: f32, g: f32, b: f32, a: f32) -> Srgba {
    Srgba::new(r, g, b, a)
}

/// A short-hand constructor for `Hsl::new(RgbHue::from_degrees(h * 360.0), s, l)`.
///
/// The given hue expects a value between `0.0` and `1.0` where `0.0` is 0 degress and `1.0` is
/// 360 degrees (or 2 PI radians).
pub fn hsl(h: f32, s: f32, l: f32) -> Hsl {
    Hsl::new(RgbHue::from_degrees(h * 360.0), s, l)
}

/// A short-hand constructor for `Hsla::new(RgbHue::from_degrees(h * 360.0), s, l, a)`.
///
/// The given hue expects a value between `0.0` and `1.0` where `0.0` is 0 degress and `1.0` is
/// 360 degrees (or 2 PI radians).
pub fn hsla(h: f32, s: f32, l: f32, a: f32) -> Hsla {
    Hsla::new(RgbHue::from_degrees(h * 360.0), s, l, a)
}

/// A short-hand constructor for `Hsv::new(RgbHue::from_degrees(h * 360.0), s, v)`.
///
/// The given hue expects a value between `0.0` and `1.0` where `0.0` is 0 degress and `1.0` is
/// 360 degrees (or 2 PI radians).
pub fn hsv(h: f32, s: f32, v: f32) -> Hsv {
    Hsv::new(RgbHue::from_degrees(h * 360.0), s, v)
}

/// A short-hand constructor for `Hsva::new(RgbHue::from_degrees(h * 360.0), s, v, a)`.
///
/// The given hue expects a value between `0.0` and `1.0` where `0.0` is 0 degress and `1.0` is
/// 360 degrees (or 2 PI radians).
pub fn hsva(h: f32, s: f32, v: f32, a: f32) -> Hsva {
    Hsva::new(RgbHue::from_degrees(h * 360.0), s, v, a)
}
