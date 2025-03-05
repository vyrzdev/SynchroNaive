use std::cmp::{max, min};
use std::error::Error;
use hifitime::Epoch;
use nodit::NoditMap;
use crate::interval::{Interval, Moment, MERGE};
use crate::observations::Observation;
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
            interval: first.at,
            observations: Vec::from([first])
        }
    }

    fn add_new(&mut self, observation: Observation) {
        self.interval = MERGE(self.interval, observation.at);
        self.observations.push(observation);
    }

    fn merge(&mut self, other: Level) {
        self.observations.extend(other.observations);
        self.interval = MERGE(self.interval, other.interval);
    }
}

impl History {
    pub(crate) fn new() -> History {
        History { history: NoditMap::new() }
    }

    pub(crate) fn add_new(&mut self, observation: Observation) {
        let interval = observation.at;
        let mut new_level = Level::new(observation);
        for (_, level) in self.history.remove_overlapping(interval) {
            new_level.merge(level);
        }
        self.history.insert_strict(new_level.interval, new_level).unwrap() // Expect to succeed - Above remove_overlap guarantees it.
    }

    pub fn apply(&mut self, mut state: Value) -> Result<Option<Value>, Box<dyn Error>>{
        for (_, L) in self.history.iter() {
            if L.observations.len() == 1 {
                state = L.observations[0].s1.clone(); // Expect access success.
            } else {
                // TODO: Handle Logic
                todo!();
                return Ok(None) // for now illegal
            }
        }
        return Ok(Some(state));
    }
}