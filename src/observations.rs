use uuid::Uuid;
use crate::interval::Interval;
use crate::value::Value;

#[derive(Debug)]
pub struct Observation {
    pub(crate) at: Interval,
    pub(crate) s0: Value,
    pub(crate) s1: Value,
    pub(crate) source: String
}
