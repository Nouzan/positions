//! A position (finance) definition that has some good algebraic properties.

#![deny(missing_docs)]

#[cfg(test)]
mod tests;

/// Naive position without price representation.
pub mod naive_position;

/// Price representation.
pub mod representation;

/// Position with price representation.
pub mod position;

use num_traits::{Num, Signed};

pub use naive_position::{IntoNaivePosition, NaivePosition, ToNaivePosition};
pub use position::{normal, position, reversed, Position};
pub use representation::{Normal, Representation, Reversed};

/// Num trait that is required by position.
pub trait PositionNum: Num + Signed + Clone + PartialOrd {}

impl<T: Num + Signed + Clone + PartialOrd> PositionNum for T {}
