use geom::graph::node;
use geom::{self, Point3, Vector3};
use math::{deg_to_rad, turns_to_rad, Angle, BaseFloat, Euler, Quaternion, Rad};

/// Orientation properties for **Drawing** a **Node**.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Properties<S> {
    /// The orientation described by an angle along each axis.
    Axes(Axes<S>),
    /// The orientation described by looking at some other point.
    LookAt(LookAt<S>),
}

/// The orientation along each axis.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Axes<S> {
    pub x: Option<Orientation<S>>,
    pub y: Option<Orientation<S>>,
    pub z: Option<Orientation<S>>,
}

/// Describe the orientation of a node via a target towards which it is facing.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LookAt<S = geom::scalar::Default> {
    Node(node::Index),
    Point(Point3<S>),
}

/// The orientation of a node along a single axis.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Orientation<S = geom::scalar::Default> {
    /// The orientation of the node along the axis in radians.
    Absolute(S),
    /// The orientation of the node described relatively to another node in radians.
    Relative(S, Option<node::Index>),
}

impl<S> Properties<S> {
    /// If the `Properties` was set to the `LookAt` variant, this method switches to the `Axes`
    /// variant.
    ///
    /// If the `Properties` is already `Axes`, nothing changes.
    pub fn switch_to_axes(&mut self) {
        if let Properties::LookAt(_) = *self {
            *self = Properties::Axes(Default::default());
        }
    }
}

/// An API for setting the **orientation::Properties**.
pub trait SetOrientation<S>: Sized {
    /// Provide a mutable reference to the **orientation::Properties** for updating.
    fn properties(&mut self) -> &mut Properties<S>;

    // Describing orientation via a target.

    /// Describe orientation via the vector that points to the given target.
    fn look_at(mut self, target: LookAt<S>) -> Self {
        *self.properties() = Properties::LookAt(target);
        self
    }

    /// Describe orientation via the vector that points to the given node.
    fn look_at_node(self, node: node::Index) -> Self {
        self.look_at(LookAt::Node(node))
    }

    /// Describe orientation via the vector that points to the given point.
    fn look_at_point(self, point: Point3<S>) -> Self {
        self.look_at(LookAt::Point(point))
    }

    // Setters for each axis.

    /// Build with the given **Orientation** along the *x* axis.
    fn x_orientation(mut self, orientation: Orientation<S>) -> Self {
        self.properties().switch_to_axes();
        expect_axes(self.properties()).x = Some(orientation);
        self
    }

    /// Build with the given **Orientation** along the *x* axis.
    fn y_orientation(mut self, orientation: Orientation<S>) -> Self {
        self.properties().switch_to_axes();
        expect_axes(self.properties()).y = Some(orientation);
        self
    }

    /// Build with the given **Orientation** along the *x* axis.
    fn z_orientation(mut self, orientation: Orientation<S>) -> Self {
        self.properties().switch_to_axes();
        expect_axes(self.properties()).z = Some(orientation);
        self
    }

    // Absolute orientation.

    /// Specify the orientation around the *x* axis as an absolute value in radians.
    fn x_radians(self, x: S) -> Self {
        self.x_orientation(Orientation::Absolute(x))
    }

    /// Specify the orientation around the *y* axis as an absolute value in radians.
    fn y_radians(self, y: S) -> Self {
        self.y_orientation(Orientation::Absolute(y))
    }

    /// Specify the orientation around the *z* axis as an absolute value in radians.
    fn z_radians(self, z: S) -> Self {
        self.z_orientation(Orientation::Absolute(z))
    }

    /// Specify the orientation around the *x* axis as an absolute value in radians.
    fn x_degrees(self, x: S) -> Self
    where
        S: BaseFloat,
    {
        self.x_radians(deg_to_rad(x))
    }

    /// Specify the orientation around the *y* axis as an absolute value in radians.
    fn y_degrees(self, y: S) -> Self
    where
        S: BaseFloat,
    {
        self.y_radians(deg_to_rad(y))
    }

