/// Price representation.
pub trait Representation: 'static + Send + Sync + Clone + Copy + std::fmt::Debug {
    /// Is price representation reversed.
    fn is_reversed() -> bool;
}

/// Normal price representation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Normal;

/// Reversed price representation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Reversed;

impl Representation for Normal {
    fn is_reversed() -> bool {
        false
    }
}

impl Representation for Reversed {
    fn is_reversed() -> bool {
        true
    }
}
