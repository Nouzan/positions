use super::*;
use rust_decimal_macros::dec;

#[test]
fn average_cost_normal() {
    let h1 = normal((4, 1));
    let h2 = normal((2, 2));
    let h3 = normal((1, 4));
    assert_eq!((h1 + h2 + h3).price(), Some(1));
}

#[test]
fn average_cost_reversed() {
    let h1 = reversed((2.0, 4.0));
    let h2 = reversed((4.0, 4.0));
    let h3 = reversed((8.0, 8.0));
    assert_eq!((h1 + h2 + h3).price(), Some(4.0));
}

#[test]
fn close_position_normal() {
    let h1 = normal((dec!(1.0), dec!(2.0)));
    let h2 = normal((dec!(2.0), dec!(3.0)));
    let h3 = normal((dec!(1.5), dec!(-4.0)));

    assert_eq!(h1 + h2 + h3, normal((dec!(1.6), dec!(1.0), dec!(-0.4))));
}
