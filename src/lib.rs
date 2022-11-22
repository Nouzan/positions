//! A position (finance) definition that has some good algebraic properties.

#![deny(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

use num_traits::{NumAssignRef, Signed};

pub use naive_position::{IntoNaivePosition, NaivePosition, Reversed, ToNaivePosition};

/// Naive position without price representation.
pub mod naive_position;

/// Position.
#[cfg(feature = "alloc")]
pub mod position;

/// Asset.
#[cfg(feature = "alloc")]
pub mod asset;

/// Instrument.
#[cfg(feature = "alloc")]
pub mod instrument;

/// Position Tree.
#[cfg(feature = "alloc")]
pub mod tree;

/// Legacy.
#[deprecated(since = "0.2.0")]
pub mod legacy;

/// Prelude.
#[cfg(feature = "alloc")]
pub mod prelude {
    pub use crate::asset::{Asset, ParseAssetError};
    pub use crate::instrument::Instrument;
    pub use crate::naive_position::IntoNaivePosition;
    pub use crate::position::{Position, Positions};
    pub use crate::PositionNum;

    #[cfg(not(feature = "std"))]
    pub use hashbrown::HashMap;

    #[cfg(feature = "std")]
    pub use std::collections::HashMap;
}

#[cfg(feature = "alloc")]
pub use prelude::*;

/// Num trait that is required by position.
pub trait PositionNum: NumAssignRef + Signed + Clone + PartialOrd {}

impl<T: NumAssignRef + Signed + Clone + PartialOrd> PositionNum for T {}
