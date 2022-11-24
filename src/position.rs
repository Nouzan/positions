use crate::{
    instrument::Instrument, tree::PositionTree, Asset, HashMap, IntoNaivePosition, NaivePosition,
    PositionNum, Reversed,
};
use alloc::fmt;
use arcstr::ArcStr;
use core::ops::{AddAssign, Neg, SubAssign};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Position.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Position<T> {
    instrument: Instrument,
    #[cfg_attr(feature = "serde", serde(flatten))]
    naive: NaivePosition<T>,
}

impl<T> Position<T> {
    /// Get the instrument.
    pub fn instrument(&self) -> &Instrument {
        &self.instrument
    }

    /// Convert to a [`NaivePosition`]
    pub fn as_naive(&self) -> &NaivePosition<T> {
        &self.naive
    }

    /// Get the value of the position.
    pub fn value(&self) -> &T {
        &self.naive.value
    }
}

impl<T> Position<T>
where
    T: PositionNum,
{
    /// Create a new position.
    pub fn new(instrument: Instrument, position: impl IntoNaivePosition<T>) -> Self {
        Self {
            instrument,
            naive: position.into_naive(),
        }
    }

    /// Return the value when the position is closed at the given price.
    pub fn closed(&self, price: &T) -> T {
        let mut p = self.naive.clone();
        p -= (price.clone(), p.size.clone());
        p.value
    }

    /// Get the average price of the position,
    /// respecting the reversed preference of its instrument.
    pub fn price(&self) -> Option<T> {
        if self.instrument.is_prefer_reversed() {
            if self.naive.price.is_zero() {
                None
            } else {
                Some({
                    let mut v = T::one();
                    v /= &self.naive.price;
                    v
                })
            }
        } else {
            Some(self.naive.price.clone())
        }
    }

    /// Get the size of the position,
    /// respecting the reversed preference of its instrument.
    pub fn size(&self) -> T {
        if self.instrument.is_prefer_reversed() {
            self.naive.size.clone().neg()
        } else {
            self.naive.size.clone()
        }
    }

    /// Calculate the notional value of the position.
    /// Note that the notional value of a short position will be negative.
    pub fn notional_value(&self) -> T {
        let mut value = self.naive.price.clone();
        value *= &self.naive.size;
        value
    }

    /// Merge with the other position.
    /// After merging, the `other` will be the default ("zero") position.
    /// # Warning
    /// No-OP if the other position has different `instrument`.
    pub fn merge(&mut self, other: &mut Self) {
        if other.instrument == self.instrument {
            let rhs = core::mem::take(&mut other.naive);
            self.naive += rhs;
            debug_assert!(other.is_zero());
        }
    }

    /// Take the value of the position.
    #[inline]
    pub fn take(&mut self) -> T {
        self.naive.take()
    }

    /// Convert the price to the given.
    /// # Warning
    /// The `to` price is treated to be in the reversed-form
    /// if the `instrument` is reversed-prefering.
    /// # Panic
    /// Panic if `to` is in the reversed-form and is zero.
    pub fn convert(&mut self, to: T) {
        let to = if self.instrument.is_prefer_reversed() {
            if to.is_zero() {
                panic!("the price in reversed-form cannot be zero");
            }
            T::one() / to
        } else {
            to
        };
        self.naive.convert(to);
    }

    /// Is this a zero position whose `size` and `value` are both zero.
    pub fn is_zero(&self) -> bool {
        self.naive.size.is_zero() && self.naive.value.is_zero()
    }

    /// Convert to a position tree.
    pub fn as_tree(&self) -> PositionTree<'_, T> {
        PositionTree {
            asset: self.instrument.quote(),
            value: T::zero(),
            positions: HashMap::from([(&self.instrument, self)]),
            children: HashMap::default(),
        }
    }
}

impl<'a, T: PositionNum> IntoNaivePosition<T> for &'a Position<T> {
    fn into_naive(self) -> NaivePosition<T> {
        self.naive.clone()
    }
}

