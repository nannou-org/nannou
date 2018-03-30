//! Items related to describing positioning along each axis as

use geom;
use geom::graph::node;
use math::{Point2, Point3};

/// Position properties for **Drawing** a **Node**.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Properties<S = geom::DefaultScalar> {
    /// Position along the *x* axis.
    pub x: Option<Position<S>>,
    /// Position along the *y* axis.
    pub y: Option<Position<S>>,
    /// Position along the *z* axis.
    pub z: Option<Position<S>>,
}

/// A **Position** along a single axis.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Position<S = geom::DefaultScalar> {
    /// A specific position.
    Absolute(S),
    /// A position relative to some other Node.
    Relative(Relative<S>, Option<node::Index>),
}

/// Positions that are described as **Relative** to some other **Node**.
///
/// **Relative** describes a relative position along a single axis.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Relative<S = geom::DefaultScalar> {
    /// A relative scalar distance.
    Scalar(S),
    /// Aligned to either the `Start`, `Middle` or `End`.
    Align(Align<S>),
    /// A distance as a `Scalar` value over the given `Direction`.
    Direction(Direction, S),
}

/// Directionally positioned, normally relative to some other node.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Direction {
    /// Positioned forwards (*positive* **Scalar**) along some **Axis**.
    Forwards,
    /// Positioned backwards (*negative* **Scalar**) along some **Axis**.
    Backwards,
}

/// The orientation of **Align**ment along some **Axis**.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Align<S = geom::DefaultScalar> {
    /// **Align** our **Start** with the **Start** of some other node along the **Axis** with the
    /// given margin.
    Start(Option<S>),
    /// **Align** our **Middle** with the **Middle** of some other node along the **Axis**.
    Middle,
    /// **Align** our **End** with the **End** of some other node along the **Axis** with the given
    /// margin.
    End(Option<S>),
}

/// An API for setting the **position::Properties**.
pub trait SetPosition<S>: Sized {
    /// Provide a mutable reference to the **position::Properties** for updating.
    fn properties(&mut self) -> &mut Properties<S>;

    // Setters for each axis.

    /// Build with the given **Position** along the *x* axis.
    fn x_position(mut self, position: Position<S>) -> Self {
        self.properties().x = Some(position);
        self
    }

    /// Build with the given **Position** along the *y* axis.
    fn y_position(mut self, position: Position<S>) -> Self {
        self.properties().y = Some(position);
        self
    }

    /// Build with the given **Position** along the *z* axis.
    fn z_position(mut self, position: Position<S>) -> Self {
        self.properties().z = Some(position);
        self
    }

    // Absolute positioning.

    /// Build with the given **Absolute** **Position** along the *x* axis.
    fn x(self, x: S) -> Self {
        self.x_position(Position::Absolute(x))
    }

    /// Build with the given **Absolute** **Position** along the *y* axis.
    fn y(self, y: S) -> Self {
        self.y_position(Position::Absolute(y))
    }

    /// Build with the given **Absolute** **Position** along the *z* axis.
    fn z(self, y: S) -> Self {
        self.z_position(Position::Absolute(y))
    }

    /// Set the **Position** with some two-dimensional point.
    fn xy(self, p: Point2<S>) -> Self {
        self.x(p.x).y(p.y)
    }

    /// Set the **Position** with some three-dimensional point.
    fn xyz(self, p: Point3<S>) -> Self {
        self.x(p.x).y(p.y).z(p.z)
    }

    /// Set the **Position** with *x* *y* coordinates.
    fn x_y(self, x: S, y: S) -> Self {
        self.xy(Point2 { x, y })
    }

    /// Set the **Position** with *x* *y* *z* coordinates.
    fn x_y_z(self, x: S, y: S, z: S) -> Self {
        self.xyz(Point3 { x, y, z })
    }

    // Relative positioning.

    /// Set the *x* **Position** **Relative** to the previous node.
    fn x_position_relative(self, x: Relative<S>) -> Self {
        self.x_position(Position::Relative(x, None))
    }

    /// Set the *y* **Position** **Relative** to the previous node.
    fn y_position_relative(self, y: Relative<S>) -> Self {
        self.y_position(Position::Relative(y, None))
    }

    /// Set the *z* **Position** **Relative** to the previous node.
    fn z_position_relative(self, z: Relative<S>) -> Self {
        self.z_position(Position::Relative(z, None))
    }

