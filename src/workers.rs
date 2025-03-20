use std::collections::HashSet;
use std::str::FromStr;
use std::time::Duration;
use chrono::{DateTime, TimeDelta, Utc};
use squareup::models::{BatchChangeInventoryRequest, BatchRetrieveInventoryChangesRequest, InventoryChange, InventoryPhysicalCount};
use squareup::models::DateTime as SquareDateTime;
use squareup::models::enums::{InventoryChangeType, InventoryState};
use squareup::models::enums::InventoryState::InStock;
use tokio::sync::{watch};
use tokio::sync::mpsc::{Sender};
use tokio::time::sleep;
use uuid::Uuid;
use crate::interval::{Interval, Moment};
use crate::observations::{DefinitionPredicate, Observation};
use crate::observer::Observer;
use crate::value::Value;

pub async fn timestamped_record_worker(observer: &Observer, out_chan: Sender<Observation>, since: DateTime<Utc>) {

    let mut request = BatchRetrieveInventoryChangesRequest {
        catalog_object_ids: Some(vec![observer.config.target.clone()]),
        location_ids: Some(vec![observer.config.location_id.clone()]),
        types: None,
        states: None,
        updated_after: Some(SquareDateTime::from(&since)),
        updated_before: None,
        cursor: None,
        limit: None,
    };

    let mut seen = HashSet::new();

    loop {
        let new_since = SquareDateTime::from(&(Utc::now() - TimeDelta::seconds(5))) ;
        let change_records_resp = observer.api.batch_retrieve_inventory_changes(
            &request
        ).await;
        request.updated_after = Some(new_since);

        if let Ok(change_records) = change_records_resp {
            if let Some(changes) = change_records.changes {
                for change in changes {
                    match change.r#type.expect("Change had no type!") {
                        InventoryChangeType::PhysicalCount => {
                            let physical_count = change.physical_count.expect("Physical Count has no properties!");
                            if let Some(v) = physical_count.reference_id  {
                                if v == "IGNORE" {
                                    continue
                                }
                            }
                            let change_id= physical_count.id.expect("Physical Count has no ID!");
                            if seen.contains(&change_id) {
                                continue;
                            } else {
                                seen.insert(change_id);
                            }


                            let created_at: DateTime<Utc> = physical_count.created_at.expect("Physical Count had no created date!").into();
                            let (min, max) = (created_at + observer.deviation_min, created_at + observer.deviation_max);
                            let definition = DefinitionPredicate::Assignment { v_new: i64::from_str(&physical_count.quantity.expect("Physical Count had no quantity!")).expect("Quantity was not integer!") };
                            out_chan.send(
                                Observation {
                                    definition: definition,
                                    interval: Interval(Moment(min), Moment(max)),
                                    source: observer.uuid.clone(),
                                }
                            ).await.unwrap();
                            println!("Physical Count Generated!");
                        },
                        InventoryChangeType::Adjustment => {
                            let adjustment = change.adjustment.expect("Adjustment had no properties!");
                            if let Some(v) = adjustment.reference_id  {
                                if v == "IGNORE" {
                                    continue
                                }
                            }
                            let change_id= adjustment.id.expect("Physical Count has no ID!");
                            if seen.contains(&change_id) {
                                continue;
                            } else {
                                seen.insert(change_id);
                            }

                            let created_at: DateTime<Utc> = adjustment.created_at.expect("Adjustment had no created date!").into();
                            let (min, max) = (created_at + observer.deviation_min, created_at + observer.deviation_max);

                            let definition;
                            if adjustment.from_state.expect("Adjustment had no from state!") == InStock {
                                definition = DefinitionPredicate::Mutation { delta: -(i64::from_str(&adjustment.quantity.expect("Adjustment had no quantity!")).expect("Quantity was not integer!")) }
                            } else if adjustment.to_state.expect("Adjustment had no to state!") == InStock {
                                definition = DefinitionPredicate::Mutation { delta: (i64::from_str(&adjustment.quantity.expect("Adjustment had no quantity!")).expect("Quantity was not integer!")) }
                            } else {
                                println!("Unrecognized from/to state");
                                continue;
                            }


                            out_chan.send(
                                Observation {
                                    definition: definition,
                                    interval: Interval(Moment(min), Moment(max)),
                                    source: observer.uuid.clone(),
                                }
                            ).await.unwrap();
                            println!("Adjustment Generated!");
                        },
                        InventoryChangeType::Transfer => {
                            eprintln!("Experienced Transfer - We do not consider these!");
                        }
                    }
                }
            } else {
                println!("No changes!")
            }
        } else {
            eprintln!("Failed to request Records!");
            println!("{:?}", change_records_resp);

        }
        sleep(Duration::from_millis(2000)).await;
    }
}

pub enum PollingInterp {
    Mutation,
    Assignment,
    Transition
}

pub async fn poll_worker(observer: &Observer, (mut last_state, mut last_at): (Value, Moment), sync: Sender<Observation>) {
    loop {
        let (state, at) = observer.request(&observer.config.target).await.unwrap();

        if state != last_state {
            sync.send(Observation {
                definition: DefinitionPredicate::Transition {v_0: last_state, v_1: state}, // ref XXX.
                interval: Interval(last_at, at.1), // ref XXX.
                source: observer.uuid.clone(),
            }).await.unwrap()
        } else {
            last_at = at.0;
        }
    }
}

pub async fn platform_writer(observer: &Observer, mut next: watch::Receiver<Option<Value>>) {
    loop {
        next.changed().await.unwrap(); // Wait until a new value is available
        let local_next = next.borrow().clone();
        {
            if let Some(v) = local_next {
                let reference_key = "IGNORE".to_string();
                let update_resp = observer.api.batch_change_inventory(
                    &BatchChangeInventoryRequest {
                        idempotency_key: Uuid::new_v4().to_string(),
                        changes: Some(vec![
                            InventoryChange {
                                r#type: Some(InventoryChangeType::PhysicalCount),
                                physical_count: Some(
                                    InventoryPhysicalCount {
                                        id: None,
                                        reference_id: Some(reference_key.clone()),
                                        catalog_object_id: Some(observer.config.target.clone()),
                                        catalog_object_type: None,
                                        state: Some(InventoryState::InStock),
                                        location_id: Some(observer.config.location_id.clone()),
                                        quantity: Some(v.to_string()),
                                        source: None,
                                        employee_id: None,
                                        team_member_id: None,
                                        occurred_at: Some(SquareDateTime::from(&Utc::now())),
                                        created_at: None,
                                    }
                                ),
                                adjustment: None,
                                transfer: None,
                                measurement_unit: None,
                                measurement_unit_id: None,
                            }
                        ]),
                        ignore_unchanged_counts: None,
                    }
                ).await;
                if let Ok(resp) = update_resp {
                    println!("Stock Updated!")
                } else {
                    eprintln!("Failed to update stock!");
                    println!("{:?}", update_resp);
                }
            }
        }
    }
}


