//! A position (finance) definition that has some good algebraic properties.

#![deny(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

use num_traits::{Num, Signed};

pub use naive_position::{IntoNaivePosition, NaivePosition, Reversed, ToNaivePosition};

#[cfg(test)]
mod tests;

/// Naive position without price representation.
pub mod naive_position;

/// Asset.
#[cfg(feature = "alloc")]
pub mod asset;

/// Position Tree.
#[cfg(feature = "std")]
pub mod tree;

/// Price representation.
pub mod representation;

/// Position with price representation.
pub mod position;

/// Num trait that is required by position.
pub trait PositionNum: Num + Signed + Clone + PartialOrd {}

impl<T: Num + Signed + Clone + PartialOrd> PositionNum for T {}
