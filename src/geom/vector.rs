//! Implementation of the **Vector** types.
//!
//! **Note:** Much of the code in this module is inspired by or copied directly from the `cgmath`
//! crate. Originally we used the `cgmath` types directly, however we decided to switch to our own
//! implementations in order to gain some flexibility.

use crate::geom::scalar;
use crate::math::{self, BaseFloat, Bounded, InnerSpace, NumCast, One, Zero};
use crate::rand::distributions::{Distribution, Standard};
use crate::rand::Rng;
use crate::serde_derive::{Deserialize, Serialize};
use std::{iter, ops};

/// A 2-dimensional vector.
#[repr(C)]
#[derive(Default, Debug, PartialEq, Eq, Copy, Clone, Hash, Serialize, Deserialize)]
pub struct Vector2<S = scalar::Default> {
    pub x: S,
    pub y: S,
}

/// A 3-dimensional vector.
#[repr(C)]
#[derive(Default, Debug, PartialEq, Eq, Copy, Clone, Hash, Serialize, Deserialize)]
pub struct Vector3<S = scalar::Default> {
    pub x: S,
    pub y: S,
    pub z: S,
}

/// A 4-dimensional vector.
#[repr(C)]
#[derive(Default, Debug, PartialEq, Eq, Copy, Clone, Hash, Serialize, Deserialize)]
pub struct Vector4<S = scalar::Default> {
    pub x: S,
    pub y: S,
    pub z: S,
    pub w: S,
}

// Generates index operators for a compound type
//
// Code originally from `cgmath` macros module.
macro_rules! impl_index_operators {
    ($VectorN:ident < $S:ident > , $n:expr, $Output:ty, $I:ty) => {
        impl<$S> ::std::ops::Index<$I> for $VectorN<$S> {
            type Output = $Output;

            #[inline]
            fn index<'a>(&'a self, i: $I) -> &'a $Output {
                let v: &[$S; $n] = self.as_ref();
                &v[i]
            }
        }

        impl<$S> ::std::ops::IndexMut<$I> for $VectorN<$S> {
            #[inline]
            fn index_mut<'a>(&'a mut self, i: $I) -> &'a mut $Output {
                let v: &mut [$S; $n] = self.as_mut();
                &mut v[i]
            }
        }
    };
}

