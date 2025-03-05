use std::env;
use std::error::Error;
use tokio::sync::mpsc;
use hifitime::Epoch;
use serde::{Deserialize, Serialize};
use squareup::api::CatalogApi;
use squareup::config::{BaseUri, Configuration, Environment};
use squareup::http::client::HttpClientConfiguration;
use squareup::models::RetrieveCatalogObjectParameters;
use squareup::SquareClient;
use crate::interval::{Interval, Moment};
use crate::observations::Observation;
use crate::value::Value;
// We implement a Foo observer - polling-based over a single variable on a Square Store.

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ObserverConfig {
    pub(crate) token: String,
    pub(crate) target: String,
}

pub struct Observer {
    uuid: String,
    pub(crate) config: ObserverConfig,
    api: CatalogApi
}

impl Observer {
    pub fn new(uuid: String, config: ObserverConfig) -> Result<Observer, Box<dyn Error>> {
        env::set_var("SQUARE_API_TOKEN", &config.token);
        let api = CatalogApi::new(SquareClient::try_new(Configuration {
            environment: Environment::Sandbox, // OPTIONAL if you set the SQUARE_ENVIRONMENT env var
            http_client_config: HttpClientConfiguration::default(),
            base_uri: BaseUri::default(),
        })?);
        Ok(Observer { uuid, config, api, })
    }

    pub async fn request(&self, target: &String) -> Result<(Value, Interval), Box<dyn Error>> {
        let sent = Moment(Epoch::now().expect("Failed to take timestamp!").duration); // Assumes UTC system-time. (NOTE)
        let response = self.api.retrieve_catalog_object(target, &RetrieveCatalogObjectParameters {
            include_related_objects: None,
            catalog_version: None,
            include_category_path_to_root: None,
        }).await?;
        let replied = Moment(Epoch::now().expect("Failed to take timestamp!").duration);
        Ok((response.object.expect(format!("Target doesn't exist on {}", self.uuid).as_str()).item_data.expect("Expected item data!").name.expect("All items have names!"), Interval(sent, replied)))
    }
    pub async fn poll_worker(&self, (mut last_state, mut last_at): (Value, Moment), sync: mpsc::Sender<Observation>) {
        loop {
            let (state, at) = self.request(&self.config.target).await.unwrap();

            if state != last_state {
                sync.send(Observation {
                    at: Interval(last_at, at.1),
                    s0: last_state,
                    s1: state.clone(),
                    source: self.uuid.clone(),
                }).await.unwrap();
                last_state = state;
                last_at = at.0;
            } else {
                last_at = at.0;
            }
        }
    }
}

