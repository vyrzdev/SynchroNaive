extern crate core;
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub(crate) profiling_directory: String,
    pub(crate) observers: Vec<(String, ObserverConfig)>
}
mod observer;
mod observations;
mod interval;
mod coordinator;
mod history;
mod value;

use std::fs;
use hifitime::Epoch;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use crate::coordinator::coordinator;
use crate::interval::Moment;
use crate::observer::{Observer, ObserverConfig};
use crate::value::Value;

const CHANNEL_BUFFER: usize = 100;
#[tokio::main]
async fn main() {
    let config: Config = serde_json::from_slice(&fs::read("config.json").expect("Failed to read config.json!")).expect("Failed to parse config!");

    let observers: Vec<Observer> = config.observers.into_iter().map(|(uid, cfg)| Observer::new(uid, cfg).unwrap()).collect::<Vec<Observer>>();

    let mut initial_value = None;
    let mut initial_at = None;
    'outer: while initial_value.is_none() {
        let mut tentative_value: Option<Value> = None;
        for observer in &observers {
            let observed = observer.request(&observer.config.target).await.unwrap();
            if let Some(v) = &tentative_value {
                if (!v.eq(&observed.0)) {
                    println!("Platforms do not agree on initial value! Retrying!");
                    continue 'outer;
                }
            } else {
                tentative_value = Some(observed.0);
            }
        }
        // Only reachable if the inital value was equal for all.
        initial_at = Some(Moment(Epoch::now().unwrap().duration));

        for observer in &observers {
            let observed = observer.request(&observer.config.target).await.unwrap();
            if !(observed.0.eq(tentative_value.as_ref().unwrap())) {
                println!("Confirmation Pass Failed - Value changed! Retrying Initial Value.");
                continue 'outer;
            }
        }
        initial_value = tentative_value;
    }

    let initial_value = initial_value.unwrap();
    let initial_at = initial_at.unwrap();

    println!("Initial value established: {:?}", initial_value);

    let (tx, rx) = mpsc::channel(CHANNEL_BUFFER);
    let mut observer_pool = JoinSet::new();
    for observer in observers {
        let val_local = initial_value.clone();
        let at_local = initial_at.clone();
        let tx_local = tx.clone();
       observer_pool.spawn((async move || observer.poll_worker((val_local, at_local), tx_local).await)());
    }
    let coordinator_fut = coordinator(initial_value, rx);

    futures::join!(observer_pool.join_all(), coordinator_fut);
}
