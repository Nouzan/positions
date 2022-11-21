use crate::{asset::Asset, NaivePosition};

#[cfg(feature = "std")]
use std::collections::HashMap;

/// Position.
#[derive(Debug, Clone)]
pub struct Position<T> {
    asset: Asset,
    naive: NaivePosition<T>,
}

#[cfg(feature = "std")]
/// A table of positions.
#[derive(Debug, Clone)]
pub struct Positions<T> {
    values: HashMap<Asset, HashMap<Asset, Position<T>>>,
}