// Utility macro for generating associated functions for the vectors
macro_rules! impl_vector {
    ($VectorN:ident { $($field:ident),+ }, $n:expr, $constructor:ident) => {
        impl<S> $VectorN<S> {
            /// Construct a new vector, using the provided values.
            #[inline]
            pub fn new($($field: S),+) -> $VectorN<S> {
                $VectorN { $($field: $field),+ }
            }

            /// Construct a vector using the given value for each field.
            #[inline]
            pub fn from_value(scalar: S) -> $VectorN<S>
            where
                S: Clone,
            {
                $VectorN { $($field: scalar.clone()),+ }
            }

            /// The number of dimensions in the vector.
            #[inline]
            pub fn len(&self) -> usize {
                $n
            }

            /// Perform the given operation on each field in the vector, returning a new vector
            /// constructed from the operations.
            #[inline]
            pub fn map<U, F>(self, mut f: F) -> $VectorN<U>
            where
                F: FnMut(S) -> U,
            {
                $VectorN { $($field: f(self.$field)),+ }
            }

            /// Perform the given operation on each each field on both vectors, returning a new
            /// vector constructed from the operations.
            #[inline]
            pub fn zip_map<T, U, F>(self, other: $VectorN<T>, mut f: F) -> $VectorN<U>
            where
                F: FnMut(S, T) -> U,
            {
                $VectorN { $($field: f(self.$field, other.$field)),+ }
            }

            /// Test whether or not the vector is infinite.
            pub fn is_finite(&self) -> bool
            where
                S: BaseFloat,
            {
                $(self.$field.is_finite())&&+
            }

            /// Component-wise casting to another type.
            #[inline]
            pub fn cast<T>(&self) -> Option<$VectorN<T>>
            where
                S: NumCast + Clone,
                T: NumCast,
            {
                $(
                    let $field = match NumCast::from(self.$field.clone()) {
                        Some(field) => field,
                        None => return None
                    };
                )+
                Some($VectorN { $($field),+ })
            }

            /// A zeroed vector.
            #[inline]
            pub fn zero() -> $VectorN<S>
            where
                S: Zero,
            {
                $VectorN { $($field: S::zero()),+ }
            }

            /// Whether or not the vector is zeroed.
            #[inline]
            pub fn is_zero(&self) -> bool
            where
                S: PartialEq + Zero,
            {
                *self == $VectorN::zero()
            }

            /// A vector with `1` for each element.
            #[inline]
            pub fn one() -> $VectorN<S>
            where
                S: One,
            {
                $VectorN { $($field: S::one()),+ }
            }

            /// Whether or not each element in the vector is equal to `1`.
            #[inline]
            pub fn is_one(&self) -> bool
            where
                S: PartialEq + One,
            {
                *self == $VectorN::one()
            }

            /// Tests whether or not any of the vector's elements is `NaN`.
            #[inline]
            pub fn is_nan(&self) -> bool
            where
                S: BaseFloat,
            {
                $(self.$field.is_nan())||+
            }

            /// Sum the fields of the vector.
            #[inline]
            pub fn sum(self) -> S
            where
                S: ops::Add<Output = S> + Copy,
            {
                math::Array::sum(self)
            }

            /// The product of the fields of the vector.
            #[inline]
            pub fn product(self) -> S
            where
                S: ops::Mul<Output = S> + Copy,
            {
                math::Array::product(self)
            }

            /// Return a vector whose magnitude is limited to the given value.
            #[inline]
            pub fn limit_magnitude(self, limit: S) -> Self
            where
                S: BaseFloat,
            {
                limit_magnitude(self, limit)
            }

            /// Return a vector with the given magnitude.
            #[inline]
            pub fn with_magnitude(self, magnitude: S) -> Self
            where
                S: BaseFloat,
            {
                self.normalize() * magnitude
            }

            /// Return a normalized vector.
            ///
            /// If `self` `is_zero`, this returns `self`.
            pub fn normalize(self) -> Self
            where
                S: BaseFloat,
            {
                if self.is_zero() {
                    self
                } else {
                    InnerSpace::normalize(self)
                }
            }

            /// The dot product of self and the given vector.
            #[inline]
            pub fn dot(self, other: $VectorN<S>) -> S
            where
                S: BaseFloat,
            {
                InnerSpace::dot(self, other)
            }
        }

        impl<S> iter::Sum<$VectorN<S>> for $VectorN<S>
        where
            S: Zero + ops::Add<Output = S>,
        {
            #[inline]
            fn sum<I>(iter: I) -> $VectorN<S>
            where
                I: Iterator<Item = $VectorN<S>>,
            {
                iter.fold($VectorN::zero(), ops::Add::add)
            }
        }

        impl<'a, S: 'a> iter::Sum<&'a $VectorN<S>> for $VectorN<S>
        where
            S: 'a + Clone + Zero + ops::Add<Output = S>,
        {
            #[inline]
            fn sum<I>(iter: I) -> $VectorN<S>
            where
                I: Iterator<Item=&'a $VectorN<S>>,
            {
                iter.fold($VectorN::zero(), |acc, s| acc + s.clone())// ops::Add::add)
            }
        }

        // std::ops - vector vector

        impl<S> ops::Neg for $VectorN<S>
        where
            S: ops::Neg<Output = S>,
        {
            type Output = $VectorN<S>;

            #[inline]
            fn neg(self) -> $VectorN<S> {
                self.map(|s| -s)
            }
        }

        impl<S> ops::Add for $VectorN<S>
        where
            S: ops::Add<Output = S>,
        {
            type Output = $VectorN<S>;

            #[inline]
            fn add(self, other: Self) -> Self {
                self.zip_map(other, |a, b| a + b)
            }
        }

        impl<S> ops::Sub for $VectorN<S>
        where
            S: ops::Sub<Output = S>,
        {
            type Output = $VectorN<S>;

            #[inline]
            fn sub(self, other: Self) -> Self {
                self.zip_map(other, |a, b| a - b)
            }
        }

        impl<S> ops::Mul for $VectorN<S>
        where
            S: ops::Mul<Output = S>,
        {
            type Output = $VectorN<S>;

            #[inline]
            fn mul(self, other: Self) -> Self {
                self.zip_map(other, |a, b| a * b)
            }
        }

        impl<S> ops::Div for $VectorN<S>
        where
            S: ops::Div<Output = S>,
        {
            type Output = $VectorN<S>;

            #[inline]
            fn div(self, other: Self) -> Self {
                self.zip_map(other, |a, b| a / b)
            }
        }

        impl<S> ops::Rem for $VectorN<S>
        where
            S: ops::Rem<Output = S>,
        {
            type Output = $VectorN<S>;

            #[inline]
            fn rem(self, other: Self) -> Self {
                self.zip_map(other, |a, b| a % b)
            }
        }

        impl<S> ops::AddAssign for $VectorN<S>
        where
            S: ops::AddAssign,
        {
            fn add_assign(&mut self, other: Self) {
                $(self.$field += other.$field;)+
            }
        }

        impl<S> ops::SubAssign for $VectorN<S>
        where
            S: ops::SubAssign,
        {
            fn sub_assign(&mut self, other: Self) {
                $(self.$field -= other.$field;)+
            }
        }

        impl<S> ops::DivAssign for $VectorN<S>
        where
            S: Copy + ops::DivAssign,
        {
            #[inline]
            fn div_assign(&mut self, other: Self) {
                $(self.$field /= other.$field;)+
            }
        }

        impl<S> ops::MulAssign for $VectorN<S>
        where
            S: Copy + ops::MulAssign,
        {
            #[inline]
            fn mul_assign(&mut self, other: Self) {
                $(self.$field *= other.$field;)+
            }
        }

        impl<S> ops::RemAssign for $VectorN<S>
        where
            S: Copy + ops::RemAssign,
        {
            #[inline]
            fn rem_assign(&mut self, other: Self) {
                $(self.$field %= other.$field;)+
            }
        }

        // std::ops - vector scalar

        impl<S> ops::Rem<S> for $VectorN<S>
        where
            S: Copy + ops::Rem<Output = S>,
        {
            type Output = $VectorN<S>;

            #[inline]
            fn rem(self, scalar: S) -> Self {
                self.map(|s| s % scalar)
            }
        }

        impl<S> ops::Div<S> for $VectorN<S>
        where
            S: Copy + ops::Div<Output = S>,
        {
            type Output = $VectorN<S>;

            #[inline]
            fn div(self, scalar: S) -> Self {
                self.map(|s| s / scalar)
            }
        }

        impl<S> ops::Mul<S> for $VectorN<S>
        where
            S: Copy + ops::Mul<Output = S>,
        {
            type Output = $VectorN<S>;

            #[inline]
            fn mul(self, scalar: S) -> Self {
                self.map(|s| s * scalar)
            }
        }

        impl<S> ops::RemAssign<S> for $VectorN<S>
        where
            S: Copy + ops::RemAssign,
        {
            #[inline]
            fn rem_assign(&mut self, scalar: S) {
                $(self.$field %= scalar;)+
            }
        }

        impl<S> ops::DivAssign<S> for $VectorN<S>
        where
            S: Copy + ops::DivAssign,
        {
            #[inline]
            fn div_assign(&mut self, scalar: S) {
                $(self.$field /= scalar;)+
            }
        }

        impl<S> ops::MulAssign<S> for $VectorN<S>
        where
            S: Copy + ops::MulAssign,
        {
            #[inline]
            fn mul_assign(&mut self, scalar: S) {
                $(self.$field *= scalar;)+
            }
        }

        // indexing

        impl_index_operators!($VectorN<S>, $n, S, usize);
        impl_index_operators!($VectorN<S>, $n, [S], ops::Range<usize>);
        impl_index_operators!($VectorN<S>, $n, [S], ops::RangeTo<usize>);
        impl_index_operators!($VectorN<S>, $n, [S], ops::RangeFrom<usize>);
        impl_index_operators!($VectorN<S>, $n, [S], ops::RangeFull);

        // conversions

        impl<S> From<[S; $n]> for $VectorN<S>
        where
            S: Copy,
        {
            #[inline]
            fn from(v: [S; $n]) -> Self {
                let [$($field),+] = v;
                $VectorN { $($field),+ }
            }
        }

        impl<S> Into<[S; $n]> for $VectorN<S> {
            #[inline]
            fn into(self) -> [S; $n] {
                let $VectorN { $($field),+ } = self;
                [$($field),+]
            }
        }

        impl<S> AsRef<[S; $n]> for $VectorN<S> {
            #[inline]
            fn as_ref(&self) -> &[S; $n] {
                unsafe {
                    let ptr = self as *const _ as *const [S; $n];
                    &*ptr
                }
            }
        }

        impl<S> AsMut<[S; $n]> for $VectorN<S> {
            #[inline]
            fn as_mut(&mut self) -> &mut [S; $n] {
                unsafe {
                    let ptr = self as *mut _ as *mut [S; $n];
                    &mut*ptr
                }
            }
        }

        impl<S> ops::Deref for $VectorN<S> {
            type Target = [S; $n];
            #[inline]
            fn deref(&self) -> &Self::Target {
                self.as_ref()
            }
        }

        impl<S> ops::DerefMut for $VectorN<S> {
            #[inline]
            fn deref_mut(&mut self) -> &mut Self::Target {
                self.as_mut()
            }
        }

        // num-traits

        impl<S> Bounded for $VectorN<S>
        where
            S: Bounded,
        {
            #[inline]
            fn min_value() -> $VectorN<S> {
                $VectorN { $($field: S::min_value()),+ }
            }

            #[inline]
            fn max_value() -> $VectorN<S> {
                $VectorN { $($field: S::max_value()),+ }
            }
        }

        impl<S> Zero for $VectorN<S>
        where
            S: PartialEq + Zero,
        {
            #[inline]
            fn zero() -> $VectorN<S> {
                $VectorN { $($field: S::zero()),* }
            }

            #[inline]
            fn is_zero(&self) -> bool {
                *self == $VectorN::zero()
            }
        }

        // `rand` crate implementations

        impl<S> Distribution<$VectorN<S>> for Standard
        where
            Standard: Distribution<S>,
        {
            fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> $VectorN<S> {
                $VectorN { $($field: rng.gen()),+ }
            }
        }

        /// The short constructor.
        #[inline]
        pub fn $constructor<S>($($field: S),+) -> $VectorN<S> {
            $VectorN::new($($field),+)
        }
    };
}

