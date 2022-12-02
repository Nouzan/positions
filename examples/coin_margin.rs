use positions::prelude::*;
use rust_decimal_macros::dec;

fn main() {
    // Let's first define the "coin-margin" instrument.
    // `BTC-USD-SWAP` is a "coin-margin" instrument, whose base asset is `USD`.
    let inst = Instrument::try_new("SWAP:BTC-USD-SWAP", &Asset::USD, &Asset::BTC)
        .unwrap()
        .prefer_reversed(true);
    // We use the `Instrument::prefer_reversed` method to mark an instrument as a reversed instrument,
    // whose price unit and the position side that shown by the exchange (and `positions`) are actually reversed.
    //
    // Take `BTC-USD-SWAP` as an example, what it actually means that we are holding a $100 "long" position
    // of `BTC-USD-SWAP` contract at the "price" of 16000 USD/BTC is we short $100 at the price of (1/16000) BTC/USD.
    // That is what the exchange actually using in its formula when it calculates your total position as well as your profit.

    // We can represent this case directly by using `Reversed`:
    let mut p = inst.position(Reversed((dec!(16000), dec!(100))));

    // If we print it, we will see the reversed form (which is the same as what you see in the exchange) of the position.
    // That is because we have marked the `inst` to be "reversed-preferring".
    assert_eq!(p.to_string(), "(16000, 100 USD)*");

    // The return value of `Position::price` and `Position::size` methods also respect this setting.
    assert_eq!(p.price().unwrap(), dec!(16000));
    assert_eq!(p.size(), dec!(100));

    // But what is really being stored and calculated is the "true form".
    assert_eq!(p.as_naive().price, dec!(1) / dec!(16000));
    assert_eq!(p.as_naive().size, dec!(-100));

    // Let's add another reversed position to it to see what will happen.
    p += Reversed((dec!(15000), dec!(100)));
    assert_eq!(p.to_string(), "(15483.870967741935483870951759, 200 USD)*");
    // It is not the same result when we are calculating with the "true form", but this is the right answer.

    // See what we will get when we close this position.
    p += Reversed((dec!(15700), -dec!(200)));
    assert_eq!(p.take(), dec!(0.0001778131634819532908705000));
    assert!(p.is_zero());
    // That may seem like a small profit, but the unit is BTC, which is actually not small.
}
