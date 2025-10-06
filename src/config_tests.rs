#[cfg(test)]
mod tests {
    use crate::config::*;
    use serde_json;

    #[test]
    fn test_parse_project_json() {
        let json_data = serde_json::json!({
            "_id": "proj_123",
            "key": "my-project",
            "a0_organization": "org_456",
            "settings": {
                "edgedb": {
                    "enabled": true
                },
                "optin": {
                    "enabled": false,
                    "title": "Opt In Title",
                    "description": "Please opt in to our features",
                    "image_url": "https://example.com/image.png",
                    "colors": {
                        "primary": "#FF0000",
                        "secondary": "#00FF00"
                    }
                },
                "disable_passthrough_rollouts": false
            }
        });

        let project: Project =
            serde_json::from_value(json_data).expect("Failed to parse project JSON");

        assert_eq!(project._id, "proj_123");
        assert_eq!(project.key, "my-project");
        assert_eq!(project.a0_organization, "org_456");
        assert_eq!(project.settings.edgedb.enabled, true);
        assert_eq!(project.settings.optin.enabled, false);
        assert_eq!(project.settings.optin.title, "Opt In Title");
        assert_eq!(project.settings.optin.colors.primary, "#FF0000");
        assert_eq!(project.settings.disable_passthrough_rollouts, false);
    }

    #[test]
    fn test_parse_environment_json() {
        let json_str = r#"{
            "_id": "env_789",
            "key": "development"
        }"#;

        let environment: Environment =
            serde_json::from_str(json_str).expect("Failed to parse environment JSON");

