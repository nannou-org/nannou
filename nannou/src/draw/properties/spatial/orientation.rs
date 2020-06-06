use crate::geom::{self, Point3, Vector3};
use crate::math::{deg_to_rad, turns_to_rad, Angle, BaseFloat, Euler, Quaternion, Rad, Zero};

/// Orientation properties for **Drawing** a **Primitive**.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Properties<S = geom::scalar::Default> {
    /// The orientation described by an angle along each axis.
    Axes(Vector3<S>),
    /// The orientation described by looking at some other point.
    LookAt(Point3<S>),
}

impl<S> Properties<S>
where
    S: Zero,
{
    pub fn transform(&self) -> cgmath::Matrix4<S>
    where
        S: BaseFloat,
    {
        match *self {
            Properties::Axes(v) => {
                let euler = cgmath::Euler {
                    x: cgmath::Rad(v.x),
                    y: cgmath::Rad(v.y),
                    z: cgmath::Rad(v.z),
                };
                cgmath::Matrix4::from(euler)
            }
            Properties::LookAt(p) => {
                let eye = Vector3::new(S::zero(), S::zero(), S::zero());
                cgmath::Matrix4::look_at_dir(eye.into(), p.into(), up().into())
            }
        }
    }

    /// If the `Properties` was set to the `LookAt` variant, this method switches to the `Axes`
    /// variant.
    ///
    /// If the `Properties` is already `Axes`, nothing changes.
    pub fn switch_to_axes(&mut self) {
        if let Properties::LookAt(_) = *self {
            *self = Properties::Axes(Vector3::zero());
        }
    }
}

fn up<S: BaseFloat>() -> Vector3<S> {
    Vector3::new(S::zero(), S::one(), S::zero())
}

/// An API for setting the **orientation::Properties**.
pub trait SetOrientation<S>: Sized {
    /// Provide a mutable reference to the **orientation::Properties** for updating.
    fn properties(&mut self) -> &mut Properties<S>;

    // Describing orientation via a target.

    /// Describe orientation via the vector that points to the given target.
    fn look_at(mut self, target: Point3<S>) -> Self {
        *self.properties() = Properties::LookAt(target);
        self
    }

    // Absolute orientation.

    /// Specify the orientation around the *x* axis as an absolute value in radians.
    fn x_radians(mut self, x: S) -> Self
    where
        S: BaseFloat,
    {
        self.properties().switch_to_axes();
        expect_axes(self.properties()).x = x;
        self
    }

    /// Specify the orientation around the *y* axis as an absolute value in radians.
    fn y_radians(mut self, y: S) -> Self
    where
        S: BaseFloat,
    {
        self.properties().switch_to_axes();
        expect_axes(self.properties()).y = y;
        self
    }

    /// Specify the orientation around the *z* axis as an absolute value in radians.
    fn z_radians(mut self, z: S) -> Self
    where
        S: BaseFloat,
    {
        self.properties().switch_to_axes();
        expect_axes(self.properties()).z = z;
        self
    }

    /// Specify the orientation around the *x* axis as an absolute value in degrees.
    fn x_degrees(self, x: S) -> Self
    where
        S: BaseFloat,
    {
        self.x_radians(deg_to_rad(x))
    }

    /// Specify the orientation around the *y* axis as an absolute value in degrees.
    fn y_degrees(self, y: S) -> Self
    where
        S: BaseFloat,
    {
        self.y_radians(deg_to_rad(y))
    }

    /// Specify the orientation around the *z* axis as an absolute value in degrees.
    fn z_degrees(self, z: S) -> Self
    where
        S: BaseFloat,
    {
        self.z_radians(deg_to_rad(z))
    }

    /// Specify the orientation around the *x* axis as a number of turns around the axis.
    fn x_turns(self, x: S) -> Self
    where
        S: BaseFloat,
    {
        self.x_radians(turns_to_rad(x))
    }

    /// Specify the orientation around the *y* axis as a number of turns around the axis.
    fn y_turns(self, y: S) -> Self
    where
        S: BaseFloat,
    {
        self.y_radians(turns_to_rad(y))
    }

