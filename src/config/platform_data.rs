use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

cfg_if::cfg_if! {
    if #[cfg(target_family = "wasm")] {
        fn resolve_hostname() -> String {
            std::env::var("HOSTNAME")
                .unwrap_or_else(|_| "unknown".to_string())
        }
    } else {
        fn resolve_hostname() -> String {
            hostname::get()
                .ok()
                .and_then(|h| h.into_string().ok())
                .or_else(|| std::env::var("HOSTNAME").ok())
                .unwrap_or_else(|| "unknown".to_string())
        }
    }
}

// Global platform data storage per SDK key
pub(crate) static PLATFORM_DATA: Lazy<RwLock<HashMap<String, Arc<PlatformData>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
        PlatformData {
            sdk_type: "server".to_string(),
            sdk_version: env!("CARGO_PKG_VERSION").to_string(),
            platform_version: std::env::consts::OS.to_string(),
            device_model: "unknown".to_string(),
            platform: "Rust".to_string(),
            hostname: resolve_hostname(),
        }
    }
}

pub fn get_platform_data(sdk_key: &str) -> Result<Arc<PlatformData>, String> {
    let data = PLATFORM_DATA
        .read()
        .expect("Failed to acquire read lock on PLATFORM_DATA: lock poisoned");
    data.get(sdk_key).cloned().ok_or_else(|| {
        format!(
            "Platform data not set for SDK key: {}. Call set_platform_data() first.",
            sdk_key
        )
    })
}

pub fn set_platform_data(sdk_key: String, platform_data: PlatformData) {
    let mut data = PLATFORM_DATA
        .write()
        .expect("Failed to acquire write lock on PLATFORM_DATA (lock may be poisoned)");
    data.insert(sdk_key, Arc::new(platform_data));
}
