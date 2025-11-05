// WebAssembly bindings using wasm-bindgen
#![cfg(feature = "wasm")]

use crate::bucketing::VariableForUserResult;
use crate::config::platform_data::PlatformData;
use crate::events::EventQueueOptions;
use crate::user::{PopulatedUser, User};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use wasm_bindgen::prelude::*;

/// WASM-compatible EvalReason matching AssemblyScript SDKVariable
#[derive(Clone, Serialize, Deserialize)]
pub struct WasmEvalReason {
    pub reason: String,
    pub details: String,
    #[serde(rename = "target_id")]
    pub target_id: String,
}

/// WASM-compatible SDKVariable matching AssemblyScript SDKVariable
#[derive(Clone, Serialize, Deserialize)]
pub struct WasmSDKVariable {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "type")]
    pub variable_type: String,
    pub key: String,
    pub value: serde_json::Value,
    #[serde(rename = "_feature", skip_serializing_if = "Option::is_none")]
    pub feature: Option<String>,
    #[serde(rename = "eval")]
    pub eval_reason: WasmEvalReason,
}

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

/// Set platform data for SDK key from JSON string
#[wasm_bindgen]
pub fn set_platform_data(sdk_key: String, platform_data_json: String) -> Result<(), JsValue> {
    let platform_data: PlatformData = serde_json::from_str(&platform_data_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid platform data JSON: {:?}", e)))?;

    crate::config::platform_data::set_platform_data(sdk_key, platform_data);
    Ok(())
}

/// Set config data for SDK key from JSON string
#[wasm_bindgen]
pub fn set_config_data(sdk_key: String, config_json: String) -> Result<(), JsValue> {
    let full_config: crate::config::FullConfig = serde_json::from_str(&config_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid config JSON: {:?}", e)))?;

    let config_body = match crate::config::ConfigBody::from_full_config(full_config) {
        Ok(cfg) => cfg,
        Err(e) => {
            return Err(JsValue::from_str(&format!(
                "Error creating config body: {}",
                e
            )))
        }
    };
    crate::configmanager::set_config(&sdk_key, config_body);
    Ok(())
}

/// Check if config data exists for SDK key
#[wasm_bindgen]
pub fn has_config_data(sdk_key: String) -> bool {
    crate::configmanager::has_config(&sdk_key)
}

/// Set client custom data for SDK key from JSON string
#[wasm_bindgen]
pub fn set_client_custom_data(
    sdk_key: String,
    client_custom_data_json: String,
) -> Result<(), JsValue> {
    let client_custom_data: HashMap<String, serde_json::Value> =
        serde_json::from_str(&client_custom_data_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid client custom data JSON: {:?}", e)))?;

    crate::config::client_custom_data::set_client_custom_data(sdk_key, client_custom_data);
    Ok(())
}

/// Get variable value for user (JSON input/output), returns stringified SDKVariable or null
#[wasm_bindgen]
pub async fn variable_for_user(
    sdk_key: String,
    user_json_str: String,
    variable_key: String,
    variable_type: String,
) -> Result<JsValue, JsValue> {
    // Parse user JSON
    let user: User = serde_json::from_str(&user_json_str)
        .map_err(|e| JsValue::from_str(&format!("Invalid user JSON: {:?}", e)))?;

    // Convert to PopulatedUser
    let populated_user = user.get_populated_user(&sdk_key);

    // Get client_custom_data from global storage (or use empty if not set)
    let client_custom_data = crate::config::client_custom_data::CLIENT_CUSTOM_DATA
        .read()
        .unwrap()
        .get(&sdk_key)
        .cloned()
        .unwrap_or_else(HashMap::new);

    unsafe {
        let result = crate::bucketing::variable_for_user(
            &sdk_key,
            populated_user,
            &variable_key,
            &variable_type,
            client_custom_data,
        )
        .await;

        match result {
            Ok(VariableForUserResult {
                variable_id,
                variable_key: var_key,
                variable_type: var_type,
                variable_value: value,
                feature_id,
                variation_id,
                eval_reason,
                default_reason: _default_reason,
            }) => {
                // Check if variable type matches expected type (matching AS behavior)
                if !variable_type.is_empty() && var_type != variable_type {
                    return Err(JsValue::from_str(&format!(
                        "Type mismatch: expected '{}', got '{}'",
                        variable_type, var_type
                    )));
                }

                // Construct SDKVariable matching AssemblyScript structure
                let sdk_variable = WasmSDKVariable {
                    id: variable_id,
                    variable_type: var_type,
                    key: var_key,
                    value,
                    feature: Some(feature_id),
                    eval_reason: WasmEvalReason {
                        reason: format!("{:?}", eval_reason),
                        details: String::new(),
                        target_id: String::new(),
                    },
                };

                serde_wasm_bindgen::to_value(&sdk_variable).map_err(|e| {
                    JsValue::from_str(&format!("Error serializing SDKVariable: {:?}", e))
                })
            }
            Err(e) => {
                // Throw error
                Err(JsValue::from_str(&format!(
                    "Error getting variable: {:?}",
                    e
                )))
            }
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
) -> Result<JsValue, JsValue> {
    let user: User = serde_wasm_bindgen::from_value(user_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid user JSON: {:?}", e)))?;

    unsafe {
        let config = crate::generate_bucketed_config_from_user(&sdk_key, user)
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
    client_custom_data_json: JsValue,
) -> Result<JsValue, JsValue> {
    let populated_user: PopulatedUser = serde_wasm_bindgen::from_value(populated_user_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid populated user JSON: {:?}", e)))?;

    let client_custom_data: HashMap<String, serde_json::Value> =
        if client_custom_data_json.is_undefined() || client_custom_data_json.is_null() {
            HashMap::new()
        } else {
            serde_wasm_bindgen::from_value(client_custom_data_json)
                .map_err(|e| JsValue::from_str(&format!("Invalid client custom data: {:?}", e)))?
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
