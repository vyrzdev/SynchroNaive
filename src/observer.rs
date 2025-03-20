use std::cmp::{max, min, PartialEq};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::ops::{Add, Sub};
use std::str::FromStr;
use squareup::models::DateTime;
use chrono::{DateTime as ChronoDateTime, TimeDelta, Utc};
use std::time::Duration as SysDuration;
use tokio::sync::mpsc;
use hifitime::{Duration, Epoch};
use serde::{Deserialize, Serialize};
use squareup::api::{CatalogApi, InventoryApi};
use squareup::config::{BaseUri, Configuration, Environment};
use squareup::http::client::HttpClientConfiguration;
use squareup::models::{BatchChangeInventoryRequest, InventoryAdjustment, InventoryChange, RetrieveCatalogObjectParameters, RetrieveInventoryCountParams};
use squareup::models::enums::{InventoryChangeType, InventoryState};
use squareup::SquareClient;
use tokio::time::sleep;
use uuid::Uuid;
use crate::interval::{Interval, Moment};
use crate::observations::{DefinitionPredicate, Observation};
use crate::value::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ObserverConfig {
    pub(crate) token: String,
    pub(crate) target: String,
    pub(crate) calibration_target: String,
    pub(crate) location_id: String
}

pub struct Observer {
    pub(crate) uuid: String,
    pub(crate) config: ObserverConfig,
    pub(crate) api: InventoryApi,
    pub(crate) deviation_max: TimeDelta,
    pub(crate) deviation_min: TimeDelta,
}

impl Observer {
    pub fn new(uuid: String, config: ObserverConfig) -> Result<Observer, Box<dyn Error>> {
        env::set_var("SQUARE_API_TOKEN", &config.token);
        let api = InventoryApi::new(SquareClient::try_new(Configuration {
            environment: Environment::Sandbox, // OPTIONAL if you set the SQUARE_ENVIRONMENT env var
            http_client_config: HttpClientConfiguration::default(),
            base_uri: BaseUri::default(),
        })?);
        Ok(Observer { uuid, config, api, deviation_min: TimeDelta::new(0, 0).expect("TIMESTAMP!"), deviation_max: TimeDelta::new(0, 0).expect("TIMESTAMP!")})
    }

    pub async fn request(&self, target: &String) -> Result<(Value, Interval), Box<dyn Error>> {
        let sent = Moment(chrono::Utc::now()); // Assumes UTC system-time. (NOTE)
        let response = self.api.retrieve_inventory_count(target, RetrieveInventoryCountParams {
            location_ids: None,
            cursor: None,
        }).await?;
        let replied = Moment(chrono::Utc::now());
        Ok((response.counts.expect(format!("Target doesn't exist on {}", self.uuid).as_str()).iter().filter(|c| c.state == InventoryState::InStock).map(|c| i64::from_str(&c.quantity).expect("Invalid quantity string.")).sum(), Interval(sent, replied)))
    }

    pub async fn deviation_worker(&mut self) {
        // let mut calibration_records = HashMap::new();

        println!("Calibrating Deviation Bounds for {} - 100 requests.",self.uuid);

        for _ in 0..100 {
            let request_key = Uuid::new_v4().to_string();
            let sent: ChronoDateTime<Utc> = Utc::now();
            let resp =  self.api.batch_change_inventory(&BatchChangeInventoryRequest {
                idempotency_key: request_key.clone(),
                changes: Some(vec![
                    InventoryChange {
                        r#type: Some(InventoryChangeType::Adjustment),
                        physical_count: None,
                        adjustment: Some(InventoryAdjustment {
                            id: None,
                            reference_id: Some(request_key.clone()),
                            from_state: Some(InventoryState::None),
                            to_state: Some(InventoryState::InStock),
                            location_id: Some(self.config.location_id.clone()),
                            catalog_object_id: Some(self.config.calibration_target.clone()),
                            catalog_object_type: None,
                            quantity: Some("1".to_string()),
                            total_price_money: None,
                            occurred_at: Some(Default::default()),
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
            }).await;
            let replied: ChronoDateTime<Utc> = Utc::now();

            if let Ok(resp) = resp {
                let change = resp.changes.as_ref().expect(format!("Failed to do Calibration: {}", self.uuid.clone()).as_str()).iter().nth(0).unwrap();
                let created_at = change.adjustment.as_ref().unwrap().created_at.as_ref().unwrap();
                let timestamp_dev: ChronoDateTime<Utc> = ChronoDateTime::from(created_at.clone());
                let timestamp_min = timestamp_dev;
                let timestamp_max = timestamp_dev + TimeDelta::milliseconds(1);

                let deviation_max = timestamp_max - sent;
                let deviation_min = timestamp_min - replied;
                self.deviation_max = max(deviation_max, self.deviation_max);
                self.deviation_min = min(deviation_min, self.deviation_min);


            } else {
                eprintln!("Failed to do Calibration!");
                println!("{:?}", resp);
                return;
            }
            sleep(SysDuration::from_millis(500)).await;
        }
        println!("{} Calibrated: {}, {}", self.uuid, self.deviation_max, self.deviation_min);
    }
}