mod cgmath_impl {
    // From `cgmath`
    macro_rules! fold_array {
        (& $method:ident, { $x:expr }) => {
            *$x
        };
        (& $method:ident, { $x:expr, $y:expr }) => {
            $x.$method(&$y)
        };
        (& $method:ident, { $x:expr, $y:expr, $z:expr }) => {
            $x.$method(&$y).$method(&$z)
        };
        (& $method:ident, { $x:expr, $y:expr, $z:expr, $w:expr }) => {
            $x.$method(&$y).$method(&$z).$method(&$w)
        };
        ($method:ident, { $x:expr }) => {
            $x
        };
        ($method:ident, { $x:expr, $y:expr }) => {
            $x.$method($y)
        };
        ($method:ident, { $x:expr, $y:expr, $z:expr }) => {
            $x.$method($y).$method($z)
        };
        ($method:ident, { $x:expr, $y:expr, $z:expr, $w:expr }) => {
            $x.$method($y).$method($z).$method($w)
        };
    }

    use super::{Vector2, Vector3, Vector4};
    use crate::math::approx::ApproxEq;
    use crate::math::cgmath::{
        self, Angle, Array, BaseFloat, BaseNum, ElementWise, EuclideanSpace, InnerSpace,
        MetricSpace, Rad, VectorSpace,
    };
    use std::ops;

