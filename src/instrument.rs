use core::{borrow::Borrow, hash::Hash, ops::Deref, str::FromStr};

use crate::{
    asset::{Asset, ParseAssetError},
    prelude::Str,
    IntoNaivePosition, Position, PositionNum,
};
use alloc::fmt;

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Instrument.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Instrument {
    prefer_reversed: bool,
    symbol: Symbol,
    base: Asset,
    quote: Asset,
}

/// Symbol.
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "Str", into = "String"))]
pub struct Symbol {
    prefix: Str,
    symbol: Str,
}

impl Symbol {
    /// Empty str, the prefix of spot instruments.
    pub const SPOT_PREFIX: Str = Str::new_inline("");

    /// The delimiter of prefix and symbol.
    pub const SEP: char = ':';

    /// Get prefix.
    /// The prefix of spot instrument must be empty.
    pub fn prefix(&self) -> &str {
        self.prefix.as_str()
    }

    /// Is spot.
    pub fn is_spot(&self) -> bool {
        self.prefix.is_empty()
    }

    /// Get symbol.
    /// The symbol of spot instrument must be of this format: `"{base}-{quote}"`,
    /// where `base` and `quote` are in upppercase. For example, `"BTC-USDT"` is a
    /// valid spot symbol.
    pub fn symbol(&self) -> &str {
        self.symbol.as_str()
    }

    /// Create a symbol for the spot.
    pub fn spot(base: &Asset, quote: &Asset) -> Self {
        Self {
            prefix: Self::SPOT_PREFIX,
            symbol: Str::new(format!("{base}{}{quote}", Asset::SEP)),
        }
    }

    /// Create a new symbol from [`Str`].
    /// Return `None` if `prefix` is emtpy or contains [`Symbol::SEP`];
    pub fn from_raw(prefix: &Str, symbol: &Str) -> Option<Self> {
        if prefix.is_empty() || prefix.contains(Self::SEP) {
            return None;
        }
        Some(Self {
            prefix: prefix.clone(),
            symbol: symbol.clone(),
        })
    }

    /// Create a new symbol from text.
    /// Return `None` if `prefix` is emtpy or contains [`Symbol::SEP`];
    pub fn new(prefix: &str, symbol: &str) -> Option<Self> {
        if prefix.is_empty() || prefix.contains(Self::SEP) {
            return None;
        }
        Some(Self {
            prefix: Str::new(prefix),
            symbol: Str::new(symbol),
        })
    }
}

impl From<Symbol> for String {
    fn from(symbol: Symbol) -> Self {
        symbol.to_string()
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.prefix.is_empty() {
            write!(f, "{}", self.symbol)
        } else {
            write!(f, "{}{}{}", self.prefix, Self::SEP, self.symbol)
        }
    }
}

/// Parse symbol error.
#[derive(Debug)]
#[cfg_attr(feature = "thiserror", derive(thiserror::Error))]
pub enum ParseSymbolError {
    /// Invalid spot format.
    #[cfg_attr(feature = "thiserror", error("invalid spot format"))]
    InvalidSpotFormat,
    /// Invalid prefix.
    #[cfg_attr(feature = "thiserror", error("invalid prefix"))]
    InvalidPrefix,
    /// Asset errors.
    #[cfg_attr(feature = "thiserror", error("parse asset error: {0}"))]
    Asset(ParseAssetError),
}

#[cfg(not(feature = "thiserror"))]
impl fmt::Display for ParseSymbolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSpotFormat => write!(f, "invalid spot format"),
            Self::InvalidPrefix => write!(f, "invalid prefix"),
            Self::Asset(err) => write!(f, "parse asset error: {err}"),
        }
    }
}

impl From<ParseAssetError> for ParseSymbolError {
    fn from(err: ParseAssetError) -> Self {
        Self::Asset(err)
    }
}

impl<'a> TryFrom<&'a str> for Symbol {
    type Error = ParseSymbolError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value.split_once(Self::SEP) {
            Some((prefix, symbol)) => {
                Self::new(prefix, symbol).ok_or(ParseSymbolError::InvalidPrefix)
            }
            None => {
                if let Some((base, quote)) = value.split_once(Asset::SEP) {
                    let base = Asset::from_str(base)?;
                    let quote = Asset::from_str(quote)?;
                    Ok(Self::spot(&base, &quote))
                } else {
                    Err(ParseSymbolError::InvalidSpotFormat)
                }
            }
        }
    }
}

