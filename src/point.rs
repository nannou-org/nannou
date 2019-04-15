//! Implementation of the general laser point type used throughout the crate.

use crate::lerp::Lerp;

/// A position in 2D space represented by x and y coordinates.
pub type Position = [f32; 2];

/// Red, green and blue channels of a single colour.
pub type Rgb = [f32; 3];

/// The point type used within the laser stream API.
///
/// The point represents the location to which the scanner should point and the colour that the
/// scanner should be at this point.
///
/// If two consecutive points have two different colours, the `color` values will be linearly
/// interpolated.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Point {
    /// The position of the point. `-1` represents the minimum value along the axis and `1`
    /// represents the maximum.
    pub position: Position,
    /// The color of the point.
    pub color: Rgb,
}

impl Point {
    /// Create a blank point at `[0, 0]`.
    pub fn centered_blank() -> Self {
        Point {
            position: [0.0, 0.0],
            color: [0.0, 0.0, 0.0],
        }
    }

    /// Returns a point with the same position as `self` but with a black (blank) color.
    pub fn blanked(&self) -> Self {
        let mut blanked = *self;
        blanked.color = [0.0, 0.0, 0.0];
        blanked
    }

    /// Whether or not the point is blank (black).
    pub fn is_blank(&self) -> bool {
        let [r, g, b] = self.color;
        r == 0.0 && g == 0.0 && b == 0.0
    }
}

impl Lerp for Point {
    type Scalar = f32;
    fn lerp(&self, other: &Self, amt: f32) -> Self {
        Point {
            position: self.position.lerp(&other.position, amt),
            color: self.color.lerp(&other.color, amt),
        }
    }
}
