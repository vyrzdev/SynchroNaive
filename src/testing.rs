use std::sync::Arc;
use chrono::{TimeDelta, TimeZone, Utc};
use rand::prelude::Distribution;
use rand::Rng;
use rand_distr::{Exp, Normal};
use squareup::api::InventoryApi;
use squareup::models::{BatchChangeInventoryRequest, DateTime, InventoryAdjustment, InventoryChange, InventoryPhysicalCount};
use squareup::models::enums::{InventoryChangeType, InventoryState};
use tokio::task::JoinSet;
use tokio::time::sleep;
use uuid::Uuid;
use crate::observations::DefinitionPredicate;
use crate::observer::SquareObserverConfig;
use crate::value::Value;
use crate::workers::PollingInterpretation;
// A seller who experiences avg X sales a day
// experiences X/12 sales an hour (we assume none at midnight)
// experiences X/16/60 sales every minute on average

// TESTING IN PRACTICAL ENVIRONMENT
// EVALUATES - CONFLICT RATE, AVERAGE TIME TO CONVERGE, ADHERENCE?

async fn make_sale(api: Arc<InventoryApi>, location_id: String, target: String) {
    let request = BatchChangeInventoryRequest {
        idempotency_key: uuid::Uuid::new_v4().to_string(),
        changes: Some(vec![
            InventoryChange {
                r#type: Some(InventoryChangeType::Adjustment),
                physical_count: None,
                adjustment: Some(InventoryAdjustment {
                    id: None,
                    reference_id: None,
                    from_state: Some(InventoryState::InStock),
                    to_state: Some(InventoryState::Sold),
                    location_id: Some(location_id),
                    catalog_object_id: Some(target.clone()),
                    catalog_object_type: None,
                    quantity: Some("1".to_string()),
                    total_price_money: None,
                    occurred_at: Some(DateTime::new()),
                    created_at: None,
                    source: None,
                    employee_id: None,
                    team_member_id: None,
                    transaction_id: None,
                    refund_id: None,
                    purchase_order_id: None,
                    goods_receipt_id: None,
                    adjustment_group: None,
                }),
                transfer: None,
                measurement_unit: None,
                measurement_unit_id: None,
            }
        ]),
        ignore_unchanged_counts: None,
    };

    api.batch_change_inventory(&request).await.unwrap();
    println!("Made Sale!")
}

