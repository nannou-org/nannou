//! "Diff"ing iterators for caching elements to sequential collections without requiring the new
//! elements' iterator to be `Clone`.
//!
//! - [**IterDiff**](./enum.IterDiff.html) (produced by the [**iter_diff**](./fn.iter_diff.html)
//! function) describes the difference between two non-`Clone` iterators `a` and `b` after breaking
//! ASAP from a comparison with enough data to update `a`'s collection.
//! - [**copy_on_diff**](./fn.copy_on_diff.html) is an application of [**iter_diff**] that compares
//! two iterators `a` and `b`, borrowing the source of `a` if they are the same or creating a new
//! owned collection with `b`'s elements if they are different.

use std::borrow::{Cow, ToOwned};
use std::iter;

/// A type returned by the [`iter_diff`](./fn.iter_diff.html) function.
///
/// `IterDiff` represents the way in which the elements (of type `E`) yielded by the iterator `I`
/// differ to some other iterator yielding borrowed elements of the same type.
///
/// `I` is some `Iterator` yielding elements of type `E`.
pub enum IterDiff<E, I> {
    /// The index of the first non-matching element along with the iterator's remaining elements
    /// starting with the first mis-matched element.
    FirstMismatch(usize, iter::Chain<iter::Once<E>, I>),
    /// The remaining elements of the iterator.
    Longer(iter::Chain<iter::Once<E>, I>),
    /// The total number of elements that were in the iterator.
    Shorter(usize),
}

/// Compares every element yielded by both elems and new_elems in lock-step and returns an
/// `IterDiff` which describes how `b` differs from `a`.
///
/// This function is useful for caching some iterator `b` in some sequential collection without
/// requiring `b` to be `Clone` in order to compare it to the collection before determining if the
/// collection needs to be updated. The returned function returns as soon as a difference is found,
/// producing an `IterDiff` that provides the data necessary to update the collection without ever
/// requiring `B` to be `Clone`. This allows for efficiently caching iterators like `Map` or
/// `Filter` that do not implement `Clone`.
///
/// If the number of elements yielded by `b` is less than the number of elements yielded by `a`,
/// the number of `b` elements yielded will be returned as `IterDiff::Shorter`.
///
/// If the two elements of a step differ, the index of those elements along with the remaining
/// elements of `b` are returned as `IterDiff::FirstMismatch`.
///
/// If `a` becomes exhausted before `b` becomes exhausted, the remaining `b` elements will be
/// returned as `IterDiff::Longer`.
///
/// See [`copy_on_diff`](./fn.copy_on_diff.html) for an application of `iter_diff`.
pub fn iter_diff<'a, A, B>(a: A, b: B) -> Option<IterDiff<B::Item, B::IntoIter>>
    where A: IntoIterator<Item=&'a B::Item>,
          B: IntoIterator,
          B::Item: PartialEq + 'a,
{
    let mut b = b.into_iter();
    for (i, a_elem) in a.into_iter().enumerate() {
        match b.next() {
            None => return Some(IterDiff::Shorter(i)),
            Some(b_elem) => if *a_elem != b_elem {
                return Some(IterDiff::FirstMismatch(i, iter::once(b_elem).chain(b)));
            },
        }
    }
    b.next().map(|elem| IterDiff::Longer(iter::once(elem).chain(b)))
}

/// Returns `Cow::Borrowed` `a` if `a` contains the same elements as yielded by `b`'s iterator.
///
/// Collects into a new `A::Owned` and returns `Cow::Owned` if either the number of elements or the
/// elements themselves differ.
/// ```
#[allow(dead_code)]
pub fn copy_on_diff<'a, A, B, T: 'a>(a: &'a A, b: B) -> Cow<'a, A>
    where &'a A: IntoIterator<Item=&'a T>,
          <&'a A as IntoIterator>::IntoIter: Clone,
          A: ToOwned,
          <A as ToOwned>::Owned: iter::FromIterator<T>,
          B: IntoIterator<Item=T>,
          T: Clone + PartialEq,
{
    let a_iter = a.into_iter();
    match iter_diff(a_iter.clone(), b.into_iter()) {
        Some(IterDiff::FirstMismatch(i, mismatch)) =>
            Cow::Owned(a_iter.take(i).cloned().chain(mismatch).collect()),
        Some(IterDiff::Longer(remaining)) =>
            Cow::Owned(a_iter.cloned().chain(remaining).collect()),
        Some(IterDiff::Shorter(num_new_elems)) =>
            Cow::Owned(a_iter.cloned().take(num_new_elems).collect()),
        None => Cow::Borrowed(a),
    }
}
