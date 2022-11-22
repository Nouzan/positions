use super::PositionNum;
use core::ops::{Add, AddAssign, Neg, Sub, SubAssign};
use num_traits::Zero;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[deprecated(since = "0.2.0")]
mod legacy;

/// Naive position (in normal representation).
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NaivePosition<T> {
    /// Price.
    pub price: T,
    /// Size.
    pub size: T,
    /// Value.
    pub value: T,
}

impl<T: PositionNum> Default for NaivePosition<T> {
    fn default() -> Self {
        Self {
            price: T::one(),
            size: T::zero(),
            value: T::zero(),
        }
    }
}

impl<T: PositionNum> NaivePosition<T> {
    /// Create a new [`NaivePosition`].
    pub fn new(price: T, size: T, value: T) -> Self {
        Self { price, size, value }
    }

    /// Return a new position that consumes its `value`. (Equivalence I).
    ///
    /// Return `None` if `size` is zero.
    pub fn consumed(&self) -> Option<Self> {
        if self.size.is_zero() {
            None
        } else {
            let mut price = self.value.clone().neg();
            price /= &self.size;
            price += &self.price;
            Some(Self {
                price,
                size: self.size.clone(),
                value: T::zero(),
            })
        }
    }

    /// Consume the `value`.
    ///
    /// No effect if `size` is zero.
    pub fn consume(&mut self) {
        if !self.size.is_zero() {
            let mut delta = self.value.clone();
            delta /= &self.size;
            self.price -= delta;
            self.value = T::zero();
        }
    }

    /// Return a new position with the given `price`
    /// but keep equivalent to the original position.
    /// (Equivalence II)
    pub fn converted(&self, price: T) -> Self {
        let mut value = price.clone();
        value -= &self.price;
        value *= &self.size;
        Self {
            price,
            size: self.size.clone(),
            value,
        }
    }

    /// Convert the `price` to the given
    /// but keep equivalent to the original.
    /// (Equivalence II)
    pub fn convert(&mut self, price: T) {
        self.value = price.clone();
        self.value -= &self.price;
        self.value *= &self.size;
        self.price = price;
    }

    /// Take the `value` and keep the `price` and `size` unchanged.
    ///
    /// After the operation, the new position is no longer
    /// equivalent to the original.
    pub fn take(&mut self) -> T {
        let mut value = T::zero();
        core::mem::swap(&mut self.value, &mut value);
        value
    }
}

impl<T: PositionNum, H> PartialEq<H> for NaivePosition<T>
where
    H: ToNaivePosition<T>,
{
    fn eq(&self, other: &H) -> bool {
        let other = other.to_naive();
        if self.size.eq(&other.size) {
            if self.price.eq(&other.price) && self.value.eq(&other.value) {
                true
            } else if self.size.is_zero() && other.size.is_zero() {
                self.value.eq(&other.value)
            } else {
                match (self.clone().consumed(), other.consumed()) {
                    (Some(lhs), Some(rhs)) => lhs.price.eq(&rhs.price),
                    _ => false,
                }
            }
        } else {
            false
        }
    }
}

impl<T: PositionNum> Eq for NaivePosition<T> {}

impl<T: PositionNum> Zero for NaivePosition<T> {
    fn zero() -> Self {
        Self::default()
    }

    fn is_zero(&self) -> bool {
        self.size.is_one() && self.value.is_zero()
    }
}

/// Types that can convert into a [`NaivePosition`].
pub trait IntoNaivePosition<T: PositionNum> {
    /// Convert to a `NaivePosition`.
    fn into_naive(self) -> NaivePosition<T>;

    /// Mark this position as reversed-form.
    fn reversed(self) -> Reversed<Self>
    where
        Self: Sized,
    {
        Reversed(self)
    }
}

impl<T: PositionNum> IntoNaivePosition<T> for NaivePosition<T> {
    fn into_naive(self) -> NaivePosition<T> {
        self
    }
}

impl<T: PositionNum> IntoNaivePosition<T> for (T, T, T) {
    fn into_naive(self) -> NaivePosition<T> {
        NaivePosition {
            price: self.0,
            size: self.1,
            value: self.2,
        }
    }
}

impl<T: PositionNum> IntoNaivePosition<T> for (T, T) {
    fn into_naive(self) -> NaivePosition<T> {
        NaivePosition {
            price: self.0,
            size: self.1,
            value: T::zero(),
        }
    }
}

