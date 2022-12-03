use core::{borrow::Borrow, hash::Hash, str::FromStr};

use crate::{
    asset::{Asset, ParseAssetError},
    prelude::Str,
    IntoNaivePosition, Position, PositionNum,
};
use alloc::fmt;

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Instrument.
#[derive(Debug, Clone, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Instrument {
    prefer_reversed: bool,
    symbol: Symbol,
    base: Asset,
    quote: Asset,
}

impl Instrument {
    /// Create a new instrument.
    /// Return [`ParseSymbolError`] if the format of the `symbol` is not valid.
    pub fn try_new(symbol: &str, base: &Asset, quote: &Asset) -> Result<Self, ParseSymbolError> {
        Self::try_with_symbol(Symbol::try_from(symbol)?, base, quote)
    }

    /// Create a new spot.
    pub fn spot(base: &Asset, quote: &Asset) -> Self {
        Self {
            prefer_reversed: false,
            symbol: Symbol::spot(base, quote),
            base: base.clone(),
            quote: quote.clone(),
        }
    }

    /// Create a new derivative.
    /// Return [`ParseSymbolError`] if the `prefix` is not valid.
    pub fn derivative(
        prefix: &str,
        symbol: &str,
        base: &Asset,
        quote: &Asset,
    ) -> Result<Self, ParseSymbolError> {
        let symbol = Symbol::derivative(prefix, symbol)?;
        Ok(Self {
            prefer_reversed: false,
            symbol,
            base: base.clone(),
            quote: quote.clone(),
        })
    }

    /// Convert to the revsered spot.
    /// Return [`None`] if it is not a spot.
    pub fn to_reversed_spot(&self) -> Option<Self> {
        let symbol = self.symbol.to_reversed_symbol()?;
        Some(Self {
            prefer_reversed: self.prefer_reversed,
            symbol,
            base: self.quote.clone(),
            quote: self.base.clone(),
        })
    }

    /// Create a new instrument with the given symbol.
    /// Return [`ParseSymbolError`] if the `symbol` does not match the given `base` or `quote`.
    pub fn try_with_symbol(
        symbol: Symbol,
        base: &Asset,
        quote: &Asset,
    ) -> Result<Self, ParseSymbolError> {
        if let Some(pair) = symbol.as_spot() {
            if pair != (base, quote) {
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

    /// Get the symbol.
    #[inline]
    pub fn as_symbol(&self) -> &Symbol {
        &self.symbol
    }

    /// Is spot.
    #[inline]
    pub fn is_spot(&self) -> bool {
        self.symbol.is_spot()
    }

    /// Is derivative.
    #[inline]
    pub fn is_derivative(&self) -> bool {
        self.symbol.is_derivative()
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
    pub fn position<T, P>(&self, position: P) -> Position<T>
    where
        T: PositionNum,
        P: IntoNaivePosition<T>,
    {
        Position::new(self.clone(), position)
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

impl AsRef<Symbol> for Instrument {
    fn as_ref(&self) -> &Symbol {
        &self.symbol
    }
}

/// Symbol.
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Symbol(Repr);

impl Symbol {
    /// Empty str, the prefix of spot instruments.
    pub const SPOT_PREFIX: Str = Str::new_inline("");

    /// The delimiter of prefix and symbol.
    pub const SEP: char = ':';

    /// Is a derivative.
    pub fn is_derivative(&self) -> bool {
        matches!(self.0, Repr::Derivative(_, _))
    }

    /// Get the prefix and the symbol of the derivative.
    /// Return [`None`] if it is not a derivative.
    #[inline]
    pub fn as_derivative(&self) -> Option<(&str, &str)> {
        let Repr::Derivative(prefix, symbol) = &self.0 else {
            return None;
        };
        Some((prefix.as_str(), symbol.as_str()))
    }

    /// Get the prefix of the derivative.
    /// Spots has no prefix.
    #[inline]
    pub fn derivative_prefix(&self) -> Option<&str> {
        Some(self.as_derivative()?.0)
    }

    /// Get the symbol of the derivative.
    /// Return [`None`] if it is not a derivative.
    #[inline]
    pub fn derivative_symbol(&self) -> Option<&str> {
        Some(self.as_derivative()?.1)
    }

    /// Is a spot.
    #[inline]
    pub fn is_spot(&self) -> bool {
        matches!(self.0, Repr::Spot(_, _))
    }

    /// As a pair of assets.
    /// Return [`None`] if it is not a spot.
    #[inline]
    pub fn as_spot(&self) -> Option<(&Asset, &Asset)> {
        let Repr::Spot(base, quote) = &self.0 else {
            return None;
        };
        Some((base, quote))
    }

    /// Create a spot symbol.
    pub fn spot(base: &Asset, quote: &Asset) -> Self {
        Self(Repr::spot(base, quote))
    }

    /// Get the reversed spot.
    /// Return [`None`] if it is not a spot.
    pub fn to_reversed_symbol(&self) -> Option<Self> {
        let (base, quote) = self.as_spot()?;
        Some(Self::spot(quote, base))
    }

    /// Create a derivative symbol.
    /// Return [`ParseSymbolError`] if the prefix is not valid.
    pub fn derivative(prefix: &str, symbol: &str) -> Result<Self, ParseSymbolError> {
        Ok(Self(Repr::derivative(prefix, symbol)?))
    }
}

/// The internal representation of a symbol.
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "Str", into = "String"))]
enum Repr {
    /// Spot.
    Spot(Asset, Asset),
    /// Derivative
    Derivative(Str, Str),
}

impl Repr {
    #[inline]
    fn spot(base: &Asset, quote: &Asset) -> Self {
        Self::Spot(base.clone(), quote.clone())
    }

    #[inline]
    fn derivative(prefix: &str, symbol: &str) -> Result<Self, ParseSymbolError> {
        if prefix.contains(Symbol::SEP) {
            Err(ParseSymbolError::InvalidPrefix)
        } else {
            Ok(Self::Derivative(Str::new(prefix), Str::new(symbol)))
        }
    }
}

impl From<Repr> for String {
    fn from(symbol: Repr) -> Self {
        symbol.to_string()
    }
}

impl fmt::Display for Repr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spot(base, quote) => write!(f, "{base}{}{quote}", Asset::SEP),
            Self::Derivative(prefix, symbol) => write!(f, "{prefix}{}{symbol}", Symbol::SEP),
        }
    }
}

