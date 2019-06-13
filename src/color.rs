//! Color items, including everything from rgb, hsb/l/v, lap, alpha, luma and more, provided by the
//! palette crate. See [the palette docs](https://docs.rs/palette) for more details or see the
//! [**named**](./named/index.html) module for a set of provided color constants.

#[doc(inline)]
pub use palette::*;

pub use self::named::*;
pub use self::tango::*;

pub mod tango {
    //! A set of provided, named color constants.
    //!
    //! These colors come from the [Tango
    //! palette](http://tango.freedesktop.org/Tango_Icon_Theme_Guidelines) which provides
    //! aesthetically reasonable defaults for colors. Each color also comes with a light and dark
    //! version.
    use super::{Alpha, Srgb, Srgba};

    macro_rules! make_color {
        ($r:expr, $g:expr, $b:expr, $a:expr) => {
            Alpha {
                color: Srgb {
                    red: $r as f32 / 255.0,
                    green: $g as f32 / 255.0,
                    blue: $b as f32 / 255.0,
                    standard: std::marker::PhantomData,
                },
                alpha: $a as f32 / 255.0,
            }
        };
        ($r:expr, $g:expr, $b:expr) => {
            make_color!($r, $g, $b, 255)
        };
    }

    /// Scarlet Red - Light - #EF2929
    pub const LIGHT_RED: Srgba<f32> = make_color!(239, 41, 41);
    /// Scarlet Red - Regular - #CC0000
    pub const RED: Srgba<f32> = make_color!(204, 0, 0);
    /// Scarlet Red - Dark - #A30000
    pub const DARK_RED: Srgba<f32> = make_color!(164, 0, 0);

    /// Orange - Light - #FCAF3E
    pub const LIGHT_ORANGE: Srgba<f32> = make_color!(252, 175, 62);
    /// Orange - Regular - #F57900
    pub const ORANGE: Srgba<f32> = make_color!(245, 121, 0);
    /// Orange - Dark - #CE5C00
    pub const DARK_ORANGE: Srgba<f32> = make_color!(206, 92, 0);

    /// Butter - Light - #FCE94F
    pub const LIGHT_YELLOW: Srgba<f32> = make_color!(252, 233, 79);
    /// Butter - Regular - #EDD400
    pub const YELLOW: Srgba<f32> = make_color!(237, 212, 0);
    /// Butter - Dark - #C4A000
    pub const DARK_YELLOW: Srgba<f32> = make_color!(196, 160, 0);

    /// Chameleon - Light - #8AE234
    pub const LIGHT_GREEN: Srgba<f32> = make_color!(138, 226, 52);
    /// Chameleon - Regular - #73D216
    pub const GREEN: Srgba<f32> = make_color!(115, 210, 22);
    /// Chameleon - Dark - #4E9A06
    pub const DARK_GREEN: Srgba<f32> = make_color!(78, 154, 6);

    /// Sky Blue - Light - #729FCF
    pub const LIGHT_BLUE: Srgba<f32> = make_color!(114, 159, 207);
    /// Sky Blue - Regular - #3465A4
    pub const BLUE: Srgba<f32> = make_color!(52, 101, 164);
    /// Sky Blue - Dark - #204A87
    pub const DARK_BLUE: Srgba<f32> = make_color!(32, 74, 135);

    /// Plum - Light - #AD7FA8
    pub const LIGHT_PURPLE: Srgba<f32> = make_color!(173, 127, 168);
    /// Plum - Regular - #75507B
    pub const PURPLE: Srgba<f32> = make_color!(117, 80, 123);
    /// Plum - Dark - #5C3566
    pub const DARK_PURPLE: Srgba<f32> = make_color!(92, 53, 102);

    /// Chocolate - Light - #E9B96E
    pub const LIGHT_BROWN: Srgba<f32> = make_color!(233, 185, 110);
    /// Chocolate - Regular - #C17D11
    pub const BROWN: Srgba<f32> = make_color!(193, 125, 17);
    /// Chocolate - Dark - #8F5902
    pub const DARK_BROWN: Srgba<f32> = make_color!(143, 89, 2);

    /// Straight Black.
    pub const BLACK: Srgba<f32> = make_color!(0, 0, 0);
    /// Straight White.
    pub const WHITE: Srgba<f32> = make_color!(255, 255, 255);

    /// Alluminium - Light
    pub const LIGHT_GRAY: Srgba<f32> = make_color!(238, 238, 236);
    /// Alluminium - Regular
    pub const GRAY: Srgba<f32> = make_color!(211, 215, 207);
    /// Alluminium - Dark
    pub const DARK_GRAY: Srgba<f32> = make_color!(186, 189, 182);

    /// Aluminium - Light - #EEEEEC
    pub const LIGHT_GREY: Srgba<f32> = make_color!(238, 238, 236);
    /// Aluminium - Regular - #D3D7CF
    pub const GREY: Srgba<f32> = make_color!(211, 215, 207);
    /// Aluminium - Dark - #BABDB6
    pub const DARK_GREY: Srgba<f32> = make_color!(186, 189, 182);

    /// Charcoal - Light - #888A85
    pub const LIGHT_CHARCOAL: Srgba<f32> = make_color!(136, 138, 133);
    /// Charcoal - Regular - #555753
    pub const CHARCOAL: Srgba<f32> = make_color!(85, 87, 83);
    /// Charcoal - Dark - #2E3436
    pub const DARK_CHARCOAL: Srgba<f32> = make_color!(46, 52, 54);

    /// Transparent
    pub const TRANSPARENT: Srgba<f32> = make_color!(0.0, 0.0, 0.0, 0.0);
}
