use std::cmp::{max, min};
use hifitime::Epoch;
use nodit::NoditMap;
use crate::interval::{Interval, MERGE};
use crate::observations::Observation;


pub struct History {
    history: NoditMap<Epoch, Interval, Level>
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
}