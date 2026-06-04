use bevy::prelude::*;

/// Orientation properties for **Drawing** a **Primitive**.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Properties {
    /// The orientation described by an angle along each axis.
    Axes(Vec3),
    /// The orientation described by looking at some other point.
    LookAt(Vec3),
    /// Angle described by quarternion.
    Quat(Quat),
}

impl Properties {
    pub fn transform(&self) -> Mat4 {
        match *self {
            Properties::Axes(v) => Mat4::from_euler(EulerRot::XYZ, v.x, v.y, v.z),
            Properties::LookAt(p) => {
                let eye = Vec3::ZERO;
                let up = Vec3::Y;
                Mat4::look_at_rh(eye, p, up)
            }
            Properties::Quat(q) => Mat4::from_quat(q),
        }
    }

    /// If the `Properties` was set to the `LookAt` variant, this method switches to the `Axes`
    /// variant.
    ///
    /// If the `Properties` is already `Axes`, nothing changes.
    pub fn switch_to_axes(&mut self) {
        if let Properties::LookAt(_) = *self {
            *self = Properties::Axes(Vec3::ZERO);
        }
    }
}

/// An API for setting the **orientation::Properties**.
pub trait SetOrientation: Sized {
    /// Provide a mutable reference to the **orientation::Properties** for updating.
    fn properties(&mut self) -> &mut Properties;

    // Describing orientation via a target.

    /// Describe orientation via the vector that points to the given target.
    fn look_at(mut self, target: Vec3) -> Self {
        *self.properties() = Properties::LookAt(target);
        self
    }

    // Absolute orientation.

    /// Specify the orientation around the *x* axis as an absolute value in radians.
    fn x_radians(mut self, x: f32) -> Self {
        self.properties().switch_to_axes();
        expect_axes(self.properties()).x = x;
        self
    }

    /// Specify the orientation around the *y* axis as an absolute value in radians.
    fn y_radians(mut self, y: f32) -> Self {
        self.properties().switch_to_axes();
        expect_axes(self.properties()).y = y;
        self
    }

    /// Specify the orientation around the *z* axis as an absolute value in radians.
    fn z_radians(mut self, z: f32) -> Self {
        self.properties().switch_to_axes();
        expect_axes(self.properties()).z = z;
        self
    }

    /// Specify the orientation around the *x* axis as an absolute value in degrees.
    fn x_degrees(self, x: f32) -> Self {
        self.x_radians(x.to_radians())
    }

    /// Specify the orientation around the *y* axis as an absolute value in degrees.
    fn y_degrees(self, y: f32) -> Self {
        self.y_radians(y.to_radians())
    }

    /// Specify the orientation around the *z* axis as an absolute value in degrees.
    fn z_degrees(self, z: f32) -> Self {
        self.z_radians(z.to_radians())
    }

    /// Specify the orientation around the *x* axis as a number of turns around the axis.
    fn x_turns(self, x: f32) -> Self {
        self.x_radians(x * std::f32::consts::TAU)
    }

    /// Specify the orientation around the *y* axis as a number of turns around the axis.
    fn y_turns(self, y: f32) -> Self {
        self.y_radians(y * std::f32::consts::TAU)
    }

    /// Specify the orientation around the *z* axis as a number of turns around the axis.
    fn z_turns(self, z: f32) -> Self {
        self.z_radians(z * std::f32::consts::TAU)
    }

    /// Specify the orientation along each axis with the given **Vector** of radians.
    ///
    /// This has the same affect as calling `self.x_radians(v.x).y_radians(v.y).z_radians(v.z)`.
    fn radians(self, v: Vec3) -> Self {
        self.x_radians(v.x).y_radians(v.y).z_radians(v.z)
    }

    /// Specify the orientation along each axis with the given **Vector** of degrees.
    ///
    /// This has the same affect as calling `self.x_degrees(v.x).y_degrees(v.y).z_degrees(v.z)`.
    fn degrees(self, v: Vec3) -> Self {
        self.x_degrees(v.x).y_degrees(v.y).z_degrees(v.z)
    }

    /// Specify the orientation along each axis with the given **Vector** of "turns".
    ///
    /// This has the same affect as calling `self.x_turns(v.x).y_turns(v.y).z_turns(v.z)`.
    fn turns(self, v: Vec3) -> Self {
        self.x_turns(v.x).y_turns(v.y).z_turns(v.z)
    }

    /// Specify the orientation with the given euler orientation in radians.
    fn euler(self, e: Vec3) -> Self {
        self.radians(e)
    }

    /// Specify the orientation with the given **Quaternion**.
    fn quaternion(mut self, q: Quat) -> Self {
        *self.properties() = Properties::Quat(q);
        self
    }

    // Higher level methods.

    /// Specify the "pitch" of the orientation in radians.
    ///
    /// This has the same effect as calling `x_radians`.
    fn pitch(self, pitch: f32) -> Self {
        self.x_radians(pitch)
    }

    /// Specify the "yaw" of the orientation in radians.
    ///
    /// This has the same effect as calling `y_radians`.
    fn yaw(self, yaw: f32) -> Self {
        self.y_radians(yaw)
    }

    /// Specify the "roll" of the orientation in radians.
    ///
    /// This has the same effect as calling `z_radians`.
    fn roll(self, roll: f32) -> Self {
        self.z_radians(roll)
    }

    /// Assuming we're looking at a 2D plane, positive values cause a clockwise rotation where the
    /// given value is specified in radians.
    ///
    /// This is equivalent to calling the `z_radians` or `roll` methods.
    fn rotate(self, radians: f32) -> Self {
        self.z_radians(radians)
    }
}

impl SetOrientation for Properties {
    fn properties(&mut self) -> &mut Properties {
        self
    }
}

impl Default for Properties {
    fn default() -> Self {
        Properties::Axes(Vec3::ZERO)
    }
}

// Expects the `Axes` variant from the given properties.
fn expect_axes(p: &mut Properties) -> &mut Vec3 {
    match *p {
        Properties::Axes(ref mut axes) => axes,
        Properties::LookAt(_) => panic!("expected `Axes`, found `LookAt`"),
        Properties::Quat(_) => panic!("expected `Axes`, found `Quat`"),
    }
}