impl<T> fmt::Display for Position<T>
where
    T: PositionNum + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let base = self.instrument.base();
        let mark = if self.instrument.is_prefer_reversed() {
            "*"
        } else {
            ""
        };
        if let Some(price) = self.price() {
            write!(f, "({price}, {} {base}){mark}", self.size(),)?;
        } else {
            write!(f, "(Nan, {} {base}){mark}", self.size(),)?;
        }
        let value = self.value();
        if !value.is_zero() {
            let sign = if value.is_negative() { " - " } else { " + " };
            write!(f, "{sign}{} {}", value.abs(), self.instrument.quote())?;
        }
        Ok(())
    }
}

impl<T> PartialEq for Position<T>
where
    T: PositionNum,
{
    fn eq(&self, other: &Self) -> bool {
        self.instrument == other.instrument && self.naive == other.naive
    }
}

impl<T> Eq for Position<T> where T: PositionNum {}

impl<T, P> AddAssign<P> for Position<T>
where
    T: PositionNum,
    P: IntoNaivePosition<T>,
{
    fn add_assign(&mut self, rhs: P) {
        self.naive += rhs;
    }
}

impl<T, P> SubAssign<P> for Position<T>
where
    T: PositionNum,
    P: IntoNaivePosition<T>,
{
    fn sub_assign(&mut self, rhs: P) {
        self.naive -= rhs;
    }
}

impl<T> Neg for Position<T>
where
    T: PositionNum,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            instrument: self.instrument.clone(),
            naive: self.naive.neg(),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
struct SingleValue<T> {
    value: T,
    positions: HashMap<ArcStr, Position<T>>,
}

impl<T> Default for SingleValue<T>
where
    T: PositionNum,
{
    fn default() -> Self {
        Self {
            value: T::zero(),
            positions: HashMap::default(),
        }
    }
}

impl<T> SingleValue<T>
where
    T: PositionNum,
{
    fn insert(&mut self, position: Position<T>) {
        if let Some(p) = self.positions.get_mut(position.instrument.symbol()) {
            debug_assert_eq!(p.instrument, position.instrument);
            p.naive += position.naive;
        } else {
            self.positions
                .insert(position.instrument.symbol.clone(), position);
        }
    }

    fn concentrate(&mut self) {
        let value = self
            .positions
            .values_mut()
            .map(|p| p.take())
            .fold(T::zero(), T::add);
        self.value += value;
    }
}

impl<T> AddAssign<&Self> for SingleValue<T>
where
    T: PositionNum,
{
    fn add_assign(&mut self, rhs: &Self) {
        self.value += &rhs.value;
        for (inst, rhs) in rhs.positions.iter() {
            if let Some(lhs) = self.positions.get_mut(inst) {
                debug_assert_eq!(lhs.instrument, rhs.instrument);
                lhs.naive += rhs.naive.clone();
            } else {
                self.positions.insert(inst.clone(), rhs.clone());
            }
        }
    }
}

impl<T> PartialEq for SingleValue<T>
where
    T: PositionNum,
{
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.positions == other.positions
    }
}

impl<T> Eq for SingleValue<T> where T: PositionNum {}

/// A table of positions.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Positions<T> {
    values: HashMap<Asset, SingleValue<T>>,
}

impl<T> Default for Positions<T> {
    fn default() -> Self {
        Self {
            values: Default::default(),
        }
    }
}

