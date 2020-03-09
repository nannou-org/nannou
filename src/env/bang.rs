use envelope;

/// A type to use for `Point`s that have no value.
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Bang;

impl envelope::interpolation::Spatial for Bang {
    type Scalar = f32;
    fn add(&self, _: &Bang) -> Bang {
        Bang
    }
    fn sub(&self, _: &Bang) -> Bang {
        Bang
    }
    fn scale(&self, _scalar: &f32) -> Bang {
        Bang
    }
}
