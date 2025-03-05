use std::cmp::{max, min};
use hifitime::Epoch;

pub type Interval = (Epoch, Epoch);

pub const MERGE: fn(Interval, Interval) -> Interval = |a, b| (min(a.0, b.0), max(a.1, b.1));
