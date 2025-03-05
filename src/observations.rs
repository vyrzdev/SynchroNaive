use uuid::Uuid;
use crate::interval::Interval;

pub struct Observation {
    pub(crate) at: Interval,
    action: Action,
    source: Uuid
}

pub type Value = String;
pub enum Action {
    Mutation(Value),
    Assignment(Value)
}

impl poset::PartialOrderBehaviour for Observation {
    type Element = Observation;

    fn ge(&self, a: &Self::Element, b: &Self::Element) -> bool {
        a.at > b.at
    }
}
