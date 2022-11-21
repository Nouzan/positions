use crate::asset::Asset;

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
            self.values.insert(asset.clone(), SingleValue::default());
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
