//! Items related to randomness and random number generators. Also provides some high-level helper
//! functions.
//!
//! Helper functions include [**random_f32()**](./fn.random_f32.html),
//! [**random_f64()**](./fn.random_f64.html) and [**random_range(min,
//! max)**](./fn.random_range.html).

pub use self::rand::*;
pub use rand;

/// A wrapper function around the `random` function that avoids the need for specifying a type in
/// the case that it cannot be inferred. The primary purpose for this is to simplify the random API
/// for new rust users.
///
/// NOTE: This helper function relies on a thread-local RNG and is currently only available with
/// the "std" feature enabled.
#[cfg(feature = "std")]
pub fn random_f32() -> f32 {
    rand::random()
}

/// A wrapper function around the `random` function that avoids the need for specifying a type in
/// the case that it cannot be inferred. The primary purpose for this is to simplify the random API
/// for new rust users.
///
/// NOTE: This helper function relies on a thread-local RNG and is currently only available with
/// the "std" feature enabled.
#[cfg(feature = "std")]
pub fn random_f64() -> f64 {
    rand::random()
}

/// A function for generating a random value within the given range.
///
/// The generated value may be within the range [min, max). That is, the result is inclusive of
/// `min`, but will never be `max`.
///
/// If the given `min` is greater than the given `max`, they will be swapped before calling
/// `gen_range` internally to avoid triggering a `panic!`.
///
/// This calls `rand::thread_rng().gen_range(min..max)` internally, in turn using the thread-local
/// default random number generator.
///
/// NOTE: This helper function relies on a thread-local RNG and is currently only available with
/// the "std" feature enabled.
#[cfg(feature = "std")]
pub fn random_range<T>(min: T, max: T) -> T
where
    T: PartialOrd + distributions::uniform::SampleUniform,
{
    let (min, max) = if min <= max { (min, max) } else { (max, min) };
    rand::thread_rng().gen_range(min..max)
}

/// Generates and returns a random ascii character.
///
/// The ascii characters that can be generated are:
///
///  ABCDEFGHIJKLMNOPQRSTUVWXYZ\
/// abcdefghijklmnopqrstuvwxyz\
/// 0123456789)(*&^%$#@!~.
///
/// NOTE: This helper function relies on a thread-local RNG and is currently only available with
/// the "std" feature enabled.
#[cfg(feature = "std")]
pub fn random_ascii() -> char {
    const ASCIISET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789)(*&^%$#@!~. ";

    let idx = rand::thread_rng().gen_range(0..ASCIISET.len());
    ASCIISET[idx] as char
}
