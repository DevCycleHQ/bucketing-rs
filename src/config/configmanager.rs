use crate::config::ConfigBody;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub(crate) static CONFIGS: Lazy<RwLock<HashMap<String, Arc<ConfigBody>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub(crate) fn get_config(sdk_key: &str) -> Option<Arc<ConfigBody>> {
    let configs = CONFIGS.read().unwrap();
    configs.get(sdk_key).cloned() // Clones the Arc, not the ConfigBody
}

pub(crate) fn set_config(sdk_key: &str, config: ConfigBody) {
    let mut configs = CONFIGS.write().unwrap();
    configs.insert(sdk_key.to_string(), Arc::new(config));
}

pub(crate) fn has_config(sdk_key: &str) -> bool {
    let configs = CONFIGS.read().unwrap();
    configs.contains_key(sdk_key)
}
