use core::{cmp::Ordering, str::FromStr};

use alloc::{fmt, string::String, sync::Arc};

/// Asset.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Asset {
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

impl Asset {
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

impl fmt::Display for Asset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Asset {
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

impl PartialOrd for Asset {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl Ord for Asset {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}
