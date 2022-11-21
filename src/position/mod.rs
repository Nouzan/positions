use core::ops::{AddAssign, Neg, SubAssign};

use alloc::fmt;

use crate::{instrument::Instrument, IntoNaivePosition, NaivePosition, PositionNum};

#[cfg(feature = "std")]
mod table;

#[cfg(feature = "std")]
pub use self::table::Positions;

/// Position.
#[derive(Debug, Clone)]
pub struct Position<T> {
    instrument: Instrument,
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
            naive: position.into_naive_position(),
        }
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

    /// Is this a zero position whose `size` and `value` are both zero.
    pub fn is_zero(&self) -> bool {
        self.naive.size.is_zero() && self.naive.value.is_zero()
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
        let value = self.value();
        let sign = if value.is_negative() { "-" } else { "+" };
        if let Some(price) = self.price() {
            write!(
                f,
                "({price}, {} {base}){mark} {sign} {}",
                self.size(),
                value.abs(),
            )
        } else {
            write!(
                f,
                "(Nan, {} {base}){mark} {sign} {}",
                self.size(),
                value.abs()
            )
        }
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

#[cfg(test)]
mod tests {
    use crate::{asset::Asset, Reversed};

    use super::*;
    use fraction::{BigInt, GenericDecimal, Zero};

    type BigDecimal = GenericDecimal<BigInt, usize>;

    #[test]
    fn normal() {
        let mut p = Position::new(
            Instrument::from((Asset::btc(), Asset::usdt())),
            BigDecimal::zero(),
        );
        p += BigDecimal::from(1.5);
        p += (BigDecimal::from(2.3), BigDecimal::from(2.5));
        p += (BigDecimal::from(7.3), BigDecimal::from(3.4));
        p += (
            BigDecimal::from(3.7),
            BigDecimal::from(-7.8),
            BigDecimal::from(12),
        );
        println!("{p}");
        assert_eq!(
            p,
            Position::new(
                Instrument::from((Asset::btc(), Asset::usdt())),
                (
                    BigDecimal::from(3.7),
                    BigDecimal::from(-1.9),
                    BigDecimal::from(4.7),
                )
            )
        )
    }

    #[test]
    fn reversed() {
        let mut p = Position::new(
            Instrument::from((Asset::usdt(), Asset::btc())).prefer_reversed(true),
            BigDecimal::zero(),
        );
        p += Reversed(BigDecimal::from(1.5));
        p += Reversed((BigDecimal::from(3), BigDecimal::from(2)));
        p += Reversed((BigDecimal::from(4), BigDecimal::from(1)));
        p += Reversed((
            BigDecimal::from(2),
            BigDecimal::from(-7),
            BigDecimal::from(-1.4),
        ));
        println!("{p}");
        assert_eq!(
            *p.value(),
            (BigDecimal::from(-29) / BigDecimal::from(60)).set_precision(1),
        );
    }
}
