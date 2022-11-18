use core::ops::Deref;

use crate::{IntoNaivePosition, NaivePosition, PositionNum};

/// A node in the position tree.
#[derive(Debug, Clone, Copy)]
pub enum Node<T> {
    /// A Value Node.
    Value(ValueNode<T>),
    /// Position Node.
    Position(PositionNode<T>),
}

/// Value Node.
#[derive(Debug, Clone, Copy, Default)]
pub struct ValueNode<T>(pub T);

/// Position Node.
#[derive(Debug, Clone, Copy)]
pub struct PositionNode<T>(pub NaivePosition<T>);

impl<T> Default for PositionNode<T>
where
    T: PositionNum,
{
    fn default() -> Self {
        Self(NaivePosition::default())
    }
}

impl<T> Deref for PositionNode<T> {
    type Target = NaivePosition<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> PositionNode<T>
where
    T: PositionNum,
{
    /// Add a position (using the exchange rule).
    pub fn add(&mut self, position: impl IntoNaivePosition<T>) -> T {
        self.0 = self.0.clone() + position.into_naive_position();
        self.0.take()
    }

    /// Eval the position by closing.
    pub fn eval(&self, price: &T) -> T {
        if self.0.size.is_zero() {
            T::zero()
        } else {
            self.clone().add((price.clone(), -self.0.size.clone()))
        }
    }
}
