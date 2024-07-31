use core::ops;

use crate::math::num_traits::{Num, NumOps};

/// Implemented for all numeric scalar types used within `geom`.
///
/// A base set of traits that must be implemented by all types used to represent scalar values
/// within the `geom` module abstractions.
pub trait Scalar:
    Clone
    + Copy
    + Num
    + PartialOrd
    + ops::AddAssign
    + ops::SubAssign
    + ops::MulAssign
    + ops::DivAssign
    + ops::RemAssign
    + ops::Neg<Output = Self>
{
}

impl<T> Scalar for T where
    T: Clone
        + Copy
        + Num
        + NumOps
        + PartialOrd
        + ops::AddAssign
        + ops::SubAssign
        + ops::MulAssign
        + ops::DivAssign
        + ops::RemAssign
        + ops::Neg<Output = Self>
{
}

/// The default scalar type used for geometry throughout Nannou.
pub type Default = f32;