        assert_eq!(environment._id, "env_789");
        assert_eq!(environment.key, "development");
    }

    #[test]
    fn test_parse_variable_json() {
        let json_str = r#"{
            "_id": "var_123",
            "type": "Boolean",
            "key": "enable_feature"
        }"#;

        let variable: Variable =
            serde_json::from_str(json_str).expect("Failed to parse variable JSON");

        assert_eq!(variable._id, "var_123");
        assert_eq!(variable._type, "Boolean");
        assert_eq!(variable.key, "enable_feature");
    }

    #[test]
    fn test_parse_variable_json_with_different_types() {
        let test_cases = vec![
            (
                r#"{"_id": "var_1", "type": "String", "key": "text_value"}"#,
                "String",
            ),
            (
                r#"{"_id": "var_2", "type": "Number", "key": "numeric_value"}"#,
                "Number",
            ),
            (
                r#"{"_id": "var_3", "type": "JSON", "key": "json_value"}"#,
                "JSON",
            ),
        ];

        for (json_str, expected_type) in test_cases {
            let variable: Variable = serde_json::from_str(json_str)
                .expect(&format!("Failed to parse variable JSON: {}", json_str));
            assert_eq!(variable._type, expected_type);
        }
    }

    #[test]
    fn test_parse_sse_json() {
        let json_str = r#"{
            "hostname": "events.devcycle.com",
            "path": "/v1/sse"
        }"#;

        let sse: SSE = serde_json::from_str(json_str).expect("Failed to parse SSE JSON");

        assert_eq!(sse.hostname, "events.devcycle.com");
        assert_eq!(sse.path, "/v1/sse");
    }

    #[test]
    fn test_parse_bucketing_configuration_json() {
        let json_str = r#"{
            "flush_events_interval": 30000,
            "disable_automatic_event_logging": false,
            "disable_custom_event_logging": true,
            "disable_push_state_event_logging": false
        }"#;

        let config: BucketingConfiguration =
            serde_json::from_str(json_str).expect("Failed to parse bucketing configuration JSON");

        assert_eq!(config.flush_events_interval, 30000);
        assert_eq!(config.disable_automatic_event_logging, false);
        assert_eq!(config.disable_custom_event_logging, true);
        assert_eq!(config.disable_push_state_event_logging, false);
    }

    #[test]
    fn test_serialize_and_deserialize_project() {
        let project = Project {
            _id: "test_proj".to_string(),
            key: "test-key".to_string(),
            a0_organization: "test_org".to_string(),
            settings: ProjectSettings {
                edgedb: EdgeDBSettings { enabled: true },
                optin: OptInSettings {
                    enabled: true,
                    title: "Test Title".to_string(),
                    description: "Test Description".to_string(),
                    image_url: "https://test.com/image.png".to_string(),
                    colors: OptInColors {
                        primary: "#FFFFFF".to_string(),
                        secondary: "#000000".to_string(),
                    },
                }
                .into(),
                disable_passthrough_rollouts: true,
                obfuscation: None,
            },
        };

        // Serialize to JSON
        let json_string = serde_json::to_string(&project).expect("Failed to serialize project");

        // Deserialize back from JSON
        let deserialized: Project =
            serde_json::from_str(&json_string).expect("Failed to deserialize project");

        assert_eq!(project._id, deserialized._id);
        assert_eq!(project.key, deserialized.key);
        assert_eq!(project.a0_organization, deserialized.a0_organization);
        assert_eq!(
            project.settings.edgedb.enabled,
            deserialized.settings.edgedb.enabled
        );
        assert_eq!(
            project.settings.optin.title,
            deserialized.settings.optin.title
        );
    }

    #[test]
    fn test_parse_variables_array() {
        let json_str = r#"[
            {
                "_id": "var_bool_1",
                "type": "Boolean",
                "key": "enable_new_ui"
            },
            {
                "_id": "var_str_1",
                "type": "String",
                "key": "welcome_message"
            },
            {
                "_id": "var_num_1",
                "type": "Number",
                "key": "max_items"
            }
        ]"#;

        let variables: Vec<Variable> =
            serde_json::from_str(json_str).expect("Failed to parse variables array JSON");

        assert_eq!(variables.len(), 3);
        assert_eq!(variables[0]._id, "var_bool_1");
        assert_eq!(variables[0]._type, "Boolean");
        assert_eq!(variables[0].key, "enable_new_ui");

        assert_eq!(variables[1]._id, "var_str_1");
        assert_eq!(variables[1]._type, "String");
        assert_eq!(variables[1].key, "welcome_message");

        assert_eq!(variables[2]._id, "var_num_1");
        assert_eq!(variables[2]._type, "Number");
        assert_eq!(variables[2].key, "max_items");
    }

    #[test]
    fn test_invalid_json_handling() {
        let invalid_json = r#"{
            "_id": "incomplete_project"
        }"#;

        let result: Result<Project, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err(), "Should fail to parse invalid JSON");
    }

    #[test]
    fn test_missing_required_fields() {
        let json_missing_key = r#"{
            "_id": "proj_123",
            "a0_organization": "org_456"
        }"#;

        let result: Result<Project, _> = serde_json::from_str(json_missing_key);
        assert!(
            result.is_err(),
            "Should fail when required field 'key' is missing"
        );
    }

    #[test]
    fn test_parse_complex_project_config() {
        let json_data = serde_json::json!({
            "_id": "60f7c84e4f9c4a001f6b8c23",
            "key": "production-app",
            "a0_organization": "auth0_org_12345",
            "settings": {
                "edgedb": {
                    "enabled": true
                },
                "optin": {
                    "enabled": true,
                    "title": "Feature Preview",
                    "description": "Enable beta features and provide feedback to help us improve",
                    "image_url": "https://cdn.example.com/feature-preview.svg",
                    "colors": {
                        "primary": "#4285F4",
                        "secondary": "#34A853"
                    }
                },
                "disable_passthrough_rollouts": true
            }
        });

        let project: Project =
            serde_json::from_value(json_data).expect("Failed to parse complex project JSON");

        assert_eq!(project._id, "60f7c84e4f9c4a001f6b8c23");
        assert_eq!(project.key, "production-app");
        assert_eq!(project.a0_organization, "auth0_org_12345");
        assert_eq!(project.settings.edgedb.enabled, true);
        assert_eq!(project.settings.optin.enabled, true);
        assert_eq!(project.settings.optin.title, "Feature Preview");
        assert!(project.settings.optin.description.contains("beta features"));
        assert_eq!(project.settings.optin.colors.primary, "#4285F4");
        assert_eq!(project.settings.optin.colors.secondary, "#34A853");
        assert_eq!(project.settings.disable_passthrough_rollouts, true);
    }

    #[test]
    fn test_parse_environment_with_extra_fields() {
        let json_data = serde_json::json!({
            "_id": "env_101",
            "key": "production",
            "description": "Production environment",
            "enabled": true
        });

        let environment: Environment = serde_json::from_value(json_data)
            .expect("Failed to parse environment JSON with additional fields");

        assert_eq!(environment._id, "env_101");
        assert_eq!(environment.key, "production");
        // Additional fields are ignored since they're not in the struct
    }

    #[test]
    fn test_round_trip_serialization() {
        let original_variable = Variable {
            _id: "var_test".to_string(),
            _type: "String".to_string(),
            key: "test_key".to_string(),
        };

        // Serialize to JSON string
        let json_string =
            serde_json::to_string(&original_variable).expect("Failed to serialize variable");

        // Deserialize back to JSON string
        let deserialized: Variable =
            serde_json::from_str(&json_string).expect("Failed to deserialize variable");

        assert_eq!(original_variable._id, deserialized._id);
        assert_eq!(original_variable._type, deserialized._type);
        assert_eq!(original_variable.key, deserialized.key);
    }

    #[test]
    fn test_pretty_json_output() {
        let environment = Environment {
            _id: "env_pretty".to_string(),
            key: "staging".to_string(),
        };

        let pretty_json = serde_json::to_string_pretty(&environment)
            .expect("Failed to serialize environment as pretty JSON");

        println!("Pretty JSON output:\n{}", pretty_json);

        // Verify it can be parsed back
        let parsed: Environment =
            serde_json::from_str(&pretty_json).expect("Failed to parse pretty JSON");

        assert_eq!(environment._id, parsed._id);
        assert_eq!(environment.key, parsed.key);
    }

    #[test]
    fn test_bucketing_config_with_partial_fields() {
        let json_data = serde_json::json!({
            "flush_events_interval": 15000,
            "disable_automatic_event_logging": true,
            "disable_custom_event_logging": false,
            "disable_push_state_event_logging": true
        });

        let config: BucketingConfiguration =
            serde_json::from_value(json_data).expect("Failed to parse bucketing configuration");

        assert_eq!(config.flush_events_interval, 15000);
        assert_eq!(config.disable_automatic_event_logging, true);
        assert_eq!(config.disable_custom_event_logging, false);
        assert_eq!(config.disable_push_state_event_logging, true);
    }

    #[test]
    fn test_sse_config_serialization() {
        let sse = SSE {
            hostname: "sse.example.com".to_string(),
            path: "/events".to_string(),
        };

        let json_string =
            serde_json::to_string_pretty(&sse).expect("Failed to serialize SSE config");

        let deserialized: SSE =
            serde_json::from_str(&json_string).expect("Failed to deserialize SSE config");

        assert_eq!(sse.hostname, deserialized.hostname);
        assert_eq!(sse.path, deserialized.path);
    }

    #[test]
    fn test_parse_real_world_config() {
        let json_str = r#"{"project":{"_id":"638680c459f1b81cc9e6c557","key":"test-harness-data","a0_organization":"org_U9F8YMaTChTEndWw","settings":{"edgeDB":{"enabled":false},"optIn":{"enabled":false,"colors":{}},"obfuscation":{"required":false,"enabled":false},"disablePassthroughRollouts":true}},"environment":{"_id":"638680c459f1b81cc9e6c55b","key":"production"},"features":[{"_id":"689d0585cdd7cf74e6abf469","key":"redis","type":"release","variations":[{"key":"variation-on","name":"Variation On","variables":[{"_var":"689d0585cdd7cf74e6abf46f","value":true}],"_id":"689d0585cdd7cf74e6abf474"},{"key":"variation-off","name":"Variation Off","variables":[{"_var":"689d0585cdd7cf74e6abf46f","value":false}],"_id":"689d0585cdd7cf74e6abf475"}],"tags":[],"configuration":{"_id":"689d0585cdd7cf74e6abf478","targets":[{"_id":"689d05956884124802d69897","distribution":[{"_variation":"689d0585cdd7cf74e6abf474","percentage":1}],"_audience":{"_id":"","filters":{"filters":[{"_audiences":[],"values":[],"type":"all"}],"operator":"and"}},"bucketingKey":"user_id"}]}}],"variables":[{"_id":"638681f059f1b81cc9e6c7fa","key":"bool-var","type":"Boolean"},{"_id":"638681f059f1b81cc9e6c7fd","key":"json-var","type":"JSON"},{"_id":"638681f059f1b81cc9e6c7fc","key":"number-var","type":"Number"},{"_id":"689d0585cdd7cf74e6abf46f","key":"redis","type":"Boolean"},{"_id":"6386813a59f1b81cc9e6c68f","key":"schedule-feature","type":"Boolean"},{"_id":"638681f059f1b81cc9e6c7fb","key":"string-var","type":"String"},{"_id":"638680d6fcb67b96878d90e8","key":"test-harness","type":"Boolean"}],"variableHashes":{"bool-var":1457472961,"json-var":3396881128,"number-var":1339101624,"redis":2717177077,"schedule-feature":3964025047,"string-var":2569899200,"test-harness":4260170344},"audiences":{},"debugUsers":[],"sse":{"hostname":"https://sse.devcycle.com","path":"/event-stream?key=azZpGQ.64zbWw:RXCvOj0NO8V5CwpiNhiImhd1n7zsiS0QXgcOWElBxg4&v=1.2&channels=dvc_server_1604761d90fc68a649755aa89b487d1b6993a43f_v1"}}"#;

        let config: FullConfig =
            serde_json::from_str(json_str).expect("Failed to parse real-world config JSON");

        // Test project parsing
        assert_eq!(config.project._id, "638680c459f1b81cc9e6c557");
        assert_eq!(config.project.key, "test-harness-data");
        assert_eq!(config.project.a0_organization, "org_U9F8YMaTChTEndWw");

        // Test project settings
        assert_eq!(config.project.settings.edgedb.enabled, false);
        assert_eq!(config.project.settings.optin.enabled, false);
        assert_eq!(config.project.settings.disable_passthrough_rollouts, true);

        // Test obfuscation settings
        assert!(config.project.settings.obfuscation.is_some());
        let obfuscation = config.project.settings.obfuscation.unwrap();
        assert_eq!(obfuscation.required, false);
        assert_eq!(obfuscation.enabled, false);

        // Test environment
        assert_eq!(config.environment._id, "638680c459f1b81cc9e6c55b");
        assert_eq!(config.environment.key, "production");

        // Test features
        assert_eq!(config.features.len(), 1);
        let feature = &config.features[0];
        assert_eq!(feature._id, "689d0585cdd7cf74e6abf469");
        assert_eq!(feature.key, "redis");
        assert_eq!(feature._type, "release");

        // Test variables
        assert_eq!(config.variables.len(), 7);

        // Find and test specific variables
        let bool_var = config
            .variables
            .iter()
            .find(|v| v.key == "bool-var")
            .expect("bool-var should exist");
        assert_eq!(bool_var._id, "638681f059f1b81cc9e6c7fa");
        assert_eq!(bool_var._type, "Boolean");

        let redis_var = config
            .variables
            .iter()
            .find(|v| v.key == "redis")
            .expect("redis variable should exist");
        assert_eq!(redis_var._id, "689d0585cdd7cf74e6abf46f");
        assert_eq!(redis_var._type, "Boolean");

        // Test variable hashes
        assert_eq!(config.variable_hashes.len(), 7);
        assert_eq!(config.variable_hashes.get("bool-var"), Some(&1457472961));
        assert_eq!(config.variable_hashes.get("redis"), Some(&2717177077));
        assert_eq!(
            config.variable_hashes.get("test-harness"),
            Some(&4260170344)
        );

        // Test SSE config - now properly unwrapping the Option
        let sse = config.sse.as_ref().expect("SSE should be present");
        assert_eq!(sse.hostname, "https://sse.devcycle.com");
        assert!(sse.path.contains("event-stream"));
        assert!(sse.path.contains("azZpGQ.64zbWw"));

        // Test that audiences and debug users are empty
        assert!(config.audiences.is_empty());
        assert!(config.debug_users.is_empty());
    }

    #[test]
    fn test_parse_project_with_field_aliases() {
        let json_data = serde_json::json!({
            "_id": "test_id",
            "key": "test_key",
            "a0_organization": "test_org",
            "settings": {
                "edgeDB": {
                    "enabled": true
                },
                "optIn": {
                    "enabled": true,
                    "colors": {}
                },
                "disablePassthroughRollouts": false,
                "obfuscation": {
                    "required": true,
                    "enabled": true
                }
            }
        });

        let project: Project =
            serde_json::from_value(json_data).expect("Failed to parse project with field aliases");

        assert_eq!(project.settings.edgedb.enabled, true);
        assert_eq!(project.settings.optin.enabled, true);
        assert_eq!(project.settings.disable_passthrough_rollouts, false);

        assert!(project.settings.obfuscation.is_some());
        let obfuscation = project.settings.obfuscation.unwrap();
        assert_eq!(obfuscation.required, true);
        assert_eq!(obfuscation.enabled, true);
    }

    #[test]
    fn test_variable_types_from_real_config() {
        let variables_json = r#"[
            {"_id":"638681f059f1b81cc9e6c7fa","key":"bool-var","type":"Boolean"},
            {"_id":"638681f059f1b81cc9e6c7fd","key":"json-var","type":"JSON"},
            {"_id":"638681f059f1b81cc9e6c7fc","key":"number-var","type":"Number"},
            {"_id":"689d0585cdd7cf74e6abf46f","key":"redis","type":"Boolean"},
            {"_id":"6386813a59f1b81cc9e6c68f","key":"schedule-feature","type":"Boolean"},
            {"_id":"638681f059f1b81cc9e6c7fb","key":"string-var","type":"String"},
            {"_id":"638680d6fcb67b96878d90e8","key":"test-harness","type":"Boolean"}
        ]"#;

        let variables: Vec<Variable> = serde_json::from_str(variables_json)
            .expect("Failed to parse variables from real config");

        assert_eq!(variables.len(), 7);

        // Test that we have the expected variable types
        let type_counts =
            variables
                .iter()
                .fold(std::collections::HashMap::new(), |mut acc, var| {
                    *acc.entry(var._type.as_str()).or_insert(0) += 1;
                    acc
                });

        assert_eq!(type_counts.get("Boolean"), Some(&4)); // bool-var, redis, schedule-feature, test-harness
        assert_eq!(type_counts.get("String"), Some(&1)); // string-var
        assert_eq!(type_counts.get("Number"), Some(&1)); // number-var
        assert_eq!(type_counts.get("JSON"), Some(&1)); // json-var
    }

    #[tokio::test]
    async fn test_parse_production_config_from_cdn() {
        let config_url =
            "https://config-cdn.devcycle.com/config/v2/server/dvc_server_token_hash.json";

        // Fetch the config from the CDN
        let client = reqwest::Client::new();
        let response = client
            .get(config_url)
            .header("User-Agent", "devcycle-bucketing-rs/0.1.0")
            .send()
            .await
            .expect("Failed to fetch config from CDN");

        assert!(
            response.status().is_success(),
            "HTTP request failed with status: {}",
            response.status()
        );

        let config_json = response.text().await.expect("Failed to read response text");

        println!(
            "Fetched config JSON (first 500 chars): {}",
            &config_json.chars().take(500).collect::<String>()
        );

        // Parse the JSON into our config structure
        let config: FullConfig =
            serde_json::from_str(&config_json).expect("Failed to parse production config JSON");

        // Validate the basic structure
        assert!(
            !config.project._id.is_empty(),
            "Project ID should not be empty"
        );
        assert!(
            !config.project.key.is_empty(),
            "Project key should not be empty"
        );
        assert!(
            !config.environment._id.is_empty(),
            "Environment ID should not be empty"
        );
        assert!(
            !config.environment.key.is_empty(),
            "Environment key should not be empty"
        );

        // Validate we have some data
        println!("Production config summary:");
        println!("  Project: {} ({})", config.project.key, config.project._id);
        println!(
            "  Environment: {} ({})",
            config.environment.key, config.environment._id
        );
        println!("  Features: {}", config.features.len());
        println!("  Variables: {}", config.variables.len());
        println!("  Variable hashes: {}", config.variable_hashes.len());

        // Handle optional SSE field
        if let Some(ref sse) = config.sse {
            println!("  SSE hostname: {}", sse.hostname);

            // Test that SSE config is valid
            assert!(
                sse.hostname.starts_with("http"),
                "SSE hostname should be a URL"
            );
            assert!(!sse.path.is_empty(), "SSE path should not be empty");
        } else {
            println!("  SSE: Not present in config");
        }

        // Test variables if any exist
        if !config.variables.is_empty() {
            let first_var = &config.variables[0];
            assert!(!first_var._id.is_empty(), "Variable ID should not be empty");
            assert!(
                !first_var.key.is_empty(),
                "Variable key should not be empty"
            );
            assert!(
                !first_var._type.is_empty(),
                "Variable type should not be empty"
            );

            // Check that we have valid variable types
            let valid_types = ["Boolean", "String", "Number", "JSON"];
            assert!(
                valid_types.contains(&first_var._type.as_str()),
                "Variable type '{}' should be one of: {:?}",
                first_var._type,
                valid_types
            );
        }

        // Test features if any exist
        if !config.features.is_empty() {
            let first_feature = &config.features[0];
            assert!(
                !first_feature._id.is_empty(),
                "Feature ID should not be empty"
            );
            assert!(
                !first_feature.key.is_empty(),
                "Feature key should not be empty"
            );
            assert!(
                !first_feature._type.is_empty(),
                "Feature type should not be empty"
            );
        }

        // Test variable hashes correspondence
        for variable in &config.variables {
            if let Some(hash_value) = config.variable_hashes.get(&variable.key) {
                assert!(
                    *hash_value > 0,
                    "Variable hash for '{}' should be positive",
                    variable.key
                );
            }
        }

        println!("✅ Production config parsed and validated successfully!");
    }

    #[tokio::test]
    async fn test_production_config_error_handling() {
        let invalid_url = "https://config-cdn.devcycle.com/config/v2/server/nonexistent.json";

        let client = reqwest::Client::new();
        let response = client
            .get(invalid_url)
            .header("User-Agent", "devcycle-bucketing-rs/0.1.0")
            .send()
            .await
            .expect("Failed to make HTTP request");

        // This should return a 404 or similar error
        assert!(
            !response.status().is_success(),
            "Request to invalid config URL should fail, got status: {}",
            response.status()
        );

        println!(
            "✅ Error handling for invalid config URL works correctly (status: {})",
            response.status()
        );
    }

    #[test]
    fn test_parse_hardcoded_config_from_resources() {
        use std::fs;

        // Load the config from the resources file
        let config_path = "tests/resources/test_config.json";
        let config_json = fs::read_to_string(config_path).expect("Failed to read test config file");

        println!("Loaded config from resources file: {}", config_path);
        println!(
            "Config JSON (first 300 chars): {}",
            &config_json.chars().take(300).collect::<String>()
        );

        // Parse the JSON into our config structure
        let config: FullConfig =
            serde_json::from_str(&config_json).expect("Failed to parse hardcoded config JSON");

        // Validate project structure
        assert_eq!(config.project._id, "6216420c2ea68943c8833c09");
        assert_eq!(config.project.key, "default");
        assert_eq!(config.project.a0_organization, "org_NszUFyWBFy7cr95J");

        // Validate project settings
        assert_eq!(config.project.settings.edgedb.enabled, false);
        assert_eq!(config.project.settings.optin.enabled, true);
        assert_eq!(config.project.settings.optin.title, "Beta Feature Access");
        assert_eq!(
            config.project.settings.optin.description,
            "Get early access to new features below"
        );
        assert_eq!(config.project.settings.optin.colors.primary, "#0042f9");
        assert_eq!(config.project.settings.optin.colors.secondary, "#facc15");

        // Validate environment
        assert_eq!(config.environment._id, "6216420c2ea68943c8833c0b");
        assert_eq!(config.environment.key, "development");

        // Validate features
        assert_eq!(config.features.len(), 1);
        let feature = &config.features[0];
        assert_eq!(feature._id, "6216422850294da359385e8b");
        assert_eq!(feature.key, "test");
        assert_eq!(feature._type, "release");

        // Validate feature variations
        assert_eq!(feature.variations.len(), 2);

        // Check "Variation On"
        let variation_on = &feature.variations[0];
        assert_eq!(variation_on._id, "6216422850294da359385e8f");
        assert_eq!(variation_on.key, "variation-on");
        assert_eq!(variation_on.name, "Variation On");
        assert_eq!(variation_on.variables.len(), 5);

        // Check variable values in "Variation On"
        let bool_var = &variation_on.variables[0];
        assert_eq!(bool_var._var, "6216422850294da359385e8d");
        assert_eq!(bool_var.value, serde_json::Value::Bool(true));

        let number_var = &variation_on.variables[1];
        assert_eq!(number_var._var, "64de2b2486d4b575121589db");
        assert_eq!(
            number_var.value,
            serde_json::Value::Number(serde_json::Number::from(123))
        );

        let float_var = &variation_on.variables[2];
        assert_eq!(float_var._var, "64de2b9486d4b275121589d1");
        assert_eq!(float_var.value.as_f64(), Some(4.56));

        let string_var = &variation_on.variables[3];
        assert_eq!(string_var._var, "64de2b2486d4b575121589dc");
        assert_eq!(
            string_var.value,
            serde_json::Value::String("on".to_string())
        );

        let json_var = &variation_on.variables[4];
        assert_eq!(json_var._var, "64de88bcc99ba02630f3df80");
        let expected_json = serde_json::json!({"message": "a"});
        assert_eq!(json_var.value, expected_json);

        // Check "Variation Off"
        let variation_off = &feature.variations[1];
        assert_eq!(variation_off._id, "6216422850294da359385e90");
        assert_eq!(variation_off.key, "variation-off");
        assert_eq!(variation_off.name, "Variation Off");
        assert_eq!(variation_off.variables.len(), 5);

        // Check some variable values in "Variation Off"
        let bool_var_off = &variation_off.variables[0];
        assert_eq!(bool_var_off.value, serde_json::Value::Bool(false));

        let number_var_off = &variation_off.variables[1];
        assert_eq!(
            number_var_off.value,
            serde_json::Value::Number(serde_json::Number::from(0))
        );

        let json_var_off = &variation_off.variables[4];
        let expected_json_off = serde_json::json!({"message": "b"});
        assert_eq!(json_var_off.value, expected_json_off);

        // Validate feature configuration
        assert_eq!(feature.configuration._id, "621642332ea68943c8833c4a");
        assert_eq!(feature.configuration.targets.len(), 1);

        let target = &feature.configuration.targets[0];
        assert_eq!(target._id, "621642332ea68943c8833c4d");
        assert_eq!(target.distribution.len(), 2);

        // Check distribution percentages
        assert_eq!(target.distribution[0].percentage, 0.5);
        assert_eq!(target.distribution[0].variation, "6216422850294da359385e8f");
        assert_eq!(target.distribution[1].percentage, 0.5);
        assert_eq!(target.distribution[1].variation, "6216422850294da359385e90");

        // Validate variables
        assert_eq!(config.variables.len(), 5);

        let variable_types: std::collections::HashMap<&str, &str> = config
            .variables
            .iter()
            .map(|v| (v.key.as_str(), v._type.as_str()))
            .collect();

        assert_eq!(variable_types.get("test"), Some(&"Boolean"));
        assert_eq!(variable_types.get("test-number-variable"), Some(&"Number"));
        assert_eq!(variable_types.get("test-float-variable"), Some(&"Number"));
        assert_eq!(variable_types.get("test-string-variable"), Some(&"String"));
        assert_eq!(variable_types.get("test-json-variable"), Some(&"JSON"));

        // Validate variable hashes
        assert_eq!(config.variable_hashes.len(), 4); // Note: only 4 hashes in the config
        assert_eq!(config.variable_hashes.get("test"), Some(&2447239932));
        assert_eq!(
            config.variable_hashes.get("test-number-variable"),
            Some(&3332991395)
        );
        assert_eq!(
            config.variable_hashes.get("test-string-variable"),
            Some(&957171234)
        );
        assert_eq!(
            config.variable_hashes.get("test-json-variable"),
            Some(&2814889459)
        );

        // Check that SSE is not present in this config (should be None)
        assert!(
            config.sse.is_none(),
            "SSE should not be present in this test config"
        );

        // Print summary
        println!("✅ Hardcoded config validation summary:");
        println!("  Project: {} ({})", config.project.key, config.project._id);
        println!(
            "  Environment: {} ({})",
            config.environment.key, config.environment._id
        );
        println!("  Features: {}", config.features.len());
        println!("  Variables: {}", config.variables.len());
        println!("  Variable Hashes: {}", config.variable_hashes.len());
        println!("  Variations in test feature: {}", feature.variations.len());
        println!(
            "  Variables per variation: {}",
            variation_on.variables.len()
        );

        println!("✅ All validations passed for hardcoded config!");
    }
}
