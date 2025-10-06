use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

static PLATFORM_DATA: OnceLock<PlatformData> = OnceLock::new();

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

/// Sets platform data globally for this process.
///
/// This can only be called once successfully. Subsequent calls will return an error
/// with the platform data you tried to set.
///
/// # Errors
///
/// Returns `Err(platform_data)` if platform data was already set.
pub fn set_static_platform_data(platform_data: PlatformData) -> Result<(), PlatformData> {
    PLATFORM_DATA.set(platform_data)
}

/// Gets a reference to the global platform data.
///
/// If platform data was set via `set_static_platform_data()`, returns that.
/// Otherwise, automatically generates and caches default platform data on first call.
///
/// Returns a reference to avoid unnecessary cloning on each access.
pub fn get_platform_data() -> &'static PlatformData {
    PLATFORM_DATA.get_or_init(|| PlatformData::generate())
}
