use std::error::Error;
use nodit::NoditMap;
use crate::interval::{Interval, Moment, MERGE};
use crate::observations::{Observation, DefinitionPredicate};
use crate::value::Value;

pub struct History {
    history: NoditMap<Moment, Interval, Level>
}

#[derive(Debug)]
pub struct Level {
    interval: Interval,
    observations: Vec<Observation>
}


impl Level {
    fn new(first: Observation) -> Self {
        Level {
            interval: first.interval,
            observations: Vec::from([first])
        }
    }

    fn add_new(&mut self, observation: Observation) {
        self.interval = MERGE(self.interval, observation.interval);
        self.observations.push(observation);
    }

    fn merge(&mut self, other: Level) {
        self.observations.extend(other.observations); // When levels merge merge observations
        self.interval = MERGE(self.interval, other.interval); // AND grow interval
    }

    fn definition(&self) -> Option<DefinitionPredicate> {
        let mut all_mut = true;
        let mut cumulative_mut = 0;
        let mut all_last_assn = true;
        let mut value = None;

        if self.observations.len() == 0 {
            panic!("Should be Unreachable!")
        }

        if self.observations.len() == 1 {
            return Some(self.observations[0].definition.clone()); // ref XXX
        }

        for observation in &self.observations {
            match observation.definition {
                DefinitionPredicate::Transition {..} => return None, // ref XXX
                DefinitionPredicate::Assignment {v_new} => {
                    all_mut = false;

                    if let Some(v) = value {
                        if v != v_new {
                            return None // Distinct assignments cannot commute - ref XXX
                        }
                    } else {
                        value = Some(v_new); // First assignment witnessed!
                    }
                },
                DefinitionPredicate::Mutation {delta} => {
                    all_last_assn = false;
                    cumulative_mut += delta; // ref XXX
                }
            }
        }

        if all_mut {
            return Some(DefinitionPredicate::Mutation {delta: cumulative_mut });
        }
        if all_last_assn {
            return Some(DefinitionPredicate::Assignment {v_new: value.unwrap()});
        }

        return None; // Otherwise, no definition could be found!
    }
}

impl History {
    pub(crate) fn new() -> History {
        History { history: NoditMap::new() }
    }

    pub(crate) fn add_new(&mut self, observation: Observation) {
        // Add new observation to history
        let interval = observation.interval;
        let mut new_level = Level::new(observation); // Instantiate new level set for observation.

        for (_, level) in self.history.remove_overlapping(interval) { // any existing level-sets overlapping with new obs:
            new_level.merge(level); // Merged into one new level set (they can no longer be ordered)
        }
        self.history.insert_strict(new_level.interval, new_level).unwrap() // Expect to succeed - Above remove_overlap guarantees it.
    }

    pub fn apply(&mut self, init: Option<Value>) -> Option<Value> {
        let mut cumulative = init;

        for (_, level) in self.history.iter() {
            match level.definition() {
                Some(definition) => match definition {
                    DefinitionPredicate::Transition { v_0, v_1 } => {
                        if cumulative.is_some() && cumulative == Some(v_0) {
                            cumulative = Some(v_1);
                        } else {
                            cumulative = None;
                        }
                    }
                    DefinitionPredicate::Mutation { delta } => {
                        if let Some(v) = cumulative {
                            cumulative = Some(v + delta);
                        }
                    }
                    DefinitionPredicate::Assignment { v_new } => {
                        cumulative = Some(v_new);
                    }
                }
                None => cumulative = None,
            }
        }

        return cumulative;
    }
}