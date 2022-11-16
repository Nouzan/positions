use alloc::fmt;

pub use self::node::{Node, PositionNode, ValueNode};
use crate::{asset::Asset, IntoNaivePosition, PositionNum};
use core::ops::AddAssign;
use std::collections::HashMap;

/// Node.
pub mod node;

/// Position Tree.
/// # Invarience
/// The `asset` neither in `positions.keys` nor in `children.keys`.
#[derive(Debug, Clone)]
pub struct PositionTree<T> {
    asset: Asset,
    value: ValueNode<T>,
    positions: HashMap<Asset, PositionNode<T>>,
    children: HashMap<Asset, PositionTree<T>>,
}

/// Create a new empty position tree.
pub fn tree<T>(asset: &Asset) -> PositionTree<T>
where
    T: PositionNum,
{
    PositionTree::new(T::zero(), asset)
}

impl<T: PositionNum> PositionTree<T> {
    /// Create a new position tree (as a root).
    pub fn new(value: T, asset: &Asset) -> Self {
        Self {
            asset: asset.clone(),
            value: ValueNode(value),
            positions: HashMap::default(),
            children: HashMap::default(),
        }
    }

    /// Get asset.
    pub fn asset(&self) -> &Asset {
        &self.asset
    }

    /// Insert a (normal) position.
    pub fn insert_position(
        &mut self,
        position: impl IntoNaivePosition<T>,
        asset: &Asset,
    ) -> &mut Self {
        if *asset == self.asset {
            let mut position = position.into_naive_position();
            position.convert(T::one());
            self.value.0 = self.value.0.clone() + position.take();
        } else {
            let value = self
                .positions
                .entry(asset.clone())
                .or_default()
                .add(position);
            self.value.0 = self.value.0.clone() + value;
        }
        self
    }

    /// Insert a value (as a subtree).
    pub fn insert_value(&mut self, value: T, asset: &Asset) -> &mut Self {
        if *asset == self.asset {
            self.value.0 = self.value.0.clone() + value;
        } else {
            let acc = self
                .children
                .entry(asset.clone())
                .or_insert_with(|| PositionTree::new(T::zero(), asset));
            acc.value.0 = acc.value.0.clone() + value;
        }
        self
    }
}

impl<'a, T> AddAssign<(T, &'a Asset)> for PositionTree<T>
where
    T: PositionNum,
{
    fn add_assign(&mut self, (value, asset): (T, &'a Asset)) {
        self.insert_value(value, asset);
    }
}

impl<'a, T> AddAssign<(T, T, &'a Asset)> for PositionTree<T>
where
    T: PositionNum,
{
    fn add_assign(&mut self, (price, size, asset): (T, T, &'a Asset)) {
        self.insert_position((price, size), asset);
    }
}

impl<T> AddAssign for PositionTree<T>
where
    T: PositionNum,
{
    fn add_assign(&mut self, rhs: Self) {
        if self.asset == rhs.asset {
            let Self {
                value,
                positions,
                children,
                ..
            } = rhs;
            self.value.0 = self.value.0.clone() + value.0;
            for (asset, position) in positions {
                // According to the invarience, `asset` must not equal to `self.asset`.
                debug_assert_ne!(asset, self.asset);
                self.insert_position(position.0, &asset);
            }
            for (asset, tree) in children {
                // According to the invarience, `asset` must not equal to `self.asset`.
                debug_assert_ne!(asset, self.asset);
                *self += tree;
            }
        } else if let Some(lhs) = self.children.get_mut(&rhs.asset) {
            *lhs += rhs;
        } else {
            self.children.insert(rhs.asset.clone(), rhs);
        }
    }
}

impl<T> fmt::Display for PositionTree<T>
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
        let flag = !self.children.is_empty();
        for (idx, (asset, position)) in self.positions.iter().enumerate() {
            if flag || idx != 0 {
                write!(f, " + ({}, {} {asset})", position.price, position.size)?;
            } else {
                write!(f, "({}, {} {asset})", position.price, position.size)?;
            }
        }
        let flag = flag || !self.positions.is_empty();
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
        q += p;
        println!("{q}");
    }
}