    macro_rules! impl_vector_cgmath {
        ($VectorN:ident { $($field:ident),+ }, $n:expr) => {
            impl<S> From<cgmath::$VectorN<S>> for $VectorN<S> {
                #[inline]
                fn from(v: cgmath::$VectorN<S>) -> Self {
                    let cgmath::$VectorN { $($field),+ } = v;
                    $VectorN { $($field),+ }
                }
            }

            impl<S> Into<cgmath::$VectorN<S>> for $VectorN<S> {
                #[inline]
                fn into(self) -> cgmath::$VectorN<S> {
                    let $VectorN { $($field),+ } = self;
                    cgmath::$VectorN { $($field),+ }
                }
            }

            impl<S> VectorSpace for $VectorN<S>
            where
                S: BaseNum,
            {
                type Scalar = S;
            }

            impl<S> MetricSpace for $VectorN<S>
            where
                S: BaseFloat,
            {
                type Metric = S;

                #[inline]
                fn distance2(self, other: Self) -> S {
                    (other - self).magnitude2()
                }
            }

            impl<S> ApproxEq for $VectorN<S>
            where
                S: ApproxEq,
                S::Epsilon: Copy,
            {
                type Epsilon = S::Epsilon;

                #[inline]
                fn default_epsilon() -> S::Epsilon {
                    S::default_epsilon()
                }

                #[inline]
                fn default_max_relative() -> S::Epsilon {
                    S::default_max_relative()
                }

                #[inline]
                fn default_max_ulps() -> u32 {
                    S::default_max_ulps()
                }

                #[inline]
                fn relative_eq(
                    &self,
                    other: &Self,
                    epsilon: Self::Epsilon,
                    max_relative: Self::Epsilon,
                ) -> bool {
                    $(self.$field.relative_eq(&other.$field, epsilon, max_relative))&&+
                }

                #[inline]
                fn ulps_eq(&self, other: &Self, epsilon: Self::Epsilon, max_ulps: u32) -> bool {
                    $(self.$field.ulps_eq(&other.$field, epsilon, max_ulps))&&+
                }
            }

            impl<S> ElementWise<S> for $VectorN<S>
            where
                S: BaseNum,
            {
                #[inline]
                fn add_element_wise(self, rhs: S) -> $VectorN<S> {
                    $VectorN::new($(self.$field + rhs),+)
                }
                #[inline]
                fn sub_element_wise(self, rhs: S) -> $VectorN<S> {
                    $VectorN::new($(self.$field - rhs),+)
                }
                #[inline]
                fn mul_element_wise(self, rhs: S) -> $VectorN<S> {
                    $VectorN::new($(self.$field * rhs),+)
                }
                #[inline]
                fn div_element_wise(self, rhs: S) -> $VectorN<S> {
                    $VectorN::new($(self.$field / rhs),+)
                }
                #[inline]
                fn rem_element_wise(self, rhs: S) -> $VectorN<S> {
                    $VectorN::new($(self.$field % rhs),+)
                }

                #[inline]
                fn add_assign_element_wise(&mut self, rhs: S) {
                    $(self.$field += rhs);+
                }
                #[inline]
                fn sub_assign_element_wise(&mut self, rhs: S) {
                    $(self.$field -= rhs);+
                }
                #[inline]
                fn mul_assign_element_wise(&mut self, rhs: S) {
                    $(self.$field *= rhs);+
                }
                #[inline]
                fn div_assign_element_wise(&mut self, rhs: S) {
                    $(self.$field /= rhs);+
                }
                #[inline]
                fn rem_assign_element_wise(&mut self, rhs: S) {
                    $(self.$field %= rhs);+
                }
            }

            impl<S> ElementWise for $VectorN<S>
            where
                S: BaseFloat,
            {
                #[inline]
                fn add_element_wise(self, rhs: $VectorN<S>) -> $VectorN<S> {
                    $VectorN::new($(self.$field + rhs.$field),+)
                }
                #[inline]
                fn sub_element_wise(self, rhs: $VectorN<S>) -> $VectorN<S> {
                    $VectorN::new($(self.$field - rhs.$field),+)
                }
                #[inline]
                fn mul_element_wise(self, rhs: $VectorN<S>) -> $VectorN<S> {
                    $VectorN::new($(self.$field * rhs.$field),+)
                }
                #[inline]
                fn div_element_wise(self, rhs: $VectorN<S>) -> $VectorN<S> {
                    $VectorN::new($(self.$field / rhs.$field),+)
                }
                #[inline]
                fn rem_element_wise(self, rhs: $VectorN<S>) -> $VectorN<S> {
                    $VectorN::new($(self.$field % rhs.$field),+)
                }

                #[inline]
                fn add_assign_element_wise(&mut self, rhs: $VectorN<S>) {
                    $(self.$field += rhs.$field);+
                }
                #[inline]
                fn sub_assign_element_wise(&mut self, rhs: $VectorN<S>) {
                    $(self.$field -= rhs.$field);+
                }
                #[inline]
                fn mul_assign_element_wise(&mut self, rhs: $VectorN<S>) {
                    $(self.$field *= rhs.$field);+
                }
                #[inline]
                fn div_assign_element_wise(&mut self, rhs: $VectorN<S>) {
                    $(self.$field /= rhs.$field);+
                }
                #[inline]
                fn rem_assign_element_wise(&mut self, rhs: $VectorN<S>) {
                    $(self.$field %= rhs.$field);+
                }
            }

            impl<S> Array for $VectorN<S>
            where
                S: Copy,
            {
                type Element = S;

                #[inline]
                fn len() -> usize {
                    $n
                }

                #[inline]
                fn from_value(scalar: S) -> $VectorN<S> {
                    $VectorN { $($field: scalar),+ }
                }

                #[inline]
                fn sum(self) -> S
                where
                    S: ops::Add<Output = S>,
                {
                    fold_array!(add, { $(self.$field),+ })
                }

                #[inline]
                fn product(self) -> S
                where
                    S: ops::Mul<Output = S>,
                {
                    fold_array!(mul, { $(self.$field),+ })
                }
            }

            impl<S> EuclideanSpace for $VectorN<S>
            where
                S: BaseNum,
            {
                type Scalar = S;
                type Diff = $VectorN<S>;

                #[inline]
                fn origin() -> Self {
                    $VectorN { $($field: S::zero()),+ }
                }

                #[inline]
                fn from_vec(v: $VectorN<S>) -> Self {
                    $VectorN::new($(v.$field),+)
                }

                #[inline]
                fn to_vec(self) -> $VectorN<S> {
                    $VectorN::new($(self.$field),+)
                }

                #[inline]
                fn dot(self, other: $VectorN<S>) -> S {
                    $VectorN::new($(self.$field * other.$field),+).sum()
                }
            }
        }
    }

