use std::collections::HashMap;

use bevy::prelude::Color;

/// A set of styling defaults used for coloring texturing geometric primitives that have no entry
/// within the **Draw**'s inner **ColorMap**.
#[derive(Clone, Debug)]
pub struct Theme {
    /// Fill color defaults.
    pub fill_color: ThemeColor,
    /// Stroke color defaults.
    pub stroke_color: ThemeColor,
}

/// A set of defaults used for coloring.
#[derive(Clone, Debug)]
pub struct ThemeColor {
    pub default: Color,
    pub primitive: HashMap<Primitive, Color>,
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
    /// Retrieve the fill color representation for the given primitive.
    pub fn fill(&self, prim: &Primitive) -> Color {
        self.fill_color
            .primitive
            .get(prim)
            .map(|&c| c)
            .unwrap_or(self.fill_color.default)
    }

    /// Retrieve the  stroke color representation for the given primitive.
    pub fn stroke(&self, prim: &Primitive) -> Color {
        self.stroke_color
            .primitive
            .get(prim)
            .map(|&c| c)
            .unwrap_or(self.stroke_color.default)
    }
}

impl Default for Theme {
    fn default() -> Self {
        // TODO: This should be pub const.
        let default_fill = Color::rgba(1.0, 1.0, 1.0, 1.0);
        let default_stroke = Color::rgba(0.0, 0.0, 0.0, 1.0);

        let fill_color = ThemeColor {
            default: default_fill,
            primitive: Default::default(),
        };
        let mut stroke_color = ThemeColor {
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
