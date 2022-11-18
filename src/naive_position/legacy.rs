#![allow(deprecated)]

use crate::{position::Position, representation::Representation, NaivePosition, PositionNum};

impl<T: PositionNum> NaivePosition<T> {
    /// Create a [`Position`] from the [`NaivePosition`] directly,
    /// without changing its price or size according to the representation.
    pub fn into_position<Rep: Representation>(self) -> Position<Rep, T> {
        #[allow(deprecated)]
        Position {
            naive: self,
            _rep: core::marker::PhantomData::default(),
        }
    }
}