    /// Set the *x* and *y* **Position**s **Relative** to the previous node.
    fn x_y_position_relative(self, x: Relative<S>, y: Relative<S>) -> Self {
        self.x_position_relative(x).y_position_relative(y)
    }

    /// Set the *x*, *y* and *z* **Position**s **Relative** to the previous node.
    fn x_y_z_position_relative(self, x: Relative<S>, y: Relative<S>, z: Relative<S>) -> Self {
        self.x_y_position_relative(x, y).z_position_relative(z)
    }

    /// Set the *x* **Position** **Relative** to the given node.
    fn x_position_relative_to(self, other: node::Index, x: Relative<S>) -> Self {
        self.x_position(Position::Relative(x, Some(other)))
    }

    /// Set the *y* **Position** **Relative** to the given node.
    fn y_position_relative_to(self, other: node::Index, y: Relative<S>) -> Self {
        self.y_position(Position::Relative(y, Some(other)))
    }

    /// Set the *y* **Position** **Relative** to the given node.
    fn z_position_relative_to(self, other: node::Index, z: Relative<S>) -> Self {
        self.z_position(Position::Relative(z, Some(other)))
    }

    /// Set the *x* and *y* **Position**s **Relative** to the given node.
    fn x_y_position_relative_to(self, other: node::Index, x: Relative<S>, y: Relative<S>) -> Self {
        self.x_position_relative_to(other, x)
            .y_position_relative_to(other, y)
    }

    /// Set the *x*, *y* and *z* **Position**s **Relative** to the given node.
    fn x_y_z_position_relative_to(
        self,
        other: node::Index,
        x: Relative<S>,
        y: Relative<S>,
        z: Relative<S>,
    ) -> Self {
        self.x_y_position_relative_to(other, x, y)
            .z_position_relative_to(other, z)
    }

    // Relative `Scalar` positioning.

    /// Set the **Position** as a **Scalar** along the *x* axis **Relative** to the middle of
    /// previous node.
    fn x_relative(self, x: S) -> Self {
        self.x_position_relative(Relative::Scalar(x))
    }

    /// Set the **Position** as a **Scalar** along the *y* axis **Relative** to the middle of
    /// previous node.
    fn y_relative(self, y: S) -> Self {
        self.y_position_relative(Relative::Scalar(y))
    }

    /// Set the **Position** as a **Scalar** along the *z* axis **Relative** to the middle of
    /// previous node.
    fn z_relative(self, z: S) -> Self {
        self.z_position_relative(Relative::Scalar(z))
    }

    /// Set the **Position** as a **Point** **Relative** to the middle of the previous node.
    fn xy_relative(self, p: Point2<S>) -> Self {
        self.x_relative(p.x).y_relative(p.y)
    }

    /// Set the **Position** as a **Point** **Relative** to the middle of the previous node.
    fn xyz_relative(self, p: Point3<S>) -> Self {
        self.x_relative(p.x).y_relative(p.y).z_relative(p.z)
    }

    /// Set the **Position** as **Scalar**s along the *x* and *y* axes **Relative** to the middle
    /// of the previous node.
    fn x_y_relative(self, x: S, y: S) -> Self {
        self.xy_relative(Point2 { x, y })
    }

    /// Set the **Position** as **Scalar**s along the *x*, *y* and *z* axes **Relative** to the
    /// middle of the previous node.
    fn x_y_z_relative(self, x: S, y: S, z: S) -> Self {
        self.xyz_relative(Point3 { x, y, z })
    }

    /// Set the position relative to the node with the given node::Index.
    fn x_relative_to(self, other: node::Index, x: S) -> Self {
        self.x_position_relative_to(other, Relative::Scalar(x))
    }

    /// Set the position relative to the node with the given node::Index.
    fn y_relative_to(self, other: node::Index, y: S) -> Self {
        self.y_position_relative_to(other, Relative::Scalar(y))
    }

    /// Set the position relative to the node with the given node::Index.
    fn z_relative_to(self, other: node::Index, z: S) -> Self {
        self.z_position_relative_to(other, Relative::Scalar(z))
    }

    /// Set the position relative to the node with the given node::Index.
    fn xy_relative_to(self, other: node::Index, p: Point2<S>) -> Self {
        self.x_relative_to(other, p.x).y_relative_to(other, p.y)
    }

