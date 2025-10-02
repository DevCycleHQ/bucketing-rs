use std::collections::HashMap;
use std::sync::RwLock;
use serde_json::Value;

// Global client custom data storage
static CLIENT_CUSTOM_DATA: RwLock<HashMap<String, HashMap<String, Value>>> = RwLock::new(HashMap::new());

pub fn get_client_custom_data(sdk_key: &str) -> Option<HashMap<String, Value>> {
    let data = CLIENT_CUSTOM_DATA.read().unwrap();
    data.get(sdk_key).cloned()
}

pub fn set_client_custom_data(sdk_key: String, data: HashMap<String, Value>) {
    let mut client_data = CLIENT_CUSTOM_DATA.write().unwrap();
    client_data.insert(sdk_key, data);
}
