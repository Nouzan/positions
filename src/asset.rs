use core::{cmp::Ordering, hash::Hash, ops::Deref, str::FromStr};

use alloc::{fmt, string::String, sync::Arc};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "serde")]
use serde_with::{serde_as, DisplayFromStr};

/// Asset.
#[cfg(feature = "serde")]
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    prefer_reversed: bool,
    value_contained: bool,
    #[serde_as(as = "DisplayFromStr")]
    kind: AssetKind,
}

/// Asset.
#[cfg(not(feature = "serde"))]
#[derive(Debug, Clone)]
pub struct Asset {
    prefer_reversed: bool,
    value_contained: bool,
    kind: AssetKind,
}

impl Asset {
    /// Is prefer reversed.
    pub fn is_prefer_reversed(&self) -> bool {
        self.prefer_reversed
    }

    /// Is value contained.
    pub fn is_value_contained(&self) -> bool {
        self.value_contained
    }

    /// Set prefering reversed to be `true`.
    pub fn prefer_reversed(mut self) -> Self {
        self.prefer_reversed = true;
        self
    }

    /// Set `value_contained` to be `true`.
    pub fn value_contained(mut self) -> Self {
        self.value_contained = true;
        self
    }

    /// Usdt.
    pub fn usdt() -> Self {
        Self::from(AssetKind::Usdt)
    }

    /// Usd.
    pub fn usd() -> Self {
        Self::from(AssetKind::Usd)
    }

    /// Btc.
    pub fn btc() -> Self {
        Self::from(AssetKind::Btc)
    }

    /// Eth.
    pub fn eth() -> Self {
        Self::from(AssetKind::Eth)
    }
}

impl From<AssetKind> for Asset {
    fn from(kind: AssetKind) -> Self {
        Self {
            prefer_reversed: false,
            value_contained: false,
            kind,
        }
    }
}

impl Deref for Asset {
    type Target = AssetKind;

    fn deref(&self) -> &Self::Target {
        &self.kind
    }
}

impl fmt::Display for Asset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl FromStr for Asset {
    type Err = ParseAssetError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let kind = AssetKind::from_str(s)?;
        Ok(Self {
            prefer_reversed: false,
            value_contained: false,
            kind,
        })
    }
}

impl PartialEq for Asset {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl Eq for Asset {}

impl Hash for Asset {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
    }
}

impl PartialOrd for Asset {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.kind.partial_cmp(&other.kind)
    }
}

impl Ord for Asset {
    fn cmp(&self, other: &Self) -> Ordering {
        self.kind.cmp(&other.kind)
    }
}

/// Asset Kind.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AssetKind {
    /// USD.
    Usd,
    /// USDT.
    Usdt,
    /// BTC.
    Btc,
    /// ETH.
    Eth,
    /// Extension.
    Extesntion(Arc<String>),
}

impl AssetKind {
    /// Conver to str.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Usd => "USD",
            Self::Usdt => "USDT",
            Self::Btc => "BTC",
            Self::Eth => "ETH",
            Self::Extesntion(asset) => asset.as_str(),
        }
    }
}

/// Parse Asset Error.
#[derive(Debug)]
#[cfg_attr(feature = "thiserror", derive(thiserror::Error))]
pub enum ParseAssetError {}

#[cfg(not(feature = "thiserror"))]
impl fmt::Display for ParseAssetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parse Asset Error")
    }
}

impl fmt::Display for AssetKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for AssetKind {
    type Err = ParseAssetError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "USD" | "usd" | "Usd" => Ok(Self::Usd),
            "USDT" | "usdt" | "Usdt" => Ok(Self::Usdt),
            "BTC" | "btc" | "Btc" => Ok(Self::Btc),
            "ETH" | "eth" | "Eth" => Ok(Self::Eth),
            s => Ok(Self::Extesntion(Arc::new(s.to_uppercase()))),
        }
    }
}

impl PartialOrd for AssetKind {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl Ord for AssetKind {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}
