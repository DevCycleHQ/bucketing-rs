#[cfg(test)]
mod tests {
    use crate::bucketing;
    use crate::config::*;
    use crate::configmanager;
    use crate::platform_data::PlatformData;
    use crate::user::*;
    use chrono::Utc;
    use serde_json;
    use serde_json::Value;
    use std::collections::HashMap;

    fn load_test_config() -> FullConfig {
        let config_json = include_str!("../tests/resources/test_config.json");
        serde_json::from_str(config_json).expect("Failed to parse test config")
    }

    fn load_test_config_v2() -> FullConfig {
        let config_json = include_str!("../tests/resources/fixture_test_v2_config.json");
        serde_json::from_str(config_json).expect("Failed to parse test config v2")
    }

    fn create_config_body_from_full_config(full_config: FullConfig) -> ConfigBody<'static> {
        // Create static hashmaps for the lifetime requirements
        let static_audiences: &'static HashMap<String, crate::filters::NoIdAudience> = Box::leak(Box::new(HashMap::new()));

        let mut variable_id_map = HashMap::new();
        let mut variable_key_map = HashMap::new();
        let mut variable_id_to_feature_map = HashMap::new();

        // Populate the maps
        for variable in &full_config.variables {
            variable_id_map.insert(variable._id.clone(), variable.clone());
            variable_key_map.insert(variable.key.clone(), variable.clone());
        }

        // For each variable, find the feature that contains it
        for feature in &full_config.features {
            for variation in &feature.variations {
                for var_ref in &variation.variables {
                    variable_id_to_feature_map.insert(var_ref._var.clone(), feature.clone());
                }
            }
        }