impl TryFrom<Str> for Symbol {
    type Error = ParseSymbolError;

    fn try_from(value: Str) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl FromStr for Symbol {
    type Err = ParseSymbolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

impl Instrument {
    /// Create a new instrument.
    pub fn try_new(
        symbol: impl AsRef<str>,
        base: &Asset,
        quote: &Asset,
    ) -> Result<Self, ParseSymbolError> {
        Self::try_with_symbol(Symbol::try_from(symbol.as_ref())?, base, quote)
    }

    /// Create a new spot instrument.
    pub fn spot(base: &Asset, quote: &Asset) -> Self {
        Self {
            prefer_reversed: false,
            symbol: Symbol::spot(base, quote),
            base: base.clone(),
            quote: quote.clone(),
        }
    }

    /// Create a new instrument with the given [`Str`] as symbol.
    pub fn try_with_symbol(
        symbol: Symbol,
        base: &Asset,
        quote: &Asset,
    ) -> Result<Self, ParseSymbolError> {
        if symbol.is_spot() {
            let valid_symbol = Symbol::spot(base, quote);
            if valid_symbol != symbol {
                return Err(ParseSymbolError::InvalidSpotFormat);
            }
        }
        Ok(Self {
            prefer_reversed: false,
            symbol,
            base: base.clone(),
            quote: quote.clone(),
        })
    }

    /// Whether to mark this instrument as a reversed-prefering.
    /// Default to `false`.
    pub fn prefer_reversed(mut self, reversed: bool) -> Self {
        self.prefer_reversed = reversed;
        self
    }

    /// Is this instrument reversed-prefering.
    /// Default to `false`.
    pub fn is_prefer_reversed(&self) -> bool {
        self.prefer_reversed
    }

    /// Is this instrument a derivative.
    pub fn is_derivative(&self) -> bool {
        !self.symbol.is_spot()
    }

    /// Get the symbol.
    #[inline]
    pub fn as_symbol(&self) -> &Symbol {
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
        Self::spot(&base, &quote)
    }
}

impl fmt::Display for Instrument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol)
    }
}

impl PartialEq for Instrument {
    fn eq(&self, other: &Self) -> bool {
        self.prefer_reversed == other.prefer_reversed
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

impl PartialEq<Symbol> for Instrument {
    fn eq(&self, other: &Symbol) -> bool {
        self.symbol.eq(other)
    }
}

impl Borrow<Symbol> for Instrument {
    fn borrow(&self) -> &Symbol {
        &self.symbol
    }
}

impl<'a> Borrow<Symbol> for &'a Instrument {
    fn borrow(&self) -> &Symbol {
        &self.symbol
    }
}

impl Deref for Instrument {
    type Target = Symbol;

    fn deref(&self) -> &Self::Target {
        &self.symbol
    }
}

impl AsRef<Symbol> for Instrument {
    fn as_ref(&self) -> &Symbol {
        &self.symbol
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbol_from_str() {
        let swap = Symbol::from_str("SWAP:btcusdt").unwrap();
        assert!(!swap.is_spot());
        assert_eq!(swap, Symbol::new("SWAP", "btcusdt").unwrap());
        assert_eq!(swap.to_string(), "SWAP:btcusdt");
        let spot = Symbol::from_str("btc-usdt").unwrap();
        assert!(spot.is_spot());
        assert_eq!(spot, Symbol::spot(&Asset::BTC, &Asset::USDT));
        assert_eq!(spot.to_string(), "BTC-USDT");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn symbol_serde() -> anyhow::Result<()> {
        use alloc::{vec, vec::Vec};

        let value = serde_json::json!(["futures:BTC-USDT-210101", "USDT-BTC"]);
        let assets: Vec<Symbol> = serde_json::from_value(value)?;
        assert_eq!(
            assets,
            [
                Symbol::new("futures", "BTC-USDT-210101").unwrap(),
                Symbol::spot(&Asset::USDT, &Asset::BTC)
            ]
        );
        let s = serde_json::to_string(&assets)?;
        assert_eq!(s, r#"["futures:BTC-USDT-210101","USDT-BTC"]"#);
        Ok(())
    }
}