    /// Set the position relative to the node with the given node::Index.
    fn xyz_relative_to(self, other: node::Index, p: Point3<S>) -> Self {
        self.x_relative_to(other, p.x)
            .y_relative_to(other, p.y)
            .z_relative_to(other, p.z)
    }

    /// Set the position relative to the node with the given node::Index.
    fn x_y_relative_to(self, other: node::Index, x: S, y: S) -> Self {
        self.xy_relative_to(other, Point2 { x, y })
    }

    /// Set the position relative to the node with the given node::Index.
    fn x_y_z_relative_to(self, other: node::Index, x: S, y: S, z: S) -> Self {
        self.xyz_relative_to(other, Point3 { x, y, z })
    }

    // Directional positioning.

    /// Build with the **Position** along the *x* axis as some distance from another node.
    fn x_direction(self, direction: Direction, x: S) -> Self {
        self.x_position_relative(Relative::Direction(direction, x))
    }

    /// Build with the **Position** along the *y* axis as some distance from another node.
    fn y_direction(self, direction: Direction, y: S) -> Self {
        self.y_position_relative(Relative::Direction(direction, y))
    }

    /// Build with the **Position** along the *z* axis as some distance from another node.
    fn z_direction(self, direction: Direction, z: S) -> Self {
        self.z_position_relative(Relative::Direction(direction, z))
    }

    /// Build with the **Position** as some distance to the left of another node.
    fn left(self, x: S) -> Self {
        self.x_direction(Direction::Backwards, x)
    }

    /// Build with the **Position** as some distance to the right of another node.
    fn right(self, x: S) -> Self {
        self.x_direction(Direction::Forwards, x)
    }

    /// Build with the **Position** as some distance below another node.
    fn down(self, y: S) -> Self {
        self.y_direction(Direction::Backwards, y)
    }

    /// Build with the **Position** as some distance above another node.
    fn up(self, y: S) -> Self {
        self.y_direction(Direction::Forwards, y)
    }

    /// Build with the **Position** as some distance in front of another node.
    fn backwards(self, z: S) -> Self {
        self.z_direction(Direction::Backwards, z)
    }

    /// Build with the **Position** as some distance behind another node.
    fn forwards(self, z: S) -> Self {
        self.z_direction(Direction::Forwards, z)
    }

    /// Build with the **Position** along the *x* axis as some distance from the given node.
    fn x_direction_from(self, other: node::Index, direction: Direction, x: S) -> Self {
        self.x_position_relative_to(other, Relative::Direction(direction, x))
    }

    /// Build with the **Position** along the *y* axis as some distance from the given node.
    fn y_direction_from(self, other: node::Index, direction: Direction, y: S) -> Self {
        self.y_position_relative_to(other, Relative::Direction(direction, y))
    }

    /// Build with the **Position** along the *z* axis as some distance from the given node.
    fn z_direction_from(self, other: node::Index, direction: Direction, z: S) -> Self {
        self.z_position_relative_to(other, Relative::Direction(direction, z))
    }

    /// Build with the **Position** as some distance to the left of the given node.
    fn left_from(self, other: node::Index, x: S) -> Self {
        self.x_direction_from(other, Direction::Backwards, x)
    }

    /// Build with the **Position** as some distance to the right of the given node.
    fn right_from(self, other: node::Index, x: S) -> Self {
        self.x_direction_from(other, Direction::Forwards, x)
    }

    /// Build with the **Position** as some distance below the given node.
    fn down_from(self, other: node::Index, y: S) -> Self {
        self.y_direction_from(other, Direction::Backwards, y)
    }

    /// Build with the **Position** as some distance above the given node.
    fn up_from(self, other: node::Index, y: S) -> Self {
        self.y_direction_from(other, Direction::Forwards, y)
    }

    /// Build with the **Position** as some distance in front of the given node.
    fn backwards_from(self, other: node::Index, z: S) -> Self {
        self.z_direction_from(other, Direction::Backwards, z)
    }

    /// Build with the **Position** as some distance above the given node.
    fn forwards_from(self, other: node::Index, z: S) -> Self {
        self.z_direction_from(other, Direction::Forwards, z)
    }

    // Alignment positioning.

    /// Align the **Position** of the node along the *x* axis.
    fn x_align(self, align: Align<S>) -> Self {
        self.x_position_relative(Relative::Align(align))
    }

