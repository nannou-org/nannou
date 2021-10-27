use crate::color::LinSrgba;
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
    pub default: LinSrgba,
    pub primitive: HashMap<Primitive, LinSrgba>,
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

pub enum ColorType {
    Fill,
    Stroke,
}

impl Theme {
    /// Retrieve the linear sRGBA fill color representation for the given primitive.
    pub fn fill_lin_srgba(&self, prim: Primitive) -> LinSrgba {
        self.fill_color
            .primitive
            .get(&prim)
            .copied()
            .unwrap_or(self.fill_color.default)
    }

    /// Retrieve the linear sRGBA stroke color representation for the given primitive.
    pub fn stroke_lin_srgba(&self, prim: Primitive) -> LinSrgba {
        self.stroke_color
            .primitive
            .get(&prim)
            .copied()
            .unwrap_or(self.stroke_color.default)
    }

    pub fn resolve_color<T>(&self, color: Option<LinSrgba>, prim: Primitive, options: T) -> LinSrgba
    where
        T: Into<ColorType>,
    {
        color.unwrap_or_else(|| match options.into() {
            ColorType::Fill => self.fill_lin_srgba(prim),
            ColorType::Stroke => self.stroke_lin_srgba(prim),
        })
    }
}

impl Default for Theme {
    fn default() -> Self {
        // TODO: This should be pub const.
        let default_fill = LinSrgba::new(1.0, 1.0, 1.0, 1.0);
        let default_stroke = LinSrgba::new(0.0, 0.0, 0.0, 1.0);

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
