use rand::prelude::{Distribution, ThreadRng};
use rand_distr::{Exp, Normal};
use rand_distr::num_traits::ToPrimitive;
use crate::interval::Tick;
use crate::observations::DefinitionPredicate;

pub type Lambda = f64;
pub type Event = (DefinitionPredicate, Tick);

pub fn exp(lambda: Lambda, rng: &mut ThreadRng) -> Tick {
    Exp::new(lambda).unwrap().sample(rng).to_u64().unwrap()
}

pub fn norm(lambda: Lambda, std_dev: Lambda, rng: &mut ThreadRng) -> Tick {
    Normal::new(lambda, std_dev).unwrap().sample(rng).to_u64().unwrap()
}






// A seller who experiences avg X sales a day
// experiences X/12 sales an hour (we assume none at midnight)
// experiences X/16/60 sales every minute on average

// TESTING IN PRACTICAL ENVIRONMENT
// EVALUATES - CONFLICT RATE, AVERAGE TIME TO CONVERGE, ADHERENCE?
//
// async fn make_sale(observer: Arc<SquareObserver>, location_id: String, target: String) {
//     let request = BatchChangeInventoryRequest {
//         idempotency_key: uuid::Uuid::new_v4().to_string(),
//         changes: Some(vec![
//             InventoryChange {
//                 r#type: Some(InventoryChangeType::Adjustment),
//                 physical_count: None,
//                 adjustment: Some(InventoryAdjustment {
//                     id: None,
//                     reference_id: None,
//                     from_state: Some(InventoryState::InStock),
//                     to_state: Some(InventoryState::Sold),
//                     location_id: Some(location_id),
//                     catalog_object_id: Some(target.clone()),
//                     catalog_object_type: None,
//                     quantity: Some("1".to_string()),
//                     total_price_money: None,
//                     occurred_at: Some(DateTime::new()),
//                     created_at: None,
//                     source: None,
//                     employee_id: None,
//                     team_member_id: None,
//                     transaction_id: None,
//                     refund_id: None,
//                     purchase_order_id: None,
//                     goods_receipt_id: None,
//                     adjustment_group: None,
//                 }),
//                 transfer: None,
//                 measurement_unit: None,
//                 measurement_unit_id: None,
//             }
//         ]),
//         ignore_unchanged_counts: None,
//     };
//
//     observer.inventory_api.batch_change_inventory(&request).await.unwrap();
//     println!("Made Sale!")
// }
//
// async fn make_recount(observer: Arc<SquareObserver>, location_id: String, target: String, value: String) {
//     observer.inventory_api.batch_change_inventory(
//         &BatchChangeInventoryRequest {
//             idempotency_key: Uuid::new_v4().to_string(),
//             changes: Some(vec![
//                 InventoryChange {
//                     r#type: Some(InventoryChangeType::PhysicalCount),
//                     physical_count: Some(
//                         InventoryPhysicalCount {
//                             id: None,
//                             reference_id: None,
//                             catalog_object_id: Some(target),
//                             catalog_object_type: None,
//                             state: Some(InventoryState::InStock),
//                             location_id: Some(location_id),
//                             quantity: Some(value),
//                             source: None,
//                             employee_id: None,
//                             team_member_id: None,
//                             occurred_at: Some(DateTime::now()),
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
//     ).await.unwrap();
// }
//
// pub async fn external_users(observer: Arc<SquareObserver>, config: SquareObserverConfig, lambda: f64) {
//     // Average Rate of Sales per minute = lambda
//     // Therefore: Average time between sales is given by EXP 1/lambda
//     let exp = Exp::new(lambda).unwrap();
//
//
//     let first_wait = exp.sample(&mut rand::rng()) * (60000f64);
//     sleep(core::time::Duration::from_millis(5000 + (first_wait as u64))).await;
//     info!("Started Mocking External Users!");
//
//
//     let mut join_set = JoinSet::new();
//     loop {
//         let wait = exp.sample(&mut rand::rng()) * (60000f64); // Get milliseconds wait (1/lambda * 60000)
//
//         join_set.spawn(make_sale(observer.clone(), config.location_id.clone(), config.target.clone()));
//
//         if wait < 0.01 {
//             continue // Don't send yet - add another one!
//         } else {
//             join_set.join_all().await; // Wait for them to send (and reply)
//             join_set = JoinSet::new(); // Wipe it clear.
//             sleep(core::time::Duration::from_millis(wait as u64)).await; // wait for given time.
//         }
//     }
// }
//
// pub async fn manual_editor(observer: Arc<SquareObserver>, config: SquareObserverConfig, lambda: f64) {
//     // Average rate of manual edits per minute = lambda
//     // Therefore: Average time between edits is given by EXP 1/lambda
//     let exp = Exp::new(lambda).unwrap();
//
//     let first_wait = exp.sample(&mut rand::rng()) * (60000f64);
//     sleep(core::time::Duration::from_millis(5000 + (first_wait as u64))).await;
//     info!("Started Mocking Manual Editors!");
//     let mut join_set = JoinSet::new();
//     loop {
//         let wait = exp.sample(&mut rand::rng()) * (60000f64); // Get milliseconds wait (1/lambda * 60000)
//
//         join_set.spawn(make_recount(observer.clone(), config.location_id.clone(), config.target.clone(), "10".to_string()));
//
//         if wait < 0.01 {
//             continue // Don't send yet - add another one!
//         } else {
//             join_set.join_all().await; // Wait for them to send (and reply)
//             join_set = JoinSet::new(); // Wipe it clear.
//             sleep(core::time::Duration::from_millis(wait as u64)).await; // wait for given time.
//         }
//     }
// }


