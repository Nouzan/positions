use alloc::fmt;
use arcstr::ArcStr;

use crate::asset::Asset;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Instrument.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Instrument {
    prefer_reversed: bool,
    derivative: bool,
    symbol: ArcStr,
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
