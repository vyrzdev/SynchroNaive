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

impl DefinitionPredicate {
    pub(crate) fn apply(&self, input: &Value) -> Option<Value>{
        match self {
            DefinitionPredicate::Transition { v_0, v_1 } => if input == v_0 { Some(v_1.clone()) } else { None },
            DefinitionPredicate::Mutation { delta } => {Some(input + delta)}
            DefinitionPredicate::Assignment { v_new } => {Some(v_new.clone())}
        }
    }
}

#[derive(Debug, Clone)]
pub struct Observation {
    pub definition: DefinitionPredicate,
    pub interval: Interval,
    pub source: String
}

pub enum PollingInterpretation {
    Transition,
    Assignment,
    Mutation
}

pub(crate) type Tick = u64;