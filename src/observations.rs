use std::cmp::Ordering;
use crate::inference::interval::{Interval, GT, LT, OVERLAP};
use crate::observations::SourceKind::Polling;
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

#[derive(Clone, Debug)]
pub enum SourceKind {
    Polling(String),
    Record(String)
}

#[derive(Debug, Clone)]
pub struct Observation {
    pub definition: DefinitionPredicate,
    pub interval: Interval,
    pub source: SourceKind
}
impl Observation {
    pub(crate) fn pretty_output(&self) -> String {
        let label = match self.definition {
            DefinitionPredicate::Transition { v_0, v_1 } => {format!("TR ({} -> {})", v_0, v_1)},
            DefinitionPredicate::Mutation { delta } => {format!("MU ({delta})")},
            DefinitionPredicate::Assignment { v_new } => {format!("AS ({v_new})")},
        };

        format!(
            "{label} over {}, {} from: {:?}",
            ((self.interval.0.0 as f64)/1000.0),
            ((self.interval.1.0 as f64)/1000.0),
            self.source
        )
    }
}
impl PartialEq<Self> for Observation {
    fn eq(&self, other: &Self) -> bool {
        return false; // No two observations are equal!
        // let mut ordered = false;
        //
        // // If intervals do not overlap - they are ordered.
        // if !OVERLAP(self.interval, other.interval) {
        //     ordered = true;
        // }
        //
        // // If they are from the same platform (and it is polling) they are ordered.
        // if let (Polling(name_1), Polling(name_2)) = (&self.source, &other.source) {
        //     if name_1 == name_2 {
        //         ordered = true;
        //     }
        // }
        //
        // return !ordered;
    }
}

impl PartialOrd for Observation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if LT(self.interval, other.interval) {
            return Some(Ordering::Less);
        }
        if GT(self.interval, other.interval) {
            return Some(Ordering::Greater);
        }
        if let (Polling(name_1), Polling(name_2)) = (&self.source, &other.source) {
            if name_1 == name_2 {
                if self.interval.0 < other.interval.0 {
                    return Some(Ordering::Less);
                }
                if self.interval.0 > other.interval.0 {
                    return Some(Ordering::Greater);
                }

                panic!("Found Overlapping Polls!");
            }
        }

        return None; // No ordering possible!
    }
}



pub enum PollingInterpretation {
    Transition,
    Assignment,
    Mutation
}

pub(crate) type Tick = u64;