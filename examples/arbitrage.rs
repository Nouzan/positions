use fraction::{BigInt, GenericDecimal};
use positions::prelude::*;

type Decimal = GenericDecimal<BigInt, usize>;

const FEE_RATE: f64 = -0.001;

fn buy_btc(size: Decimal, at: Decimal) -> Positions<Decimal> {
    let mut p = Positions::default();
    p += (-at * size.clone(), &Asset::usdt());
    p += (size.clone(), &Asset::btc());
    p += (size.abs() * Decimal::from(FEE_RATE), &Asset::btc());
    p
}

fn buy_swap(size: Decimal, at: Decimal) -> Positions<Decimal> {
    let p = Position::new(
        Instrument::new("BTC-USD-SWAP", Asset::usd(), Asset::btc()).prefer_reversed(true),
        (at, size).reversed(),
    );
    let fee =
        (p.as_naive().price.clone() * p.as_naive().size.clone()).abs() * Decimal::from(FEE_RATE);
    let quote = p.instrument().quote().clone();
    let mut p = p.into();
    p += (fee, &quote);
    p
}

fn interest(value: Decimal, rate: Decimal) -> Positions<Decimal> {
    let mut p = Positions::default();
    p += (value * rate, &Asset::btc());
    p
}

fn main() -> anyhow::Result<()> {
    let btc = Asset::btc();
    let usdt = Asset::usdt();
    let btc_usdt = Instrument::from((btc.clone(), usdt.clone()));
    let btc_usd_swap =
        Instrument::new("BTC-USD-SWAP", Asset::usd(), Asset::btc()).prefer_reversed(true);
    let mut prices = HashMap::from([
        (btc_usdt.clone(), Decimal::from(16000)),
        (btc_usd_swap.clone(), Decimal::from(16005)),
    ]);

    let mut account = Positions::default();
    account += buy_btc(Decimal::from(1), prices.get(&btc_usdt).unwrap().clone());
    println!("{account}");
    let price = prices.get(&btc_usd_swap).unwrap().clone();
    let size = price.clone() * account.get_value(&btc).unwrap().clone();
    account += buy_swap(-size, price);
    println!("{account}");

    println!("------ 8 hours later -------\n");

    *prices.get_mut(&btc_usdt).unwrap() = Decimal::from(17000);
    *prices.get_mut(&btc_usd_swap).unwrap() = Decimal::from(17004);

    let p = account.get_position(&btc_usd_swap).unwrap().clone();
    let value = p.as_naive().price.clone() * p.as_naive().size.clone();
    account += interest(value, Decimal::from(0.000038));
    println!("{account}");
    account += buy_swap(-p.size(), prices.get(&btc_usd_swap).unwrap().clone());
    println!("{account}");
    let btc = account.get_value(&btc).unwrap().clone();
    account += buy_btc(-btc, prices.get(&btc_usdt).unwrap().clone());
    println!("{account}");
    println!(
        "value = {} {usdt}",
        account.as_tree(&usdt).eval(&prices).unwrap()
    );
    Ok(())
}
