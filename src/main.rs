extern crate core;
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub(crate) profiling_directory: String,
    pub(crate) observers: Vec<(String, SquareObserverConfig)>
}

mod observer;
mod observations;
mod interval;
mod coordinator;
mod history;
mod value;
mod workers;
mod testing;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use chrono::{TimeDelta};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, watch, Mutex};
use tokio::task::JoinSet;
use crate::coordinator::coordinator;
use crate::observer::{SquareObserver, SquareObserverConfig};
use crate::workers::{poll_worker, record_worker, polling_write_worker, record_write_worker};
use crate::workers::PollingInterpretation::Transition;

async fn real_evaluation() {
    // Load configuration.
    let config: Config = serde_json::from_reader(File::open(Path::new("config.json")).unwrap()).unwrap();
    info!("MAIN - Configuration Loaded Successfully.");

    let mut observers = HashMap::new();
    // Initialise Observers
    for (uid, cfg) in config.observers {
        observers.insert(uid.clone(), Arc::new(SquareObserver::new(uid, cfg)));
    }
    info!("MAIN - Initialised Observers");
    // TODO: Deviation Calculations.

    let mut initial_value = None;
    for obs in observers.values() {
        let (value, _, _) = obs.request(obs.target.clone()).await.unwrap();
        if initial_value.is_none() {
            initial_value.replace(value); // If no replies yet - set!
        } else if let Some(v) = initial_value {
            if v != value {
                initial_value = None;
                break; // Stop iterating - initial value stays none.
            }
            // Otherwise - is same as others - still might be a shared init value!
        }
    }
    info!("MAIN - Initial Value : {initial_value:?}");

    // We will test polling with subset:
    let polling_observers = vec!["Vendor A"];

    // We will test timestamped records with subset:
    let record_observers = vec!["Vendor B"];

    // Initialise Coordinator Channels.
    let (obs_tx, obs_rx) = mpsc::channel(1024);
    let (consensus_tx, consensus_rx) = watch::channel(None);


    // Initialise Polling Workers (and writers)
    let mut polling_workers = JoinSet::new();
    let mut polling_write_workers = JoinSet::new();
    info!("MAIN - Initialising Polling Workers for {polling_observers:?}");
    for name in polling_observers {
        // Clones for move.
        let local_observer = observers.get(name).unwrap().clone();
        let local_obs_tx = obs_tx.clone();
        let local_target = local_observer.target.clone();
        let local_mutex = Arc::new(Mutex::new(initial_value));
        let local_mutex_writer = local_mutex.clone();
        // Initialise Worker.
        polling_workers.spawn((async move || poll_worker(
            local_observer,
            Transition,  // Interpret changes as Transitions.
            local_target,
            TimeDelta::seconds(1),
            name.to_string(),
            local_mutex,
            local_obs_tx // Send observations here.
        ).await)());

        let local_observer = observers.get(name).unwrap().clone();
        let local_target = local_observer.target.clone();
        let local_consensus_rx = consensus_rx.clone();


        polling_write_workers.spawn((async move || polling_write_worker(
            local_observer,
            local_target,
            local_mutex_writer,
            local_consensus_rx
        ).await)());
    }

    // Initialise Record Workers:
    let mut record_workers = JoinSet::new();
    let mut record_write_workers = JoinSet::new();
    info!("MAIN - Initialising Record Workers for {record_observers:?}");
    for name in record_observers {
        // Clone for move.
        let local_obs_tx = obs_tx.clone();
        let local_observer = observers.get(name).unwrap().clone();
        let local_target = local_observer.target.clone();

        debug!("Spawning: {}", name);
        // Initialise Worker.
        record_workers.spawn((async move || record_worker(
            local_observer,
            local_target,
            TimeDelta::seconds(1),
            local_obs_tx // Send observations here.
        ).await)());

        let local_observer = observers.get(name).unwrap().clone();
        let local_target = local_observer.target.clone();
        let local_consensus_rx = consensus_rx.clone();

        record_write_workers.spawn((async move || record_write_worker(
            local_observer,
            local_target,
            local_consensus_rx
        ).await)());
    }

    // Initialise Writers:

    // let mut write_workers = JoinSet::new();
    // info!("MAIN - Initialising Write Workers!");
    // for observer in observers.values() {
    //     // Clone for Move
    //     let local_consensus_rx = consensus_rx.clone();
    //     let local_target = observer.target.clone();
    //     let local_observer = observer.clone();
    //
    //     // Initialise Worker.
    //     write_workers.spawn((async move || write_worker(
    //         local_observer,
    //         local_target,
    //         local_consensus_rx, // Receive new values here.
    //     ).await)());
    // }

    // Initialise Coordinator
    info!("MAIN - Initialising Coordinator Worker!");
    let coordinator_future = coordinator(initial_value, obs_rx, consensus_tx);

    // Initialise Testing Workers
    // TODO: Integrate testing workers.

    // Run Threads!
    info!("MAIN - INITIALISED - RUNNING!");
    futures::join!(polling_workers.join_all(), record_workers.join_all(), polling_write_workers.join_all(), record_write_workers.join_all(), coordinator_future);
}

