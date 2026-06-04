//! Parameters which a **Drawing** instance may use to describe certain properties of a drawing.
//!
//! Each time a new method is chained onto a **Drawing** instance, it uses the given values to set
//! one or more properties for the drawing.
//!
//! Each **Drawing** instance is associated with a specific **Node** in the geometry graph and has
//! a unique **node::Index** to simplify this.

pub use self::color::SetColor;
pub use self::fill::SetFill;
pub use self::spatial::dimension::SetDimensions;
pub use self::spatial::orientation::SetOrientation;
pub use self::spatial::position::SetPosition;
pub use self::stroke::SetStroke;

pub mod color;
pub mod fill;
pub mod spatial;
pub mod stroke;
pub mod tex_coords;
