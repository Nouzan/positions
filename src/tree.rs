use crate::{Asset, HashMap, Instrument, Position, PositionNum, ToNaivePosition};
use alloc::{boxed::Box, fmt};

/// Position Tree.
#[derive(Debug, Clone)]
pub struct PositionTree<'a, T> {
    pub(crate) asset: &'a Asset,
    pub(crate) value: T,
    pub(crate) positions: HashMap<&'a Instrument, &'a Position<T>>,
    pub(crate) children: HashMap<Instrument, PositionTree<'a, T>>,
}

impl<'a, T> PositionTree<'a, T>
where
    T: PositionNum,
{
    /// Instruments that the tree cares.
    pub fn instruments(&self) -> impl Iterator<Item = &Instrument> {
        let children: Box<dyn Iterator<Item = &Instrument>> =
            Box::new(self.children.values().flat_map(|t| t.instruments()));
        let pairs = self.children.keys();
        let positions = self.positions.keys().copied();
        children.chain(pairs).chain(positions)
    }

    /// Evaluate the position tree with the given prices.
    /// Return `None` if there are missing prcies.
    pub fn eval(&self, prices: &HashMap<Instrument, T>) -> Option<T> {
        let children = self
            .children
            .iter()
            .map(|(inst, t)| {
                let mut value = t.eval(prices)?;
                value *= prices.get(inst)?;
                Some(value)
            })
            .try_fold(T::zero(), |acc, x| Some(acc + x?))?;
        let mut ans = self
            .positions
            .iter()
            .map(|(inst, p)| Some(p.closed(prices.get(*inst)?)))
            .try_fold(children, |acc, x| Some(acc + x?))?;
        ans += &self.value;
        Some(ans)
    }

    /// Evaluate the position tree with the result price of the given function.
    /// Return `None` if there is something wrong.
    pub fn eval_with<F>(&self, mut f: F) -> Option<T>
    where
        F: FnMut(&Instrument, &dyn ToNaivePosition<T>) -> Option<T>,
    {
        let children = self
            .children
            .iter()
            .map(|(inst, t)| {
                let value = t.eval_with(&mut f)?;
                (f)(inst, &(T::zero(), value))
            })
            .try_fold(T::zero(), |acc, x| Some(acc + x?))?;
        let mut ans = self
            .positions
            .iter()
            .map(|(inst, p)| (f)(inst, p))
            .try_fold(children, |acc, x| Some(acc + x?))?;
        ans += &self.value;
        Some(ans)
    }
}

fn write_position<T>(
    f: &mut fmt::Formatter<'_>,
    price: &T,
    size: &T,
    inst: &Instrument,
) -> fmt::Result
where
    T: fmt::Display + PositionNum,
{
    let asset = inst.base();
    if inst.is_prefer_reversed() {
        if price.is_zero() {
            write!(f, "(Nan, {} {asset})*", -size.clone())
        } else {
            let mut real_price = T::one();
            real_price /= price;
            write!(f, "({}, {} {asset})*", real_price, -size.clone())
        }
    } else {
        write!(f, "({price}, {size} {asset})")
    }
}

impl<'a, T> fmt::Display for PositionTree<'a, T>
where
    T: fmt::Display + PositionNum,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (idx, tree) in self.children.values().enumerate() {
            if idx != 0 {
                write!(f, " + {tree}")?;
            } else {
                write!(f, "{tree}")?;
            }
        }
        let flag = !self.children.is_empty();
        let mut value = self.value.clone();
        for (idx, (inst, position)) in self.positions.iter().enumerate() {
            if flag || idx != 0 {
                write!(f, " + ")?;
            }
            write_position(
                f,
                &position.as_naive().price,
                &position.as_naive().size,
                inst,
            )?;
            value += position.value();
        }
        let flag = flag || !self.positions.is_empty();
        if value.is_positive() && flag {
            write!(f, " + {} {}", value, self.asset)
        } else if value.is_negative() && flag {
            write!(f, " - {} {}", value.abs(), self.asset)
        } else if value.is_negative() {
            write!(f, "- {} {}", value.abs(), self.asset)
        } else {
            write!(f, "{} {}", value, self.asset)
        }
    }
}