        ConfigBody {
            project: full_config.project,
            audiences: static_audiences,
            environment: full_config.environment,
            features: full_config.features,
            variables: full_config.variables,
            sse: SSE {
                hostname: "localhost".to_string(),
                path: "/sse".to_string(),
            },
            variable_id_map,
            variable_key_map,
            variable_id_to_feature_map,
            etag: "test-etag".to_string(),
            ray_id: "test-ray-id".to_string(),
            last_modified: Utc::now(),
        }
    }

    fn create_test_user(user_id: &str) -> PopulatedUser {
        let platform_data = PlatformData {
            sdk_type: "server".to_string(),
            sdk_version: "1.0.0".to_string(),
            platform_version: "1.0.0".to_string(),
            device_model: "test-device".to_string(),
            platform: "linux".to_string(),
            hostname: "localhost".to_string(),
        };

        PopulatedUser {
            user_id: user_id.to_string(),
            email: format!("{}@test.com", user_id),
            name: format!("Test User {}", user_id),
            language: "en".to_string(),
            country: "US".to_string(),
            app_version: "1.0.0".to_string(),
            app_build: "100".to_string(),
            custom_data: HashMap::new(),
            private_custom_data: HashMap::new(),
            device_model: "test-device".to_string(),
            last_seen_date: Utc::now(),
            platform_data,
            created_date: Utc::now(),
        }
    }

    fn create_test_user_v2(user_id: &str) -> PopulatedUser {
        let platform_data = PlatformData {
            sdk_type: "server".to_string(),
            sdk_version: "2.0.0".to_string(),
            platform_version: "1.1.2".to_string(),
            device_model: "test-device-v2".to_string(),
            platform: "linux".to_string(),
            hostname: "localhost".to_string(),
        };
        let mut custom_data : HashMap<String, Value> = HashMap::new();
        custom_data.insert("favouriteNull".to_string(), Value::Null);

        PopulatedUser {
            user_id: user_id.to_string(),
            email: format!("{}@test.com", user_id),
            name: format!("Test User {}", user_id),
            language: "en".to_string(),
            country: "US".to_string(),
            app_version: "1.0.0".to_string(),
            app_build: "100".to_string(),
            custom_data,
            private_custom_data: HashMap::new(),
            device_model: "test-device".to_string(),
            last_seen_date: Utc::now(),
            platform_data,
            created_date: Utc::now(),
        }
    }


    fn setup_test_config(sdk_key: &str) {
        let full_config = load_test_config();
        let config_body = create_config_body_from_full_config(full_config);

        // Store the config in the global CONFIGS map
        let mut configs = configmanager::CONFIGS.write().unwrap();
        configs.insert(sdk_key.to_string(), config_body.into());
    }

    fn setup_test_config_v2(sdk_key: &str) {
        let full_config = load_test_config();
        let config_body = create_config_body_from_full_config(full_config);

        // Store the config in the global CONFIGS map
        let mut configs = configmanager::CONFIGS.write().unwrap();
        configs.insert(sdk_key.to_string(), config_body.into());
    }

    #[tokio::test]
    async fn test_generate_bucketed_config_basic_user() {
        let sdk_key = "test-sdk-key-1";
        setup_test_config(sdk_key);

        let user = create_test_user("user123");
        let client_custom_data = HashMap::new();

        let result = unsafe {
            bucketing::generate_bucketed_config(sdk_key, user.clone(), client_custom_data).await
        };

        assert!(result.is_ok(), "Failed to generate bucketed config: {:?}", result.err());

        let bucketed_config = result.unwrap();
        assert_eq!(bucketed_config.user.user_id, "user123");
        assert_eq!(bucketed_config.project, "6216420c2ea68943c8833c09");
        assert_eq!(bucketed_config.environment, "6216420c2ea68943c8833c0b");
    }

    #[tokio::test]
    async fn test_generate_bucketed_config_with_custom_data() {
        let sdk_key = "test-sdk-key-2";
        setup_test_config(sdk_key);

        let mut user = create_test_user("user456");
        user.custom_data.insert("age".to_string(), serde_json::Value::Number(serde_json::Number::from(25)));
        user.custom_data.insert("tier".to_string(), serde_json::Value::String("premium".to_string()));

        let mut client_custom_data = HashMap::new();
        client_custom_data.insert("session_id".to_string(), serde_json::Value::String("session123".to_string()));

        let result = unsafe {
            bucketing::generate_bucketed_config(sdk_key, user.clone(), client_custom_data).await
        };

        assert!(result.is_ok(), "Failed to generate bucketed config with custom data: {:?}", result.err());

        let bucketed_config = result.unwrap();
        assert_eq!(bucketed_config.user.user_id, "user456");
        assert_eq!(bucketed_config.user.custom_data.get("age").unwrap(), &serde_json::Value::Number(serde_json::Number::from(25)));
        assert_eq!(bucketed_config.user.custom_data.get("tier").unwrap(), &serde_json::Value::String("premium".to_string()));
    }

    #[tokio::test]
    async fn test_generate_bucketed_config_multiple_users() {
        let sdk_key = "test-sdk-key-3";
        setup_test_config(sdk_key);

        let user_ids = vec!["user1", "user2", "user3", "user4", "user5"];
        let mut results = Vec::new();

        for user_id in user_ids {
            let user = create_test_user(user_id);
            let client_custom_data = HashMap::new();

            let result = unsafe {
                bucketing::generate_bucketed_config(sdk_key, user.clone(), client_custom_data).await
            };

            assert!(result.is_ok(), "Failed to generate bucketed config for user {}: {:?}", user_id, result.err());
            results.push(result.unwrap());
        }

        // Verify all configs were generated successfully
        assert_eq!(results.len(), 5);
        for (i, config) in results.iter().enumerate() {
            assert_eq!(config.user.user_id, format!("user{}", i + 1));
            assert_eq!(config.project, "6216420c2ea68943c8833c09");
            assert_eq!(config.environment, "6216420c2ea68943c8833c0b");
        }
    }

    #[tokio::test]
    async fn test_generate_bucketed_config_different_countries() {
        let sdk_key = "test-sdk-key-4";
        setup_test_config(sdk_key);

        let countries = vec!["US", "CA", "GB", "DE", "FR"];

        for country in countries {
            let mut user = create_test_user(&format!("user_{}", country));
            user.country = country.to_string();

            let client_custom_data = HashMap::new();

            let result = unsafe {
                bucketing::generate_bucketed_config(sdk_key, user.clone(), client_custom_data).await
            };

            assert!(result.is_ok(), "Failed to generate bucketed config for country {}: {:?}", country, result.err());

            let bucketed_config = result.unwrap();
            assert_eq!(bucketed_config.user.country, country);
        }
    }

    #[tokio::test]
    async fn test_generate_bucketed_config_missing_sdk_key() {
        let user = create_test_user("user_missing");
        let client_custom_data = HashMap::new();

        let result = unsafe {
            bucketing::generate_bucketed_config("nonexistent-sdk-key", user.clone(), client_custom_data).await
        };

        assert!(result.is_err(), "Expected error for missing SDK key");
    }

    #[tokio::test]
    async fn test_generate_bucketed_config_with_private_custom_data() {
        let sdk_key = "test-sdk-key-5";
        setup_test_config(sdk_key);

        let mut user = create_test_user("user_private");
        user.private_custom_data.insert("internal_id".to_string(), serde_json::Value::String("internal123".to_string()));
        user.private_custom_data.insert("score".to_string(), serde_json::Value::Number(serde_json::Number::from(95)));

        let client_custom_data = HashMap::new();

        let result = unsafe {
            bucketing::generate_bucketed_config(sdk_key, user.clone(), client_custom_data).await
        };

        assert!(result.is_ok(), "Failed to generate bucketed config with private custom data: {:?}", result.err());

        let bucketed_config = result.unwrap();
        assert_eq!(bucketed_config.user.private_custom_data.get("internal_id").unwrap(), &serde_json::Value::String("internal123".to_string()));
        assert_eq!(bucketed_config.user.private_custom_data.get("score").unwrap(), &serde_json::Value::Number(serde_json::Number::from(95)));
    }

    #[tokio::test]
    async fn test_config_loading_and_structure() {
        let config = load_test_config();

        // Verify project structure
        assert_eq!(config.project._id, "6216420c2ea68943c8833c09");
        assert_eq!(config.project.key, "default");
        assert_eq!(config.project.a0_organization, "org_NszUFyWBFy7cr95J");

        // Verify environment structure
        assert_eq!(config.environment._id, "6216420c2ea68943c8833c0b");
        assert_eq!(config.environment.key, "development");

        // Verify features exist
        assert!(!config.features.is_empty(), "Config should have features");

        // Verify variables exist
        assert!(!config.variables.is_empty(), "Config should have variables");

        // Check specific feature structure
        let test_feature = config.features.iter().find(|f| f.key == "test");
        assert!(test_feature.is_some(), "Should have 'test' feature");

        let test_feature = test_feature.unwrap();
        assert_eq!(test_feature._type, "release");
        assert_eq!(test_feature.variations.len(), 2);

        // Check variations
        assert_eq!(test_feature.variations[0].key, "variation-on");
        assert_eq!(test_feature.variations[1].key, "variation-off");
    }

    #[test]
    fn test_user_creation_and_data_merging() {
        let mut user = create_test_user("test_merge");

        // Test combined custom data
        user.custom_data.insert("public_key".to_string(), serde_json::Value::String("public_value".to_string()));
        user.private_custom_data.insert("private_key".to_string(), serde_json::Value::String("private_value".to_string()));

        let combined = user.combined_custom_data();
        assert_eq!(combined.len(), 2);
        assert_eq!(combined.get("public_key").unwrap(), &serde_json::Value::String("public_value".to_string()));
        assert_eq!(combined.get("private_key").unwrap(), &serde_json::Value::String("private_value".to_string()));
    }

    // Production config tests using the real CDN config
    fn load_production_config() -> FullConfig {
        let config_json = include_str!("../tests/resources/production_config.json");
        serde_json::from_str(config_json).expect("Failed to parse production config")
    }

    fn setup_production_config(sdk_key: &str) {
        let full_config = load_production_config();
        let config_body = create_config_body_from_full_config(full_config);

        // Store the config in the global CONFIGS map
        let mut configs = configmanager::CONFIGS.write().unwrap();
        configs.insert(sdk_key.to_string(), config_body.into());
    }

    #[tokio::test]
    async fn test_production_config_basic_bucketing() {
        let sdk_key = "dvc_server_token_hash";
        setup_production_config(sdk_key);

        let user = create_test_user("prod_user_123");
        let client_custom_data = HashMap::new();

        let result = unsafe {
            bucketing::generate_bucketed_config(sdk_key, user.clone(), client_custom_data).await
        };

        assert!(result.is_ok(), "Failed to generate bucketed config from production config: {:?}", result.err());

        let bucketed_config = result.unwrap();
        assert_eq!(bucketed_config.user.user_id, "prod_user_123");
        assert_eq!(bucketed_config.project, "6216420c2ea68943c8833c09");
        assert_eq!(bucketed_config.environment, "6216420c2ea68943c8833c0b");
    }

    #[tokio::test]
    async fn test_production_config_with_targeting_data() {
        let sdk_key = "prod-sdk-key-2";
        setup_production_config(sdk_key);

        let mut user = create_test_user("prod_user_targeted");
        // Add targeting data that might affect bucketing
        user.country = "CA".to_string();
        user.custom_data.insert("beta_user".to_string(), serde_json::Value::Bool(true));
        user.custom_data.insert("subscription_tier".to_string(), serde_json::Value::String("premium".to_string()));

        let mut client_custom_data = HashMap::new();
        client_custom_data.insert("feature_flag_context".to_string(), serde_json::Value::String("production".to_string()));

        let result = unsafe {
            bucketing::generate_bucketed_config(sdk_key, user.clone(), client_custom_data).await
        };

        assert!(result.is_ok(), "Failed to generate bucketed config with targeting data: {:?}", result.err());

        let bucketed_config = result.unwrap();
        assert_eq!(bucketed_config.user.country, "CA");
        assert_eq!(bucketed_config.user.custom_data.get("beta_user").unwrap(), &serde_json::Value::Bool(true));
        assert_eq!(bucketed_config.user.custom_data.get("subscription_tier").unwrap(), &serde_json::Value::String("premium".to_string()));
    }

    #[tokio::test]
    async fn test_production_config_multiple_user_scenarios() {
        let sdk_key = "prod-sdk-key-3";
        setup_production_config(sdk_key);

        // Test different user scenarios that might exist in production
        let test_scenarios = vec![
            ("mobile_user", "1.5.0", "mobile", "US"),
            ("web_user", "2.1.3", "web", "GB"),
            ("api_user", "3.0.0", "server", "DE"),
            ("beta_user", "4.0.0-beta", "desktop", "FR"),
            ("legacy_user", "1.0.0", "legacy", "JP"),
        ];

        for (user_id, app_version, platform, country) in test_scenarios {
            let mut user = create_test_user(user_id);
            user.app_version = app_version.to_string();
            user.platform_data.platform = platform.to_string();
            user.country = country.to_string();

            let client_custom_data = HashMap::new();

            let result = unsafe {
                bucketing::generate_bucketed_config(sdk_key, user.clone(), client_custom_data).await
            };

            assert!(result.is_ok(), "Failed to generate bucketed config for user scenario {}: {:?}", user_id, result.err());

            let bucketed_config = result.unwrap();
            assert_eq!(bucketed_config.user.user_id, user_id);
            assert_eq!(bucketed_config.user.app_version, app_version);
            assert_eq!(bucketed_config.user.country, country);
        }
    }

    #[tokio::test]
    async fn test_production_config_stress_test() {
        let sdk_key = "prod-sdk-key-stress";
        setup_production_config(sdk_key);

        // Generate bucketed configs for many users to test performance and consistency
        let mut successful_buckets = 0;
        let total_users = 100;

        for i in 0..total_users {
            let user = create_test_user(&format!("stress_user_{}", i));
            let client_custom_data = HashMap::new();

            let result = unsafe {
                bucketing::generate_bucketed_config(sdk_key, user.clone(), client_custom_data).await
            };

            if result.is_ok() {
                successful_buckets += 1;
                let bucketed_config = result.unwrap();
                assert_eq!(bucketed_config.user.user_id, format!("stress_user_{}", i));
            }
        }

        // Ensure all bucketing operations succeeded
        assert_eq!(successful_buckets, total_users, "Not all users were successfully bucketed");
    }

    #[test]
    fn test_production_config_structure_validation() {
        let config = load_production_config();

        // Verify production config has the expected structure
        assert_eq!(config.project._id, "6216420c2ea68943c8833c09");
        assert_eq!(config.project.key, "default");
        assert_eq!(config.environment._id, "6216420c2ea68943c8833c0b");
        assert_eq!(config.environment.key, "development");

        // Verify features and variables exist
        assert!(!config.features.is_empty(), "Production config should have features");
        assert!(!config.variables.is_empty(), "Production config should have variables");

        // Verify specific test feature exists
        let test_feature = config.features.iter().find(|f| f.key == "test");
        assert!(test_feature.is_some(), "Production config should have 'test' feature");

        let test_feature = test_feature.unwrap();
        assert_eq!(test_feature._type, "release");
        assert_eq!(test_feature.variations.len(), 2);

        // Verify variable hashes exist
        assert!(!config.variable_hashes.is_empty(), "Production config should have variable hashes");
        assert!(config.variable_hashes.contains_key("test"), "Should have hash for 'test' variable");
    }

    #[tokio::test]
    async fn test_production_vs_test_config_consistency() {
        // Test both configs to ensure they produce similar results
        let test_sdk_key = "test-config-key";
        let prod_sdk_key = "prod-config-key";

        setup_test_config(test_sdk_key);
        setup_production_config(prod_sdk_key);

        let user = create_test_user("consistency_test_user");
        let client_custom_data = HashMap::new();

        let test_result = unsafe {
            bucketing::generate_bucketed_config(test_sdk_key, user.clone(), client_custom_data.clone()).await
        };

        let prod_result = unsafe {
            bucketing::generate_bucketed_config(prod_sdk_key, user.clone(), client_custom_data).await
        };

        assert!(test_result.is_ok(), "Test config bucketing failed");
        assert!(prod_result.is_ok(), "Production config bucketing failed");

        let test_config = test_result.unwrap();
        let prod_config = prod_result.unwrap();

        // Both should have the same project and environment IDs since they're the same config
        assert_eq!(test_config.project, prod_config.project);
        assert_eq!(test_config.environment, prod_config.environment);
        assert_eq!(test_config.user.user_id, prod_config.user.user_id);
    }

    #[tokio::test]
    async fn test_generate_bucketed_config_v2_user() {
        let user = create_test_user_v2("client_test_3");
        load_test_config_v2();
        let bucketing_result = unsafe {
            bucketing::generate_bucketed_config("test-sdk-key-1", user.clone(), HashMap::new()).await
        };
        assert!(bucketing_result.is_ok(), "Failed to generate bucketed config for v2 user: {:?}", bucketing_result.err());
        let bucketed_config = bucketing_result.unwrap();

        let json = serde_json::to_string_pretty(&bucketed_config);
        assert!(json.is_ok(), "Failed to serialize bucketed config to JSON: {:?}", json.err());
        println!("Bucketed Config JSON:\n{}", json.unwrap());
    }
}