// const CHANNEL_BUFFER: usize = 100;
#[tokio::main]
async fn main() {
    colog::init();
    info!("MAIN - Starting Real Evaluation");
    real_evaluation().await;
    //
    // // println!("Performing Deviation Calibration for each Observer:");
    // // for observer in &mut observers {
    // //     observer.deviation_worker().await;
    // // }
    //
    // let mut initial_value = None;
    // let mut initial_at = None;
    // 'outer: while initial_value.is_none() {
    //     let mut tentative_value: Option<Value> = None;
    //     for observer in &observers {
    //         let observed = observer.request(&observer.config.target).await.unwrap();
    //         if let Some(v) = &tentative_value {
    //             if (!v.eq(&observed.0)) {
    //                 println!("Platforms do not agree on initial value! Retrying!");
    //                 continue 'outer;
    //             }
    //         } else {
    //             tentative_value = Some(observed.0);
    //         }
    //     }
    //
    //     initial_at = Some(Utc::now());
    //     // Only reachable if the inital value was equal for all.
    //     // initial_at = Some(Moment(Epoch::now().unwrap().duration));
    //
    //     for observer in &observers {
    //         let observed = observer.request(&observer.config.target).await.unwrap();
    //         if !(observed.0.eq(tentative_value.as_ref().unwrap())) {
    //             println!("Confirmation Pass Failed - Value changed! Retrying Initial Value.");
    //             continue 'outer;
    //         }
    //     }
    //     initial_value = tentative_value;
    // }
    //
    // let initial_value = initial_value.unwrap();
    // let initial_at = initial_at.unwrap();
    //
    // println!("Initial value established: {:?}", initial_value);
    //
    // let (tx, rx) = mpsc::channel(CHANNEL_BUFFER);
    // let (w_tx, mut w_rx) = watch::channel(None);
    // let mut observer_pool = JoinSet::new();
    // let mut writer_pool = JoinSet::new();
    // for mut observer in observers {
    //     let val_local = initial_value.clone();
    //     let at_local = initial_at.clone();
    //     let tx_local = tx.clone();
    //     let w_rx_local = w_rx.clone();
    //     let observer_ref_1 = observer.clone();
    //     let observer_ref_2 = observer;
    //     observer_pool.spawn((async move || timestamped_record_worker(&*observer_ref_1, tx_local, initial_at).await)());
    //     writer_pool.spawn((async move || platform_writer(&*observer_ref_2, w_rx_local).await)());
    // }
    //
    //
    // let coordinator_fut = coordinator(initial_value, rx, w_tx);
    // let (x, y) = test_apis.pop().expect("Failed to get coordinator!");
}






// async fn tester(test_apis: Vec<(InventoryApi, ObserverConfig)>) {
//     sleep(core::time::Duration::from_secs(5)).await;
//
//     let mut futures = JoinSet::new();
//     for (api, cfg) in test_apis {
//         let r_target = cfg.target.clone();
//         let r_loc_id = cfg.location_id.clone();
//         futures.spawn((async move || {
//             let resp = api.batch_change_inventory(&BatchChangeInventoryRequest {
//                 idempotency_key: Uuid::new_v4().to_string(),
//                 changes: Some(vec![
//                     InventoryChange {
//                         r#type: Some(InventoryChangeType::Adjustment),
//                         physical_count: None,
//                         adjustment: Some(InventoryAdjustment {
//                             id: None,
//                             reference_id: None,
//                             from_state: Some(InventoryState::None),
//                             to_state: Some(InventoryState::InStock),
//                             location_id: Some(r_loc_id),
//                             catalog_object_id: Some(r_target),
//                             catalog_object_type: None,
//                             quantity: Some("5".to_string()),
//                             total_price_money: None,
//                             occurred_at: Some(DateTime::new()),
//                             created_at: None,
//                             source: None,
//                             employee_id: None,
//                             team_member_id: None,
//                             transaction_id: None,
//                             refund_id: None,
//                             purchase_order_id: None,
//                             goods_receipt_id: None,
//                             adjustment_group: None,
//                         }),
//                         transfer: None,
//                         measurement_unit: None,
//                         measurement_unit_id: None,
//                     }
//                 ]),
//                 ignore_unchanged_counts: None,
//             }).await;
//             println!("{:?}", resp);
//         })());
//     }
//     futures.join_all().await;
//     println!("DONE!");
// }