impl<T> Positions<T>
where
    T: PositionNum,
{
    /// Convert to a position tree, using the given asset as root.
    pub fn as_tree<'a>(&'a self, root: &'a Asset) -> PositionTree<'a, T> {
        let children = self
            .values
            .iter()
            .filter_map(|(asset, sv)| {
                if *asset == *root {
                    None
                } else {
                    let inst = Instrument::from((asset.clone(), root.clone()));
                    Some((
                        inst,
                        PositionTree {
                            asset,
                            value: sv.value.clone(),
                            positions: sv.positions.values().map(|p| (p.instrument(), p)).collect(),
                            children: HashMap::default(),
                        },
                    ))
                }
            })
            .collect();
        if let Some(sv) = self.values.get(root) {
            PositionTree {
                asset: root,
                value: sv.value.clone(),
                positions: sv.positions.values().map(|p| (p.instrument(), p)).collect(),
                children,
            }
        } else {
            PositionTree {
                asset: root,
                value: T::zero(),
                positions: HashMap::default(),
                children,
            }
        }
    }

    /// Insert a position.
    pub fn insert_position(&mut self, position: Position<T>) -> &mut Self {
        self.values
            .entry(position.instrument.quote().clone())
            .or_default()
            .insert(position);
        self
    }

    /// Insert an value.
    pub fn insert_value(&mut self, value: T, asset: &Asset) -> &mut Self {
        if let Some(sv) = self.values.get_mut(asset) {
            sv.value += value;
        } else {
            self.values.insert(
                asset.clone(),
                SingleValue {
                    value,
                    ..Default::default()
                },
            );
        }
        self
    }

    /// Get the reference of the position of the given instrument.
    pub fn get_position(&self, instrument: &Instrument) -> Option<&Position<T>> {
        self.values
            .get(instrument.quote())?
            .positions
            .get(&instrument.symbol)
    }

    /// Get the reference of the value of the given asset.
    pub fn get_value(&self, asset: &Asset) -> Option<&T> {
        Some(&self.values.get(asset)?.value)
    }

    /// Get the mutable reference of the position of the given instrument.
    pub fn get_position_mut(&mut self, instrument: &Instrument) -> Option<&mut Position<T>> {
        self.values
            .get_mut(instrument.quote())?
            .positions
            .get_mut(&instrument.symbol)
    }

    /// Get the mutable reference of the value of the given asset.
    pub fn get_value_mut(&mut self, asset: &Asset) -> Option<&mut T> {
        Some(&mut self.values.get_mut(asset)?.value)
    }

    /// Concentrate the values.
    pub fn concentrate(&mut self) {
        for sv in self.values.values_mut() {
            sv.concentrate();
        }
    }
}

impl<T> PartialEq for Positions<T>
where
    T: PositionNum,
{
    fn eq(&self, other: &Self) -> bool {
        self.values == other.values
    }
}

impl<T> Eq for Positions<T> where T: PositionNum {}

impl<T> AddAssign<&Self> for Positions<T>
where
    T: PositionNum,
{
    fn add_assign(&mut self, rhs: &Self) {
        for (asset, rhs) in rhs.values.iter() {
            if let Some(lhs) = self.values.get_mut(asset) {
                *lhs += rhs;
            } else {
                self.values.insert(asset.clone(), rhs.clone());
            }
        }
    }
}

impl<T> AddAssign for Positions<T>
where
    T: PositionNum,
{
    fn add_assign(&mut self, rhs: Self) {
        *self += &rhs;
    }
}

impl<T> AddAssign<Position<T>> for Positions<T>
where
    T: PositionNum,
{
    fn add_assign(&mut self, rhs: Position<T>) {
        self.insert_position(rhs);
    }
}