    // A macro to simplify the implementation of the point conversion traits.
    macro_rules! impl_point_conversions {
        ($VectorN:ident { $($field:ident),+ }, $PointN:ident) => {
            impl<S> From<cgmath::$PointN<S>> for $VectorN<S> {
                #[inline]
                fn from(v: cgmath::$PointN<S>) -> Self {
                    let cgmath::$PointN { $($field),+ } = v;
                    $VectorN { $($field),+ }
                }
            }

            impl<S> Into<cgmath::$PointN<S>> for $VectorN<S> {
                #[inline]
                fn into(self) -> cgmath::$PointN<S> {
                    let $VectorN { $($field),+ } = self;
                    cgmath::$PointN { $($field),+ }
                }
            }
        };
    }

    impl_vector_cgmath!(Vector2 { x, y }, 2);
    impl_vector_cgmath!(Vector3 { x, y, z }, 3);
    impl_vector_cgmath!(Vector4 { x, y, z, w }, 4);

    impl_point_conversions!(Vector2 { x, y }, Point2);
    impl_point_conversions!(Vector3 { x, y, z }, Point3);

    impl<S> InnerSpace for Vector2<S>
    where
        S: BaseFloat,
    {
        #[inline]
        fn dot(self, other: Vector2<S>) -> S {
            Vector2::mul_element_wise(self, other).sum()
        }

        #[inline]
        fn angle(self, other: Vector2<S>) -> Rad<S> {
            Rad::atan2(Self::perp_dot(self, other), Self::dot(self, other))
        }
    }

