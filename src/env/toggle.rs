use envelope;

/// A wrapper around a boolean value for a Point implementation.
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Toggle(pub bool);

impl ::std::ops::Deref for Toggle {
    type Target = bool;
    fn deref<'a>(&'a self) -> &'a bool {
        let Toggle(ref b) = *self;
        b
    }
}

impl ::std::ops::DerefMut for Toggle {
    fn deref_mut<'a>(&'a mut self) -> &'a mut bool {
        let Toggle(ref mut b) = *self;
        b
    }
}

impl envelope::interpolation::Spatial for Toggle {
    type Scalar = f32;
    fn add(&self, other: &Toggle) -> Toggle {
        if !**other {
            *self
        } else {
            Toggle(!**self)
        }
    }
    fn sub(&self, other: &Toggle) -> Toggle {
        if !**other {
            *self
        } else {
            Toggle(!**self)
        }
    }
    fn scale(&self, _scalar: &f32) -> Toggle {
        *self
    }
}

impl From<Toggle> for bool {
    fn from(Toggle(b): Toggle) -> Self {
        b
    }
}

impl From<bool> for Toggle {
    fn from(b: bool) -> Self {
        Toggle(b)
    }
}
