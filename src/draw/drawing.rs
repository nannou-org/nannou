use draw::{self, Draw};
use draw::properties::spatial::{dimension, position};
use draw::properties::{ColorScalar, IntoDrawn, IntoRgba, Primitive, SetColor, SetDimensions, SetPosition};
use geom;
use geom::graph::node;
use math::{BaseFloat, Point2, Point3, Vector2, Vector3};
use std::marker::PhantomData;

/// A **Drawing** in progress.
///
/// **Drawing** provides a way of chaining together method calls describing properties of the thing
/// that we are drawing. **Drawing** ends when the instance is **Drop**ped, at which point the
/// properties of the drawing are inserted into the **Draw** type.
///
/// When a **Drawing** begins, a node is immediately created for the drawing within the **Draw**'s
/// inner **geom::Graph**. This ensures the correct instantiation order is maintained within the
/// graph. As a result, each **Drawing** is associated with a single, unique node. Thus a
/// **Drawing** can be thought of as a way of specifying properties for a node.
#[derive(Debug)]
pub struct Drawing<'a, T, S = geom::DefaultScalar>
where
    T: IntoDrawn<S>,
    S: 'a + BaseFloat,
{
    // The `Draw` instance used to create this drawing.
    draw: &'a Draw<S>,
    // The `Index` of the node that was created.
    //
    // This may not be accessed by the user until drawing is complete. This is because the
    // **Drawing** may yet describe further positioning, orientation or scaling and in turn using
    // the index to refer to a node before these properties are set may yield unexpected behaviour.
    index: node::Index,
    // The node type currently being drawn.
    _ty: PhantomData<T>,
}

/// Construct a new **Drawing** instance.
pub fn new<'a, T, S>(draw: &'a Draw<S>, index: node::Index) -> Drawing<'a, T, S>
where
    T: IntoDrawn<S>,
    S: BaseFloat,
{
    let _ty = PhantomData;
    Drawing { draw, index, _ty }
}

impl<'a, T, S> Drop for Drawing<'a, T, S>
where
    T: IntoDrawn<S>,
    S: BaseFloat,
{
    fn drop(&mut self) {
        self.finish_inner()
            .expect("the drawing contained a relative edge that would have caused a cycle within the geometry graph");
    }
}

impl<'a, T, S> Drawing<'a, T, S>
where
    T: IntoDrawn<S>,
    S: BaseFloat,
{
    // Shared between the **finish** method and the **Drawing**'s **Drop** implementation.
    //
    // 1. Create vertices based on node-specific position, points, etc.
    // 2. Insert edges into geom graph based on
    fn finish_inner(&mut self) -> Result<(), geom::graph::WouldCycle<S>> {
        if let Ok(mut state) = self.draw.state.try_borrow_mut() {
            if let Some(prim) = state.drawing.remove(&self.index) {
                let index = self.index;
                draw::draw_primitive(&mut state, index, prim)?;
            }
        }
        Ok(())
    }

    /// Complete the drawing and insert it into the parent **Draw** instance.
    ///
    /// This will be called when the **Drawing** is **Drop**ped if it has not yet been called.
    pub fn finish(mut self) -> Result<(), geom::graph::WouldCycle<S>> {
        self.finish_inner()
    }

    /// Complete the drawing and return its unique identifier.
    ///
    /// **Panics** if adding the edge would cause a cycle in the graph.
    pub fn id(self) -> node::Index {
        let id = self.index;
        self.finish().expect(draw::WOULD_CYCLE);
        id
    }

    // Map the given function onto the primitive stored within **Draw** at `index`.
    //
    // The functionn is only applied if the node has not yet been **Drawn**.
    fn map_primitive<F>(self, map: F) -> Self
    where
        F: FnOnce(draw::properties::Primitive<S>) -> draw::properties::Primitive<S>,
    {
        if let Ok(mut state) = self.draw.state.try_borrow_mut() {
            if let Some(mut primitive) = state.drawing.remove(&self.index) {
                primitive = map(primitive);
                state.drawing.insert(self.index, primitive);
            }
        }
        self
    }

    /// Apply the given function to the type stored within **Draw**.
    ///
    /// The function is only applied if the node has not yet been **Drawn**.
    ///
    /// **Panics** if the primitive does not contain type **T**.
    pub fn map_ty<F, T2>(self, map: F) -> Self
    where
        F: FnOnce(T) -> T2,
        T2: Into<Primitive<S>>,
        Primitive<S>: Into<Option<T>>,
    {
        self.map_primitive(|prim| {
            let maybe_ty: Option<T> = prim.into();
            let ty = maybe_ty.expect("expected `T` but primitive contained different type");
            let ty2 = map(ty);
            ty2.into()
        })
    }
}

// SetColor implementations.