async fn make_recount(api: Arc<InventoryApi>, location_id: String, target: String, value: String) {
    api.batch_change_inventory(
        &BatchChangeInventoryRequest {
            idempotency_key: Uuid::new_v4().to_string(),
            changes: Some(vec![
                InventoryChange {
                    r#type: Some(InventoryChangeType::PhysicalCount),
                    physical_count: Some(
                        InventoryPhysicalCount {
                            id: None,
                            reference_id: None,
                            catalog_object_id: Some(target),
                            catalog_object_type: None,
                            state: Some(InventoryState::InStock),
                            location_id: Some(location_id),
                            quantity: Some(value),
                            source: None,
                            employee_id: None,
                            team_member_id: None,
                            occurred_at: Some(DateTime::now()),
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
    ).await.unwrap();
}

async fn external_users(api: InventoryApi, config: SquareObserverConfig, lambda: f64) {
    let api_ref = Arc::new(api);

    // Average Rate of Sales per minute = lambda
    // Therefore: Average time between sales is given by EXP 1/lambda
    let exp = Exp::new(lambda).unwrap();

    let mut join_set = JoinSet::new();
    loop {
        let wait = exp.sample(&mut rand::rng()) * (60000f64); // Get milliseconds wait (1/lambda * 60000)

        join_set.spawn(make_sale(api_ref.clone(), config.location_id.clone(), config.target.clone()));

        if wait < 0.01 {
            continue // Don't send yet - add another one!
        } else {
            join_set.join_all().await; // Wait for them to send (and reply)
            join_set = JoinSet::new(); // Wipe it clear.
            sleep(core::time::Duration::from_millis(wait as u64)).await; // wait for given time.
        }
    }
}

async fn manual_editor(api: InventoryApi, config: SquareObserverConfig, lambda: f64) {
    let api_ref = Arc::new(api);

    // Average rate of manual edits per minute = lambda
    // Therefore: Average time between edits is given by EXP 1/lambda
    let exp = Exp::new(lambda).unwrap();

    let mut join_set = JoinSet::new();
    loop {
        let wait = exp.sample(&mut rand::rng()) * (60000f64); // Get milliseconds wait (1/lambda * 60000)

        join_set.spawn(make_recount(api_ref.clone(), config.location_id.clone(), config.target.clone(), "10".to_string()));

        if wait < 0.01 {
            continue // Don't send yet - add another one!
        } else {
            join_set.join_all().await; // Wait for them to send (and reply)
            join_set = JoinSet::new(); // Wipe it clear.
            sleep(core::time::Duration::from_secs(wait as u64)).await; // wait for given time.
        }
    }
}


// TESTING IN SYNTHETIC ENVIRONMENT
// EVALUATES: Correctness
#[derive(Debug)]
pub struct MockObservation {
    // true_interval: (chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>),
    uncertainty_interval: (chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>),
    true_actions: Vec<(DefinitionPredicate, chrono::DateTime<chrono::Utc>)>,
    observed_definition: Option<DefinitionPredicate>,
    visible_at: chrono::DateTime<chrono::Utc>
}

pub fn generate_poll_series(sale_lambda: f64, count_lambda: f64, rtt_lambda: f64, poll_count: u64, initial_value: Value, backoff: chrono::TimeDelta, default_interp: PollingInterpretation) -> Vec<MockObservation> {
    // Sale/Edit lambda are given per-minute.
    // Polling backoff is timedelta
    // RTT is in milliseconds.
    // TODO: What scale should realtime be? Nanosecond or Millisecond?
    // TODO: Seems that it should be higher than observable level.

    // BUILDING POLL RESULTS:
    // Assuming AVG time to platform, and AVG reply time is 1/2 avg RTT:
    let ttp = Normal::new(0.5 * rtt_lambda, 1.0).unwrap();
    let ttr = Normal::new(0.5 * rtt_lambda, 1.0).unwrap();

    let mut results = Vec::with_capacity(poll_count as usize);

    let mut real_time = chrono::DateTime::<Utc>::MIN_UTC; // From some time 0.
    let mut last_poll_sent = chrono::DateTime::<Utc>::MIN_UTC;
    let mut last_poll_processed_at = chrono::DateTime::<Utc>::MIN_UTC;
    for poll in 0..poll_count {
        let sent = real_time;
        real_time += TimeDelta::milliseconds(ttp.sample(&mut rand::rng()) as i64); // Poll in Transit to Platform
        let processed = real_time; // Poll processed by platform.
        real_time += TimeDelta::milliseconds(ttr.sample(&mut rand::rng()) as i64); // Poll Reply in Transit
        let replied = real_time;
        real_time += backoff; // Add the Polling Backoff - Wait this time before next poll.

        if poll != 0 {
            // If has some previous poll...
            // Changes may have occurred over the interval:
            results.push(((last_poll_processed_at, processed),MockObservation {
                // true_interval: (last_poll_processed_at, processed),
                uncertainty_interval: (last_poll_sent, replied),
                true_actions: vec![],
                observed_definition: None,
                visible_at: replied
            }));
        }
        last_poll_sent = sent;
        last_poll_processed_at = processed;
    }

    let final_real_time = real_time;
    real_time = chrono::DateTime::<Utc>::MIN_UTC;

    // Assuming Exponential Distribution for Sales
    let tts = Exp::new(sale_lambda).unwrap();

    // Generating Sales
    while real_time < final_real_time {
        let next_sale = TimeDelta::milliseconds(tts.sample(&mut rand::rng()) as i64);
        real_time += next_sale;

        if real_time < final_real_time {
            for result in &mut results {
                if real_time > result.0.0 && real_time < result.0.1 {
                    result.1.true_actions.push((DefinitionPredicate::Mutation {
                        delta: -1
                    }, real_time));
                }
            }
        }
    }

    real_time = chrono::DateTime::<Utc>::MIN_UTC;

    // Assuming Exponential Distribution for Counts
    let ttc = Exp::new(count_lambda).unwrap();

    // Generating Counts
    while real_time < final_real_time {
        let next_sale = TimeDelta::milliseconds(ttc.sample(&mut rand::rng()) as i64);
        real_time += next_sale;

        if real_time < final_real_time {
            for result in &mut results {
                if real_time > result.0.0 && real_time < result.0.1 {
                    result.1.true_actions.push((DefinitionPredicate::Assignment {
                        v_new: 10 //TODO: Does the number matter?
                    }, real_time));
                }
            }
        }
    }

    // Sort all actions in true actions by time.
    // TODO: Is precision enough to trust ordering?
    for result in &mut results {
        result.1.true_actions.sort_by_key(|x| x.1);
    }

    // Generate observed definitions
    let mut cumulative = initial_value;
    let mut build = Vec::new();
    for mut result in results {
        // Since all predicates in mocked are assn or mut - all are defined for arbitrary input
        let mut next = cumulative;
        for (predicate, _) in &result.1.true_actions {
            match predicate {
                DefinitionPredicate::Transition { .. } => panic!(), // Should be impossible!
                DefinitionPredicate::Mutation { delta } => {next = next + delta}
                DefinitionPredicate::Assignment { v_new } => {next = *v_new }
            }
        }

        match default_interp {
            PollingInterpretation::Mutation => {
                result.1.observed_definition = Some(DefinitionPredicate::Mutation {delta: (next - cumulative)})
            }
            PollingInterpretation::Assignment => {
                result.1.observed_definition = Some(DefinitionPredicate::Assignment {v_new: next})
            }
            PollingInterpretation::Transition => {
                result.1.observed_definition = Some(DefinitionPredicate::Transition {v_0: cumulative, v_1: next})
            }
        }
        build.push(result.1);
        cumulative = next;
    };

    return build;
}


fn random_timestamp_within(start: chrono::DateTime<Utc>, end: chrono::DateTime<Utc>) -> chrono::DateTime<Utc> {
    let start_timestamp = start.timestamp(); // Seconds since Unix epoch
    let end_timestamp = end.timestamp();

    // Generate a random timestamp in the range
    let random_timestamp = rand::rng().random_range(start_timestamp..=end_timestamp);

    // Convert back to DateTime<Utc>
    Utc.timestamp_opt(random_timestamp, 0).unwrap()
}
pub fn generate_record_series(sale_lambda: f64, count_lambda: f64, until: chrono::DateTime<chrono::Utc>, min_deviation: chrono::TimeDelta, max_deviation: chrono::TimeDelta, rtt_lambda: f64, backoff: chrono::TimeDelta) -> Vec<MockObservation> {
    // realtime on the observer side.
    let mut real_time = chrono::DateTime::<Utc>::MIN_UTC;
    let mut events = Vec::new();

    // Generate Sales
    let tts = Exp::new(sale_lambda).unwrap();

    while real_time < until {
        let next_sale = TimeDelta::milliseconds(tts.sample(&mut rand::rng()) as i64);
        real_time += next_sale;

        events.push((real_time, DefinitionPredicate::Mutation {delta: -1}))
    }

    real_time = chrono::DateTime::<Utc>::MIN_UTC;
    // Generate Counts
    let ttc = Exp::new(count_lambda).unwrap();

    while real_time < until {
        let next_sale = TimeDelta::milliseconds(ttc.sample(&mut rand::rng()) as i64);
        real_time += next_sale;

        events.push((real_time, DefinitionPredicate::Assignment {v_new: 10})) // TODO: Does this number matter?
    }

    // Generate Polls (for records)
    // Assuming AVG time to platform, and AVG reply time is 1/2 avg RTT:
    let ttp = Normal::new(0.5 * rtt_lambda, 1.0).unwrap();
    let ttr = Normal::new(0.5 * rtt_lambda, 1.0).unwrap();

    let mut polls_at = Vec::new();
    while real_time < until {
        let sent = real_time;
        real_time += TimeDelta::milliseconds(ttp.sample(&mut rand::rng()) as i64); // Poll in Transit to Platform
        let processed = real_time; // Poll processed by platform.
        real_time += TimeDelta::milliseconds(ttr.sample(&mut rand::rng()) as i64); // Poll Reply in Transit
        let replied = real_time;
        real_time += backoff; // Add the Polling Backoff - Wait this time before next poll.

        polls_at.push((sent, processed, replied));
    }

    // Sort events by realtime occurrence
    events.sort_by_key(|x| x.0);

    let mut results = Vec::new();
    let mut next_poll = 0;
    // Okay now we model observations of these records.
    // Each will have a timestamp - this timestamp will be 𝑝 = 𝑡 + 𝜎
    // therefore, the timestamp will lie within [p_min, p_max] = [t+sigma_min, t+sigma_max]
    // timestamp will be UTC - we don't need to care.
    for (t, event) in events {
        let timestamp = random_timestamp_within(t + min_deviation, t+max_deviation);

        // TODO: Ignoring precision limitations.
        // Observed uncertainty for event occurring at t in real time.
        let uncertainty_interval = (timestamp + min_deviation, timestamp + max_deviation);

        if t > polls_at[next_poll].1 { // If event occurred after next poll's processing time...
            next_poll += 1; // Then this event is visible at next next poll.
        } // Otherwise, this event is visible at next poll.

        results.push(MockObservation {
            uncertainty_interval,
            true_actions: vec![(event, t)],
            observed_definition: Some(event),
            visible_at: polls_at[next_poll].2 // Only visible after poll reply.
        })
    }

    return results;
}