// TESTING IN SYNTHETIC ENVIRONMENT
// EVALUATES: Correctness
// #[derive(Debug)]
// pub struct MockObservation {
//     // true_interval: (chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>),
//     pub(crate) uncertainty_interval: (chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>),
//     pub(crate) true_actions: Vec<(DefinitionPredicate, chrono::DateTime<chrono::Utc>)>,
//     pub(crate) observed_definition: Option<DefinitionPredicate>,
//     pub(crate) visible_at: chrono::DateTime<chrono::Utc>
// }
//
// impl MockObservation {
//     pub(crate) fn pretty_output(&self, from: &chrono::DateTime<Utc>) -> String {
//         let label = match self.observed_definition.unwrap() {
//             DefinitionPredicate::Transition { v_0, v_1 } => {format!("TR ({} -> {})", v_0, v_1)},
//             DefinitionPredicate::Mutation { delta } => {format!("MU ({delta})")},
//             DefinitionPredicate::Assignment { v_new } => {format!("AS ({v_new})")},
//         };
//
//         format!(
//             "{label} over {}, {} at {}",
//             ((self.uncertainty_interval.0 - from).num_milliseconds() as f64)/1000.0,
//             ((self.uncertainty_interval.1 - from).num_milliseconds() as f64)/1000.0,
//             ((self.visible_at - from).num_milliseconds() as f64)/1000.0
//         )
//     }
// }
//
// pub fn pretty_print_event_time_pair((event, at): &(DefinitionPredicate, chrono::DateTime<Utc>), from: &chrono::DateTime<Utc>) -> String {
//     let label = match &event {
//         DefinitionPredicate::Transition { .. } => {"TR"}
//         DefinitionPredicate::Mutation { .. } => {"MU"}
//         DefinitionPredicate::Assignment { .. } => {"AS"}
//     };
//     format!("| {label} @ {} |", ((*at - from).num_milliseconds() as f64)/1000.0)
// }
//
//
// pub fn generate_sales(sales_lambda: f64, from: chrono::DateTime<Utc>, until: chrono::DateTime<Utc>) -> Vec<(DefinitionPredicate, chrono::DateTime<Utc>)> {
//     // Assuming Exponential Distribution for Sales
//     // Sales occur at average rate of sales_lambda per-minute.
//     // Exp(sales_lambda) -> time to next sale as a multiplier of minutes.
//     let tts = Exp::new(sales_lambda).unwrap();
//
//
//     let mut build = Vec::new(); // TODO: With Capacity?
//     let mut real_time = from;
//     while real_time < until {
//         let next_sale_multiplier = tts.sample(&mut rand::rng());
//         let next_sale_in = TimeDelta::milliseconds((next_sale_multiplier * 60000f64) as i64); // Next sale in X milliseconds.
//         info!("Time: {}, Next In: {}", real_time, next_sale_in);
//         real_time += next_sale_in;
//
//         if real_time < until {
//             build.push((DefinitionPredicate::Mutation { delta: -1}, real_time.clone()));
//         }
//     }
//     return build;
// }
// pub fn generate_edits(edit_lambda: f64, from: chrono::DateTime<Utc>, until: chrono::DateTime<Utc>, to: Value) -> Vec<(DefinitionPredicate, chrono::DateTime<Utc>)> {
//     // Assuming Exponential Distribution for Edits
//     // Edits occur at average rate of edit_lambda per-minute.
//     // Exp(edit_lambda) -> time to next edit as a multiplier of minutes.
//     let tte = Exp::new(edit_lambda).unwrap();
//
//
//     let mut build = Vec::new(); // TODO: With Capacity?
//     let mut real_time = from;
//     while real_time < until {
//         let next_edit_multiplier = tte.sample(&mut rand::rng());
//         let next_edit_in = TimeDelta::milliseconds((next_edit_multiplier * 60000f64) as i64); // Next edit in X milliseconds.
//         real_time += next_edit_in;
//
//         if real_time < until {
//             build.push((DefinitionPredicate::Assignment { v_new: to.clone()}, real_time.clone()));
//         }
//     }
//     return build;
// }
//
// pub fn generate_polled_records(
//     events: Vec<(DefinitionPredicate, chrono::DateTime<Utc>)>,
//     from: chrono::DateTime<Utc>,
//     backoff: TimeDelta,
//     rtt_lambda: f64,
//     deviation_lambda: f64,
//     observed_min_deviation: chrono::TimeDelta,
//     observed_max_deviation: chrono::TimeDelta,
// ) -> Vec<MockObservation> {
//     // Assume normal distribution for RTT.
//     // Assuming AVG send/reply time is 1/2 RTT:
//     let ttp = Normal::new(0.5 * rtt_lambda, 1.0).unwrap();
//     let ttr = Normal::new(0.5 * rtt_lambda, 1.0).unwrap();
//
//     // Assume normal distribution for Deviation.
//     let deviation = Normal::new(deviation_lambda, 0.0).unwrap(); // TODO: STD DEV
//
//     // Assuming events are time ordered.
//     let last_event_at = events.last().unwrap().1;
//
//     let mut real_time = from;
//     let mut next_event = 0;
//     let mut results = Vec::new();
//
//     let mut poll_number = 0;
//     let mut last_sent = from;
//
//     loop {
//         if next_event == events.len() {
//             break; // Last event included in last interval!
//         }
//         let sent = real_time;
//         real_time += TimeDelta::milliseconds(ttp.sample(&mut rand::rng()) as i64); // In Transit to Platform.
//         let processed = real_time;
//         real_time += TimeDelta::milliseconds(ttr.sample(&mut rand::rng()) as i64); // In Transit to Observer.
//         let replied = real_time;
//
//         let mut true_events = Vec::new();
//         while &events[next_event].1 < &processed {
//             true_events.push(events[next_event]);
//             next_event += 1;
//             if next_event == events.len() {
//                 break; // Last event included in last interval!
//             }
//         }
//
//         for event in true_events {
//             let reported_timestamp = event.1 + TimeDelta::milliseconds(deviation.sample(&mut rand::rng()) as i64);
//
//             results.push(MockObservation {
//                 uncertainty_interval: ((reported_timestamp + observed_min_deviation), min((reported_timestamp+ observed_max_deviation), replied)),
//                 true_actions: vec![(event.0, event.1)],
//                 observed_definition: Some(event.0),
//                 visible_at: replied,
//             });
//         }
//
//         real_time += backoff;
//         poll_number += 1;
//     }
//     return results;
// }
// pub fn generate_polls(
//     events: Vec<(DefinitionPredicate, chrono::DateTime<Utc>)>,
//     from: chrono::DateTime<Utc>,
//     initial_value: Value,
//     backoff: TimeDelta,
//     rtt_lambda: f64,
//     interpretation: PollingInterpretation
// ) -> Vec<MockObservation> {
//     if events.is_empty() {
//         return Vec::new();
//     }
//
//     // Assume normal distribution for RTT.
//     // Assuming AVG send/reply time is 1/2 RTT:
//     let ttp = Normal::new(0.5 * rtt_lambda, 1.0).unwrap();
//     let ttr = Normal::new(0.5 * rtt_lambda, 1.0).unwrap();
//
//     // Assuming events are time ordered.
//     let last_event_at = events.last().unwrap().1;
//
//     let mut real_time = from;
//     let mut value = initial_value;
//     let mut next_event = 0;
//     let mut results = Vec::new();
//
//     let mut poll_number = 0;
//     let mut last_sent = from;
//
//     loop {
//         if next_event == events.len() {
//             break; // Last event included in last interval!
//         }
//         let sent = real_time;
//         real_time += TimeDelta::milliseconds(ttp.sample(&mut rand::rng()) as i64); // In Transit to Platform.
//         let processed = real_time;
//         real_time += TimeDelta::milliseconds(ttr.sample(&mut rand::rng()) as i64); // In Transit to Observer.
//         let replied = real_time;
//
//         let v_0 = value; // Value at the last poll.
//         if poll_number > 0 {
//             let mut true_events = Vec::new();
//             while &events[next_event].1 < &processed {
//                 value = events[next_event].0.apply(&value).unwrap(); // Since all are mut or assn - always defined.
//                 true_events.push(events[next_event]);
//                 next_event += 1;
//                 if next_event == events.len() {
//                     break; // Last event included in last interval!
//                 }
//             }
//
//             let v_1 = value; // Value at this poll.
//
//             if v_1 != v_0 {
//                 results.push(MockObservation {
//                     uncertainty_interval: (last_sent, replied),
//                     true_actions: true_events,
//                     observed_definition: match &interpretation {
//                         PollingInterpretation::Mutation => {
//                             Some(DefinitionPredicate::Mutation {delta: (v_1 - v_0)})
//                         }
//                         PollingInterpretation::Assignment => {
//                             Some(DefinitionPredicate::Assignment {v_new: v_1})
//                         }
//                         PollingInterpretation::Transition => {
//                             Some(DefinitionPredicate::Transition { v_0, v_1 })
//                         }
//                     },
//                     visible_at: replied,
//                 });
//             }
//         }
//
//         last_sent = sent;
//         real_time += backoff;
//         poll_number += 1;
//     }
//
//     return results;
// }