impl From<Symbol> for String {
    fn from(symbol: Symbol) -> Self {
        symbol.to_string()
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
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

impl<'a> TryFrom<&'a str> for Repr {
    type Error = ParseSymbolError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value.split_once(Symbol::SEP) {
            Some((prefix, symbol)) => Self::derivative(prefix, symbol),
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

impl TryFrom<Str> for Repr {
    type Error = ParseSymbolError;

    fn try_from(value: Str) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl<'a> TryFrom<&'a str> for Symbol {
    type Error = ParseSymbolError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Ok(Self(Repr::try_from(value)?))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbol_from_str() {
        let swap = Symbol::from_str("SWAP:btcusdt").unwrap();
        assert!(!swap.is_spot());
        assert_eq!(swap, Symbol::derivative("SWAP", "btcusdt").unwrap());
        assert_eq!(swap.to_string(), "SWAP:btcusdt");
        let spot = Symbol::from_str("btc-usdt").unwrap();
        assert!(spot.is_spot());
        assert_eq!(spot, Symbol::spot(&Asset::BTC, &Asset::USDT));
        assert_eq!(spot.to_string(), "BTC-USDT");
    }

    #[test]
    fn reversed_spot_symbol() {
        let spot: Symbol = "BTC-USDT".parse().unwrap();
        assert_eq!(
            spot.to_reversed_symbol(),
            Some(Symbol::spot(&Asset::USDT, &Asset::BTC))
        );
    }

    #[test]
    fn reversed_spot() {
        let spot = Instrument::spot(&Asset::BTC, &Asset::USDT);
        assert_eq!(
            spot.to_reversed_spot(),
            Some(Instrument::spot(&Asset::USDT, &Asset::BTC))
        );
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
                Symbol::derivative("futures", "BTC-USDT-210101").unwrap(),
                Symbol::spot(&Asset::USDT, &Asset::BTC)
            ]
        );
        let s = serde_json::to_string(&assets)?;
        assert_eq!(s, r#"["futures:BTC-USDT-210101","USDT-BTC"]"#);
        Ok(())
    }
}
