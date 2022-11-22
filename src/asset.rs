use alloc::fmt;
use arcstr::{literal, ArcStr};
use core::{hash::Hash, ops::Deref, str::FromStr};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{PositionNum, Positions};

/// Asset.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Asset {
    #[cfg_attr(feature = "serde", serde(flatten))]
    inner: ArcStr,
}

impl fmt::Display for Asset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl<'a> From<&'a ArcStr> for Asset {
    fn from(value: &'a ArcStr) -> Self {
        let s = value.to_uppercase();
        if s == *value {
            Self {
                inner: value.clone(),
            }
        } else {
            Self {
                inner: ArcStr::from(s),
            }
        }
    }
}

impl FromStr for Asset {
    type Err = core::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            inner: ArcStr::from(s.to_uppercase()),
        })
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

impl Asset {
    /// Usdt.
    pub fn usdt() -> Self {
        Self {
            inner: literal!("USDT"),
        }
    }

    /// Usd.
    pub fn usd() -> Self {
        Self {
            inner: literal!("USD"),
        }
    }

    /// Btc.
    pub fn btc() -> Self {
        Self {
            inner: literal!("BTC"),
        }
    }

    /// Eth.
    pub fn eth() -> Self {
        Self {
            inner: literal!("ETH"),
        }
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
