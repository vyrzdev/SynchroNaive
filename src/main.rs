extern crate core;
// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct Config {
//     pub(crate) profiling_directory: String,
//     pub(crate) observers: Vec<(String, SquareObserverConfig)>
// }

mod observations;
mod coordinator;
mod value;
mod workers;
mod testing;
mod observers;
mod inference;

use std::cmp::max;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use chrono::{TimeDelta, Utc};
use futures::poll;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, watch, Mutex};
use tokio::task::JoinSet;
use crate::inference::history::{NewHistory};
use crate::observations::{DefinitionPredicate, PollingInterpretation, Tick};
use crate::observers::mocked::mock_writer::{instant_write, InstantWriter};
use crate::observers::mocked::poll_platform::{MockPlatform, MockPlatformConfig};
use crate::observers::mocked::polling::MockPoller;
use crate::observers::mocked::record::MockRecordPoller;
use crate::observers::mocked::record_platform::{MockRecordPlatform, MockRecordPlatformConfig};
// use crate::workers::{poll_worker, record_worker, polling_write_worker, record_write_worker, PollingInterpretation};
// use crate::workers::PollingInterpretation::Transition;


// async fn real_evaluation() {
//     // Load configuration.
//     let config: Config = serde_json::from_reader(File::open(Path::new("config.json")).unwrap()).unwrap();
//     info!("MAIN - Configuration Loaded Successfully.");
//
//     let mut observers = HashMap::new();
//     // Initialise Observers and Testers.
//     let mut testing_workers = JoinSet::new();
//
//     for (uid, cfg) in config.observers {
//         let new_observer = Arc::new(SquareObserver::new(uid.clone(), cfg.clone()));
//         let local_observer_1 = new_observer.clone();
//         let local_observer_2 = new_observer.clone();
//         let local_config_2 = cfg.clone();
//         testing_workers.spawn((async move || manual_editor(local_observer_1, cfg.clone(), cfg.testing_config.edit_lambda).await)());
//         testing_workers.spawn((async move || external_users(local_observer_2, local_config_2.clone(), local_config_2.testing_config.sale_lambda).await)());
//
//         observers.insert(uid, new_observer);
//     }
//     info!("MAIN - Initialised Observers");
//     // TODO: Deviation Calculations.
//
//     let mut initial_value = None;
//     for obs in observers.values() {
//         let (value, _, _) = obs.request(obs.target.clone()).await.unwrap();
//         if initial_value.is_none() {
//             initial_value.replace(value); // If no replies yet - set!
//         } else if let Some(v) = initial_value {
//             if v != value {
//                 initial_value = None;
//                 break; // Stop iterating - initial value stays none.
//             }
//             // Otherwise - is same as others - still might be a shared init value!
//         }
//     }
//     info!("MAIN - Initial Value : {initial_value:?}");
//
//     // We will test polling with subset:
//     let polling_observers = vec!["Vendor A"];
//
//     // We will test timestamped records with subset:
//     let record_observers = vec!["Vendor B"];
//
//     // Initialise Coordinator Channels.
//     let (obs_tx, obs_rx) = mpsc::channel(1024);
//     let (consensus_tx, consensus_rx) = watch::channel(None);
//
//
//     // Initialise Polling Workers (and writers)
//     let mut polling_workers = JoinSet::new();
//     let mut polling_write_workers = JoinSet::new();
//     info!("MAIN - Initialising Polling Workers for {polling_observers:?}");
//     for name in polling_observers {
//         // Clones for move.
//         let local_observer = observers.get(name).unwrap().clone();
//         let local_obs_tx = obs_tx.clone();
//         let local_target = local_observer.target.clone();
//         let local_mutex = Arc::new(Mutex::new(initial_value));
//         let local_mutex_writer = local_mutex.clone();
//         // Initialise Worker.
//         polling_workers.spawn((async move || poll_worker(
//             local_observer,
//             Transition,  // Interpret changes as Transitions.
//             local_target,
//             TimeDelta::seconds(1),
//             name.to_string(),
//             local_mutex,
//             local_obs_tx // Send observations here.
//         ).await)());
//
//         let local_observer = observers.get(name).unwrap().clone();
//         let local_target = local_observer.target.clone();
//         let local_consensus_rx = consensus_rx.clone();
//
//
//         polling_write_workers.spawn((async move || polling_write_worker(
//             local_observer,
//             local_target,
//             local_mutex_writer,
//             local_consensus_rx
//         ).await)());
//     }
//
//     // Initialise Record Workers:
//     let mut record_workers = JoinSet::new();
//     let mut record_write_workers = JoinSet::new();
//     info!("MAIN - Initialising Record Workers for {record_observers:?}");
//     for name in record_observers {
//         // Clone for move.
//         let local_obs_tx = obs_tx.clone();
//         let local_observer = observers.get(name).unwrap().clone();
//         let local_target = local_observer.target.clone();
//
//         debug!("Spawning: {}", name);
//         // Initialise Worker.
//         record_workers.spawn((async move || record_worker(
//             local_observer,
//             local_target,
//             TimeDelta::seconds(1),
//             local_obs_tx // Send observations here.
//         ).await)());
//
//         let local_observer = observers.get(name).unwrap().clone();
//         let local_target = local_observer.target.clone();
//         let local_consensus_rx = consensus_rx.clone();
//
//         record_write_workers.spawn((async move || record_write_worker(
//             local_observer,
//             local_target,
//             local_consensus_rx
//         ).await)());
//     }
//
//     // Initialise Coordinator
//     info!("MAIN - Initialising Coordinator Worker!");
//     let coordinator_future = coordinator(initial_value, obs_rx, consensus_tx);
//
//
//     // Run Threads!
//     info!("MAIN - INITIALISED - RUNNING!");
//     futures::join!(polling_workers.join_all(), record_workers.join_all(), polling_write_workers.join_all(), record_write_workers.join_all(), coordinator_future, testing_workers.join_all());
// }