    impl<S> InnerSpace for Vector3<S>
    where
        S: BaseFloat,
    {
        #[inline]
        fn dot(self, other: Vector3<S>) -> S {
            Vector3::mul_element_wise(self, other).sum()
        }

        #[inline]
        fn angle(self, other: Vector3<S>) -> Rad<S> {
            Rad::atan2(self.cross(other).magnitude(), Self::dot(self, other))
        }
    }

    impl<S> InnerSpace for Vector4<S>
    where
        S: BaseFloat,
    {
        #[inline]
        fn dot(self, other: Vector4<S>) -> S {
            Vector4::mul_element_wise(self, other).sum()
        }
    }
}

impl_vector!(Vector2 { x, y }, 2, vec2);
impl_vector!(Vector3 { x, y, z }, 3, vec3);
impl_vector!(Vector4 { x, y, z, w }, 4, vec4);

// tuple conversions

impl<S> From<(S, S)> for Vector2<S> {
    fn from((x, y): (S, S)) -> Self {
        Vector2 { x, y }
    }
}

impl<S> From<(S, S, S)> for Vector3<S> {
    fn from((x, y, z): (S, S, S)) -> Self {
        Vector3 { x, y, z }
    }
}

impl<S> From<(S, S, S, S)> for Vector4<S> {
    fn from((x, y, z, w): (S, S, S, S)) -> Self {
        Vector4 { x, y, z, w }
    }
}