impl<'a, T, S> Drawing<'a, T, S>
where
    T: IntoDrawn<S> + SetColor<ColorScalar> + Into<Primitive<S>>,
    Primitive<S>: Into<Option<T>>,
    S: BaseFloat,
{
    /// Specify a color.
    ///
    /// This method supports any color type that can be converted into RGBA.
    ///
    /// Colors that have no alpha channel will be given an opaque alpha channel value `1.0`.
    pub fn color<C>(self, color: C) -> Self
    where
        C: IntoRgba<ColorScalar>,
    {
        self.map_ty(|ty| SetColor::color(ty, color))
    }

    /// Specify the color via red, green and blue channels.
    pub fn rgb(self, r: ColorScalar, g: ColorScalar, b: ColorScalar) -> Self {
        self.map_ty(|ty| SetColor::rgb(ty, r, g, b))
    }

    /// Specify the color via red, green, blue and alpha channels.
    pub fn rgba(self, r: ColorScalar, g: ColorScalar, b: ColorScalar, a: ColorScalar) -> Self {
        self.map_ty(|ty| SetColor::rgba(ty, r, g, b, a))
    }

    /// Specify the color via hue, saturation and luminance.
    ///
    /// If you're looking for HSVA or HSBA, use the `hsva` method instead.
    ///
    /// The given hue expects a value between `0.0` and `1.0` where `0.0` is 0 degress and `1.0` is
    /// 360 degrees (or 2 PI radians).
    ///
    /// See the [wikipedia entry](https://en.wikipedia.org/wiki/HSL_and_HSV) for more details on
    /// this color space.
    pub fn hsl(self, h: ColorScalar, s: ColorScalar, l: ColorScalar) -> Self {
        self.map_ty(|ty| SetColor::hsl(ty, h, s, l))
    }

    /// Specify the color via hue, saturation, luminance and an alpha channel.
    ///
    /// If you're looking for HSVA or HSBA, use the `hsva` method instead.
    ///
    /// The given hue expects a value between `0.0` and `1.0` where `0.0` is 0 degress and `1.0` is
    /// 360 degrees (or 2 PI radians).
    ///
    /// See the [wikipedia entry](https://en.wikipedia.org/wiki/HSL_and_HSV) for more details on
    /// this color space.
    pub fn hsla(self, h: ColorScalar, s: ColorScalar, l: ColorScalar, a: ColorScalar) -> Self {
        self.map_ty(|ty| SetColor::hsla(ty, h, s, l, a))
    }

    /// Specify the color via hue, saturation and *value* (brightness).
    ///
    /// This is sometimes also known as "hsb".
    ///
    /// The given hue expects a value between `0.0` and `1.0` where `0.0` is 0 degress and `1.0` is
    /// 360 degrees (or 2 PI radians).
    ///
    /// See the [wikipedia entry](https://en.wikipedia.org/wiki/HSL_and_HSV) for more details on
    /// this color space.
    pub fn hsv(self, h: ColorScalar, s: ColorScalar, v: ColorScalar) -> Self {
        self.map_ty(|ty| SetColor::hsv(ty, h, s, v))
    }

    /// Specify the color via hue, saturation, *value* (brightness) and an alpha channel.
    ///
    /// This is sometimes also known as "hsba".
    ///
    /// The given hue expects a value between `0.0` and `1.0` where `0.0` is 0 degress and `1.0` is
    /// 360 degrees (or 2 PI radians).
    ///
    /// See the [wikipedia entry](https://en.wikipedia.org/wiki/HSL_and_HSV) for more details on
    /// this color space.
    pub fn hsva(self, h: ColorScalar, s: ColorScalar, v: ColorScalar, a: ColorScalar) -> Self {
        self.map_ty(|ty| SetColor::hsva(ty, h, s, v, a))
    }
}

// SetDimensions implementations.

