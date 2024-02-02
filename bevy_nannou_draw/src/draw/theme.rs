use nannou_core::color::{LinSrgba, Srgba};
use std::collections::HashMap;

/// A set of styling defaults used for coloring texturing geometric primitives that have no entry
/// within the **Draw**'s inner **ColorMap**.
#[derive(Clone, Debug)]
pub struct Theme {
    /// Fill color defaults.
    pub fill_color: Color,
    /// Stroke color defaults.
    pub stroke_color: Color,
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
    Arrow,
    Cuboid,
    Ellipse,
    Line,
    Mesh,
    Path,
    Polygon,
    Quad,
    Rect,
    Text,
    Texture,
    Tri,
}

impl Theme {
    /// Retrieve the non-linear sRGBA fill color representation for the given primitive.
    pub fn fill_srgba(&self, prim: &Primitive) -> Srgba {
        self.fill_color
            .primitive
            .get(prim)
            .map(|&c| c)
            .unwrap_or(self.fill_color.default)
    }

    /// Retrieve the linaer sRGBA fill color representation for the given primitive.
    pub fn fill_lin_srgba(&self, prim: &Primitive) -> LinSrgba {
        self.fill_srgba(prim).into_linear()
    }

    /// Retrieve the non-linear sRGBA stroke color representation for the given primitive.
    pub fn stroke_srgba(&self, prim: &Primitive) -> Srgba {
        self.stroke_color
            .primitive
            .get(prim)
            .map(|&c| c)
            .unwrap_or(self.stroke_color.default)
    }

    /// Retrieve the linaer sRGBA stroke color representation for the given primitive.
    pub fn stroke_lin_srgba(&self, prim: &Primitive) -> LinSrgba {
        self.stroke_srgba(prim).into_linear()
    }
}

impl Default for Theme {
    fn default() -> Self {
        // TODO: This should be pub const.
        let default_fill = Srgba::new(1.0, 1.0, 1.0, 1.0);
        let default_stroke = Srgba::new(0.0, 0.0, 0.0, 1.0);

        let fill_color = Color {
            default: default_fill,
            primitive: Default::default(),
        };
        let mut stroke_color = Color {
            default: default_stroke,
            primitive: Default::default(),
        };
        stroke_color
            .primitive
            .insert(Primitive::Arrow, default_fill);

        Theme {
            fill_color,
            stroke_color,
        }
    }
}