impl<T: PositionNum> IntoNaivePosition<T> for T {
    fn into_naive(self) -> NaivePosition<T> {
        NaivePosition {
            price: T::one(),
            size: T::zero(),
            value: self,
        }
    }
}

/// Position in reversed form.
#[derive(Debug, Clone, Copy)]
pub struct Reversed<P>(pub P);

impl<T, P> IntoNaivePosition<T> for Reversed<P>
where
    T: PositionNum,
    P: IntoNaivePosition<T>,
{
    /// Convert into a naive position in reversed form.
    /// # Panic
    /// Panic if the `price` is zero.
    fn into_naive(self) -> NaivePosition<T> {
        let NaivePosition { price, size, value } = self.0.into_naive();
        if price.is_zero() {
            panic!("zero price cannot be convert into reversed form");
        }
        NaivePosition {
            price: T::one() / price,
            size: -size,
            value,
        }
    }
}

/// Types that can convert to [`NaivePosition`] by ref.
pub trait ToNaivePosition<T: PositionNum> {
    /// Convert to a `NaivePosition`.
    fn to_naive(&self) -> NaivePosition<T>;
}

impl<T: PositionNum, H: Clone + IntoNaivePosition<T>> ToNaivePosition<T> for H {
    fn to_naive(&self) -> NaivePosition<T> {
        self.clone().into_naive()
    }
}

impl<T: PositionNum, H> AddAssign<H> for NaivePosition<T>
where
    H: IntoNaivePosition<T>,
{
    fn add_assign(&mut self, rhs: H) {
        let mut rhs = rhs.into_naive();
        if self.size.abs() <= rhs.size.abs() {
            core::mem::swap(self, &mut rhs);
        }
        if rhs.size.is_zero() {
            self.value += rhs.value;
        } else if (self.size.is_positive() && rhs.size.is_positive())
            || (self.size.is_negative() && rhs.size.is_negative())
        {
            let mut v1 = self.price.clone();
            v1 *= &self.size;
            rhs.price *= &rhs.size;
            let mut total = self.size.clone();
            total += &rhs.size;
            self.price = v1 + rhs.price;
            self.price /= total;
            self.size += rhs.size;
            self.value += rhs.value;
        } else {
            self.size += &rhs.size;
            rhs.price -= &self.price;
            self.value += rhs.value + rhs.price * rhs.size.neg();
        }
    }
}

impl<T: PositionNum, H> Add<H> for NaivePosition<T>
where
    H: IntoNaivePosition<T>,
{
    type Output = Self;

    fn add(mut self, rhs: H) -> Self::Output {
        let rhs = rhs.into_naive();
        self += rhs;
        self
    }
}

impl<T: PositionNum> Neg for NaivePosition<T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            price: self.price,
            size: self.size.neg(),
            value: self.value.neg(),
        }
    }
}

impl<T: PositionNum, H> Sub<H> for NaivePosition<T>
where
    H: IntoNaivePosition<T>,
{
    type Output = Self;

    fn sub(self, rhs: H) -> Self::Output {
        let rhs = rhs.into_naive().neg();
        self.add(rhs)
    }
}

impl<T: PositionNum, H> SubAssign<H> for NaivePosition<T>
where
    H: IntoNaivePosition<T>,
{
    fn sub_assign(&mut self, rhs: H) {
        self.add_assign(rhs.into_naive().neg());
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic_ops() {
        let h = NaivePosition::default();
        assert_eq!(h + 2, 2);
        assert_eq!(h + (3, 4), (3, 4));
        assert_eq!(h + (5, 6, 7), (5, 6, 7));
        assert_eq!(h + (5, 6, 7) + (5, 6, 7), (5, 12, 14));
        assert_eq!(h + (5, 1, 7) + (7, -1, 0), (1, 0, 9));
        assert_eq!(h + (5, 1, 7) + (7, -1, 1), 10);
        assert_eq!(h + (5, 2, 7) + (7, -1, 0), (5, 1, 9));
        assert_eq!(h + (5, 2, 8), (1, 2, 0));
        assert_eq!(h + (5, 2, 7) - (5, 2, 7), 0);
    }

    #[test]
    fn basic_consuming() {
        let mut h = NaivePosition::new(1, 2, 0);
        let p = (h + 4).consumed().unwrap();
        assert_eq!(p.value, 0);
        assert_eq!(p.price, -1);
        assert_eq!(p.size, 2);
        h += 4;
        h.consume();
        assert_eq!(h.value, 0);
        assert_eq!(h.price, -1);
        assert_eq!(h.size, 2);
    }
}