impl<'a, T> AddAssign<(T, &'a Asset)> for Positions<T>
where
    T: PositionNum,
{
    fn add_assign(&mut self, (value, asset): (T, &'a Asset)) {
        self.insert_value(value, asset);
    }
}

impl<'a, T> AddAssign<(T, T, &'a Instrument)> for Positions<T>
where
    T: PositionNum,
{
    fn add_assign(&mut self, (price, size, instrument): (T, T, &'a Instrument)) {
        self.insert_position(Position::new(instrument.clone(), (price, size)));
    }
}

impl<'a, T> AddAssign<(T, T, T, &'a Instrument)> for Positions<T>
where
    T: PositionNum,
{
    fn add_assign(&mut self, (price, size, value, instrument): (T, T, T, &'a Instrument)) {
        self.insert_position(Position::new(instrument.clone(), (price, size, value)));
    }
}

impl<'a, T> AddAssign<Reversed<(T, T, &'a Instrument)>> for Positions<T>
where
    T: PositionNum,
{
    fn add_assign(
        &mut self,
        Reversed((price, size, instrument)): Reversed<(T, T, &'a Instrument)>,
    ) {
        self.insert_position(Position::new(instrument.clone(), Reversed((price, size))));
    }
}

impl<'a, T> AddAssign<Reversed<(T, T, T, &'a Instrument)>> for Positions<T>
where
    T: PositionNum,
{
    fn add_assign(
        &mut self,
        Reversed((price, size, value, instrument)): Reversed<(T, T, T, &'a Instrument)>,
    ) {
        self.insert_position(Position::new(
            instrument.clone(),
            Reversed((price, size, value)),
        ));
    }
}

impl<T> From<Position<T>> for Positions<T>
where
    T: PositionNum,
{
    fn from(p: Position<T>) -> Self {
        let asset = p.instrument.quote().clone();
        let inst = p.instrument.symbol.clone();
        let sv = SingleValue {
            value: T::zero(),
            positions: HashMap::from([(inst, p)]),
        };
        Self {
            values: HashMap::from([(asset, sv)]),
        }
    }
}

impl<T> fmt::Display for SingleValue<T>
where
    T: fmt::Display + PositionNum,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const MIDDLE: &str = "├ ";
        const LAST: &str = "└ ";
        let len = self.positions.len();
        for (idx, (inst, p)) in self.positions.iter().enumerate() {
            if p.is_zero() {
                continue;
            }
            if idx == len - 1 {
                writeln!(f, "{LAST}{inst} => {p}")?;
            } else {
                writeln!(f, "{MIDDLE}{inst} => {p}")?;
            }
        }
        Ok(())
    }
}

