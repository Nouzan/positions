use positions::prelude::*;
use rust_decimal_macros::dec;

fn main() {
    // Firstly, we open a position of 1.5 BTC at 16000 USDT/BTC.
    let inst = Instrument::spot(&Asset::BTC, &Asset::USDT);
    let mut p = inst.position((dec!(16000), dec!(1.5)));

    // Later, we add 1.5 BTC to the position at 15000 USDT/BTC.
    p += (dec!(15000), dec!(1.5));
    // The total position now should be holding 3.0 BTC at the cost of 1550 USDT/BTC.
    assert_eq!(p, inst.position((dec!(15500), dec!(3.0))));

    // Finally, we close all the position at 15700 USDT/BTC,
    p += (dec!(15700), -dec!(3.0));
    // which make us a profit of 600 USDT.
    assert_eq!(*p.value(), dec!(600));

    // And now we should have no positions.
    assert_eq!(p.size(), dec!(0));

    // If we take the profit,
    assert_eq!(p.take(), dec!(600));
    // then we will have a "true" zero position.
    assert!(p.is_zero());

    // The same calculation should also work for the short positions.
    let mut p = inst.position((dec!(16000), dec!(-1.5)));
    p += (dec!(15000), dec!(-1.5));
    assert_eq!(p, inst.position((dec!(15500), dec!(-3.0))));
    p += (dec!(15700), dec!(3.0));
    assert_eq!(p.take(), dec!(-600));
    assert!(p.is_zero());
}
