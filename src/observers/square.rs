use std::env;
use std::str::FromStr;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use squareup::api::{CatalogApi, InventoryApi};
use squareup::config::{BaseUri, Configuration, Environment};
use squareup::http::client::HttpClientConfiguration;
use squareup::models::enums::InventoryState::InStock;
use squareup::models::errors::SquareApiError;
use squareup::models::RetrieveInventoryCountParams;
use squareup::SquareClient;
use crate::value::{Target, Value};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SquareObserverConfig {
    pub(crate) token: String,
    pub(crate) target: String,
    pub(crate) calibration_target: String,
    pub(crate) location_id: String,
    pub(crate) testing_config: SquareTestingConfig
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SquareTestingConfig {
    pub(crate) sale_lambda: f64,
    pub(crate) edit_lambda: f64
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
        let catalog_api = CatalogApi::new(SquareClient::try_new(Configuration {
            environment: Environment::Sandbox, // Testing in Sandbox Environment
            http_client_config: HttpClientConfiguration::default(),
            base_uri: BaseUri::default(),
        }).unwrap());

        // Initialise Inventory API
        let inventory_api = InventoryApi::new(SquareClient::try_new(Configuration {
            environment: Environment::Sandbox, // Testing in Sandbox Environment
            http_client_config: HttpClientConfiguration::default(),
            base_uri: BaseUri::default(),
        }).unwrap());

        return SquareObserver { name, catalog_api, inventory_api, target: (config.location_id, config.target) }
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
}