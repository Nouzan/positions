use smol_str::SmolStr;

use crate::asset::Asset;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Instrument.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Instrument {
    symbol: SmolStr,
    base: Asset,
    quote: Asset,
}

impl Instrument {
    /// Create a new instrument.
    pub fn new(symbol: impl AsRef<str>, base: Asset, quote: Asset) -> Self {
        Self {
            symbol: SmolStr::new(symbol),
            base,
            quote,
        }
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
            symbol: SmolStr::new(format!("{base}-{quote}")),
            base,
            quote,
        }
    }
}
