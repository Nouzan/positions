use alloc::fmt;
use core::{hash::Hash, ops::Deref, str::FromStr};
use smol_str::SmolStr;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Asset.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Asset {
    #[cfg_attr(feature = "serde", serde(flatten))]
    inner: SmolStr,
}

impl fmt::Display for Asset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl FromStr for Asset {
    type Err = ParseAssetError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            inner: SmolStr::new(s.to_uppercase()),
        })
    }
}

impl Deref for Asset {
    type Target = SmolStr;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AsRef<str> for Asset {
    fn as_ref(&self) -> &str {
        self.inner.as_str()
    }
}

impl Asset {
    /// Usdt.
    pub fn usdt() -> Self {
        Self {
            inner: SmolStr::new_inline("USDT"),
        }
    }

    /// Usd.
    pub fn usd() -> Self {
        Self {
            inner: SmolStr::new_inline("USD"),
        }
    }

    /// Btc.
    pub fn btc() -> Self {
        Self {
            inner: SmolStr::new_inline("BTC"),
        }
    }

    /// Eth.
    pub fn eth() -> Self {
        Self {
            inner: SmolStr::new_inline("ETH"),
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