    /// Specify the orientation around the *z* axis as an absolute value in radians.
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
    fn radians(self, v: Vector3<S>) -> Self {
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

    // Relative orientation.

    /// Specify the orientation around the *x* axis as a relative value in radians.
    fn x_radians_relative(self, x: S) -> Self {
        self.x_orientation(Orientation::Relative(x, None))
    }

    /// Specify the orientation around the *y* axis as a relative value in radians.
    fn y_radians_relative(self, y: S) -> Self {
        self.y_orientation(Orientation::Relative(y, None))
    }

    /// Specify the orientation around the *z* axis as a relative value in radians.
    fn z_radians_relative(self, z: S) -> Self {
        self.z_orientation(Orientation::Relative(z, None))
    }

    /// Specify the orientation around the *x* axis as a relative value in radians.
    fn x_radians_relative_to(self, other: node::Index, x: S) -> Self {
        self.x_orientation(Orientation::Relative(x, Some(other)))
    }

    /// Specify the orientation around the *y* axis as a relative value in radians.
    fn y_radians_relative_to(self, other: node::Index, y: S) -> Self {
        self.y_orientation(Orientation::Relative(y, Some(other)))
    }

    /// Specify the orientation around the *z* axis as a relative value in radians.
    fn z_radians_relative_to(self, other: node::Index, z: S) -> Self {
        self.z_orientation(Orientation::Relative(z, Some(other)))
    }

    /// Specify the orientation around the *x* axis as a relative value in degrees.
    fn x_degrees_relative(self, x: S) -> Self
    where
        S: BaseFloat,
    {
        self.x_radians_relative(deg_to_rad(x))
    }

    /// Specify the orientation around the *y* axis as a relative value in degrees.
    fn y_degrees_relative(self, y: S) -> Self
    where
        S: BaseFloat,
    {
        self.y_radians_relative(deg_to_rad(y))
    }

    /// Specify the orientation around the *z* axis as a relative value in degrees.
    fn z_degrees_relative(self, z: S) -> Self
    where
        S: BaseFloat,
    {
        self.z_radians_relative(deg_to_rad(z))
    }

    /// Specify the orientation around the *x* axis as a relative value in degrees.
    fn x_degrees_relative_to(self, other: node::Index, x: S) -> Self
    where
        S: BaseFloat,
    {
        self.x_radians_relative_to(other, deg_to_rad(x))
    }

    /// Specify the orientation around the *y* axis as a relative value in degrees.
    fn y_degrees_relative_to(self, other: node::Index, y: S) -> Self
    where
        S: BaseFloat,
    {
        self.y_radians_relative_to(other, deg_to_rad(y))
    }

    /// Specify the orientation around the *z* axis as a relative value in degrees.
    fn z_degrees_relative_to(self, other: node::Index, z: S) -> Self
    where
        S: BaseFloat,
    {
        self.z_radians_relative_to(other, deg_to_rad(z))
    }

    /// Specify the relative orientation around the *x* axis as a number of turns around the axis.
    fn x_turns_relative(self, x: S) -> Self
    where
        S: BaseFloat,
    {
        self.x_radians_relative(turns_to_rad(x))
    }

    /// Specify the relative orientation around the *y* axis as a number of turns around the axis.
    fn y_turns_relative(self, y: S) -> Self
    where
        S: BaseFloat,
    {
        self.y_radians_relative(turns_to_rad(y))
    }

    /// Specify the relative orientation around the *z* axis as a number of turns around the axis.
    fn z_turns_relative(self, z: S) -> Self
    where
        S: BaseFloat,
    {
        self.z_radians_relative(turns_to_rad(z))
    }

    /// Specify the relative orientation around the *x* axis as a number of turns around the axis.
    fn x_turns_relative_to(self, other: node::Index, x: S) -> Self
    where
        S: BaseFloat,
    {
        self.x_radians_relative_to(other, turns_to_rad(x))
    }

    /// Specify the relative orientation around the *y* axis as a number of turns around the axis.
    fn y_turns_relative_to(self, other: node::Index, y: S) -> Self
    where
        S: BaseFloat,
    {
        self.y_radians_relative_to(other, turns_to_rad(y))
    }

    /// Specify the relative orientation around the *z* axis as a number of turns around the axis.
    fn z_turns_relative_to(self, other: node::Index, z: S) -> Self
    where
        S: BaseFloat,
    {
        self.z_radians_relative_to(other, turns_to_rad(z))
    }

    /// Specify a relative orientation along each axis with the given **Vector** of radians.
    ///
    /// This has the same affect as the following:
    ///
    /// ```ignore
    /// self.x_radians_relative(v.x)
    ///     .y_radians_relative(v.y)
    ///     .z_radians_relative(v.z)
    /// ```
    fn radians_relative(self, v: Vector3<S>) -> Self {
        self.x_radians_relative(v.x)
            .y_radians_relative(v.y)
            .z_radians_relative(v.z)
    }

    /// Specify a relative orientation along each axis with the given **Vector** of radians.
    ///
    /// This has the same affect as the following:
    ///
    /// ```ignore
    /// self.x_radians_relative_to(other, v.x)
    ///     .y_radians_relative_to(other, v.y)
    ///     .z_radians_relative_to(other, v.z)
    /// ```
    fn radians_relative_to(self, other: node::Index, v: Vector3<S>) -> Self {
        self.x_radians_relative_to(other, v.x)
            .y_radians_relative_to(other, v.y)
            .z_radians_relative_to(other, v.z)
    }

    /// Specify a relative orientation along each axis with the given **Vector** of degrees.
    ///
    /// This has the same affect as the following:
    ///
    /// ```ignore
    /// self.x_degrees_relative(v.x)
    ///     .y_degrees_relative(v.y)
    ///     .z_degrees_relative(v.z)
    /// ```
    fn degrees_relative(self, v: Vector3<S>) -> Self
    where
        S: BaseFloat,
    {
        self.x_degrees_relative(v.x)
            .y_degrees_relative(v.y)
            .z_degrees_relative(v.z)
    }

    /// Specify a relative orientation along each axis with the given **Vector** of degrees.
    ///
    /// This has the same affect as the following:
    ///
    /// ```ignore
    /// self.x_degrees_relative_to(other, v.x)
    ///     .y_degrees_relative_to(other, v.y)
    ///     .z_degrees_relative_to(other, v.z)
    /// ```
    fn degrees_relative_to(self, other: node::Index, v: Vector3<S>) -> Self
    where
        S: BaseFloat,
    {
        self.x_degrees_relative_to(other, v.x)
            .y_degrees_relative_to(other, v.y)
            .z_degrees_relative_to(other, v.z)
    }

    /// Specify a relative orientation along each axis with the given **Vector** of "turns".
    ///
    /// This has the same affect as the following:
    ///
    /// ```ignore
    /// self.x_turns_relative(v.x)
    ///     .y_turns_relative(v.y)
    ///     .z_turns_relative(v.z)
    /// ```
    fn turns_relative(self, v: Vector3<S>) -> Self
    where
        S: BaseFloat,
    {
        self.x_turns_relative(v.x)
            .y_turns_relative(v.y)
            .z_turns_relative(v.z)
    }

    /// Specify a relative orientation along each axis with the given **Vector** of "turns".
    ///
    /// This has the same affect as the following:
    ///
    /// ```ignore
    /// self.x_turns_relative_to(other, v.x)
    ///     .y_turns_relative_to(other, v.y)
    ///     .z_turns_relative_to(other, v.z)
    /// ```
    fn turns_relative_to(self, other: node::Index, v: Vector3<S>) -> Self
    where
        S: BaseFloat,
    {
        self.x_turns_relative_to(other, v.x)
            .y_turns_relative_to(other, v.y)
            .z_turns_relative_to(other, v.z)
    }

    /// Specify a relative orientation with the given **Euler**.
    ///
    /// The euler can be specified in either radians (via **Rad**) or degrees (via **Deg**).
    fn euler_relative<A>(self, e: Euler<A>) -> Self
    where
        S: BaseFloat,
        A: Angle + Into<Rad<S>>,
    {
        self.radians_relative(euler_to_vec3(e))
    }

    /// Specify a relative orientation with the given **Euler**.
    ///
    /// The euler can be specified in either radians (via **Rad**) or degrees (via **Deg**).
    fn euler_relative_to<A>(self, other: node::Index, e: Euler<A>) -> Self
    where
        S: BaseFloat,
        A: Angle + Into<Rad<S>>,
    {
        self.radians_relative_to(other, euler_to_vec3(e))
    }

    // Higher level methods.

    /// Specify the "pitch" of the orientation in radians.
    ///
    /// This has the same effect as calling `x_radians`.
    fn pitch(self, pitch: S) -> Self {
        self.x_radians(pitch)
    }

    /// Specify the "yaw" of the orientation in radians.
    ///
    /// This has the same effect as calling `y_radians`.
    fn yaw(self, yaw: S) -> Self {
        self.y_radians(yaw)
    }

    /// Specify the "roll" of the orientation in radians.
    ///
    /// This has the same effect as calling `z_radians`.
    fn roll(self, roll: S) -> Self {
        self.z_radians(roll)
    }

    /// Assuming we're looking at a 2D plane, positive values cause a clockwise rotation where the
    /// given value is specified in radians.
    ///
    /// This is equivalent to calling the `z_radians` or `roll` methods.
    fn rotate(self, radians: S) -> Self {
        self.z_radians(radians)
    }
}

impl<S> SetOrientation<S> for Properties<S> {
    fn properties(&mut self) -> &mut Properties<S> {
        self
    }
}

impl<S> Default for Properties<S> {
    fn default() -> Self {
        Properties::Axes(Default::default())
    }
}

impl<S> Default for Axes<S> {
    fn default() -> Self {
        Axes {
            x: None,
            y: None,
            z: None,
        }
    }
}

impl<S> From<node::Index> for LookAt<S> {
    fn from(node: node::Index) -> Self {
        LookAt::Node(node)
    }
}

impl<S> From<Point3<S>> for LookAt<S> {
    fn from(point: Point3<S>) -> Self {
        LookAt::Point(point)
    }
}

// Expects the `Axes` variant from the given properties.
fn expect_axes<S>(p: &mut Properties<S>) -> &mut Axes<S> {
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
