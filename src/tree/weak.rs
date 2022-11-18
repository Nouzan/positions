use core::ops::AddAssign;
use std::collections::HashMap;

use alloc::fmt;

use crate::{asset::Asset, IntoNaivePosition, PositionNum, Reversed};

use super::{PositionNode, ValueNode};

/// Weak Position Tree.
/// # Invarience
/// The `asset` must be not in `positions.keys`.
#[derive(Debug, Clone)]
pub struct WeakTree<'a, T> {
    pub(super) asset: &'a Asset,
    pub(super) value: ValueNode<T>,
    pub(super) positions: HashMap<&'a Asset, PositionNode<T>>,
}

impl<'a, T: PositionNum> WeakTree<'a, T> {
    /// Create a new weak tree without positions.
    pub fn new(value: T, asset: &'a Asset) -> Self {
        Self {
            asset,
            value: ValueNode(value),
            positions: HashMap::default(),
        }
    }

    /// Get asset.
    pub fn asset(&self) -> &Asset {
        self.asset
    }

    /// Insert a (normal) position.
    pub fn insert_position(
        &mut self,
        position: impl IntoNaivePosition<T>,
        asset: &'a Asset,
    ) -> &mut Self {
        if *asset == *(self.asset) {
            let mut position = position.into_naive_position();
            position.convert(T::one());
            self.value.0 += position.take();
        } else {
            self.value.0 += self.positions.entry(asset).or_default().add(position);
        }
        self
    }

    /// Get reference asset-pairs.
    pub fn pairs(&self) -> impl Iterator<Item = (&Asset, &Asset)> {
        self.positions.keys().map(|n| (*n, self.asset))
    }

    /// Evaluate the weak tree by closing all positions.
    /// Return `None` if missing prices.
    pub fn eval_weak(&self, prices: &HashMap<(&Asset, &Asset), T>) -> Option<T> {
        let mut value = self.value.0.clone();
        for (asset, p) in self.positions.iter() {
            let price = prices.get(&(*asset, self.asset))?;
            value += p.eval(price);
        }
        Some(value)
    }
}

impl<'a, T> AddAssign<(T, T, &'a Asset)> for WeakTree<'a, T>
where
    T: PositionNum,
{
    fn add_assign(&mut self, (price, size, asset): (T, T, &'a Asset)) {
        self.insert_position((price, size), asset);
    }
}

impl<'a, T> AddAssign<Reversed<(T, T, &'a Asset)>> for WeakTree<'a, T>
where
    T: PositionNum,
{
    fn add_assign(&mut self, Reversed((price, size, asset)): Reversed<(T, T, &'a Asset)>) {
        self.insert_position(Reversed((price, size)), asset);
    }
}

impl<'a, T> AddAssign<T> for WeakTree<'a, T>
where
    T: PositionNum,
{
    fn add_assign(&mut self, value: T) {
        self.value.0 += value;
    }
}

impl<'a, T> fmt::Display for WeakTree<'a, T>
where
    T: fmt::Display + PositionNum,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (idx, (asset, position)) in self.positions.iter().enumerate() {
            if idx != 0 {
                write!(f, " + ")?;
            }
            super::utils::write_position(f, &position.price, &position.size, asset)?;
        }
        let flag = !self.positions.is_empty();
        if self.value.0.is_positive() && flag {
            write!(f, " + {} {}", self.value.0, self.asset)
        } else if self.value.0.is_negative() && flag {
            write!(f, " - {} {}", self.value.0.abs(), self.asset)
        } else if self.value.0.is_negative() {
            write!(f, "- {} {}", self.value.0.abs(), self.asset)
        } else {
            write!(f, "{} {}", self.value.0, self.asset)
        }
    }
}
