use alloc::fmt;

pub use self::node::{Node, PositionNode, ValueNode};
use crate::{asset::Asset, IntoNaivePosition, PositionNum, Reversed};
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
            value = value.clone() + p.eval(price);
        }
        Some(value)
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
            value = value.clone() + price.clone() * weak_value;
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
        self.value.0 = self.value.0.clone() + value;
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

fn write_position<T>(f: &mut fmt::Formatter<'_>, price: &T, size: &T, asset: &Asset) -> fmt::Result
where
    T: fmt::Display + PositionNum,
{
    if asset.is_prefer_reversed() {
        if price.is_zero() {
            write!(f, "(Nan, {} {asset})*", -size.clone())
        } else {
            write!(
                f,
                "({}, {} {asset})*",
                T::one() / price.clone(),
                -size.clone()
            )
        }
    } else {
        write!(f, "({price}, {size} {asset})")
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
            write_position(f, &position.price, &position.size, asset)?;
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
        let flag = !self.children.is_empty();
        for (idx, (asset, position)) in self.positions.iter().enumerate() {
            if flag || idx != 0 {
                write!(f, " + ")?;
            }
            write_position(f, &position.price, &position.size, asset)?;
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
        let usdt = Asset::usdt();
        let btc = Asset::btc();
        let btcusdt_swap = Asset::from_str("btc-usdt-swap").unwrap().value_contained();
        let btcusd_swap = Asset::from_str("btc-usd-swap")
            .unwrap()
            .value_contained()
            .prefer_reversed();
        let mut p = tree(&usdt);
        p += (dec!(2), &btc);
        *p += (dec!(16000), dec!(12), &btcusdt_swap);
        p += (dec!(-1), &usdt);
        *p += (dec!(14000), dec!(-2), &btcusdt_swap);
        println!("{p}");
        let mut q = tree(&btc);
        q += (dec!(2), &btc);
        *q += (dec!(1) / dec!(16000), dec!(-200), &btcusd_swap);
        println!("{q}");
        p += q;
        println!("{p}");
        *p.get_weak_mut(&btc).unwrap() += (dec!(0), dec!(1), &btc);
        println!("{p}");
        *p.get_weak_mut(&btc).unwrap() += dec!(-1);
        println!("{p}");
        for (a, b) in p.all_pairs() {
            if a.is_value_contained() {
                println!("{a}");
            } else {
                println!("{a}-{b}");
            }
        }
    }

    #[test]
    fn reversed() {
        let usdt = Asset::usdt();
        let btc = Asset::btc();
        let btc_usd_swap = Asset::from_str("BTC-USDT-SWAP")
            .unwrap()
            .prefer_reversed()
            .value_contained();
        let mut p = tree(&usdt);
        p += (dec!(-16000), &usdt);
        p += (dec!(1), &btc);
        *p.get_weak_mut(&btc).unwrap() += Reversed((dec!(16000), dec!(-16000), &btc_usd_swap));
        println!("{p}");
        let mut prices = HashMap::default();
        for (a, b) in p.all_pairs() {
            if a.is_value_contained() {
                println!("{a}");
            } else {
                println!("{a}-{b}");
            }
            if a.is_prefer_reversed() {
                prices.insert((a, b), dec!(1) / dec!(17000));
            } else {
                prices.insert((a, b), dec!(17000));
            }
        }
        println!("{}", p.eval(&prices).unwrap());
    }
}
