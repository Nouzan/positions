use core::str::FromStr;

use alloc::{borrow::Cow, fmt, string::String};

/// Asset.
#[derive(Debug, Clone)]
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
    Extesntion(Cow<'static, String>),
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
            s => Ok(Self::Extesntion(Cow::Owned(s.to_uppercase()))),
        }
    }
}
