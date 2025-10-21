use once_cell::sync::Lazy;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::RwLock;

// Global client custom data storage
pub(crate) static CLIENT_CUSTOM_DATA: Lazy<RwLock<HashMap<String, HashMap<String, Value>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));
pub(crate) fn get_client_custom_data(sdk_key: String) -> HashMap<String, Value> {
    let data = CLIENT_CUSTOM_DATA.read().unwrap();
    let values = data.get(&sdk_key).cloned();
    return values.unwrap();
}

pub(crate) fn set_client_custom_data(
    sdk_key: String,
    data: HashMap<String, Value>,
) -> Option<HashMap<String, Value>> {
    let mut client_data = CLIENT_CUSTOM_DATA.write().unwrap();
    client_data.insert(sdk_key, data)
}
