use super::{IntoNaivePosition, NaivePosition, Normal, PositionNum, Representation, Reversed};
use std::fmt;
use std::marker::PhantomData;
use std::ops::{Add, Neg, Sub};

/// Position.
#[derive(Debug)]
pub struct Position<Rep: Representation, T: PositionNum> {
    naive: NaivePosition<T>,
    _rep: PhantomData<Rep>,
}

impl<Rep: Representation, T: PositionNum + fmt::Display> fmt::Display for Position<Rep, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mark = if Rep::is_reversed() { "R" } else { "N" };
        let price = self
            .price()
            .map(|p| p.to_string())
            .unwrap_or("NaN".to_string());
        let size = self.size();
        let value = self.value();
        write!(f, "{}({}, {}) + {}", mark, price, size, value)
    }
}

impl<Rep: Representation, T: PositionNum> Clone for Position<Rep, T> {
    fn clone(&self) -> Self {
        Self {
            naive: self.naive.clone(),
            _rep: PhantomData::default(),
        }
    }
}

impl<Rep: Representation, T: PositionNum + Copy> Copy for Position<Rep, T> {}

impl<Rep: Representation, T: PositionNum> Default for Position<Rep, T> {
    fn default() -> Self {
        Self {
            naive: NaivePosition::default(),
            _rep: PhantomData::default(),
        }
    }
}

impl<Rep: Representation, T: PositionNum> Position<Rep, T> {
    /// Create a [`Position`] directly from [`IntoNaivePosition`].
    fn with_naive<H: IntoNaivePosition<T>>(naive: H) -> Self {
        Self {
            naive: naive.into_naive_position(),
            _rep: PhantomData::default(),
        }
    }

    /// Create a new [`Position`].
    pub fn new(price: T, size: T, value: T) -> Option<Self> {
        if Rep::is_reversed() {
            if price.is_zero() {
                None
            } else {
                let price = T::one() / price;
                let size = size.neg();
                Some(Self::with_naive(NaivePosition::new(price, size, value)))
            }
        } else {
            Some(Self::with_naive(NaivePosition::new(price, size, value)))
        }
    }

    /// Create a new [`Position`] from [`IntoNaivePosition`].
    ///
    /// This method will convert the price and size of `naive` by `Rep`.
    pub fn from_naive<H: IntoNaivePosition<T>>(naive: H) -> Option<Self> {
        let NaivePosition { price, size, value } = naive.into_naive_position();
        Self::new(price, size, value)
    }

    /// Return a new position that consumes its `value`. (Equivalence I).
    ///
    /// Return `None` if `size` is zero.
    pub fn consumed(&self) -> Option<Self> {
        self.naive.consumed().map(Self::with_naive)
    }

    /// Consume `value`.
    ///
    /// No effect if `size` is zero.
    pub fn consume(&mut self) {
        self.naive.consume();
    }

    /// Return a new position with the given `price`
    /// but keep equivalent to the original position.
    /// (Equivalence II)
    pub fn converted(&self, price: T) -> Self {
        Self::with_naive(self.naive.converted(price))
    }

    /// Convert the price to the given
    /// but keep equivalent to the original.
    /// (Equivalence II)
    pub fn convert(&mut self, price: T) {
        self.naive.convert(price);
    }

    /// Price in `Rep` representation.
    ///
    /// Return `None` if naive price is zero when `Rep` is reversed.
    pub fn price(&self) -> Option<T> {
        if Rep::is_reversed() {
            if self.naive.price.is_zero() {
                None
            } else {
                Some(T::one() / self.naive.price.clone())
            }
        } else {
            Some(self.naive.price.clone())
        }
    }

    /// Size in `Rep` representation.
    pub fn size(&self) -> T {
        if Rep::is_reversed() {
            self.naive.size.clone().neg()
        } else {
            self.naive.size.clone()
        }
    }

    /// Value.
    pub fn value(&self) -> &T {
        &self.naive.value
    }

    /// Naive price.
    pub fn naive_price(&self) -> &T {
        &self.naive.price
    }

    /// Naive size.
    pub fn naive_size(&self) -> &T {
        &self.naive.size
    }

    /// Take the `value` and keep the `price` and `size` unchanged.
    ///
    /// After the operation, the new position is no longer
    /// equivalent to the original.
    pub fn take(&mut self) -> T {
        self.naive.take()
    }
}

impl<Rep: Representation, T: PositionNum> IntoNaivePosition<T> for Position<Rep, T> {
    fn into_naive_position(self) -> NaivePosition<T> {
        self.naive
    }
}

impl<Rep: Representation, T: PositionNum> PartialEq for Position<Rep, T> {
    fn eq(&self, other: &Self) -> bool {
        self.naive.eq(&other.naive)
    }
}

impl<Rep: Representation, T: PositionNum> Eq for Position<Rep, T> {}

impl<Rep: Representation, T: PositionNum> Add for Position<Rep, T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::with_naive(self.naive.add(rhs.naive))
    }
}

impl<Rep: Representation, T: PositionNum> Neg for Position<Rep, T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::with_naive(self.naive.neg())
    }
}

impl<Rep: Representation, T: PositionNum> Sub for Position<Rep, T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self.add(rhs.neg())
    }
}

/// Create a normal representation position from naive.
///
/// # Example
/// ```
/// use positions::normal;
///
/// let h = normal((1, 2, 3));
/// assert_eq!(h.price().unwrap(), 1);
/// assert_eq!(h.size(), 2);
/// assert_eq!(*h.value(), 3);
/// assert_eq!(*h.naive_price(), 1);
/// assert_eq!(*h.naive_size(), 2);
///
/// ```
pub fn normal<T: PositionNum, H: IntoNaivePosition<T>>(naive: H) -> Position<Normal, T> {
    Position::from_naive(naive).unwrap()
}

/// Create a reversed representation position from naive.
///
/// # Panics
/// Panic if naive `price` is zero.
///
/// # Example
/// ```
/// use positions::reversed;
///
/// let h = reversed((2.0, 2.0, 3.0));
/// assert_eq!(h.price().unwrap(), 2.0);
/// assert_eq!(h.size(), 2.0);
/// assert_eq!(*h.value(), 3.0);
/// assert_eq!(*h.naive_price(), 0.5);
/// assert_eq!(*h.naive_size(), -2.0);
///
/// ```
pub fn reversed<T: PositionNum, H: IntoNaivePosition<T>>(naive: H) -> Position<Reversed, T> {
    Position::from_naive(naive).expect("`price` cannot be zero in reversed representation.")
}

/// Create position with the given representation from naive.
///
/// # Panics
/// Panic if naive `price` is zero.
///
/// # Example
/// ```
/// use positions::{position, Reversed};
///
/// let h = position::<Reversed, _, _>((2.0, 2.0, 3.0));
/// assert_eq!(h.price().unwrap(), 2.0);
/// assert_eq!(h.size(), 2.0);
/// assert_eq!(*h.value(), 3.0);
/// assert_eq!(*h.naive_price(), 0.5);
/// assert_eq!(*h.naive_size(), -2.0);
///
/// ```
pub fn position<Rep: Representation, T: PositionNum, H: IntoNaivePosition<T>>(
    naive: H,
) -> Position<Rep, T> {
    Position::from_naive(naive).expect("`price` cannot be zero in reversed representation.")
}
