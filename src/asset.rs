use alloc::{fmt, string::String};
use core::{borrow::Borrow, hash::Hash, ops::Deref, str::FromStr};
use smol_str::SmolStr as Str;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{PositionNum, Positions};

/// Asset.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(from = "Str", into = "Str"))]
pub struct Asset {
    inner: Str,
}

impl fmt::Display for Asset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

// impl From<Asset> for Str {
//     #[inline]
//     fn from(asset: Asset) -> Self {
//         asset.inner
//     }
// }

impl From<Str> for Asset {
    fn from(value: Str) -> Self {
        let s = value.to_uppercase();
        if s == value {
            Self { inner: value }
        } else {
            Self {
                inner: Str::from(s),
            }
        }
    }
}

impl From<String> for Asset {
    #[inline]
    fn from(s: String) -> Self {
        Self {
            inner: Str::from(s.to_uppercase()),
        }
    }
}

impl<'a> From<&'a str> for Asset {
    #[inline]
    fn from(s: &'a str) -> Self {
        Self {
            inner: Str::from(s.to_uppercase()),
        }
    }
}

impl<'a> From<&'a Str> for Asset {
    fn from(value: &'a Str) -> Self {
        let s = value.to_uppercase();
        if s == *value {
            Self {
                inner: value.clone(),
            }
        } else {
            Self {
                inner: Str::from(s),
            }
        }
    }
}

impl FromStr for Asset {
    type Err = core::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

impl Deref for Asset {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AsRef<str> for Asset {
    fn as_ref(&self) -> &str {
        self.inner.as_str()
    }
}

impl Borrow<str> for Asset {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<'a> Borrow<str> for &'a Asset {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Asset {
    /// Usdt.
    pub const USDT: Self = Self::new_inline("USDT");
    /// Usd.
    pub const USD: Self = Self::new_inline("USD");
    /// Btc.
    pub const BTC: Self = Self::new_inline("BTC");
    /// Eth.
    pub const ETH: Self = Self::new_inline("ETH");

    /// Create a new [`Asset`] from an "inline" str.
    /// # Panic
    /// Panics if s.len() > 22.
    /// # Warning
    /// Must make sure the asset format is valid.
    const fn new_inline(s: &str) -> Self {
        Self {
            inner: Str::new_inline(s),
        }
    }

    /// Usdt.
    pub fn usdt() -> Self {
        Self::USDT
    }

    /// Usd.
    pub fn usd() -> Self {
        Self::USD
    }

    /// Btc.
    pub fn btc() -> Self {
        Self::BTC
    }

    /// Eth.
    pub fn eth() -> Self {
        Self::ETH
    }

    /// Convert to [`&str`]
    pub fn as_str(&self) -> &str {
        self.inner.as_str()
    }

    /// Create a [`Positions`] with only value of this asset.
    pub fn to_positions<T>(&self, value: T) -> Positions<T>
    where
        T: PositionNum,
    {
        let mut p = Positions::default();
        p.insert_value(value, self);
        p
    }
}

impl PartialEq<str> for Asset {
    fn eq(&self, other: &str) -> bool {
        self.inner.eq_ignore_ascii_case(other)
    }
}

impl<'a> PartialEq<&'a str> for Asset {
    fn eq(&self, other: &&'a str) -> bool {
        self.inner.eq_ignore_ascii_case(other)
    }
}

impl PartialEq<String> for Asset {
    fn eq(&self, other: &String) -> bool {
        self == other.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str() {
        let asset = Asset::from("usdt");
        assert_eq!(asset.as_str(), "USDT");
    }

    #[test]
    fn equal() {
        let asset = Asset::from("usdt");
        assert_eq!(asset, Asset::usdt());
        assert_eq!(asset, *"usdt");
        assert_eq!(asset, "usdt");
        assert_eq!(asset, String::from("uSdt"));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde() -> anyhow::Result<()> {
        use alloc::{vec, vec::Vec};

        let value = serde_json::json!(["usdt", "BTC"]);
        let assets: Vec<Asset> = serde_json::from_value(value)?;
        assert_eq!(assets, [Asset::usdt(), Asset::btc()]);
        let s = serde_json::to_string(&assets)?;
        assert_eq!(s, r#"["USDT","BTC"]"#);
        Ok(())
    }
}
