//! A basic implementation of linear interpolation for use with laser points.

/// Types that can be linearly interpolated.
pub trait Lerp {
    /// The type used to describe the amount of interpolation.
    type Scalar;
    /// Linearly interpolate from `self` to `dest` by the given `amt`.
    fn lerp(&self, dest: &Self, amt: Self::Scalar) -> Self;
}

impl Lerp for f32 {
    type Scalar = Self;
    fn lerp(&self, dest: &Self, amt: Self::Scalar) -> Self {
        *self + ((*dest - *self) * amt)
    }
}

impl Lerp for f64 {
    type Scalar = Self;
    fn lerp(&self, dest: &Self, amt: Self::Scalar) -> Self {
        *self + ((*dest - *self) * amt)
    }
}

// A macro for generating fixed-size array `Lerp` implementations.
macro_rules! impl_lerp_for_array {
    ($($N:expr),*) => {
        $(
            impl<T> Lerp for [T; $N]
            where
                T: Default + Lerp,
                T::Scalar: Clone,
            {
                type Scalar = T::Scalar;
                fn lerp(&self, dest: &Self, amt: Self::Scalar) -> Self {
                    let mut arr: [T; $N] = Default::default();
                    for ((src, dst), out) in self.iter().zip(dest).zip(&mut arr) {
                        *out = src.lerp(dst, amt.clone());
                    }
                    arr
                }
            }
        )*
    };
}

impl_lerp_for_array!(
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32
);
