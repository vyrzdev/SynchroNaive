use std::iter::Iterator;
 use crate::interval::Interval;
use crate::value::Value;

#[derive(Debug, Clone, Copy)]
pub enum DefinitionPredicate {
    Transition {
        v_0: Value,
        v_1: Value,
    },
    Mutation {
        delta: Value
    },
    Assignment {
        v_new: Value
    }
}

#[derive(Debug, Clone)]
pub struct Observation {
    pub definition: DefinitionPredicate,
    pub interval: Interval,
    pub source: String
}


