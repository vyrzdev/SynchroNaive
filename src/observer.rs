use std::env;
use std::str::FromStr;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use squareup::api::{CatalogApi, InventoryApi};
use squareup::config::{BaseUri, Configuration, Environment};
use squareup::http::client::HttpClientConfiguration;
use squareup::models::{RetrieveInventoryCountParams};
use squareup::models::enums::InventoryState::InStock;
use squareup::models::errors::SquareApiError;
use squareup::SquareClient;
use crate::value::{Target, Value};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SquareObserverConfig {
    pub(crate) token: String,
    pub(crate) target: String,
    pub(crate) calibration_target: String,
    pub(crate) location_id: String
}

pub struct SquareObserver {
    pub(crate) name: String,
    pub(crate) target: Target,
    pub(crate) catalog_api: CatalogApi,
    pub(crate) inventory_api: InventoryApi,
}

impl SquareObserver {
    pub fn new(name: String, config: SquareObserverConfig) -> SquareObserver {
        // Set Auth Token (in config)
        env::set_var("SQUARE_API_TOKEN", config.token);

        // Initialise Catalog API
        let catalog_api = CatalogApi::new(SquareClient::try_new( Configuration {
            environment: Environment::Sandbox, // Testing in Sandbox Environment
            http_client_config: HttpClientConfiguration::default(),
            base_uri: BaseUri::default(),
        }).unwrap());

        // Initialise Inventory API
        let inventory_api = InventoryApi::new(SquareClient::try_new( Configuration {
            environment: Environment::Sandbox, // Testing in Sandbox Environment
            http_client_config: HttpClientConfiguration::default(),
            base_uri: BaseUri::default(),
        }).unwrap());

        return SquareObserver { name, catalog_api, inventory_api, target: (config.location_id, config.target)}
    }

    pub async fn request(&self, target: Target) -> Result<(Value, chrono::DateTime<Utc>, chrono::DateTime<Utc>), SquareApiError> {
        let sent = chrono::Utc::now();
        let response = self.inventory_api.retrieve_inventory_count(target.1, RetrieveInventoryCountParams {
            location_ids: Some(vec![target.0]),
            cursor: None,
        }).await?;
        let replied = chrono::Utc::now();

        let value: i64 = response.counts.expect("Target does not exist on platform!")
            .iter()
            .filter(|c| c.state == InStock)
            .map(|c| Value::from_str(&c.quantity).expect("Failed to parse value from count!"))
            .sum();

        return Ok((value, sent, replied));
    }

    // pub async fn deviation_worker(&mut self) {
    //     // let mut calibration_records = HashMap::new();
    //
    //     println!("Calibrating Deviation Bounds for {} - 100 requests.",self.uuid);
    //
    //     for _ in 0..100 {
    //         let request_key = Uuid::new_v4().to_string();
    //         let sent: ChronoDateTime<Utc> = Utc::now();
    //         let resp =  self.api.batch_change_inventory(&BatchChangeInventoryRequest {
    //             idempotency_key: request_key.clone(),
    //             changes: Some(vec![
    //                 InventoryChange {
    //                     r#type: Some(InventoryChangeType::Adjustment),
    //                     physical_count: None,
    //                     adjustment: Some(InventoryAdjustment {
    //                         id: None,
    //                         reference_id: Some(request_key.clone()),
    //                         from_state: Some(InventoryState::None),
    //                         to_state: Some(InventoryState::InStock),
    //                         location_id: Some(self.config.location_id.clone()),
    //                         catalog_object_id: Some(self.config.calibration_target.clone()),
    //                         catalog_object_type: None,
    //                         quantity: Some("1".to_string()),
    //                         total_price_money: None,
    //                         occurred_at: Some(Default::default()),
    //                         created_at: None,
    //                         source: None,
    //                         employee_id: None,
    //                         team_member_id: None,
    //                         transaction_id: None,
    //                         refund_id: None,
    //                         purchase_order_id: None,
    //                         goods_receipt_id: None,
    //                         adjustment_group: None,
    //                     }),
    //                     transfer: None,
    //                     measurement_unit: None,
    //                     measurement_unit_id: None,
    //                 }
    //             ]),
    //             ignore_unchanged_counts: None,
    //         }).await;
    //         let replied: ChronoDateTime<Utc> = Utc::now();
    //
    //         if let Ok(resp) = resp {
    //             let change = resp.changes.as_ref().expect(format!("Failed to do Calibration: {}", self.uuid.clone()).as_str()).iter().nth(0).unwrap();
    //             let created_at = change.adjustment.as_ref().unwrap().created_at.as_ref().unwrap();
    //             let timestamp_dev: ChronoDateTime<Utc> = ChronoDateTime::from(created_at.clone());
    //             let timestamp_min = timestamp_dev;
    //             let timestamp_max = timestamp_dev + TimeDelta::milliseconds(1);
    //
    //             let deviation_max = timestamp_max - sent;
    //             let deviation_min = timestamp_min - replied;
    //             self.deviation_max = max(deviation_max, self.deviation_max);
    //             self.deviation_min = min(deviation_min, self.deviation_min);
    //
    //
    //         } else {
    //             eprintln!("Failed to do Calibration!");
    //             println!("{:?}", resp);
    //             return;
    //         }
    //         sleep(SysDuration::from_millis(500)).await;
    //     }
    //     println!("{} Calibrated: {}, {}", self.uuid, self.deviation_max, self.deviation_min);
    // }
}

