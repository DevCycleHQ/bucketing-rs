use serde::{Deserialize, Serialize};

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
    fn default() -> Self {
        PlatformData {
            sdk_type: "unknown".to_string(),
            sdk_version: "0.0.0".to_string(),
            platform_version: "0.0.0".to_string(),
            device_model: "unknown".to_string(),
            platform: "unknown".to_string(),
            hostname: "unknown".to_string(),
        }
    }
}
