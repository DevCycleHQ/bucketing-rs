use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// Global platform data storage per SDK key
pub(crate) static PLATFORM_DATA: Lazy<RwLock<HashMap<String, Arc<PlatformData>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformData {
    pub sdk_type: String,
    pub sdk_version: String,
    pub platform_version: String,
    pub device_model: String,
    pub platform: String,
    pub hostname: String,
}

impl PlatformData {
    pub fn generate() -> Self {
        let hostname = hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "unknown".to_string());

        PlatformData {
            sdk_type: "server".to_string(),
            sdk_version: env!("CARGO_PKG_VERSION").to_string(),
            platform_version: std::env::consts::OS.to_string(),
            device_model: "unknown".to_string(),
            platform: "Rust".to_string(),
            hostname,
        }
    }
}

pub fn get_platform_data(sdk_key: &str) -> Result<Arc<PlatformData>, String> {
    let data = PLATFORM_DATA.read().unwrap();
    data.get(sdk_key).cloned().ok_or_else(|| {
        format!(
            "Platform data not set for SDK key: {}. Call set_platform_data() first.",
            sdk_key
        )
    })
}

pub fn set_platform_data(sdk_key: String, platform_data: PlatformData) {
    let mut data = PLATFORM_DATA.write().unwrap();
    data.insert(sdk_key, Arc::new(platform_data));
}
