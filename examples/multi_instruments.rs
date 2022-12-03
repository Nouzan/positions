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
