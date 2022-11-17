use alloc::fmt;

pub use self::node::{Node, PositionNode, ValueNode};
use crate::{asset::Asset, IntoNaivePosition, PositionNum};
use core::ops::{AddAssign, Deref, DerefMut};
use std::collections::HashMap;

/// Node.
pub mod node;

/// Weak Position Tree.
/// # Invarience
/// The `asset` must be not in `positions.keys`.
#[derive(Debug, Clone)]
pub struct WeakTree<'a, T> {
    asset: &'a Asset,
    value: ValueNode<T>,
    positions: HashMap<&'a Asset, PositionNode<T>>,
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
            self.value.0 = self.value.0.clone() + position.take();
        } else {
            let value = self.positions.entry(asset).or_default().add(position);
            self.value.0 = self.value.0.clone() + value;
        }
        self
    }
}

/// Position Tree (the stronge tree).
/// # Invarience
/// The `asset` neither in `positions.keys` nor in `children.keys`.
#[derive(Debug, Clone)]
pub struct PositionTree<'a, T> {
    weak: WeakTree<'a, T>,
    children: HashMap<&'a Asset, WeakTree<'a, T>>,
}

/// Create a new empty position tree.
pub fn tree<T>(asset: &Asset) -> PositionTree<T>
where
    T: PositionNum,
{
    PositionTree::new(T::zero(), asset)
}

impl<'a, T: PositionNum> PositionTree<'a, T> {
    /// Create a new position tree (as a root).
    pub fn new(value: T, asset: &'a Asset) -> Self {
        Self {
            weak: WeakTree::new(value, asset),
            children: HashMap::default(),
        }
    }

    /// Insert a value.
    pub fn insert_value(&mut self, value: T, asset: &'a Asset) -> &mut Self {
        if *asset == *(self.weak.asset) {
            self.weak.value.0 = self.weak.value.0.clone() + value;
        } else {
            let acc = self
                .children
                .entry(asset)
                .or_insert_with(|| WeakTree::new(T::zero(), asset));
            acc.value.0 = acc.value.0.clone() + value;
        }
        self
    }

    /// Get the mutable reference of a weak tree.
    pub fn get_weak_mut(&mut self, asset: &Asset) -> Option<&mut WeakTree<'a, T>> {
        if *asset == *(self.weak.asset) {
            Some(&mut self.weak)
        } else {
            self.children.get_mut(asset)
        }
    }
}

impl<'a, T> AddAssign<(T, &'a Asset)> for PositionTree<'a, T>
where
    T: PositionNum,
{
    fn add_assign(&mut self, (value, asset): (T, &'a Asset)) {
        self.insert_value(value, asset);
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

impl<'a, T> AddAssign<(T, T, &'a Asset)> for PositionTree<'a, T>
where
    T: PositionNum,
{
    fn add_assign(&mut self, (price, size, asset): (T, T, &'a Asset)) {
        self.weak += (price, size, asset);
    }
}

impl<'a, T> Deref for PositionTree<'a, T> {
    type Target = WeakTree<'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.weak
    }
}

impl<'a, T> DerefMut for PositionTree<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.weak
    }
}

impl<'a, T> AddAssign<WeakTree<'a, T>> for PositionTree<'a, T>
where
    T: PositionNum,
{
    fn add_assign(&mut self, rhs: WeakTree<'a, T>) {
        if self.asset == rhs.asset {
            let WeakTree {
                value, positions, ..
            } = rhs;
            self.value.0 = self.value.0.clone() + value.0;
            for (asset, position) in positions {
                // According to the invarience, `asset` must not equal to `self.asset`.
                debug_assert_ne!(*asset, *self.asset);
                self.insert_position(position.0, asset);
            }
        } else if let Some(lhs) = self.children.get_mut(&rhs.asset) {
            lhs.value.0 = lhs.value.0.clone() + rhs.value.0;
            for (asset, position) in rhs.positions {
                // According to the invarience, `asset` must not equal to `self.asset`.
                debug_assert_ne!(*asset, *lhs.asset);
                lhs.insert_position(position.0, asset);
            }
        } else {
            self.children.insert(rhs.asset, rhs);
        }
    }
}

impl<'a, T> AddAssign for PositionTree<'a, T>
where
    T: PositionNum,
{
    fn add_assign(&mut self, rhs: Self) {
        let Self { weak, children } = rhs;
        *self += weak;
        for (asset, weak) in children {
            // According to the invarience, `asset` must not equal to `self.asset`.
            debug_assert_ne!(*asset, *self.asset);
            *self += weak;
        }
    }
}

impl<'a, T> fmt::Display for WeakTree<'a, T>
where
    T: fmt::Display + PositionNum,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (idx, (asset, position)) in self.positions.iter().enumerate() {
            if idx != 0 {
                write!(f, " + ({}, {} {asset})", position.price, position.size)?;
            } else {
                write!(f, "({}, {} {asset})", position.price, position.size)?;
            }
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

impl<'a, T> fmt::Display for PositionTree<'a, T>
where
    T: fmt::Display + PositionNum,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (idx, tree) in self.children.values().enumerate() {
            if idx != 0 {
                write!(f, " + {tree}")?;
            } else {
                write!(f, "{tree}")?;
            }
        }
        if !self.children.is_empty() {
            write!(f, " + ")?
        }
        write!(f, "{}", self.weak)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::str::FromStr;
    use rust_decimal_macros::dec;

    #[test]
    fn basic() {
        let usdt = Asset::Usdt;
        let btc = Asset::Btc;
        let btcusdt_swap = Asset::from_str("btc-usdt-swap").unwrap();
        let mut p = tree(&usdt);
        p += (dec!(2), &btc);
        p += (dec!(16000), dec!(12), &btcusdt_swap);
        p += (dec!(-1), &usdt);
        p += (dec!(14000), dec!(-2), &btcusdt_swap);
        println!("{p}");
        let mut q = tree(&btc);
        q += (dec!(2), &btc);
        q += (dec!(1) / dec!(16000), dec!(-200), &usdt);
        println!("{q}");
        p += q;
        println!("{p}");
    }
}