impl<'a, T, S> Drawing<'a, T, S>
where
    T: IntoDrawn<S> + SetDimensions<S> + Into<Primitive<S>>,
    Primitive<S>: Into<Option<T>>,
    S: BaseFloat,
{
    // Setters for each axis.

    /// Set the length along the x axis.
    pub fn x_dimension(self, x: dimension::Dimension<S>) -> Self {
        self.map_ty(|ty| SetDimensions::x_dimension(ty, x))
    }

    /// Set the length along the y axis.
    pub fn y_dimension(self, y: dimension::Dimension<S>) -> Self {
        self.map_ty(|ty| SetDimensions::y_dimension(ty, y))
    }

    /// Set the length along the z axis.
    pub fn z_dimension(self, z: dimension::Dimension<S>) -> Self {
        self.map_ty(|ty| SetDimensions::z_dimension(ty, z))
    }

    // Absolute dimensions.

    /// Set the absolute width for the node.
    pub fn width(self, w: S) -> Self {
        self.map_ty(|ty| SetDimensions::width(ty, w))
    }

    /// Set the absolute height for the node.
    pub fn height(self, h: S) -> Self {
        self.map_ty(|ty| SetDimensions::height(ty, h))
    }

    /// Set the absolute depth for the node.
    pub fn depth(self, d: S) -> Self {
        self.map_ty(|ty| SetDimensions::depth(ty, d))
    }

    /// Short-hand for the **width** method.
    pub fn w(self, w: S) -> Self {
        self.map_ty(|ty| SetDimensions::w(ty, w))
    }

    /// Short-hand for the **height** method.
    pub fn h(self, h: S) -> Self {
        self.map_ty(|ty| SetDimensions::h(ty, h))
    }

    /// Short-hand for the **depth** method.
    pub fn d(self, d: S) -> Self {
        self.map_ty(|ty| SetDimensions::d(ty, d))
    }

    /// Set the **x** and **y** dimensions for the node.
    pub fn wh(self, v: Vector2<S>) -> Self {
        self.map_ty(|ty| SetDimensions::wh(ty, v))
    }

    /// Set the **x**, **y** and **z** dimensions for the node.
    pub fn whd(self, v: Vector3<S>) -> Self {
        self.map_ty(|ty| SetDimensions::whd(ty, v))
    }

    /// Set the width and height for the node.
    pub fn w_h(self, x: S, y: S) -> Self {
        self.map_ty(|ty| SetDimensions::w_h(ty, x, y))
    }

    /// Set the width and height for the node.
    pub fn w_h_d(self, x: S, y: S, z: S) -> Self {
        self.map_ty(|ty| SetDimensions::w_h_d(ty, x, y, z))
    }

    // Relative dimensions.

    /// Some relative dimension along the **x** axis.
    pub fn x_dimension_relative(self, other: node::Index, x: dimension::Relative<S>) -> Self {
        self.map_ty(|ty| SetDimensions::x_dimension_relative(ty, other, x))
    }

    /// Some relative dimension along the **y** axis.
    pub fn y_dimension_relative(self, other: node::Index, y: dimension::Relative<S>) -> Self {
        self.map_ty(|ty| SetDimensions::y_dimension_relative(ty, other, y))
    }

    /// Some relative dimension along the **z** axis.
    pub fn z_dimension_relative(self, other: node::Index, z: dimension::Relative<S>) -> Self {
        self.map_ty(|ty| SetDimensions::z_dimension_relative(ty, other, z))
    }

    /// Set the x-axis dimension as the width of the node at the given index.
    pub fn w_of(self, other: node::Index) -> Self {
        self.map_ty(|ty| SetDimensions::w_of(ty, other))
    }

    /// Set the y-axis dimension as the height of the node at the given index.
    pub fn h_of(self, other: node::Index) -> Self {
        self.map_ty(|ty| SetDimensions::h_of(ty, other))
    }

    /// Set the z-axis dimension as the depth of the node at the given index.
    pub fn d_of(self, other: node::Index) -> Self {
        self.map_ty(|ty| SetDimensions::d_of(ty, other))
    }

    /// Set the dimensions as the dimensions of the node at the given index.
    pub fn wh_of(self, other: node::Index) -> Self {
        self.map_ty(|ty| SetDimensions::wh_of(ty, other))
    }

    /// Set the dimensions as the dimensions of the node at the given index.
    pub fn whd_of(self, other: node::Index) -> Self {
        self.map_ty(|ty| SetDimensions::whd_of(ty, other))
    }

    /// Set the width as the width of the node at the given index padded at both ends by the
    /// given Scalar.
    pub fn padded_w_of(self, other: node::Index, pad: S) -> Self {
        self.map_ty(|ty| SetDimensions::padded_w_of(ty, other, pad))
    }

    /// Set the height as the height of the node at the given index padded at both ends by the
    /// given Scalar.
    pub fn padded_h_of(self, other: node::Index, pad: S) -> Self {
        self.map_ty(|ty| SetDimensions::padded_h_of(ty, other, pad))
    }

    /// Set the depth as the depth of the node at the given index padded at both ends by the
    /// given Scalar.
    pub fn padded_d_of(self, other: node::Index, pad: S) -> Self {
        self.map_ty(|ty| SetDimensions::padded_d_of(ty, other, pad))
    }

    /// Set the dimensions as the dimensions of the node at the given index with each dimension
    /// padded by the given scalar.
    pub fn padded_wh_of(self, other: node::Index, pad: S) -> Self
    where
        S: Clone,
    {
        self.map_ty(|ty| SetDimensions::padded_wh_of(ty, other, pad))
    }

    /// Set the dimensions as the dimensions of the node at the given index with each dimension
    /// padded by the given scalar.
    pub fn padded_whd_of(self, other: node::Index, pad: S) -> Self
    where
        S: Clone,
    {
        self.map_ty(|ty| SetDimensions::padded_whd_of(ty, other, pad))
    }

    /// Set the width as the width of the node at the given index multiplied by the given **scale**
    /// Scalar value.
    pub fn scaled_w_of(self, other: node::Index, scale: S) -> Self {
        self.map_ty(|ty| SetDimensions::scaled_w_of(ty, other, scale))
    }

    /// Set the height as the height of the node at the given index multiplied by the given **scale**
    /// Scalar value.
    pub fn scaled_h_of(self, other: node::Index, scale: S) -> Self {
        self.map_ty(|ty| SetDimensions::scaled_h_of(ty, other, scale))
    }

    /// Set the depth as the depth of the node at the given index multiplied by the given **scale**
    /// Scalar value.
    pub fn scaled_d_of(self, other: node::Index, scale: S) -> Self {
        self.map_ty(|ty| SetDimensions::scaled_d_of(ty, other, scale))
    }

    /// Set the dimensions as the dimensions of the node at the given index multiplied by the given
    /// **scale** Scalar value.
    pub fn scaled_wh_of(self, other: node::Index, scale: S) -> Self
    where
        S: Clone,
    {
        self.map_ty(|ty| SetDimensions::scaled_wh_of(ty, other, scale))
    }

    /// Set the dimensions as the dimensions of the node at the given index multiplied by the given
    /// **scale** Scalar value.
    pub fn scaled_whd_of(self, other: node::Index, scale: S) -> Self
    where
        S: Clone,
    {
        self.map_ty(|ty| SetDimensions::scaled_whd_of(ty, other, scale))
    }
}

