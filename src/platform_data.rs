pub mod platform_data {
    #[derive(Debug, Clone)]
    pub struct PlatformData {
        sdk_type: String,
        sdk_version: String,
        platform_version: String,
        device_model: String,
        platform: String,
        hostname: String,
    }
}