    /// Align the **Position** of the node along the *y* axis.
    fn y_align(self, align: Align<S>) -> Self {
        self.y_position_relative(Relative::Align(align))
    }

    /// Align the **Position** of the node along the *z* axis.
    fn z_align(self, align: Align<S>) -> Self {
        self.z_position_relative(Relative::Align(align))
    }

    /// Align the position to the left.
    fn align_left(self) -> Self {
        self.x_align(Align::Start(None))
    }

    /// Align the position to the left.
    fn align_left_with_margin(self, margin: S) -> Self {
        self.x_align(Align::Start(Some(margin)))
    }

    /// Align the position to the middle.
    fn align_middle_x(self) -> Self {
        self.x_align(Align::Middle)
    }

    /// Align the position to the right.
    fn align_right(self) -> Self {
        self.x_align(Align::End(None))
    }

    /// Align the position to the right.
    fn align_right_with_margin(self, margin: S) -> Self {
        self.x_align(Align::End(Some(margin)))
    }

    /// Align the position to the bottom.
    fn align_bottom(self) -> Self {
        self.y_align(Align::Start(None))
    }

    /// Align the position to the bottom.
    fn align_bottom_with_margin(self, margin: S) -> Self {
        self.y_align(Align::Start(Some(margin)))
    }

    /// Align the position to the middle.
    fn align_middle_y(self) -> Self {
        self.y_align(Align::Middle)
    }

    /// Align the position to the top.
    fn align_top(self) -> Self {
        self.y_align(Align::End(None))
    }

    /// Align the position to the top.
    fn align_top_with_margin(self, margin: S) -> Self {
        self.y_align(Align::End(Some(margin)))
    }

    /// Align the position to the front.
    fn align_front(self) -> Self {
        self.z_align(Align::Start(None))
    }

    /// Align the position to the front.
    fn align_front_with_margin(self, margin: S) -> Self {
        self.z_align(Align::Start(Some(margin)))
    }

    /// Align the position to the middle.
    fn align_middle_z(self) -> Self {
        self.z_align(Align::Middle)
    }

    /// Align the position to the back.
    fn align_back(self) -> Self {
        self.z_align(Align::End(None))
    }

    /// Align the position to the back.
    fn align_back_with_margin(self, margin: S) -> Self {
        self.z_align(Align::End(Some(margin)))
    }

    /// Align the **Position** of the node with the given node along the *x* axis.
    fn x_align_to(self, other: node::Index, align: Align<S>) -> Self {
        self.x_position_relative_to(other, Relative::Align(align))
    }

    /// Align the **Position** of the node with the given node along the *y* axis.
    fn y_align_to(self, other: node::Index, align: Align<S>) -> Self {
        self.y_position_relative_to(other, Relative::Align(align))
    }

    /// Align the **Position** of the node with the given node along the *z* axis.
    fn z_align_to(self, other: node::Index, align: Align<S>) -> Self {
        self.z_position_relative_to(other, Relative::Align(align))
    }

    /// Align the position to the left.
    fn align_left_of(self, other: node::Index) -> Self {
        self.x_align_to(other, Align::Start(None))
    }

    /// Align the position to the left.
    fn align_left_of_with_margin(self, other: node::Index, margin: S) -> Self {
        self.x_align_to(other, Align::Start(Some(margin)))
    }

    /// Align the position to the middle.
    fn align_middle_x_of(self, other: node::Index) -> Self {
        self.x_align_to(other, Align::Middle)
    }

    /// Align the position to the right.
    fn align_right_of(self, other: node::Index) -> Self {
        self.x_align_to(other, Align::End(None))
    }

    /// Align the position to the right.
    fn align_right_of_with_margin(self, other: node::Index, margin: S) -> Self {
        self.x_align_to(other, Align::End(Some(margin)))
    }

    /// Align the position to the bottom.
    fn align_bottom_of(self, other: node::Index) -> Self {
        self.y_align_to(other, Align::Start(None))
    }

    /// Align the position to the bottom.
    fn align_bottom_of_with_margin(self, other: node::Index, margin: S) -> Self {
        self.y_align_to(other, Align::Start(Some(margin)))
    }

    /// Align the position to the middle.
    fn align_middle_y_of(self, other: node::Index) -> Self {
        self.y_align_to(other, Align::Middle)
    }