// SetPosition methods.

impl<'a, T, S> Drawing<'a, T, S>
where
    T: IntoDrawn<S> + SetPosition<S> + Into<Primitive<S>>,
    Primitive<S>: Into<Option<T>>,
    S: BaseFloat,
{
    /// Build with the given **Position** along the *x* axis.
    pub fn x_position(self, position: position::Position<S>) -> Self {
        self.map_ty(|ty| SetPosition::x_position(ty, position))
    }

    /// Build with the given **Position** along the *y* axis.
    pub fn y_position(self, position: position::Position<S>) -> Self {
        self.map_ty(|ty| SetPosition::y_position(ty, position))
    }

    /// Build with the given **Position** along the *z* axis.
    pub fn z_position(self, position: position::Position<S>) -> Self {
        self.map_ty(|ty| SetPosition::z_position(ty, position))
    }

    // Absolute positioning.

    /// Build with the given **Absolute** **Position** along the *x* axis.
    pub fn x(self, x: S) -> Self {
        self.map_ty(|ty| SetPosition::x(ty, x))
    }

    /// Build with the given **Absolute** **Position** along the *y* axis.
    pub fn y(self, y: S) -> Self {
        self.map_ty(|ty| SetPosition::y(ty, y))
    }

    /// Build with the given **Absolute** **Position** along the *z* axis.
    pub fn z(self, z: S) -> Self {
        self.map_ty(|ty| SetPosition::z(ty, z))
    }

    /// Set the **Position** with some two-dimensional point.
    pub fn xy(self, p: Point2<S>) -> Self {
        self.map_ty(|ty| SetPosition::xy(ty, p))
    }

    /// Set the **Position** with some three-dimensional point.
    pub fn xyz(self, p: Point3<S>) -> Self {
        self.map_ty(|ty| SetPosition::xyz(ty, p))
    }

    /// Set the **Position** with *x* *y* coordinates.
    pub fn x_y(self, x: S, y: S) -> Self {
        self.map_ty(|ty| SetPosition::x_y(ty, x, y))
    }

    /// Set the **Position** with *x* *y* *z* coordinates.
    pub fn x_y_z(self, x: S, y: S, z: S) -> Self {
        self.map_ty(|ty| SetPosition::x_y_z(ty, x, y, z))
    }

    // Relative positioning.

    /// Set the *x* **Position** **Relative** to the previous node.
    pub fn x_position_relative(self, x: position::Relative<S>) -> Self {
        self.map_ty(|ty| SetPosition::x_position_relative(ty, x))
    }

    /// Set the *y* **Position** **Relative** to the previous node.
    pub fn y_position_relative(self, y: position::Relative<S>) -> Self {
        self.map_ty(|ty| SetPosition::y_position_relative(ty, y))
    }

    /// Set the *z* **Position** **Relative** to the previous node.
    pub fn z_position_relative(self, z: position::Relative<S>) -> Self {
        self.map_ty(|ty| SetPosition::z_position_relative(ty, z))
    }

    /// Set the *x* and *y* **Position**s **Relative** to the previous node.
    pub fn x_y_position_relative(self, x: position::Relative<S>, y: position::Relative<S>) -> Self {
        self.map_ty(|ty| SetPosition::x_y_position_relative(ty, x, y))
    }

    /// Set the *x*, *y* and *z* **Position**s **Relative** to the previous node.
    pub fn x_y_z_position_relative(
        self,
        x: position::Relative<S>,
        y: position::Relative<S>,
        z: position::Relative<S>,
    ) -> Self {
        self.map_ty(|ty| SetPosition::x_y_z_position_relative(ty, x, y, z))
    }

    /// Set the *x* **Position** **Relative** to the given node.
    pub fn x_position_relative_to(self, other: node::Index, x: position::Relative<S>) -> Self {
        self.map_ty(|ty| SetPosition::x_position_relative_to(ty, other, x))
    }

    /// Set the *y* **Position** **Relative** to the given node.
    pub fn y_position_relative_to(self, other: node::Index, y: position::Relative<S>) -> Self {
        self.map_ty(|ty| SetPosition::y_position_relative_to(ty, other, y))
    }

    /// Set the *y* **Position** **Relative** to the given node.
    pub fn z_position_relative_to(self, other: node::Index, z: position::Relative<S>) -> Self {
        self.map_ty(|ty| SetPosition::z_position_relative_to(ty, other, z))
    }

    /// Set the *x* and *y* **Position**s **Relative** to the given node.
    pub fn x_y_position_relative_to(
        self,
        other: node::Index,
        x: position::Relative<S>,
        y: position::Relative<S>,
    ) -> Self {
        self.map_ty(|ty| SetPosition::x_y_position_relative_to(ty, other, x, y))
    }

    /// Set the *x*, *y* and *z* **Position**s **Relative** to the given node.
    pub fn x_y_z_position_relative_to(
        self,
        other: node::Index,
        x: position::Relative<S>,
        y: position::Relative<S>,
        z: position::Relative<S>,
    ) -> Self {
        self.map_ty(|ty| SetPosition::x_y_z_position_relative_to(ty, other, x, y, z))
    }

    // Relative `Scalar` positioning.

    /// Set the **Position** as a **Scalar** along the *x* axis **Relative** to the middle of
    /// previous node.
    pub fn x_relative(self, x: S) -> Self {
        self.map_ty(|ty| SetPosition::x_relative(ty, x))
    }

    /// Set the **Position** as a **Scalar** along the *y* axis **Relative** to the middle of
    /// previous node.
    pub fn y_relative(self, y: S) -> Self {
        self.map_ty(|ty| SetPosition::y_relative(ty, y))
    }

    /// Set the **Position** as a **Scalar** along the *z* axis **Relative** to the middle of
    /// previous node.
    pub fn z_relative(self, z: S) -> Self {
        self.map_ty(|ty| SetPosition::z_relative(ty, z))
    }

    /// Set the **Position** as a **Point** **Relative** to the middle of the previous node.
    pub fn xy_relative(self, p: Point2<S>) -> Self {
        self.map_ty(|ty| SetPosition::xy_relative(ty, p))
    }

    /// Set the **Position** as a **Point** **Relative** to the middle of the previous node.
    pub fn xyz_relative(self, p: Point3<S>) -> Self {
        self.map_ty(|ty| SetPosition::xyz_relative(ty, p))
    }

    /// Set the **Position** as **Scalar**s along the *x* and *y* axes **Relative** to the middle
    /// of the previous node.
    pub fn x_y_relative(self, x: S, y: S) -> Self {
        self.map_ty(|ty| SetPosition::x_y_relative(ty, x, y))
    }

    /// Set the **Position** as **Scalar**s along the *x*, *y* and *z* axes **Relative** to the
    /// middle of the previous node.
    pub fn x_y_z_relative(self, x: S, y: S, z: S) -> Self {
        self.map_ty(|ty| SetPosition::x_y_z_relative(ty, x, y, z))
    }

    /// Set the position relative to the node with the given node::Index.
    pub fn x_relative_to(self, other: node::Index, x: S) -> Self {
        self.map_ty(|ty| SetPosition::x_relative_to(ty, other, x))
    }

    /// Set the position relative to the node with the given node::Index.
    pub fn y_relative_to(self, other: node::Index, y: S) -> Self {
        self.map_ty(|ty| SetPosition::y_relative_to(ty, other, y))
    }

    /// Set the position relative to the node with the given node::Index.
    pub fn z_relative_to(self, other: node::Index, z: S) -> Self {
        self.map_ty(|ty| SetPosition::z_relative_to(ty, other, z))
    }

    /// Set the position relative to the node with the given node::Index.
    pub fn xy_relative_to(self, other: node::Index, p: Point2<S>) -> Self {
        self.map_ty(|ty| SetPosition::xy_relative_to(ty, other, p))
    }

    /// Set the position relative to the node with the given node::Index.
    pub fn xyz_relative_to(self, other: node::Index, p: Point3<S>) -> Self {
        self.map_ty(|ty| SetPosition::xyz_relative_to(ty, other, p))
    }

    /// Set the position relative to the node with the given node::Index.
    pub fn x_y_relative_to(self, other: node::Index, x: S, y: S) -> Self {
        self.map_ty(|ty| SetPosition::x_y_relative_to(ty, other, x, y))
    }

    /// Set the position relative to the node with the given node::Index.
    pub fn x_y_z_relative_to(self, other: node::Index, x: S, y: S, z: S) -> Self {
        self.map_ty(|ty| SetPosition::x_y_z_relative_to(ty, other, x, y, z))
    }

    // Directional positioning.

    /// Build with the **Position** along the *x* axis as some distance from another node.
    pub fn x_direction(self, direction: position::Direction, x: S) -> Self {
        self.map_ty(|ty| SetPosition::x_direction(ty, direction, x))
    }

    /// Build with the **Position** along the *y* axis as some distance from another node.
    pub fn y_direction(self, direction: position::Direction, y: S) -> Self {
        self.map_ty(|ty| SetPosition::y_direction(ty, direction, y))
    }

    /// Build with the **Position** along the *z* axis as some distance from another node.
    pub fn z_direction(self, direction: position::Direction, z: S) -> Self {
        self.map_ty(|ty| SetPosition::z_direction(ty, direction, z))
    }

    /// Build with the **Position** as some distance to the left of another node.
    pub fn left(self, x: S) -> Self {
        self.map_ty(|ty| SetPosition::left(ty, x))
    }

    /// Build with the **Position** as some distance to the right of another node.
    pub fn right(self, x: S) -> Self {
        self.map_ty(|ty| SetPosition::right(ty, x))
    }

    /// Build with the **Position** as some distance below another node.
    pub fn down(self, y: S) -> Self {
        self.map_ty(|ty| SetPosition::down(ty, y))
    }

    /// Build with the **Position** as some distance above another node.
    pub fn up(self, y: S) -> Self {
        self.map_ty(|ty| SetPosition::up(ty, y))
    }

    /// Build with the **Position** as some distance in front of another node.
    pub fn backwards(self, z: S) -> Self {
        self.map_ty(|ty| SetPosition::backwards(ty, z))
    }

    /// Build with the **Position** as some distance behind another node.
    pub fn forwards(self, z: S) -> Self {
        self.map_ty(|ty| SetPosition::forwards(ty, z))
    }

    /// Build with the **Position** along the *x* axis as some distance from the given node.
    pub fn x_direction_from(
        self,
        other: node::Index,
        direction: position::Direction,
        x: S,
    ) -> Self {
        self.map_ty(|ty| SetPosition::x_direction_from(ty, other, direction, x))
    }

    /// Build with the **Position** along the *y* axis as some distance from the given node.
    pub fn y_direction_from(
        self,
        other: node::Index,
        direction: position::Direction,
        y: S,
    ) -> Self {
        self.map_ty(|ty| SetPosition::y_direction_from(ty, other, direction, y))
    }

    /// Build with the **Position** along the *z* axis as some distance from the given node.
    pub fn z_direction_from(
        self,
        other: node::Index,
        direction: position::Direction,
        z: S,
    ) -> Self {
        self.map_ty(|ty| SetPosition::z_direction_from(ty, other, direction, z))
    }

    /// Build with the **Position** as some distance to the left of the given node.
    pub fn left_from(self, other: node::Index, x: S) -> Self {
        self.map_ty(|ty| SetPosition::left_from(ty, other, x))
    }

    /// Build with the **Position** as some distance to the right of the given node.
    pub fn right_from(self, other: node::Index, x: S) -> Self {
        self.map_ty(|ty| SetPosition::right_from(ty, other, x))
    }

    /// Build with the **Position** as some distance below the given node.
    pub fn down_from(self, other: node::Index, y: S) -> Self {
        self.map_ty(|ty| SetPosition::down_from(ty, other, y))
    }

    /// Build with the **Position** as some distance above the given node.
    pub fn up_from(self, other: node::Index, y: S) -> Self {
        self.map_ty(|ty| SetPosition::up_from(ty, other, y))
    }

    /// Build with the **Position** as some distance in front of the given node.
    pub fn backwards_from(self, other: node::Index, z: S) -> Self {
        self.map_ty(|ty| SetPosition::backwards_from(ty, other, z))
    }

    /// Build with the **Position** as some distance above the given node.
    pub fn forwards_from(self, other: node::Index, z: S) -> Self {
        self.map_ty(|ty| SetPosition::forwards_from(ty, other, z))
    }

    // Alignment positioning.

    /// Align the **Position** of the node along the *x* axis.
    pub fn x_align(self, align: position::Align<S>) -> Self {
        self.map_ty(|ty| SetPosition::x_align(ty, align))
    }

    /// Align the **Position** of the node along the *y* axis.
    pub fn y_align(self, align: position::Align<S>) -> Self {
        self.map_ty(|ty| SetPosition::y_align(ty, align))
    }

    /// Align the **Position** of the node along the *z* axis.
    pub fn z_align(self, align: position::Align<S>) -> Self {
        self.map_ty(|ty| SetPosition::z_align(ty, align))
    }

    /// Align the position to the left.
    pub fn align_left(self) -> Self {
        self.map_ty(|ty| SetPosition::align_left(ty))
    }

    /// Align the position to the left.
    pub fn align_left_with_margin(self, margin: S) -> Self {
        self.map_ty(|ty| SetPosition::align_left_with_margin(ty, margin))
    }

    /// Align the position to the middle.
    pub fn align_middle_x(self) -> Self {
        self.map_ty(|ty| SetPosition::align_middle_x(ty))
    }

    /// Align the position to the right.
    pub fn align_right(self) -> Self {
        self.map_ty(|ty| SetPosition::align_right(ty))
    }

    /// Align the position to the right.
    pub fn align_right_with_margin(self, margin: S) -> Self {
        self.map_ty(|ty| SetPosition::align_right_with_margin(ty, margin))
    }

    /// Align the position to the bottom.
    pub fn align_bottom(self) -> Self {
        self.map_ty(|ty| SetPosition::align_bottom(ty))
    }

    /// Align the position to the bottom.
    pub fn align_bottom_with_margin(self, margin: S) -> Self {
        self.map_ty(|ty| SetPosition::align_bottom_with_margin(ty, margin))
    }

    /// Align the position to the middle.
    pub fn align_middle_y(self) -> Self {
        self.map_ty(|ty| SetPosition::align_middle_y(ty))
    }

    /// Align the position to the top.
    pub fn align_top(self) -> Self {
        self.map_ty(|ty| SetPosition::align_top(ty))
    }

    /// Align the position to the top.
    pub fn align_top_with_margin(self, margin: S) -> Self {
        self.map_ty(|ty| SetPosition::align_top_with_margin(ty, margin))
    }

    /// Align the position to the front.
    pub fn align_front(self) -> Self {
        self.map_ty(|ty| SetPosition::align_front(ty))
    }

    /// Align the position to the front.
    pub fn align_front_with_margin(self, margin: S) -> Self {
        self.map_ty(|ty| SetPosition::align_front_with_margin(ty, margin))
    }

    /// Align the position to the middle.
    pub fn align_middle_z(self) -> Self {
        self.map_ty(|ty| SetPosition::align_middle_z(ty))
    }

    /// Align the position to the back.
    pub fn align_back(self) -> Self {
        self.map_ty(|ty| SetPosition::align_back(ty))
    }

    /// Align the position to the back.
    pub fn align_back_with_margin(self, margin: S) -> Self {
        self.map_ty(|ty| SetPosition::align_back_with_margin(ty, margin))
    }

    /// Align the **Position** of the node with the given node along the *x* axis.
    pub fn x_align_to(self, other: node::Index, align: position::Align<S>) -> Self {
        self.map_ty(|ty| SetPosition::x_align_to(ty, other, align))
    }

    /// Align the **Position** of the node with the given node along the *y* axis.
    pub fn y_align_to(self, other: node::Index, align: position::Align<S>) -> Self {
        self.map_ty(|ty| SetPosition::y_align_to(ty, other, align))
    }

    /// Align the **Position** of the node with the given node along the *z* axis.
    pub fn z_align_to(self, other: node::Index, align: position::Align<S>) -> Self {
        self.map_ty(|ty| SetPosition::z_align_to(ty, other, align))
    }

    /// Align the position to the left.
    pub fn align_left_of(self, other: node::Index) -> Self {
        self.map_ty(|ty| SetPosition::align_left_of(ty, other))
    }

    /// Align the position to the left.
    pub fn align_left_of_with_margin(self, other: node::Index, margin: S) -> Self {
        self.map_ty(|ty| SetPosition::align_left_of_with_margin(ty, other, margin))
    }

    /// Align the position to the middle.
    pub fn align_middle_x_of(self, other: node::Index) -> Self {
        self.map_ty(|ty| SetPosition::align_middle_x_of(ty, other))
    }

    /// Align the position to the right.
    pub fn align_right_of(self, other: node::Index) -> Self {
        self.map_ty(|ty| SetPosition::align_right_of(ty, other))
    }

    /// Align the position to the right.
    pub fn align_right_of_with_margin(self, other: node::Index, margin: S) -> Self {
        self.map_ty(|ty| SetPosition::align_right_of_with_margin(ty, other, margin))
    }

    /// Align the position to the bottom.
    pub fn align_bottom_of(self, other: node::Index) -> Self {
        self.map_ty(|ty| SetPosition::align_bottom_of(ty, other))
    }

    /// Align the position to the bottom.
    pub fn align_bottom_of_with_margin(self, other: node::Index, margin: S) -> Self {
        self.map_ty(|ty| SetPosition::align_bottom_of_with_margin(ty, other, margin))
    }

    /// Align the position to the middle.
    pub fn align_middle_y_of(self, other: node::Index) -> Self {
        self.map_ty(|ty| SetPosition::align_middle_y_of(ty, other))
    }

    /// Align the position to the top.
    pub fn align_top_of(self, other: node::Index) -> Self {
        self.map_ty(|ty| SetPosition::align_top_of(ty, other))
    }

    /// Align the position to the top.
    pub fn align_top_of_with_margin(self, other: node::Index, margin: S) -> Self {
        self.map_ty(|ty| SetPosition::align_top_of_with_margin(ty, other, margin))
    }

    /// Align the position to the front.
    pub fn align_front_of(self, other: node::Index) -> Self {
        self.map_ty(|ty| SetPosition::align_front_of(ty, other))
    }

    /// Align the position to the front.
    pub fn align_front_of_with_margin(self, other: node::Index, margin: S) -> Self {
        self.map_ty(|ty| SetPosition::align_front_of_with_margin(ty, other, margin))
    }

    /// Align the position to the middle.
    pub fn align_middle_z_of(self, other: node::Index) -> Self {
        self.map_ty(|ty| SetPosition::align_middle_z_of(ty, other))
    }

    /// Align the position to the back.
    pub fn align_back_of(self, other: node::Index) -> Self {
        self.map_ty(|ty| SetPosition::align_back_of(ty, other))
    }

    /// Align the position to the back.
    pub fn align_back_of_with_margin(self, other: node::Index, margin: S) -> Self {
        self.map_ty(|ty| SetPosition::align_back_of_with_margin(ty, other, margin))
    }

    // Alignment combinations.

    /// Align the node to the middle of the last node.
    pub fn middle(self) -> Self {
        self.map_ty(|ty| SetPosition::middle(ty))
    }

    /// Align the node to the bottom left of the last node.
    pub fn bottom_left(self) -> Self {
        self.map_ty(|ty| SetPosition::bottom_left(ty))
    }

    /// Align the node to the middle left of the last node.
    pub fn mid_left(self) -> Self {
        self.map_ty(|ty| SetPosition::mid_left(ty))
    }

    /// Align the node to the top left of the last node.
    pub fn top_left(self) -> Self {
        self.map_ty(|ty| SetPosition::top_left(ty))
    }

    /// Align the node to the middle top of the last node.
    pub fn mid_top(self) -> Self {
        self.map_ty(|ty| SetPosition::mid_top(ty))
    }

    /// Align the node to the top right of the last node.
    pub fn top_right(self) -> Self {
        self.map_ty(|ty| SetPosition::top_right(ty))
    }

    /// Align the node to the middle right of the last node.
    pub fn mid_right(self) -> Self {
        self.map_ty(|ty| SetPosition::mid_right(ty))
    }

    /// Align the node to the bottom right of the last node.
    pub fn bottom_right(self) -> Self {
        self.map_ty(|ty| SetPosition::bottom_right(ty))
    }

    /// Align the node to the middle bottom of the last node.
    pub fn mid_bottom(self) -> Self {
        self.map_ty(|ty| SetPosition::mid_bottom(ty))
    }

    /// Align the node in the middle of the given Node.
    pub fn middle_of(self, other: node::Index) -> Self {
        self.map_ty(|ty| SetPosition::middle_of(ty, other))
    }

    /// Align the node to the bottom left of the given Node.
    pub fn bottom_left_of(self, other: node::Index) -> Self {
        self.map_ty(|ty| SetPosition::bottom_left_of(ty, other))
    }

    /// Align the node to the middle left of the given Node.
    pub fn mid_left_of(self, other: node::Index) -> Self {
        self.map_ty(|ty| SetPosition::mid_left_of(ty, other))
    }

    /// Align the node to the top left of the given Node.
    pub fn top_left_of(self, other: node::Index) -> Self {
        self.map_ty(|ty| SetPosition::top_left_of(ty, other))
    }

    /// Align the node to the middle top of the given Node.
    pub fn mid_top_of(self, other: node::Index) -> Self {
        self.map_ty(|ty| SetPosition::mid_top_of(ty, other))
    }

    /// Align the node to the top right of the given Node.
    pub fn top_right_of(self, other: node::Index) -> Self {
        self.map_ty(|ty| SetPosition::top_right_of(ty, other))
    }

    /// Align the node to the middle right of the given Node.
    pub fn mid_right_of(self, other: node::Index) -> Self {
        self.map_ty(|ty| SetPosition::mid_right_of(ty, other))
    }

    /// Align the node to the bottom right of the given Node.
    pub fn bottom_right_of(self, other: node::Index) -> Self {
        self.map_ty(|ty| SetPosition::bottom_right_of(ty, other))
    }

    /// Align the node to the middle bottom of the given Node.
    pub fn mid_bottom_of(self, other: node::Index) -> Self {
        self.map_ty(|ty| SetPosition::mid_bottom_of(ty, other))
    }
}
