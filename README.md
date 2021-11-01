# Positions

A position (finance) definition with some good algebraic properties.

## Example

Add `positions` as a dependency of your project.

```toml
[dependencies]
positions = "0.1.0"

# `rust_decimal` is added to make the example code work,
# but optional for using `positions`.
rust_decimal = "1.17.0"
rust_decimal_macros = "1.17.0"
```

And then, you can try this codes.

```rust
use positions::normal;
use rust_decimal_macros::dec;

let h1 = normal((dec!(1.0), dec!(2.0)));
let h2 = normal((dec!(2.0), dec!(3.0)));
let h3 = normal((dec!(1.5), dec!(-4.0)));

assert_eq!(h1 + h2 + h3, normal((dec!(1.6), dec!(1.0), dec!(-0.4))));
```