impl<S> Into<(S, S)> for Vector2<S> {
    fn into(self) -> (S, S) {
        let Vector2 { x, y } = self;
        (x, y)
    }
}

impl<S> Into<(S, S, S)> for Vector3<S> {
    fn into(self) -> (S, S, S) {
        let Vector3 { x, y, z } = self;
        (x, y, z)
    }
}

impl<S> Into<(S, S, S, S)> for Vector4<S> {
    fn into(self) -> (S, S, S, S) {
        let Vector4 { x, y, z, w } = self;
        (x, y, z, w)
    }
}

// expanding tuple conversions

impl<S> From<(S, S)> for Vector3<S>
where
    S: Zero,
{
    fn from((x, y): (S, S)) -> Self {
        let z = S::zero();
        Vector3 { x, y, z }
    }
}

impl<S> From<(S, S)> for Vector4<S>
where
    S: Zero,
{
    fn from((x, y): (S, S)) -> Self {
        let z = S::zero();
        let w = S::zero();
        Vector4 { x, y, z, w }
    }
}

impl<S> From<(S, S, S)> for Vector4<S>
where
    S: Zero,
{
    fn from((x, y, z): (S, S, S)) -> Self {
        let w = S::zero();
        Vector4 { x, y, z, w }
    }
}

// expanding fixed-size array conversions

impl<S> From<[S; 2]> for Vector3<S>
where
    S: Zero,
{
    fn from([x, y]: [S; 2]) -> Self {
        let z = S::zero();
        Vector3 { x, y, z }
    }
}

impl<S> From<[S; 2]> for Vector4<S>
where
    S: Zero,
{
    fn from([x, y]: [S; 2]) -> Self {
        let z = S::zero();
        let w = S::zero();
        Vector4 { x, y, z, w }
    }
}

impl<S> From<[S; 3]> for Vector4<S>
where
    S: Zero,
{
    fn from([x, y, z]: [S; 3]) -> Self {
        let w = S::zero();
        Vector4 { x, y, z, w }
    }
}

// expanding vector conversions

impl<S> From<Vector2<S>> for Vector3<S>
where
    S: Zero,
{
    fn from(Vector2 { x, y }: Vector2<S>) -> Self {
        let z = S::zero();
        Vector3 { x, y, z }
    }
}

impl<S> From<Vector2<S>> for Vector4<S>
where
    S: Zero,
{
    fn from(Vector2 { x, y }: Vector2<S>) -> Self {
        let z = S::zero();
        let w = S::zero();
        Vector4 { x, y, z, w }
    }
}

impl<S> From<Vector3<S>> for Vector4<S>
where
    S: Zero,
{
    fn from(Vector3 { x, y, z }: Vector3<S>) -> Self {
        let w = S::zero();
        Vector4 { x, y, z, w }
    }
}

// Vector 2

impl<S> Vector2<S> {
    /// A unit vector in the `x` direction.
    #[inline]
    pub fn unit_x() -> Vector2<S>
    where
        S: Zero + One,
    {
        Vector2::new(S::one(), S::zero())
    }

    /// A unit vector in the `y` direction.
    #[inline]
    pub fn unit_y() -> Vector2<S>
    where
        S: Zero + One,
    {
        Vector2::new(S::zero(), S::one())
    }

    /// The perpendicular dot product of the vector and `other`.
    #[inline]
    pub fn perp_dot(self, other: Vector2<S>) -> S
    where
        S: ops::Sub<Output = S> + ops::Mul<Output = S>,
    {
        (self.x * other.y) - (self.y * other.x)
    }

    /// Create a `Vector3`, using the `x` and `y` values from this vector, and the
    /// provided `z`.
    #[inline]
    pub fn extend(self, z: S) -> Vector3<S> {
        Vector3::new(self.x, self.y, z)
    }

