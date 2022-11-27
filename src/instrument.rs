use core::{borrow::Borrow, hash::Hash};

use alloc::fmt;
use arcstr::ArcStr;

use crate::{asset::Asset, IntoNaivePosition, Position, PositionNum};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Instrument.
#[derive(Debug, Clone, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Instrument {
    prefer_reversed: bool,
    derivative: bool,
    pub(crate) symbol: ArcStr,
    base: Asset,
    quote: Asset,
}

impl Instrument {
    /// Create a new instrument.
    pub fn new(symbol: impl AsRef<str>, base: Asset, quote: Asset) -> Self {
        Self {
            prefer_reversed: false,
            derivative: true,
            symbol: ArcStr::from(symbol.as_ref()),
            base,
            quote,
        }
    }

    /// Create with symbol provided by `ArcStr`.
    pub fn with_arcstr(symbol: ArcStr, base: Asset, quote: Asset) -> Self {
        Self {
            prefer_reversed: false,
            derivative: true,
            symbol,
            base,
            quote,
        }
    }

    /// Whether to mark this instrument as a reversed-prefering.
    /// Default to `false`.
    pub fn prefer_reversed(mut self, reversed: bool) -> Self {
        self.prefer_reversed = reversed;
        self
    }

    /// Whether to mark this insturment as a derivative.
    /// Default to `true` if constructed by [`Instrument::new`],
    /// and `false` if constructed from [`Asset`] pairs.
    pub fn derivative(mut self, derivative: bool) -> Self {
        self.derivative = derivative;
        self
    }

    /// Is this instrument reversed-prefering.
    /// Default to `false`.
    pub fn is_prefer_reversed(&self) -> bool {
        self.prefer_reversed
    }

    /// Is this instrument a derivative.
    /// Default to `true` if constructed by [`Instrument::new`],
    /// and `false` if constructed from [`Asset`] pairs.
    pub fn is_derivative(&self) -> bool {
        self.derivative
    }

    /// Get the symbol.
    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    /// Get the base asset.
    pub fn base(&self) -> &Asset {
        &self.base
    }

    /// Get the quote asset.
    pub fn quote(&self) -> &Asset {
        &self.quote
    }

    /// Create a [`Position`] with the given position of this instrument.
    #[inline]
    pub fn into_position<T, P>(self, position: P) -> Position<T>
    where
        T: PositionNum,
        P: IntoNaivePosition<T>,
    {
        Position::new(self, position)
    }
}

impl From<(Asset, Asset)> for Instrument {
    fn from((base, quote): (Asset, Asset)) -> Self {
        Self {
            prefer_reversed: false,
            derivative: false,
            symbol: arcstr::format!("{base}-{quote}"),
            base,
            quote,
        }
    }
}

impl fmt::Display for Instrument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

impl PartialEq for Instrument {
    fn eq(&self, other: &Self) -> bool {
        self.prefer_reversed == other.prefer_reversed
            && self.derivative == other.derivative
            && self.symbol == other.symbol
            && self.base == other.base
            && self.quote == other.quote
    }
}

impl Eq for Instrument {}

impl Hash for Instrument {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.symbol.hash(state);
    }
}

impl Borrow<str> for Instrument {
    fn borrow(&self) -> &str {
        self.symbol()
    }
}

impl<'a> Borrow<str> for &'a Instrument {
    fn borrow(&self) -> &str {
        self.symbol()
    }
}
