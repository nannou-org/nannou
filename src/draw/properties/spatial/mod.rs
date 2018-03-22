use geom;

pub mod dimension;
pub mod position;
// pub mod orientation;

pub use self::dimension::SetDimensions;
pub use self::position::SetPosition;
// pub use self::orientation::SetOrientation;

/// Types that may be positioned, sized and oriented within 3D space.
pub trait SetSpatial<S>: SetDimensions<S> + SetPosition<S> {}

impl<S, T> SetSpatial<S> for T
where
    T: SetDimensions<S> + SetPosition<S>,
{
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Properties<S = geom::DefaultScalar> {
    pub position: position::Properties<S>,
    pub dimensions: dimension::Properties<S>,
    // pub orientation: orientation::Properties<S>,
}

impl<S> Default for Properties<S> {
    fn default() -> Self {
        let position = Default::default();
        let dimensions = Default::default();
        //let orientation = Default::default();
        Properties {
            position,
            dimensions,
        }
    }
}

impl<S> SetPosition<S> for Properties<S> {
    fn properties(&mut self) -> &mut position::Properties<S> {
        self.position.properties()
    }
}

impl<S> SetDimensions<S> for Properties<S> {
    fn properties(&mut self) -> &mut dimension::Properties<S> {
        self.dimensions.properties()
    }
}