// #[derive(Debug)]
// pub struct MockTestingConfig {
//     edit_lambda: f64,
//     sale_lambda: f64,
//     min_deviation: Option<chrono::TimeDelta>,
//     max_deviation: Option<chrono::TimeDelta>,
// }

// async fn fake_evaluation(from: chrono::DateTime<Utc>, until: chrono::DateTime<Utc>, polling_platforms: Vec<MockTestingConfig>, record_platforms: Vec<MockTestingConfig>) {
//
//     // let mut mock_observations = Vec::new();
//
//     // For each platform - generate events, and then observations
//
//     let mut global_observations = Vec::new();
//     let mut global_events = Vec::new();
//
//     for cfg in polling_platforms {
//         let mut edits = generate_edits(cfg.edit_lambda, from, until, 10);
//         let mut events = generate_sales(cfg.sale_lambda, from, until);
//         events.append(&mut edits);
//
//         // Sort events by time of occurrence.
//         events.sort_by_key(|x| x.1);
//
//
//         let mut poll_observations = generate_polls(events.clone(), from, 10, TimeDelta::seconds(1), 20.0, Transition);
//
//         global_events.append(&mut events);
//         global_observations.append(&mut poll_observations);
//     }
//
//     for cfg in record_platforms {
//         let mut edits = generate_edits(cfg.edit_lambda, from, until, 10);
//         let mut events = generate_sales(cfg.sale_lambda, from, until);
//         events.append(&mut edits);
//
//         // Sort events by time of occurrence.
//         events.sort_by_key(|x| x.1);
//
//         let mut record_observations = generate_polled_records(events.clone(), from, TimeDelta::seconds(1), 20.0, 200.0, TimeDelta::milliseconds(-400), TimeDelta::milliseconds(600));
//
//         global_events.append(&mut events);
//         global_observations.append(&mut record_observations);
//     }
//
//     // Sort events by time of occurrence.
//     global_events.sort_by_key(|x| x.1);
//
//     info!("Events:");
//     info!("{}", global_events.iter().map(|x| pretty_print_event_time_pair(x, &from)).collect::<Vec<String>>().join("|"));
//
//     // Sort observation by time of visibility.
//     global_observations.sort_by_key(|x| x.visible_at);
//
//     info!("Observations: ");
//     info!("{}", global_observations.iter().map(|x| x.pretty_output(&from)).collect::<Vec<String>>().join("\n"));
// }
// // const CHANNEL_BUFFER: usize = 100;
fn simulate(until: Tick) {
    let mut time: Tick = 0; // Simulated RealTime.

    let initial_value = 10000;

    let mut test_polling_platform = MockPlatform::new(MockPlatformConfig {
        name: "PollPlatform".to_string(),
        sale_lambda: 0.00003,
        edit_lambda: 0.0,
    }, 10000);

    let mut test_poller = MockPoller::new(
        40.0,
        1.0,
        1000,
        PollingInterpretation::Mutation
    );

    // Test Writer
    let mut test_poll_writer = InstantWriter::new();


    let mut test_record_platform = MockRecordPlatform::new(MockRecordPlatformConfig {
        name: "RecordPlatform".to_string(),
        sale_lambda: 0.00003,
        edit_lambda: 0.0,
        deviation_lambda: 500.0,
        deviation_std_dev: 0.2,
        clock_precision: 1,
    }, 10000);

    let mut test_record_poller = MockRecordPoller::new(
        40.0,
        1.0,
        1000,
        1
    );

    // TODO: Record Pollers don't need writers - underlying value is irrelevant.
    // We assume that a value gets written, and that we can identify and ignore actions originating from us.
    // therefore - event stream is unchanged by writes, and thus derived value is unchanged.

    let mut true_history = Vec::new();
    let mut observed_history = NewHistory::new();



    let mut consistent = true;
    let mut conflict = false;
    let mut time_to_convergence = 0;
    let mut convergence_times = Vec::new();

    let mut observed_value = Some(initial_value);
    let mut true_value = initial_value;

    while time <= until {
        let mut new_event = false;
        let mut new_observation = false;
       if let Some(event) = test_polling_platform.do_tick(&time) {
           // debug!("Simulator - Event: {event:?} at {time}");
           true_history.push(event);
           new_event = true;
       }

       if let Some(event) = test_record_platform.do_tick(&time) {
           // debug!("Simulator - Event: {event:?} at {time}");
           true_history.push(event);
           new_event = true;
       }

       if let Some(obs) = test_poller.do_tick(&time, &test_polling_platform) {
           // info!("Simulator - {obs:?} at {time}");
           observed_history.add_new(obs);

           new_observation = true;
       }

       if let Some(obs) = test_record_poller.do_tick(&time, &mut test_record_platform) {
           // debug!("Simulator - {obs:?} at {time}");
           for o in obs {
               observed_history.add_new(o);
           }
           new_observation = true;
       }

        if new_event {
            true_value = initial_value;
            for (event, _) in &true_history {
                true_value = event.apply(&true_value).unwrap();
            }
        }

        if new_observation {
            observed_value = observed_history.apply(Some(initial_value));
            // TODO: Writing Values :)
            // We consider both Instantaneous (conflict free writes)
            if let Some(new_value) = observed_value {
                // info!("InstantWriter - Wrote: {new_value:?} at {time}");
                test_poll_writer.do_write(new_value, &mut test_poller, &time);
            }
            // AND potentially lossy-writes (realistic, no mutex)

        }

        // Do Writer Tick
        test_poll_writer.do_tick(&mut test_polling_platform, &mut test_poller, &time);

        if observed_value.is_none() {
            info!("Simulator - Conflict, ending simulation.");
            conflict = true;
            break;
        }

        if observed_value.unwrap() != true_value {
            consistent = false;
        }

        if !consistent {
            if observed_value.unwrap() == true_value {
                consistent = true;
                convergence_times.push(time_to_convergence);
                time_to_convergence = 0;
            } else {
                time_to_convergence += 1;
            }
        }

        if time % (until/10) == 0 {
            info!("10% MARK")
        }

        time += 1;
    }
    info!("Simulation Complete!");
    info!("Convergence Times: {:?}", convergence_times);
    info!("Average Time To Convergence: {}", convergence_times.iter().sum::<u64>() / convergence_times.len() as u64);
    if conflict {
        info!("Time to Conflict: {}", time);
    } else {
        info!("No Conflicts!")
    }

}


#[tokio::main]
async fn main() {
    colog::init();
    info!("MAIN - Starting Simulation");
    simulate(600000);
    // fake_evaluation(
    //     Utc::now(),
    //     (Utc::now() + TimeDelta::seconds(60)),
    //     vec![
    //         MockTestingConfig {
    //             edit_lambda: 0.25,
    //             sale_lambda: 1.0,
    //             min_deviation: None,
    //             max_deviation: None,
    //         }
    //     ],
    //     vec![
    //         MockTestingConfig {
    //             edit_lambda: 0.25,
    //             sale_lambda: 1.0,
    //             min_deviation: Some(chrono::TimeDelta::milliseconds(-500)),
    //             max_deviation: Some(chrono::TimeDelta::milliseconds(600)),
    //         }
    //     ]
    // ).await;
    // info!("MAIN - Starting Real Evaluation");
    // real_evaluation().await;
    //
}