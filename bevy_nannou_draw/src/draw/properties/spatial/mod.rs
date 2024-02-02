pub mod dimension;
pub mod orientation;
pub mod position;

pub use self::dimension::SetDimensions;
pub use self::orientation::SetOrientation;
pub use self::position::SetPosition;

/// Types that may be positioned, sized and oriented within 3D space.
pub trait SetSpatial: SetDimensions + SetPosition + SetOrientation {}

impl<T> SetSpatial for T where T: SetDimensions + SetPosition + SetOrientation {}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Properties {
    pub position: position::Properties,
    pub dimensions: dimension::Properties,
    pub orientation: orientation::Properties,
}

impl Default for Properties {
    fn default() -> Self {
        let position = Default::default();
        let dimensions = Default::default();
        let orientation = Default::default();
        Properties {
            position,
            dimensions,
            orientation,
        }
    }
}

impl SetPosition for Properties {
    fn properties(&mut self) -> &mut position::Properties {
        self.position.properties()
    }
}

impl SetDimensions for Properties {
    fn properties(&mut self) -> &mut dimension::Properties {
        self.dimensions.properties()
    }
}

impl SetOrientation for Properties {
    fn properties(&mut self) -> &mut orientation::Properties {
        self.orientation.properties()
    }
}
