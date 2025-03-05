use uuid::Uuid;
use crate::interval::Interval;
#[derive(Debug)]
pub struct Observation {
    pub(crate) at: Interval,
    action: Action,
    source: Uuid
}

pub type Value = String;
#[derive(Debug)]
pub enum Action {
    Mutation(Value),
    Assignment(Value)
}
