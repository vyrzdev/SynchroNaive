use std::cmp::{max, min};
use hifitime::Epoch;

#[derive(Debug, Copy, Clone)]
pub struct Interval(Epoch, Epoch);

pub const MERGE: fn(Interval, Interval) -> Interval = |a, b| Interval(min(a.0, b.0), max(a.1, b.1));
pub const LT: fn(Interval, Interval) -> bool = |a, b| (a.1 < b.0);
pub const GT: fn(Interval, Interval) -> bool = |a,b| (a.0 > b.1);

pub const OVERLAP: fn(Interval, Interval) -> bool = |a, b| (!LT(a, b) && !GT(a, b));