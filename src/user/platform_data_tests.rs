#[cfg(test)]
mod tests {
    use crate::platform_data::{get_platform_data, set_platform_data, PlatformData};
    use std::sync::Arc;

    #[test]
    fn test_generate_creates_valid_platform_data() {
        let platform_data = PlatformData::generate();

        assert_eq!(platform_data.sdk_type, "server");
        assert_eq!(platform_data.sdk_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(platform_data.platform_version, std::env::consts::OS);
        assert_eq!(platform_data.device_model, "unknown");
        assert_eq!(platform_data.platform, "Rust");
        assert!(!platform_data.hostname.is_empty());
    }

    #[test]
    fn test_get_platform_data_returns_error_when_not_set() {
        let sdk_key = "non-existent-sdk-key-12345";
        let result = get_platform_data(sdk_key);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Platform data not set for SDK key"));
    }

    #[test]
    fn test_get_platform_data_per_sdk_key() {
        let sdk_key1 = "test-sdk-key-1";
        let sdk_key2 = "test-sdk-key-2";

        let platform_data1 = PlatformData {
            sdk_type: "mobile".to_string(),
            sdk_version: "1.0.0".to_string(),
            platform_version: "iOS 15.0".to_string(),
            device_model: "iPhone13,2".to_string(),
            platform: "iOS".to_string(),
            hostname: "test-device-1".to_string(),
        };

        let platform_data2 = PlatformData {
            sdk_type: "server".to_string(),
            sdk_version: "2.0.0".to_string(),
            platform_version: "Linux".to_string(),
            device_model: "unknown".to_string(),
            platform: "Rust".to_string(),
            hostname: "test-device-2".to_string(),
        };

        set_platform_data(sdk_key1.to_string(), platform_data1);
        set_platform_data(sdk_key2.to_string(), platform_data2);

        let retrieved1 = get_platform_data(sdk_key1).unwrap();
        let retrieved2 = get_platform_data(sdk_key2).unwrap();

        assert_eq!(retrieved1.sdk_type, "mobile");
        assert_eq!(retrieved1.hostname, "test-device-1");
        assert_eq!(retrieved2.sdk_type, "server");
        assert_eq!(retrieved2.hostname, "test-device-2");
    }

    #[test]
    fn test_get_platform_data_returns_same_arc_for_same_sdk_key() {
        let sdk_key = "test-consistent-arc";

        let platform_data = PlatformData::generate();
        set_platform_data(sdk_key.to_string(), platform_data);

        let data1 = get_platform_data(sdk_key).unwrap();
        let data2 = get_platform_data(sdk_key).unwrap();

        assert_eq!(Arc::as_ptr(&data1), Arc::as_ptr(&data2));
    }

    #[test]
    fn test_platform_data_serialization() {
        let original = PlatformData {
            sdk_type: "mobile".to_string(),
            sdk_version: "3.5.1".to_string(),
            platform_version: "17.0".to_string(),
            device_model: "iPhone13,2".to_string(),
            platform: "iOS".to_string(),
            hostname: "test-device".to_string(),
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: PlatformData = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.sdk_type, original.sdk_type);
        assert_eq!(deserialized.sdk_version, original.sdk_version);
        assert_eq!(deserialized.platform, original.platform);
        assert_eq!(deserialized.hostname, original.hostname);
    }

    #[test]
    fn test_platform_data_concurrent_access() {
        use std::thread;

        let sdk_key = "test-sdk-concurrent";

        let platform_data = PlatformData {
            sdk_type: "server".to_string(),
            sdk_version: "1.0.0".to_string(),
            platform_version: "test".to_string(),
            device_model: "test-device".to_string(),
            platform: "test".to_string(),
            hostname: "concurrent-test-host".to_string(),
        };

        set_platform_data(sdk_key.to_string(), platform_data);

        let handles: Vec<_> = (0..10)
            .map(|_| {
                let key = sdk_key.to_string();
                thread::spawn(move || {
                    let data = get_platform_data(&key).unwrap();
                    assert!(!data.sdk_type.is_empty());
                    assert_eq!(data.hostname, "concurrent-test-host");
                    Arc::as_ptr(&data) as usize
                })
            })
            .collect();

        let addresses: Vec<usize> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        let first_address = addresses[0];
        for address in &addresses {
            assert_eq!(
                *address, first_address,
                "All threads should see the same Arc reference"
            );
        }
    }
}