    /// Align the position to the top.
    fn align_top_of(self, other: node::Index) -> Self {
        self.y_align_to(other, Align::End(None))
    }

    /// Align the position to the top.
    fn align_top_of_with_margin(self, other: node::Index, margin: S) -> Self {
        self.y_align_to(other, Align::End(Some(margin)))
    }

    /// Align the position to the front.
    fn align_front_of(self, other: node::Index) -> Self {
        self.z_align_to(other, Align::Start(None))
    }

    /// Align the position to the front.
    fn align_front_of_with_margin(self, other: node::Index, margin: S) -> Self {
        self.z_align_to(other, Align::Start(Some(margin)))
    }

    /// Align the position to the middle.
    fn align_middle_z_of(self, other: node::Index) -> Self {
        self.z_align_to(other, Align::Middle)
    }

    /// Align the position to the back.
    fn align_back_of(self, other: node::Index) -> Self {
        self.z_align_to(other, Align::End(None))
    }

    /// Align the position to the back.
    fn align_back_of_with_margin(self, other: node::Index, margin: S) -> Self {
        self.z_align_to(other, Align::End(Some(margin)))
    }

    // Alignment combinations.

    /// Align the node to the middle of the last node.
    fn middle(self) -> Self {
        self.align_middle_x().align_middle_y().align_middle_z()
    }

    /// Align the node to the bottom left of the last node.
    fn bottom_left(self) -> Self {
        self.align_left().align_bottom()
    }

    /// Align the node to the middle left of the last node.
    fn mid_left(self) -> Self {
        self.align_left().align_middle_y()
    }

    /// Align the node to the top left of the last node.
    fn top_left(self) -> Self {
        self.align_left().align_top()
    }

    /// Align the node to the middle top of the last node.
    fn mid_top(self) -> Self {
        self.align_middle_x().align_top()
    }

    /// Align the node to the top right of the last node.
    fn top_right(self) -> Self {
        self.align_right().align_top()
    }

    /// Align the node to the middle right of the last node.
    fn mid_right(self) -> Self {
        self.align_right().align_middle_y()
    }

    /// Align the node to the bottom right of the last node.
    fn bottom_right(self) -> Self {
        self.align_right().align_bottom()
    }

    /// Align the node to the middle bottom of the last node.
    fn mid_bottom(self) -> Self {
        self.align_middle_x().align_bottom()
    }

    /// Align the node in the middle of the given Node.
    fn middle_of(self, other: node::Index) -> Self {
        self.align_middle_x_of(other)
            .align_middle_y_of(other)
            .align_middle_z_of(other)
    }

    /// Align the node to the bottom left of the given Node.
    fn bottom_left_of(self, other: node::Index) -> Self {
        self.align_left_of(other).align_bottom_of(other)
    }

    /// Align the node to the middle left of the given Node.
    fn mid_left_of(self, other: node::Index) -> Self {
        self.align_left_of(other).align_middle_y_of(other)
    }

    /// Align the node to the top left of the given Node.
    fn top_left_of(self, other: node::Index) -> Self {
        self.align_left_of(other).align_top_of(other)
    }

    /// Align the node to the middle top of the given Node.
    fn mid_top_of(self, other: node::Index) -> Self {
        self.align_middle_x_of(other).align_top_of(other)
    }

    /// Align the node to the top right of the given Node.
    fn top_right_of(self, other: node::Index) -> Self {
        self.align_right_of(other).align_top_of(other)
    }

    /// Align the node to the middle right of the given Node.
    fn mid_right_of(self, other: node::Index) -> Self {
        self.align_right_of(other).align_middle_y_of(other)
    }

    /// Align the node to the bottom right of the given Node.
    fn bottom_right_of(self, other: node::Index) -> Self {
        self.align_right_of(other).align_bottom_of(other)
    }

    /// Align the node to the middle bottom of the given Node.
    fn mid_bottom_of(self, other: node::Index) -> Self {
        self.align_middle_x_of(other).align_bottom_of(other)
    }
}

impl<S> SetPosition<S> for Properties<S> {
    fn properties(&mut self) -> &mut Properties<S> {
        self
    }
}

impl<S> Default for Properties<S> {
    fn default() -> Self {
        let x = None;
        let y = None;
        let z = None;
        Properties { x, y, z }
    }
}