impl<T> fmt::Display for Positions<T>
where
    T: PositionNum + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (asset, sv) in self.values.iter() {
            writeln!(f, "{asset} => {} {asset}", sv.value)?;
            write!(f, "{sv}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{asset::Asset, Reversed};
    use fraction::{BigInt, GenericDecimal, Zero};

    type Decimal = GenericDecimal<BigInt, usize>;

    #[test]
    fn normal() {
        let mut p = Position::new(
            Instrument::from((Asset::btc(), Asset::usdt())),
            Decimal::zero(),
        );
        p += Decimal::from(1.5);
        p += (Decimal::from(2.3), Decimal::from(2.5));
        p += (Decimal::from(7.3), Decimal::from(3.4));
        p += (Decimal::from(3.7), Decimal::from(-7.8), Decimal::from(12));
        #[cfg(feature = "std")]
        println!("{p}");
        assert_eq!(
            p,
            Position::new(
                Instrument::from((Asset::btc(), Asset::usdt())),
                (Decimal::from(3.7), Decimal::from(-1.9), Decimal::from(4.7),)
            )
        );
        #[cfg(feature = "std")]
        println!("{}", p.as_tree());
    }

    #[test]
    fn reversed() {
        let mut p = Position::new(
            Instrument::from((Asset::usdt(), Asset::btc())).prefer_reversed(true),
            Decimal::zero(),
        );
        p += Reversed(Decimal::from(1.5));
        p += Reversed((Decimal::from(3), Decimal::from(2)));
        p += Reversed((Decimal::from(4), Decimal::from(1)));
        p += Reversed((Decimal::from(2), Decimal::from(-7), Decimal::from(-1.4)));
        #[cfg(feature = "std")]
        println!("{p}");
        assert_eq!(
            *p.value(),
            (Decimal::from(-29) / Decimal::from(60)).set_precision(1),
        );
        #[cfg(feature = "std")]
        println!("{}", p.as_tree());
    }

    #[test]
    fn basic_positions() {
        let btc = Asset::btc();
        let usdt = Asset::usdt();
        let btc_usdt_swap = Instrument::new("BTC-USDT-SWAP", Asset::btc(), Asset::usdt());
        let btc_usd_swap =
            Instrument::new("BTC-USD-SWAP", Asset::usd(), Asset::btc()).prefer_reversed(true);
        let eth_btc_swap = Instrument::new("ETH-BTC-SWAP", Asset::eth(), Asset::btc());
        let mut p = Positions::default();
        p += (Decimal::from(-16000), &usdt);
        p += (Decimal::from(1), &btc);
        p += Reversed((Decimal::from(16000), Decimal::from(-16000), &btc_usd_swap));
        p += (Decimal::from(0.067), Decimal::from(-21.5), &eth_btc_swap);
        p += (
            Decimal::from(16001),
            Decimal::from(-1.5),
            Decimal::from(-2.7),
            &btc_usdt_swap,
        );
        #[cfg(feature = "std")]
        println!("{p}");
    }

    #[test]
    fn positions_as_tree() {
        let btc = Asset::btc();
        let usdt = Asset::usdt();
        let btc_usdt_swap = Instrument::new("BTC-USDT-SWAP", Asset::btc(), Asset::usdt());
        let btc_usd_swap =
            Instrument::new("BTC-USD-SWAP", Asset::usd(), Asset::btc()).prefer_reversed(true);
        let eth_btc_swap = Instrument::new("ETH-BTC-SWAP", Asset::eth(), Asset::btc());
        let mut p = Positions::default();
        p += (Decimal::from(-16000), &usdt);
        p += (Decimal::from(1), &btc);
        p += Reversed((Decimal::from(16000), Decimal::from(-16000), &btc_usd_swap));
        p += (Decimal::from(0.067), Decimal::from(-21.5), &eth_btc_swap);
        p += (
            Decimal::from(16001),
            Decimal::from(-1.5),
            Decimal::from(-2.7),
            &btc_usdt_swap,
        );
        let tree = p.as_tree(&usdt);
        #[cfg(feature = "std")]
        println!("{tree}");
        let prices = HashMap::from([
            (eth_btc_swap.clone(), Decimal::from(0.059)),
            (
                btc_usd_swap.clone(),
                Decimal::from(1) / Decimal::from(17000),
            ),
            (btc_usdt_swap.clone(), Decimal::from(17002)),
            (
                Instrument::from((btc.clone(), usdt.clone())),
                Decimal::from(17000),
            ),
        ]);
        #[cfg(feature = "std")]
        for inst in tree.instruments() {
            println!("{inst}");
        }
        let ans = tree.eval(&prices).unwrap().set_precision(1);
        #[cfg(feature = "std")]
        println!("{ans}");
        assert_eq!(ans, Decimal::from(1419.8).set_precision(1));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_single_value() -> anyhow::Result<()> {
        use rust_decimal_macros::dec;

        let inst = Instrument::new("BTC-USD-SWAP", Asset::usd(), Asset::btc());
        let sv = SingleValue {
            value: dec!(1.2),
            positions: HashMap::from([(
                inst.symbol.clone(),
                inst.into_position((dec!(1.4), dec!(2))),
            )]),
        };
        let s = serde_json::to_string(&sv)?;
        #[cfg(feature = "std")]
        println!("{s}");
        assert!(!s.is_empty());
        Ok(())
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_positions() -> anyhow::Result<()> {
        use rust_decimal_macros::dec;
        let inst = Instrument::new("BTC-USD-SWAP", Asset::usd(), Asset::btc());
        let sv = SingleValue {
            value: dec!(1.2),
            positions: HashMap::from([(
                inst.symbol.clone(),
                inst.clone().into_position((dec!(1.4), dec!(2))),
            )]),
        };
        let positoins = Positions {
            values: HashMap::from([(inst.quote().clone(), sv)]),
        };
        let s = serde_json::to_string(&positoins)?;
        #[cfg(feature = "std")]
        println!("{s}");
        assert!(!s.is_empty());
        Ok(())
    }
}
