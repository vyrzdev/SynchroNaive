// use std::collections::HashSet;
// use std::str::FromStr;
// use std::sync::Arc;
// use chrono::{DateTime, TimeDelta, Utc};
// use squareup::models::{BatchChangeInventoryRequest, BatchChangeInventoryResponse, BatchRetrieveInventoryChangesRequest, InventoryChange, InventoryPhysicalCount};
// use squareup::models::DateTime as SquareDateTime;
// use squareup::models::enums::{InventoryChangeType, InventoryState};
// use squareup::models::enums::InventoryState::InStock;
// use tokio::sync::{watch, Mutex};
// use tokio::sync::mpsc::{Sender};
// use tokio::time::sleep;
// use uuid::Uuid;
// use log::{debug, error, info};
// use squareup::models::errors::SquareApiError;
// use crate::interval::{Interval, Moment};
// use crate::observations::{DefinitionPredicate, Observation};
// use crate::SquareObserver;
// use crate::value::{Target, Value};
//
// const IGNORE: &'static str = "IGNORE";
//
// pub fn parse_change(
//     change: InventoryChange,
//     seen: &mut HashSet<String>,
//     deviation: &Deviation,
//     name: String
// ) -> Option<Observation> {
//     match &change.r#type.as_ref().expect("No type in change!") {
//         InventoryChangeType::PhysicalCount => {
//             // Physical Count == Assignment
//
//             // Get Properties.
//             let physical_count = change.physical_count
//                 .expect("Physical Count has no properties!");
//
//             // If tagged IGNORE - is from the writers - don't observe!
//             if let Some(v) = physical_count.reference_id  { if v.eq(IGNORE) { return None }}
//
//             // If seen before - don't observe!
//             let change_id= physical_count.id.expect("Physical Count has no ID!");
//             if seen.contains(&change_id) {
//                 return None;
//             } else {
//                 seen.insert(change_id); // If not seen, seen it now.
//             }
//
//             // Construct Observation.
//             let created_at: DateTime<Utc> = physical_count.created_at.expect("Physical Count had no created date!").into();
//             // TODO: Deviations.
//             let (min, max) = (created_at, created_at);
//             let new_value = Value::from_str(
//                 &physical_count.quantity.expect("Physical Count had no quantity!")
//             ).expect("Unable to parse value from Physical Count!");
//
//             return Some(Observation {
//                 definition: DefinitionPredicate::Assignment { v_new: new_value },
//                 interval: Interval(Moment(min), Moment(max)),
//                 source: name,
//             })
//         },
//         InventoryChangeType::Adjustment => {
//             // Adjustment == Mutation
//
//             // Get Properties.
//             let adjustment = change.adjustment
//                 .expect("Adjustment has no properties!");
//
//             // If tagged IGNORE - is from the writers - don't observe!
//             if let Some(v) = adjustment.reference_id  { if v.eq(IGNORE) { return None }}
//
//             // If seen before - don't observe!
//             let change_id= adjustment.id.expect("Physical Count has no ID!");
//             if seen.contains(&change_id) {
//                 return None;
//             } else {
//                 seen.insert(change_id); // If not seen, seen it now.
//             }
//
//             // Construct Observation.
//             let created_at: DateTime<Utc> = adjustment.created_at.expect("Physical Count had no created date!").into();
//             // TODO: Deviations.
//             let (min, max) = (created_at, created_at);
//
//             // We consider only sales and additions - find delta:
//             let definition;
//             if matches!(adjustment.from_state.as_ref().expect("Adjustment had no FROM state!"), InStock) {
//                 definition = DefinitionPredicate::Mutation {
//                     // Came FROM in-stock. Must be a decrement.
//                     delta: -(i64::from_str(
//                         &adjustment.quantity.expect("Adjustment had no quantity!")
//                     ).expect("Unable to parse value from Adjustment!"))
//                 }
//             } else if  matches!(adjustment.to_state.as_ref().expect("Adjustment had no FROM state!"), InStock) {
//                 definition = DefinitionPredicate::Mutation {
//                     // Went TO in-stock. Must be an increment.
//                     delta: (Value::from_str(
//                         &adjustment.quantity.expect("Adjustment had no quantity!")
//                     ).expect("Unable to parse value from Adjustment!"))
//                 }
//             } else {
//                 error!("Unrecognized FROM/TO state {:?} {:?}!", &adjustment.from_state, &adjustment.to_state);
//                 return None;
//             }
//
//             // Construct Observation.
//             return Some(Observation {
//                 definition,
//                 interval: Interval(Moment(min), Moment(max)),
//                 source: name,
//             })
//         },
//         _ => {
//             debug!("Ignoring Unknown Change Type: {:?}", change.r#type);
//             return None;
//         }
//     }
// }
//
// pub async fn record_worker(
//     observer: Arc<SquareObserver>,
//     target: Target,
//     backoff: TimeDelta,
//     output: Sender<Observation>,
// ) -> ! {
//     let since = SquareDateTime::now();
//     let mut request = BatchRetrieveInventoryChangesRequest {
//         catalog_object_ids: Some(vec![target.1]),
//         location_ids: Some(vec![target.0]),
//         types: None,
//         states: None,
//         updated_after: Some(since),
//         updated_before: None,
//         cursor: None,
//         limit: None,
//     };
//
//     // TODO: Clear Buffer When?
//     let mut seen = HashSet::new();
//
//     loop {
//         let response = observer.inventory_api.batch_retrieve_inventory_changes(
//             &mut request,
//         ).await;
//         request.updated_after = Some(SquareDateTime::from(&(Utc::now() - TimeDelta::seconds(5))));
//
//         if let Some(changes) = response.unwrap().changes {
//             for change in changes {
//                 debug!("{:?}", change);
//                 if let Some(obs) = parse_change(
//                     change,
//                     &mut seen,
//                     &(TimeDelta::new(0,0).expect("Foo"), TimeDelta::new(0,0).expect("Foo")), // TODO: Deviations.
//                     observer.name.clone()
//                 ) {
//                     info!("{} - New Observation: {:?}",observer.name,  obs);
//                     output.send(obs).await.unwrap();
//                 }
//             }
//         }
//         sleep(backoff.to_std().unwrap()).await;
//     }
// }
//
// pub enum PollingInterpretation {
//     Mutation,
//     Assignment,
//     Transition
// }
//
// // (mut last_state, mut last_at): (Value, Moment)
// pub async fn poll_worker(
//     observer: Arc<SquareObserver>,
//     interpretation: PollingInterpretation,
//     target: Target,
//     backoff: TimeDelta,
//     name: String,
//     mutex: Arc<Mutex<Option<Value>>>,
//     output: Sender<Observation>
// ) {
//     let (mut last_state, mut last_sent) = (mutex, None);
//
//     loop {
//         {
//             let mut guard = last_state.lock().await;
//
//             let (state, sent, replied) = observer.request(target.clone()).await.unwrap();
//
//             if (*guard).is_some() && (*guard) != Some(state) {
//                 output.send(Observation {
//                     definition: match interpretation {
//                         PollingInterpretation::Mutation => {
//                             DefinitionPredicate::Mutation {delta: state - (*guard).unwrap()}
//                         }
//                         PollingInterpretation::Assignment => {
//                             DefinitionPredicate::Assignment {v_new: state}
//                         }
//                         PollingInterpretation::Transition => {
//                             DefinitionPredicate::Transition {
//                                 v_0: (*guard).unwrap(), v_1: state
//                             }
//                         }
//                     },
//                     interval: Interval(Moment(last_sent.unwrap()), Moment(replied)),
//                     source: name.clone(),
//                 }).await.unwrap();
//             }
//             *guard = Some(state);
//             last_sent = Some(sent);
//         }
//         sleep(backoff.to_std().unwrap()).await;
//     }
// }
//
// pub async fn write(observer: &SquareObserver, target: Target, value: Value) -> Result<BatchChangeInventoryResponse, SquareApiError> {
//     observer.inventory_api.batch_change_inventory(
//         &BatchChangeInventoryRequest {
//             idempotency_key: Uuid::new_v4().to_string(),
//             changes: Some(vec![
//                 InventoryChange {
//                     r#type: Some(InventoryChangeType::PhysicalCount),
//                     physical_count: Some(
//                         InventoryPhysicalCount {
//                             id: None,
//                             reference_id: Some(IGNORE.to_string()),
//                             catalog_object_id: Some(target.1),
//                             catalog_object_type: None,
//                             state: Some(InventoryState::InStock),
//                             location_id: Some(target.0),
//                             quantity: Some(value.to_string()),
//                             source: None,
//                             employee_id: None,
//                             team_member_id: None,
//                             occurred_at: Some(SquareDateTime::now()), // TODO: Preserve changes since some time?
//                             created_at: None,
//                         }
//                     ),
//                     adjustment: None,
//                     transfer: None,
//                     measurement_unit: None,
//                     measurement_unit_id: None,
//                 }
//             ]),
//             ignore_unchanged_counts: None,
//         }
//     ).await
// }
//
//
// pub async fn record_write_worker(
//     observer: Arc<SquareObserver>,
//     target: Target,
//     mut next: watch::Receiver<Option<Value>>
// ) {
//     loop {
//         next.changed().await.unwrap(); // Passes when new value available.
//         let local_next = next.borrow().clone(); // Take new value (save locally so can be changed while proc)
//
//         if let Some(v) = local_next {
//             if matches!(write(&observer, target.clone(), v).await, Err(_)) {
//                 error!("Writer {} - Failed to write to target!",observer.name);
//             }
//         } else {
//             info!("Writer {} - Conflict! No Available Value", observer.name);
//         }
//     }
// }
//
// pub async fn polling_write_worker(
//     observer: Arc<SquareObserver>,
//     target: Target,
//     mutex: Arc<Mutex<Option<Value>>>,
//     mut next: watch::Receiver<Option<Value>>
// ) {
//     loop {
//         next.changed().await.unwrap(); // Passes when new value available.
//         let local_next = next.borrow().clone(); // Take new value (save locally so can be changed while proc)
//         if let Some(v) = local_next {
//             // Wait for mutex over value
//             {
//                 let mut lock = mutex.lock().await;
//                 *lock = Some(v.clone()); // Last value is now new value.
//                 if matches!(write(&observer, target.clone(), v).await, Err(_)) {
//                     error!("Writer {} - Failed to write to target!",observer.name);
//                 }
//             }
//         } else {
//             info!("Writer {} - Conflict! No Available Value", observer.name);
//         }
//     }
// }


