use crate::{asset::Asset, Reversed};

use super::*;
use im::{hashmap, HashMap};

#[derive(Debug, Clone)]
struct SingleValue<T> {
    value: T,
    positions: HashMap<Instrument, Position<T>>,
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
        if let Some(p) = self.positions.get_mut(&position.instrument) {
            debug_assert_eq!(p.instrument, position.instrument);
            p.naive += position.naive;
        } else {
            self.positions.insert(position.instrument.clone(), position);
        }
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

/// A table of positions.
#[derive(Debug, Clone, Default)]
pub struct Positions<T> {
    values: HashMap<Asset, SingleValue<T>>,
}

impl<T> Positions<T>
where
    T: PositionNum,
{
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
            .get(instrument)
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
            .get_mut(instrument)
    }

    /// Get the mutable reference of the value of the given asset.
    pub fn get_value_mut(&mut self, asset: &Asset) -> Option<&mut T> {
        Some(&mut self.values.get_mut(asset)?.value)
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
        let inst = p.instrument.clone();
        let sv = SingleValue {
            value: T::zero(),
            positions: hashmap! { inst => p },
        };
        Self {
            values: hashmap! { asset => sv },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fraction::{BigInt, GenericDecimal};

    type Decimal = GenericDecimal<BigInt, usize>;

    #[test]
    fn basic() {
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
        println!("{p}");
    }
}
