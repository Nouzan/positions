use fraction::{BigInt, GenericDecimal};
use positions::prelude::*;

type Decimal = GenericDecimal<BigInt, usize>;

const FEE_RATE: f64 = -0.001;
const BTC_USD_SWAP: &str = "SWAP:BTC-USD-SWAP";

fn buy_btc(size: Decimal, at: Decimal) -> Positions<Decimal> {
    let mut p = Asset::usdt().value(-at.clone() * size.clone());
    p += (size.clone(), &Asset::btc());
    if size.is_sign_positive() {
        p += (size.abs() * Decimal::from(FEE_RATE), &Asset::btc());
    } else {
        p += (size.abs() * at * Decimal::from(FEE_RATE), &Asset::usdt());
    }
    p
}

fn buy_swap(size: Decimal, at: Decimal) -> Positions<Decimal> {
    let p = Instrument::try_new(BTC_USD_SWAP, &Asset::usd(), &Asset::btc())
        .unwrap()
        .prefer_reversed(true)
        .position((at, size).reversed());
    let fee =
        (p.as_naive().price.clone() * p.as_naive().size.clone()).abs() * Decimal::from(FEE_RATE);
    let quote = p.instrument().quote().clone();
    let mut p = p.into();
    p += (fee, &quote);
    p
}

fn interest(value: Decimal, rate: Decimal) -> Positions<Decimal> {
    Asset::btc().value(value * rate)
}

fn main() -> anyhow::Result<()> {
    let btc = Asset::btc();
    let usdt = Asset::usdt();
    let btc_usdt = Instrument::from((btc.clone(), usdt.clone()));
    let btc_usd_swap = Instrument::try_new(BTC_USD_SWAP, &Asset::usd(), &Asset::btc())
        .unwrap()
        .prefer_reversed(true);
    let mut prices = HashMap::from([
        (btc_usdt.clone(), Decimal::from(16000.00)),
        (btc_usd_swap.clone(), Decimal::from(16003.00)),
    ]);

    let mut account = Positions::default();
    account += buy_btc(
        Decimal::from(1.0000),
        prices.get(&btc_usdt).unwrap().clone(),
    );
    println!("{account}");
    let price = prices.get(&btc_usd_swap).unwrap().clone();
    let size = price.clone()
        * (account.get_value(&btc).unwrap().clone() * (Decimal::from(1) + Decimal::from(FEE_RATE)));
    account += buy_swap(-size, price);
    println!("{account}");
    println!("Converting the prices to be the same.\n");
    account
        .get_position_mut(&btc_usd_swap)
        .unwrap()
        .convert(prices.get(&btc_usdt).unwrap().clone());
    println!("{account}");

    println!("------ 8 hours later -------\n");

    *prices.get_mut(&btc_usdt).unwrap() = Decimal::from(17000.00);
    *prices.get_mut(&btc_usd_swap).unwrap() = Decimal::from(17004.00);

    let p = account.get_position(&btc_usd_swap).unwrap().clone();
    let value = p.as_naive().price.clone() * p.as_naive().size.clone();
    account += interest(value, Decimal::from(0.004));
    println!("{account}");
    account += buy_swap(-p.size(), prices.get(&btc_usd_swap).unwrap().clone());
    account.concentrate();
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
