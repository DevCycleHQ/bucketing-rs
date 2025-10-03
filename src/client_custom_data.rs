use std::collections::HashMap;
use std::sync::RwLock;
use once_cell::sync::Lazy;
use serde_json::Value;
use crate::config::ConfigBody;

// Global client custom data storage
pub(crate) static CLIENT_CUSTOM_DATA: Lazy<RwLock<HashMap<String, HashMap<String, Value>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));
pub fn get_client_custom_data(sdk_key: &str) -> Option<HashMap<String, Value>> {
    let data = CLIENT_CUSTOM_DATA.read().unwrap();
    data.get(sdk_key).cloned()
}

pub fn set_client_custom_data(sdk_key: String, data: HashMap<String, Value>) {
    let mut client_data = CLIENT_CUSTOM_DATA.write().unwrap();
    client_data.insert(sdk_key, data);
}
