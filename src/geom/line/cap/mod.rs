//! Items related to capping the ends of **Line**s or **Polyline**s.
//!
//! Line caps are the geometry that describes the end of a line.
//!
//! Nannou provides three common types of line caps:
//!
//! - [**butt**](./butt/index.html): Ends the stroke flush with the start or end of the line. This
//!   is the default.
//! - [**round**](./round/index.html): Ends the line with a half circle whose radius is half the
//!   line thickness.
//! - [**square**](./square/index.html): Extends the line on either end by half the thickness.
//!
//! ```ignore
//! start                      end
//!   |                         |
//!   v                         v
//!
//!   ---------------------------
//!   |                         |   butt (default)
//!   ---------------------------
//!
//!  /---------------------------\
//! (                             ) round
//!  \---------------------------/
//!
//! -------------------------------
//! |                             | square
//! -------------------------------
//!
//!   ^                         ^
//!   |                         |
//! start                      end
//! ```

pub mod butt;
pub mod round;
pub mod square;

use crate::geom::{ellipse, quad, Point2};
use crate::math::BaseFloat;

/// An iterator yielding the vertices of a line cap.
#[derive(Clone, Debug)]
pub enum Vertices<S> {
    Butt,
    Round(ellipse::Circumference<S>),
    Square(quad::Vertices<Point2<S>>),
}

impl<S> Iterator for Vertices<S>
where
    S: BaseFloat,
{
    type Item = Point2<S>;
    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            Vertices::Butt => None,
            Vertices::Round(ref mut iter) => iter.next(),
            Vertices::Square(ref mut iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<S> ExactSizeIterator for Vertices<S>
where
    S: BaseFloat,
{
    fn len(&self) -> usize {
        match *self {
            Vertices::Butt => 0,
            Vertices::Round(ref iter) => iter.len(),
            Vertices::Square(ref iter) => iter.len(),
        }
    }
}
