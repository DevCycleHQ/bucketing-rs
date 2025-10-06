#[cfg(test)]
mod tests {
    use crate::platform_data::{get_platform_data, PlatformData};

    #[test]
    fn test_generate_creates_valid_platform_data() {
        let platform_data = PlatformData::generate();

        // Verify all required fields are populated with correct defaults
        assert_eq!(platform_data.sdk_type, "server");
        assert_eq!(platform_data.sdk_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(platform_data.platform_version, std::env::consts::OS);
        assert_eq!(platform_data.device_model, "unknown");
        assert_eq!(platform_data.platform, "Rust");
        assert!(!platform_data.hostname.is_empty());
    }

    #[test]
    fn test_get_platform_data_returns_consistent_reference() {
        // Verify OnceLock returns the same static reference every time
        let data1 = get_platform_data();
        let data2 = get_platform_data();

        // Should return the exact same memory address
        assert_eq!(data1 as *const _, data2 as *const _);
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

        // Test round-trip: serialize and deserialize
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: PlatformData = serde_json::from_str(&json).unwrap();

        // Verify data integrity after round-trip
        assert_eq!(deserialized.sdk_type, original.sdk_type);
        assert_eq!(deserialized.sdk_version, original.sdk_version);
        assert_eq!(deserialized.platform, original.platform);
        assert_eq!(deserialized.hostname, original.hostname);
    }

    #[test]
    fn test_platform_data_concurrent_access() {
        use std::thread;

        // Spawn multiple threads that all try to access platform_data simultaneously
        let handles: Vec<_> = (0..10)
            .map(|_| {
                thread::spawn(|| {
                    // Each thread accesses platform data
                    let data = get_platform_data();

                    // Verify the data is valid (not checking specific values since
                    // #[ctor] in bucketing_tests may have already set it)
                    assert!(!data.sdk_type.is_empty());
                    assert!(!data.hostname.is_empty());

                    // Return the pointer address to verify all threads see the same reference
                    data as *const PlatformData as usize
                })
            })
            .collect();

        // Collect all thread results
        let addresses: Vec<usize> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        // All threads should have received the exact same memory address
        let first_address = addresses[0];
        for address in &addresses {
            assert_eq!(
                *address, first_address,
                "All threads should see the same static reference"
            );
        }
    }
}
