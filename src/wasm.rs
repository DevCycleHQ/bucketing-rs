// WebAssembly bindings using wasm-bindgen
#![cfg(feature = "wasm")]

use crate::errors::DevCycleError;
use crate::events::EventQueueOptions;
use crate::user::{BucketedUserConfig, PopulatedUser, User};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Panic hook for better error messages in browser console
#[wasm_bindgen(start)]
pub fn init_wasm() {
    console_error_panic_hook::set_once();
}

/// WASM-compatible EventQueueOptions
#[wasm_bindgen]
#[derive(Clone, Serialize, Deserialize)]
pub struct WasmEventQueueOptions {
    flush_events_interval_ms: u64,
    disable_automatic_event_logging: bool,
    disable_custom_event_logging: bool,
    max_event_queue_size: i32,
    max_user_event_queue_size: i32,
    flush_events_batch_size: i32,
    flush_events_queue_size: i32,
    events_api_base_uri: String,
}

#[wasm_bindgen]
impl WasmEventQueueOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let default = EventQueueOptions::default();
        Self {
            flush_events_interval_ms: default.flush_events_interval.as_millis() as u64,
            disable_automatic_event_logging: default.disable_automatic_event_logging,
            disable_custom_event_logging: default.disable_custom_event_logging,
            max_event_queue_size: default.max_event_queue_size,
            max_user_event_queue_size: default.max_user_event_queue_size,
            flush_events_batch_size: default.flush_events_batch_size,
            flush_events_queue_size: default.flush_events_queue_size,
            events_api_base_uri: default.events_api_base_uri,
        }
    }

    #[wasm_bindgen(setter)]
    pub fn set_flush_events_interval_ms(&mut self, value: u64) {
        self.flush_events_interval_ms = value;
    }

    #[wasm_bindgen(getter)]
    pub fn flush_events_interval_ms(&self) -> u64 {
        self.flush_events_interval_ms
    }

    #[wasm_bindgen(setter)]
    pub fn set_disable_automatic_event_logging(&mut self, value: bool) {
        self.disable_automatic_event_logging = value;
    }

    #[wasm_bindgen(getter)]
    pub fn disable_automatic_event_logging(&self) -> bool {
        self.disable_automatic_event_logging
    }

    #[wasm_bindgen(setter)]
    pub fn set_events_api_base_uri(&mut self, value: String) {
        self.events_api_base_uri = value;
    }

    #[wasm_bindgen(getter)]
    pub fn events_api_base_uri(&self) -> String {
        self.events_api_base_uri.clone()
    }
}

impl From<WasmEventQueueOptions> for EventQueueOptions {
    fn from(wasm_opts: WasmEventQueueOptions) -> Self {
        EventQueueOptions {
            flush_events_interval: Duration::from_millis(wasm_opts.flush_events_interval_ms),
            disable_automatic_event_logging: wasm_opts.disable_automatic_event_logging,
            disable_custom_event_logging: wasm_opts.disable_custom_event_logging,
            max_event_queue_size: wasm_opts.max_event_queue_size,
            max_user_event_queue_size: wasm_opts.max_user_event_queue_size,
            flush_events_batch_size: wasm_opts.flush_events_batch_size,
            flush_events_queue_size: wasm_opts.flush_events_queue_size,
            events_api_base_uri: wasm_opts.events_api_base_uri,
        }
    }
}

/// Initialize event queue
#[wasm_bindgen]
pub async fn init_event_queue(
    sdk_key: String,
    options: Option<WasmEventQueueOptions>,
) -> Result<(), JsValue> {
    let event_options = options
        .map(|o| o.into())
        .unwrap_or_else(EventQueueOptions::default);

    unsafe {
        crate::init_event_queue(&sdk_key, event_options)
            .await
            .map_err(|e| JsValue::from_str(&format!("Error initializing event queue: {:?}", e)))
    }
}

/// Generate bucketed config from user (JSON input/output)
#[wasm_bindgen]
pub async fn generate_bucketed_config_from_user(
    sdk_key: String,
    user_json: JsValue,
    client_custom_data_json: Option<JsValue>,
) -> Result<JsValue, JsValue> {
    let user: User = serde_wasm_bindgen::from_value(user_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid user JSON: {:?}", e)))?;

    let client_custom_data: HashMap<String, serde_json::Value> =
        if let Some(data) = client_custom_data_json {
            serde_wasm_bindgen::from_value(data)
                .map_err(|e| JsValue::from_str(&format!("Invalid client custom data: {:?}", e)))?
        } else {
            HashMap::new()
        };

    unsafe {
        let config = crate::generate_bucketed_config_from_user(&sdk_key, user, client_custom_data)
            .await
            .map_err(|e| {
                JsValue::from_str(&format!("Error generating bucketed config: {:?}", e))
            })?;

        serde_wasm_bindgen::to_value(&config)
            .map_err(|e| JsValue::from_str(&format!("Error serializing config: {:?}", e)))
    }
}

/// Generate bucketed config from populated user (JSON input/output)
#[wasm_bindgen]
pub async fn generate_bucketed_config(
    sdk_key: String,
    populated_user_json: JsValue,
    client_custom_data_json: Option<JsValue>,
) -> Result<JsValue, JsValue> {
    let populated_user: PopulatedUser = serde_wasm_bindgen::from_value(populated_user_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid populated user JSON: {:?}", e)))?;

    let client_custom_data: HashMap<String, serde_json::Value> =
        if let Some(data) = client_custom_data_json {
            serde_wasm_bindgen::from_value(data)
                .map_err(|e| JsValue::from_str(&format!("Invalid client custom data: {:?}", e)))?
        } else {
            HashMap::new()
        };

    unsafe {
        let config = crate::generate_bucketed_config(&sdk_key, populated_user, client_custom_data)
            .await
            .map_err(|e| {
                JsValue::from_str(&format!("Error generating bucketed config: {:?}", e))
            })?;

        serde_wasm_bindgen::to_value(&config)
            .map_err(|e| JsValue::from_str(&format!("Error serializing config: {:?}", e)))
    }
}

/// Get library version
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