    /// Returns the angle of the vector in radians.
    ///
    /// # Examples
    /// ```
    /// # use nannou::prelude::*;
    /// # use nannou::Draw;
    /// # fn main() {
    /// let vector = Vector2::new(-0.5, 0.5);
    /// let theta = vector.angle() * -1.0;
    /// # let draw = Draw::new();
    /// draw.quad()
    /// .rotate(theta);
    /// assert_eq!(theta, -2.356194490192345);
    /// # }
    /// ```
    ///
    pub fn angle(self) -> S
    where
        S: BaseFloat,
    {
        self.y.atan2(self.x)
    }

    //impl_swizzle_functions!(Vector1, Vector2, Vector3, Vector4, S, xy);
}

// Vector 3

impl<S> Vector3<S> {
    /// A unit vector in the `x` direction.
    #[inline]
    pub fn unit_x() -> Vector3<S>
    where
        S: Zero + One,
    {
        Vector3::new(S::one(), S::zero(), S::zero())
    }

    /// A unit vector in the `y` direction.
    #[inline]
    pub fn unit_y() -> Vector3<S>
    where
        S: Zero + One,
    {
        Vector3::new(S::zero(), S::one(), S::zero())
    }

    /// A unit vector in the `z` direction.
    #[inline]
    pub fn unit_z() -> Vector3<S>
    where
        S: Zero + One,
    {
        Vector3::new(S::zero(), S::zero(), S::one())
    }

    /// Returns the cross product of the vector and `other`.
    #[inline]
    pub fn cross(self, other: Vector3<S>) -> Vector3<S>
    where
        S: Copy + ops::Sub<Output = S> + ops::Mul<Output = S>,
    {
        Vector3::new(
            (self.y * other.z) - (self.z * other.y),
            (self.z * other.x) - (self.x * other.z),
            (self.x * other.y) - (self.y * other.x),
        )
    }

    /// Create a `Vector4`, using the `x`, `y` and `z` values from this vector, and the
    /// provided `w`.
    #[inline]
    pub fn extend(self, w: S) -> Vector4<S> {
        Vector4::new(self.x, self.y, self.z, w)
    }

    /// Create a `Vector2`, dropping the `z` value.
    #[inline]
    pub fn truncate(self) -> Vector2<S> {
        Vector2::new(self.x, self.y)
    }

    // impl_swizzle_functions!(Vector1, Vector2, Vector3, Vector4, S, xyz);
}

// Vector 4

impl<S> Vector4<S> {
    /// A unit vector in the `x` direction.
    #[inline]
    pub fn unit_x() -> Vector4<S>
    where
        S: Zero + One,
    {
        Vector4::new(S::one(), S::zero(), S::zero(), S::zero())
    }

    /// A unit vector in the `y` direction.
    #[inline]
    pub fn unit_y() -> Vector4<S>
    where
        S: Zero + One,
    {
        Vector4::new(S::zero(), S::one(), S::zero(), S::zero())
    }

    /// A unit vector in the `z` direction.
    #[inline]
    pub fn unit_z() -> Vector4<S>
    where
        S: Zero + One,
    {
        Vector4::new(S::zero(), S::zero(), S::one(), S::zero())
    }

    /// A unit vector in the `w` direction.
    #[inline]
    pub fn unit_w() -> Vector4<S>
    where
        S: Zero + One,
    {
        Vector4::new(S::zero(), S::zero(), S::zero(), S::one())
    }

    /// Create a `Vector3`, dropping the `w` value.
    #[inline]
    pub fn truncate(self) -> Vector3<S> {
        Vector3::new(self.x, self.y, self.z)
    }

    /// Create a `Vector3`, dropping the nth element.
    #[inline]
    pub fn truncate_n(&self, n: isize) -> Vector3<S>
    where
        S: Copy,
    {
        match n {
            0 => Vector3::new(self.y, self.z, self.w),
            1 => Vector3::new(self.x, self.z, self.w),
            2 => Vector3::new(self.x, self.y, self.w),
            3 => Vector3::new(self.x, self.y, self.z),
            _ => panic!("{:?} is out of range", n),
        }
    }

    //impl_swizzle_functions!(Vector1, Vector2, Vector3, Vector4, S, xyzw);
}

// utility functions

fn limit_magnitude<V>(v: V, limit: V::Scalar) -> V
where
    V: InnerSpace,
    V::Scalar: BaseFloat,
{
    let magnitude2 = v.magnitude2();
    if magnitude2 <= limit * limit {
        v
    } else {
        v.normalize() * limit
    }
}
