use alloc::fmt;

use super::weak::WeakTree;
use crate::{asset::Asset, PositionNum};
use core::ops::{AddAssign, Deref, DerefMut};
use std::collections::HashMap;

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
            self.weak.value.0 += value;
        } else {
            let acc = self
                .children
                .entry(asset)
                .or_insert_with(|| WeakTree::new(T::zero(), asset));
            acc.value.0 += value;
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

    /// Get all asset-pairs (including the pairs of subtrees).
    pub fn all_pairs(&self) -> impl Iterator<Item = (&Asset, &Asset)> {
        let positions = self
            .children
            .values()
            .flat_map(|c| c.pairs())
            .chain(self.pairs());
        let values = self.children.keys().map(|asset| (*asset, self.asset));
        positions.chain(values)
    }

    /// Eval the position tree by closing all positions.
    /// Return `None` if there are missing prices.
    pub fn eval(&self, prices: &HashMap<(&Asset, &Asset), T>) -> Option<T> {
        let mut value = self.weak.eval_weak(prices)?;
        for (asset, weak) in self.children.iter() {
            let price = prices.get(&(*asset, self.weak.asset))?;
            let weak_value = weak.eval_weak(prices)?;
            value += weak_value * price;
        }
        Some(value)
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

impl<'a, T> AddAssign<(T, &'a Asset)> for PositionTree<'a, T>
where
    T: PositionNum,
{
    fn add_assign(&mut self, (value, asset): (T, &'a Asset)) {
        self.insert_value(value, asset);
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
            self.value.0 += value.0;
            for (asset, position) in positions {
                // According to the invarience, `asset` must not equal to `self.asset`.
                debug_assert_ne!(*asset, *self.asset);
                self.insert_position(position.0, asset);
            }
        } else if let Some(lhs) = self.children.get_mut(&rhs.asset) {
            lhs.value.0 += rhs.value.0;
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
        let flag = !self.children.is_empty();
        for (idx, (asset, position)) in self.positions.iter().enumerate() {
            if flag || idx != 0 {
                write!(f, " + ")?;
            }
            super::utils::write_position(f, &position.price, &position.size, asset)?;
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
