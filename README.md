# Positions

A position (finance) definition with some good algebraic properties.

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/positions.svg
[crates-url]: https://crates.io/crates/positions
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/Nouzan/positions/blob/master/LICENSE
[actions-badge]: https://github.com/Nouzan/positions/workflows/CI/badge.svg
[actions-url]: https://github.com/Nouzan/positions/actions?query=workflow%3ACI+branch%3Amain

[API Docs](https://docs.rs/positions/latest/positions)

## Example

Add `positions` as a dependency of your project.

```toml
[dependencies]
positions = "0.1.1"

# `rust_decimal` is added to make the example code work,
# but optional for using `positions`.
rust_decimal = "1.17.0"
rust_decimal_macros = "1.17.0"
```

And then, you can try these codes.

```rust
use positions::normal;
use rust_decimal_macros::dec;

let h1 = normal((dec!(1.0), dec!(2.0)));
let h2 = normal((dec!(2.0), dec!(3.0)));
let h3 = normal((dec!(1.5), dec!(-4.0)));

assert_eq!(h1 + h2 + h3, normal((dec!(1.6), dec!(1.0), dec!(-0.4))));
```
