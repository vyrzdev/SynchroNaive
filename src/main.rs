extern crate core;
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub(crate) profiling_directory: String,
    pub(crate) observers: Vec<(String, ObserverConfig)>
}
// use rand::distributions::{Distribution};
use rand_distr::{Poisson, Distribution};
use rand::prelude::*;


mod observer;
mod observations;
mod interval;
mod coordinator;
mod history;
mod value;
mod workers;
mod testing;

use std::{env, fs};
use std::sync::Arc;
use chrono::{TimeDelta, Utc};
use rand;
use serde::{Deserialize, Serialize};
use squareup::api::{CatalogApi, InventoryApi};
use squareup::config::{BaseUri, Configuration, Environment};
use squareup::http::client::HttpClientConfiguration;
use squareup::SquareClient;
use tokio::sync::{mpsc, watch};
use tokio::task::JoinSet;

use crate::coordinator::coordinator;

use crate::observer::{Observer, ObserverConfig};
use crate::testing::generate_poll_series;
use crate::value::Value;
use crate::workers::{platform_writer, timestamped_record_worker};

const CHANNEL_BUFFER: usize = 100;
#[tokio::main]
async fn main() {
    generate_poll_series(0.0, 0.0, 20.0, 100, TimeDelta::seconds(1));
    panic!();

    let config: Config = serde_json::from_slice(&fs::read("config.json").expect("Failed to read config.json!")).expect("Failed to parse config!");
    let mut test_apis = Vec::new();
    for (_, cfg) in config.observers.iter() {
        env::set_var("SQUARE_API_TOKEN", &cfg.token);
        let api = CatalogApi::new(SquareClient::try_new(Configuration {
            environment: Environment::Sandbox, // OPTIONAL if you set the SQUARE_ENVIRONMENT env var
            http_client_config: HttpClientConfiguration::default(),
            base_uri: BaseUri::default(),
        }).expect("Failed to create api"));
        let test_api = InventoryApi::new(SquareClient::try_new(Configuration {
            environment: Environment::Sandbox, // OPTIONAL if you set the SQUARE_ENVIRONMENT env var
            http_client_config: HttpClientConfiguration::default(),
            base_uri: BaseUri::default(),
        }).expect("Failed to create api"));

        test_apis.push((test_api, cfg.clone()));
        // let response = api.list_catalog(&ListCatalogParameters {
        //     cursor: None,
        //     types: Some([ItemVariation].into()),
        //     catalog_version: None,
        // }).await;
        // println!("{:?}", response);
    }
    let mut observers: Vec<Arc<Observer>> = config.observers.into_iter().map(|(uid, cfg)| Arc::new(Observer::new(uid, cfg).unwrap())).collect::<Vec<Arc<Observer>>>();


    // println!("Performing Deviation Calibration for each Observer:");
    // for observer in &mut observers {
    //     observer.deviation_worker().await;
    // }

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

        initial_at = Some(Utc::now());
        // Only reachable if the inital value was equal for all.
        // initial_at = Some(Moment(Epoch::now().unwrap().duration));

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
    let (w_tx, mut w_rx) = watch::channel(None);
    let mut observer_pool = JoinSet::new();
    let mut writer_pool = JoinSet::new();
    for mut observer in observers {
        let val_local = initial_value.clone();
        let at_local = initial_at.clone();
        let tx_local = tx.clone();
        let w_rx_local = w_rx.clone();
        let observer_ref_1 = observer.clone();
        let observer_ref_2 = observer;
        observer_pool.spawn((async move || timestamped_record_worker(&*observer_ref_1, tx_local, initial_at).await)());
        writer_pool.spawn((async move || platform_writer(&*observer_ref_2, w_rx_local).await)());
    }


    let coordinator_fut = coordinator(initial_value, rx, w_tx);
    let (x, y) = test_apis.pop().expect("Failed to get coordinator!");
    futures::join!(observer_pool.join_all(), coordinator_fut);
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
