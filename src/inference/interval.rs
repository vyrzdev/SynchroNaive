use std::cmp::{max, min};
use nodit::{DiscreteFinite, InclusiveInterval};
use crate::observations::Tick;

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct Moment(Tick);

#[derive(Debug, Copy, Clone)]
pub struct Interval(pub Moment, pub Moment);

pub const MERGE: fn(Interval, Interval) -> Interval = |a, b| Interval(min(a.0, b.0), max(a.1, b.1));
pub const LT: fn(Interval, Interval) -> bool = |a, b| (a.1 < b.0);
pub const GT: fn(Interval, Interval) -> bool = |a,b| (a.0 > b.1);

pub const OVERLAP: fn(Interval, Interval) -> bool = |a, b| (!LT(a, b) && !GT(a, b));

impl DiscreteFinite for Moment {
    const MIN: Self = Moment(0);
    const MAX: Self = Moment(u64::max_value());

    fn up(self) -> Option<Self>
    where
        Self: Sized
    {
        Some(Moment(self.0 + 1))
    }

    fn down(self) -> Option<Self>
    where
        Self: Sized
    {
        Some(Moment(self.0 - 1))
    }
}
impl From<nodit::Interval<Moment>> for Interval {
    fn from(value: nodit::Interval<Moment>) -> Self {
        Interval(value.start(), value.end())
    }
}

impl InclusiveInterval<Moment> for Interval {
    fn start(&self) -> Moment {
        self.0
    }

    fn end(&self) -> Moment {
        self.1
    }
}