    /// Specify the orientation around the *z* axis as a number of turns around the axis.
    fn z_turns(self, z: S) -> Self
    where
        S: BaseFloat,
    {
        self.z_radians(turns_to_rad(z))
    }

    /// Specify the orientation along each axis with the given **Vector** of radians.
    ///
    /// This has the same affect as calling `self.x_radians(v.x).y_radians(v.y).z_radians(v.z)`.
    fn radians(self, v: Vector3<S>) -> Self
    where
        S: BaseFloat,
    {
        self.x_radians(v.x).y_radians(v.y).z_radians(v.z)
    }

    /// Specify the orientation along each axis with the given **Vector** of degrees.
    ///
    /// This has the same affect as calling `self.x_degrees(v.x).y_degrees(v.y).z_degrees(v.z)`.
    fn degrees(self, v: Vector3<S>) -> Self
    where
        S: BaseFloat,
    {
        self.x_degrees(v.x).y_degrees(v.y).z_degrees(v.z)
    }

    /// Specify the orientation along each axis with the given **Vector** of "turns".
    ///
    /// This has the same affect as calling `self.x_turns(v.x).y_turns(v.y).z_turns(v.z)`.
    fn turns(self, v: Vector3<S>) -> Self
    where
        S: BaseFloat,
    {
        self.x_turns(v.x).y_turns(v.y).z_turns(v.z)
    }

    /// Specify the orientation with the given **Euler**.
    ///
    /// The euler can be specified in either radians (via **Rad**) or degrees (via **Deg**).
    fn euler<A>(self, e: Euler<A>) -> Self
    where
        S: BaseFloat,
        A: Angle + Into<Rad<S>>,
    {
        self.radians(euler_to_vec3(e))
    }

    /// Specify the orientation with the given **Quaternion**.
    fn quaternion(self, q: Quaternion<S>) -> Self
    where
        S: BaseFloat,
    {
        self.euler(q.into())
    }

    // Higher level methods.

    /// Specify the "pitch" of the orientation in radians.
    ///
    /// This has the same effect as calling `x_radians`.
    fn pitch(self, pitch: S) -> Self
    where
        S: BaseFloat,
    {
        self.x_radians(pitch)
    }

    /// Specify the "yaw" of the orientation in radians.
    ///
    /// This has the same effect as calling `y_radians`.
    fn yaw(self, yaw: S) -> Self
    where
        S: BaseFloat,
    {
        self.y_radians(yaw)
    }

    /// Specify the "roll" of the orientation in radians.
    ///
    /// This has the same effect as calling `z_radians`.
    fn roll(self, roll: S) -> Self
    where
        S: BaseFloat,
    {
        self.z_radians(roll)
    }

    /// Assuming we're looking at a 2D plane, positive values cause a clockwise rotation where the
    /// given value is specified in radians.
    ///
    /// This is equivalent to calling the `z_radians` or `roll` methods.
    fn rotate(self, radians: S) -> Self
    where
        S: BaseFloat,
    {
        self.z_radians(radians)
    }
}

impl<S> SetOrientation<S> for Properties<S> {
    fn properties(&mut self) -> &mut Properties<S> {
        self
    }
}

impl<S> Default for Properties<S>
where
    S: Zero,
{
    fn default() -> Self {
        Properties::Axes(Vector3::zero())
    }
}

// Expects the `Axes` variant from the given properties.
fn expect_axes<S>(p: &mut Properties<S>) -> &mut Vector3<S> {
    match *p {
        Properties::Axes(ref mut axes) => axes,
        Properties::LookAt(_) => panic!("expected `Axes`, found `LookAt`"),
    }
}

// Convert the given `Euler` into a `Vector3`.
fn euler_to_vec3<A, S>(e: Euler<A>) -> Vector3<S>
where
    S: BaseFloat,
    A: Angle + Into<Rad<S>>,
{
    let x = e.x.into().0;
    let y = e.y.into().0;
    let z = e.z.into().0;
    Vector3 { x, y, z }
}
