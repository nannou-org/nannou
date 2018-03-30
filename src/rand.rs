pub extern crate rand;

pub use self::rand::*;

/// A wrapper function around the `random` function that avoids the need for specifying a type in
/// the case that it cannot be inferred. The primary purpose for this is to simplify the random API
/// for new rust users.
pub fn random_f32() -> f32 {
    random()
}

/// A wrapper function around the `random` function that avoids the need for specifying a type in
/// the case that it cannot be inferred. The primary purpose for this is to simplify the random API
/// for new rust users.
pub fn random_f64() -> f64 {
    random()
}

/// A function for generating a random value within the given range.
///
/// The generated value may be within the range [min, max). That is, the result is inclusive of
/// `min`, but will never be `max`.
///
/// This calls `rand::thread_rng().gen_range(min, max)` internally, in turn using the thread-local
/// default random number generator.
pub fn random_range<T>(min: T, max: T) -> T
where
    T: PartialOrd + distributions::range::SampleRange,
{
    rand::thread_rng().gen_range(min, max)
}
