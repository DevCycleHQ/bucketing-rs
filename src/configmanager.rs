use crate::config::ConfigBody;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;

pub(crate) static CONFIGS: Lazy<RwLock<HashMap<String, ConfigBody<'static>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub(crate) fn get_config(sdk_key: &str) -> Option<&ConfigBody<'static>> {
    let configs = CONFIGS.read().unwrap();
    if let Some(config) = configs.get(sdk_key) {
        return Some(config.clone());
    }
    None
}

pub(crate) fn set_config(sdk_key: &str, config: ConfigBody<'static>) {
    let mut configs = CONFIGS.write().unwrap();
    configs.insert(sdk_key.to_string(), config);
}

pub(crate) fn has_config(sdk_key: &str) -> bool {
    let configs = CONFIGS.read().unwrap();
    configs.contains_key(sdk_key)
}