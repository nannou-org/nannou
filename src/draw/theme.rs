use crate::color::{Alpha, Srgb, Srgba};
use std::collections::HashMap;

/// A set of styling defaults used for coloring texturing geometric primitives that have no entry
/// within the **Draw**'s inner **ColorMap**.
#[derive(Clone, Debug, Default)]
pub struct Theme {
    /// Color defaults.
    pub color: Color,
}

/// A set of defaults used for coloring.
#[derive(Clone, Debug)]
pub struct Color {
    pub default: Srgba,
    pub primitive: HashMap<Primitive, Srgba>,
}

/// Primitive geometry types that may have unique default styles.
///
/// These are used as keys into the **Theme**'s geometry primitive default values.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum Primitive {
    Cuboid,
    Ellipse,
    Line,
    Path,
    Polygon,
    Polyline,
    Quad,
    Rect,
    Tri,
}

impl Default for Color {
    fn default() -> Self {
        let default = Alpha {
            color: Srgb::new(1.0, 1.0, 1.0),
            alpha: 1.0,
        };
        let primitive = Default::default();
        Color { default, primitive }
    }
}
