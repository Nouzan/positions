pub use self::{
    node::{PositionNode, ValueNode},
    stronge::{tree, PositionTree},
    weak::WeakTree,
};

/// Node.
pub mod node;

/// Weak Tree.
pub mod weak;

/// Storage Tree.
pub mod stronge;

mod utils {
    use crate::{asset::Asset, PositionNum};
    use alloc::fmt;

    pub(super) fn write_position<T>(
        f: &mut fmt::Formatter<'_>,
        price: &T,
        size: &T,
        asset: &Asset,
    ) -> fmt::Result
    where
        T: fmt::Display + PositionNum,
    {
        if asset.is_prefer_reversed() {
            if price.is_zero() {
                write!(f, "(Nan, {} {asset})*", -size.clone())
            } else {
                write!(f, "({}, {} {asset})*", T::one() / price, -size.clone())
            }
        } else {
            write!(f, "({price}, {size} {asset})")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{asset::Asset, Reversed};
    use core::str::FromStr;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;

    #[test]
    fn basic() {
        let usdt = Asset::usdt();
        let btc = Asset::btc();
        let btcusdt_swap = Asset::from_str("btc-usdt-swap").unwrap().value_contained();
        let btcusd_swap = Asset::from_str("btc-usd-swap")
            .unwrap()
            .value_contained()
            .prefer_reversed();
        let mut p = tree(&usdt);
        p += (dec!(2), &btc);
        *p += (dec!(16000), dec!(12), &btcusdt_swap);
        p += (dec!(-1), &usdt);
        *p += (dec!(14000), dec!(-2), &btcusdt_swap);
        println!("{p}");
        let mut q = tree(&btc);
        q += (dec!(2), &btc);
        *q += (dec!(1) / dec!(16000), dec!(-200), &btcusd_swap);
        println!("{q}");
        p += q;
        println!("{p}");
        *p.get_weak_mut(&btc).unwrap() += (dec!(0), dec!(1), &btc);
        println!("{p}");
        *p.get_weak_mut(&btc).unwrap() += dec!(-1);
        println!("{p}");
        for (a, b) in p.all_pairs() {
            if a.is_value_contained() {
                println!("{a}");
            } else {
                println!("{a}-{b}");
            }
        }
    }

    #[test]
    fn reversed() {
        let usdt = Asset::usdt();
        let btc = Asset::btc();
        let btc_usd_swap = Asset::from_str("BTC-USDT-SWAP")
            .unwrap()
            .prefer_reversed()
            .value_contained();
        let mut p = tree(&usdt);
        p += (dec!(-16000), &usdt);
        p += (dec!(1), &btc);
        *p.get_weak_mut(&btc).unwrap() += Reversed((dec!(16000), dec!(-16000), &btc_usd_swap));
        println!("{p}");
        let mut prices = HashMap::default();
        for (a, b) in p.all_pairs() {
            if a.is_value_contained() {
                println!("{a}");
            } else {
                println!("{a}-{b}");
            }
            if a.is_prefer_reversed() {
                prices.insert((a, b), dec!(1) / dec!(17000));
            } else {
                prices.insert((a, b), dec!(17000));
            }
        }
        println!("{}", p.eval(&prices).unwrap());
    }
}
