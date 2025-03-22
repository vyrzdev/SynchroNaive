use std::cmp::Ordering;
use std::cmp::Ordering::Less;
use log::{debug, info};
use nodit::NoditMap;
use petgraph::{Directed, Graph, Undirected};
use petgraph::acyclic::Acyclic;
use petgraph::algo::{connected_components, tarjan_scc, toposort};
use petgraph::data::Build;
use petgraph::operator::complement;
use petgraph::prelude::{DiGraph, UnGraph};
use petgraph::visit::Walker;
use crate::inference::interval::{Interval, Moment, MERGE};
use crate::observations::{Observation, DefinitionPredicate};
use crate::value::Value;



pub struct NewHistory {
    observations: Vec<Observation>,
}

impl NewHistory {
    pub fn new() -> Self {
        Self {observations: vec![]}
    }

    pub fn add_new(&mut self, observation: Observation) {
        self.observations.push(observation);
    }

    pub fn get_execution(&self) -> Vec<Vec<Observation>> {
        let mut undirected = UnGraph::new_undirected();

        for observation in self.observations.clone() {
            undirected.add_node(observation);
        }

        for (i, o_1) in undirected.node_indices().enumerate() {
            for o_2 in undirected.node_indices().skip(i+1) {
                match undirected[o_1].partial_cmp(&undirected[o_2]) {
                    Some(Less) => {undirected.add_edge(o_1, o_2, ());},
                    _ => continue,
                }
            }
        }

        // Get Graph Complement
        let mut complement_graph = UnGraph::new_undirected();
        complement(&undirected, &mut complement_graph, ());

        // Get Connected Sections (unorderable regions)
        let conflicts = tarjan_scc(&complement_graph);

        // Build directed graph of unorderable regions
        let mut execution = DiGraph::new();
        for conflict in conflicts {
            let mut level: Vec<Observation> = Vec::new();
            for o in conflict {
                level.push(complement_graph[o].clone());
            }
            execution.add_node(level);
        }

        // Generate Edges
        for (i, o_1) in execution.node_indices().enumerate() {
            for o_2 in execution.node_indices().skip(i+1) {
                match execution[o_1][0].partial_cmp(&execution[o_2][0]) {
                    Some(Less) => {
                        execution.add_edge(o_1, o_2, ());
                    },
                    _ => continue,
                }
            }
        }

        // Finally - produce a topological sort of all nodes.
        // The graph is guaranteed to be acyclic
        let plan = toposort(&execution, None).unwrap();
        println!("{:?}", plan);
        return plan.iter().map(|x| execution[*x].clone()).collect::<Vec<Vec<Observation>>>();
    }


    pub fn apply(&self, value: Option<Value>) -> Option<Value> {
        let execution = self.get_execution();
        let mut cumulative = value;

        for level in &execution {
            match NewHistory::definition(level) {
                Some(definition) => match definition {
                    DefinitionPredicate::Transition { v_0, v_1 } => {
                        if cumulative.is_some() && cumulative == Some(v_0) {
                            cumulative = Some(v_1);
                        } else {
                            cumulative = None;
                            info!("Inference - Conflict : {level:?}")
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
                None => {
                    info!("Inference - Undefined Level: {level:?}");
                    cumulative = None
                },
            }
        }
        return cumulative;
    }

    pub fn definition(level: &Vec<Observation>) -> Option<DefinitionPredicate> {
        let mut all_mut = true;
        let mut cumulative_mut = 0;
        let mut all_last_assn = true;
        let mut value = None;

        if level.len() == 0 {
            panic!("Should be Unreachable!")
        }

        if level.len() == 1 {
            return Some(level[0].definition.clone()); // ref XXX
        }

        for observation in level {
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





//
//
// pub struct History {
//     history: NoditMap<Moment, Interval, Level>
// }
//
// #[derive(Debug)]
// pub struct Level {
//     interval: Interval,
//     observations: Vec<Observation>
// }
//
//
// impl Level {
//     fn new(first: Observation) -> Self {
//         Level {
//             interval: first.interval,
//             observations: Vec::from([first])
//         }
//     }
//
//     // fn add_new(&mut self, observation: Observation) {
//     //     self.interval = MERGE(self.interval, observation.interval);
//     //     self.observations.push(observation);
//     // }
//
//     fn merge(&mut self, other: Level) {
//         self.observations.extend(other.observations); // When levels merge merge observations
//         self.interval = MERGE(self.interval, other.interval); // AND grow interval
//     }
//
//     fn definition(&self) -> Option<DefinitionPredicate> {
//
//     }
// }
//
// impl History {
//     pub(crate) fn new() -> History {
//         History { history: NoditMap::new() }
//     }
//
//     pub(crate) fn add_new(&mut self, observation: Observation) {
//         // Add new observation to history
//         let interval = observation.interval;
//         let mut new_level = Level::new(observation); // Instantiate new level set for observation.
//
//         for (_, level) in self.history.remove_overlapping(interval) { // any existing level-sets overlapping with new obs:
//             new_level.merge(level); // Merged into one new level set (they can no longer be ordered)
//         }
//         self.history.insert_strict(new_level.interval, new_level).unwrap() // Expect to succeed - Above remove_overlap guarantees it.
//     }
//
//     pub fn apply(&mut self, init: Option<Value>) -> Option<Value> {
//         let mut cumulative = init;
//
//         for (_, level) in self.history.iter() {
//             // info!("Processing Level: {:?}", level);
//
//
//             match level.definition() {
//                 Some(definition) => match definition {
//                     DefinitionPredicate::Transition { v_0, v_1 } => {
//                         if cumulative.is_some() && cumulative == Some(v_0) {
//                             cumulative = Some(v_1);
//                         } else {
//                             cumulative = None;
//                             info!("Inference - Conflict : {level:?}")
//                         }
//                     }
//                     DefinitionPredicate::Mutation { delta } => {
//                         if let Some(v) = cumulative {
//                             cumulative = Some(v + delta);
//                         }
//                     }
//                     DefinitionPredicate::Assignment { v_new } => {
//                         cumulative = Some(v_new);
//                     }
//                 }
//                 None => {
//                     info!("Inference - Undefined Level: {level:?}");
//                     cumulative = None
//                 },
//             }
//         }
//
//         return cumulative;
//     }
// }