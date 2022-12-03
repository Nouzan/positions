# Positions

A position (finance) definition with some good algebraic properties.

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docsrs-badge]][docsrs-url]
[![MIT licensed][mit-badge]][mit-url]
[![Build Status][actions-badge]][actions-url]

[API Docs][docsrs-url]

[crates-badge]: https://img.shields.io/crates/v/positions.svg
[crates-url]: https://crates.io/crates/positions
[docsrs-badge]: https://img.shields.io/docsrs/positions
[docsrs-url]: https://docs.rs/positions/latest/positions
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/Nouzan/positions/blob/master/LICENSE
[actions-badge]: https://github.com/Nouzan/positions/workflows/CI/badge.svg
[actions-url]: https://github.com/Nouzan/positions/actions?query=workflow%3ACI+branch%3Amain

## Getting Started

1. Add `positions` as a dependency of your project.

```toml
[dependencies]
positions = "0.2.1"

# `rust_decimal` is added to make the example code work,
# but optional for using `positions`.
rust_decimal = "1.26.1"
rust_decimal_macros = "1.26.1"
```

2. And now you can calculate your positions!

```rust
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
```

## Usage
### Basic positions calculation with `Position` under the "exchange rule".
```rust
use positions::prelude::*;
use rust_decimal_macros::dec;

fn main() {
    // First, we open a position of 1.5 BTC at 16000 USDT/BTC.
    let inst = Instrument::spot(&Asset::BTC, &Asset::USDT);
    let mut p = inst.position((dec!(16000), dec!(1.5)));

    // Later, we add 1.5 BTC to the position at 15000 USDT/BTC.
    p += (dec!(15000), dec!(1.5));
    // The total position now should be holding 3.0 BTC at the
    // cost of 1550 USDT/BTC.
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
```
### Calculate the "coin-margin" contracts with `Reversed`
```rust
use positions::prelude::*;
use rust_decimal_macros::dec;

fn main() {
    // Let's start with declaring a "coin-margin" instrument.
    // `BTC-USD-SWAP` is a "coin-margin" instrument, whose base asset is `USD`.
    let inst = Instrument::try_new("SWAP:BTC-USD-SWAP", &Asset::USD, &Asset::BTC)
        .unwrap()
        .prefer_reversed(true);
    // We use the `Instrument::prefer_reversed` method to mark an instrument as
    // a reversed instrument, whose price unit and the position side that shown
    // by the exchange (and `positions`) are actually reversed.
    //
    // Take `BTC-USD-SWAP` as an example, what it actually means that we are
    // holding a $100 "long" position of `BTC-USD-SWAP` contract at the "price"
    // of 16000 USD/BTC is we short $100 at the price of (1/16000) BTC/USD.
    // That is what the exchange actually using in its formula when it calculates
    // your total position as well as your profit.

    // We can represent this case directly by using `Reversed`:
    let mut p = inst.position(Reversed((dec!(16000), dec!(100))));

    // If we print it, we will see the reversed form (which is the same as what
    // you see in the exchange) of the position. That is because we have marked
    // the `inst` to be "reversed-preferring".
    assert_eq!(p.to_string(), "(16000, 100 USD)*");

    // The return value of `Position::price` and `Position::size` methods also
    // respect this setting.
    assert_eq!(p.price().unwrap(), dec!(16000));
    assert_eq!(p.size(), dec!(100));

    // But what is really being stored and calculated is the "true form".
    assert_eq!(p.as_naive().price, dec!(1) / dec!(16000));
    assert_eq!(p.as_naive().size, dec!(-100));

    // Let's add another reversed position to it to see what will happen.
    p += Reversed((dec!(15000), dec!(100)));
    assert_eq!(p.to_string(), "(15483.870967741935483870951759, 200 USD)*");
    // It is not the same result when we are calculating with the "true form",
    // but this is the right answer.

    // See what we will get when we close this position.
    p += Reversed((dec!(15700), -dec!(200)));
    assert_eq!(p.take(), dec!(0.0001778131634819532908705000));
    assert!(p.is_zero());
    // That may seem like a small profit, but the unit is BTC, which is actually
    // not small.
}
```
### Calculate the multi-instruments positions with `Positions`
```rust
use maplit::hashmap;
use positions::prelude::*;
use rust_decimal_macros::dec;

fn main() -> anyhow::Result<()> {
    let btc: Asset = "BTC".parse()?;
    let eth: Asset = "ETH".parse()?;
    let ada: Asset = "ADA".parse()?;
    let usdt: Asset = "USDT".parse()?;
    let usd: Asset = "USD".parse()?;

    // Let's assume that we initially have `1 BTC` and `100 USDT`.
    let mut p = btc.value(dec!(1)) + usdt.value(dec!(100));
    // We can print the positions table.
    println!("{}", p);

    // Then we buy `10 ETH` with `BTC` at the price of `0.075 BTC/ETH`.
    p += (dec!(10), &eth);
    p += (dec!(-10) * dec!(0.075), &btc);
    println!("{}", p);

    // Now we will buy some contracts. We first declare them.
    let btc_usdt_swap = Instrument::derivative("SWAP", "BTC-USDT-SWAP", &btc, &usdt)?;
    let eth_usd_221209 =
        Instrument::derivative("FUTURES", "ETH-USD-221209", &usd, &eth)?.prefer_reversed(true);
    let ada_usdt_swap = Instrument::derivative("SWAP", "ADA-USDT-SWAP", &ada, &usdt)?;

    // 1. We long `1 BTC` of `BTC-USDT-SWAP` at the mark price of `16975 USDT/BTC`,
    // and the exchange charged a 8.4875 USDT fee.
    p += btc_usdt_swap.position((dec!(16975), dec!(1)));
    p += (dec!(-8.4875), &usdt);
    println!("{p}");

    // 2. We short `10000 USD` of `ETH-USD-221209` at the mark price of
    // `1278.87 USD/ETH`, and the exchange changed a 0.00391 ETH fee.
    p += eth_usd_221209.position(Reversed((dec!(1278.87), dec!(-10000))));
    p += (dec!(-0.00391), &eth);
    println!("{p}");

    // 3. We short `2100 ADA` of `ADA-USDT-SWAP` at the mark price of
    // `0.31715 USDT/ADA`, and the exchange changed a `0.333 USDT` fee.
    p += ada_usdt_swap.position((dec!(0.31715), dec!(-2100)));
    p += (dec!(0.333), &usdt);
    println!("{p}");

    // We can evaluate the equity of our positions at current prices.
    // We will use `positions::Expr` for this.
    let expr = p.as_expr();
    // This is another way of expressing the positions, we call it
    // "positions expression".
    println!("{expr}");

    // Sometimes we want to know which instruments prices can affect
    // our positions. We can use the `Expr::instruments` method, but
    // we need to decide the "root asset" (or the "unit asset") first.
    // We will choose `USDT` as our root asset.
    for inst in expr.instruments(&usdt) {
        println!("{inst}");
    }

    // To evaluate the equity of our positions, we must provide the prices
    // of the instruments above.
    let btc_usdt = Instrument::spot(&btc, &usdt);
    let eth_usdt = Instrument::spot(&eth, &usdt);
    let prices = hashmap! {
        eth_usd_221209.as_symbol().clone() => dec!(1277.09),
        eth_usdt.as_symbol().clone() => dec!(1277.71),
        ada_usdt_swap.as_symbol().clone() => dec!(0.31794),
        btc_usdt_swap.as_symbol().clone() => dec!(16961.3),
        btc_usdt.as_symbol().clone() => dec!(16964),
    };

    // Now we are ready to evaluate the equity of our positions.
    let equity = expr.eval(&usdt, &prices).unwrap();
    println!("equity={equity}\n");

    // 5. We close half of the position of `BTC-USDT-SWAP` at current
    // price.
    p += btc_usdt_swap.position((dec!(16961.3), dec!(-0.5)));
    println!("{p}");

    // Now we can evaluate the equity again.
    let prices = hashmap! {
        eth_usd_221209.as_symbol().clone() => dec!(1000.2),
        eth_usdt.as_symbol().clone() => dec!(1000.5),
        ada_usdt_swap.as_symbol().clone() => dec!(0.342),
        btc_usdt_swap.as_symbol().clone() => dec!(17000.3),
        btc_usdt.as_symbol().clone() => dec!(16999.5),
    };
    let equity = p.as_expr().eval(&usdt, &prices).unwrap();
    println!("equity={equity}");
    Ok(())
}
```