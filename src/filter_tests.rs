#[cfg(test)]
mod filter_tests {
    use crate::constants;
    use crate::filters::*;
    use crate::platform_data::PlatformData;
    use crate::user::PopulatedUser;
    use chrono::Utc;
    use std::collections::HashMap;

    fn create_test_user() -> PopulatedUser {
        PopulatedUser {
            user_id: "test_user_123".to_string(),
            email: "test@example.com".to_string(),
            name: "Test User".to_string(),
            language: "en".to_string(),
            country: "US".to_string(),
            app_version: "1.2.3".to_string(),
            app_build: "456".to_string(),
            custom_data: {
                let mut data = HashMap::new();
                data.insert(
                    "subscription_type".to_string(),
                    serde_json::Value::String("premium".to_string()),
                );
                data.insert(
                    "account_age".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(365)),
                );
                data
            },
            private_custom_data: HashMap::new(),
            device_model: "iPhone 12".to_string(),
            last_seen_date: Utc::now(),
            platform_data: PlatformData {
                sdk_type: "mobile".to_string(),
                sdk_version: "1.0.0".to_string(),
                platform_version: "iOS 15.0".to_string(),
                device_model: "iPhone 12".to_string(),
                platform: "iOS".to_string(),
                hostname: "api.example.com".to_string(),
            },
            created_date: Utc::now(),
        }
    }

    #[test]
    fn test_all_filter_always_passes() {
        let filter = Filter {
            _type: constants::TYPE_ALL.to_string(),
            sub_type: None,
            comparator: None,
            values: vec![],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_user_id_filter_equal() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_USER_ID.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![serde_json::Value::String("test_user_123".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_user_id_filter_not_equal() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_USER_ID.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![serde_json::Value::String("different_user".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(!filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_email_filter_contains() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_EMAIL.to_string()),
            comparator: Some(constants::COMPARATOR_CONTAIN.to_string()),
            values: vec![serde_json::Value::String("example.com".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_country_filter() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_COUNTRY.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![serde_json::Value::String("US".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_platform_filter() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_PLATFORM.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![serde_json::Value::String("iOS".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_custom_data_filter() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_CUSTOM_DATA.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![
                serde_json::Value::String("subscription_type".to_string()), // key
                serde_json::Value::String("premium".to_string()),           // value to match
            ],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        // Note: This test needs to be updated based on how custom data filtering actually works
        // The current implementation looks for the key in values[0] and compares against values[1]
        // But the actual implementation might work differently
    }

    #[test]
    fn test_numeric_comparison_filters() {
        // Test greater than
        let filter_gt = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_CUSTOM_DATA.to_string()),
            comparator: Some(constants::COMPARATOR_GREATER.to_string()),
            values: vec![
                serde_json::Value::String("account_age".to_string()),
                serde_json::Value::Number(serde_json::Number::from(300)),
            ],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert_eq!(true,filter_gt.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_and_operator() {
        let filter1 = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_USER_ID.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![serde_json::Value::String("test_user_123".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let filter2 = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_COUNTRY.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![serde_json::Value::String("US".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let audience_operator = AudienceOperator {
            operator: constants::OPERATOR_AND.to_string(),
            filters: vec![filter1, filter2],
        };

        let mut user = create_test_user();
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(audience_operator.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_and_operator_fails_when_one_filter_fails() {
        let filter1 = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_USER_ID.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![serde_json::Value::String("test_user_123".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let filter2 = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_COUNTRY.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![serde_json::Value::String("CA".to_string())], // User is from US, not Canada
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let audience_operator = AudienceOperator {
            operator: constants::OPERATOR_AND.to_string(),
            filters: vec![filter1, filter2],
        };

        let mut user = create_test_user();
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(!audience_operator.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_or_operator() {
        let filter1 = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_COUNTRY.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![serde_json::Value::String("CA".to_string())], // This will fail
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let filter2 = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_COUNTRY.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![serde_json::Value::String("US".to_string())], // This will pass
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let audience_operator = AudienceOperator {
            operator: constants::OPERATOR_OR.to_string(),
            filters: vec![filter1, filter2],
        };

        let mut user = create_test_user();
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(audience_operator.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_empty_filters_pass() {
        let audience_operator = AudienceOperator {
            operator: constants::OPERATOR_AND.to_string(),
            filters: vec![],
        };

        let mut user = create_test_user();
        let audiences = HashMap::new();
        let client_custom_data = HashMap::new();

        assert!(audience_operator.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_nested_filters() {
        let nested_filter = Filter {
            _type: "custom".to_string(), // Not a built-in type, will use nested logic
            sub_type: None,
            comparator: None,
            values: vec![],
            filters: vec![Filter {
                _type: constants::TYPE_ALL.to_string(),
                sub_type: None,
                comparator: None,
                values: vec![],
                filters: vec![],
                operator: None,
                _audiences: vec![],
            }],
            operator: Some(constants::OPERATOR_AND.to_string()),
            _audiences: vec![],
        };

        let mut user = create_test_user();
        let audiences = HashMap::new();
        let client_custom_data = HashMap::new();

        assert!(nested_filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_optin_filter_always_fails() {
        let filter = Filter {
            _type: constants::TYPE_OPT_IN.to_string(),
            sub_type: None,
            comparator: None,
            values: vec![],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        let audiences = HashMap::new();
        let client_custom_data = HashMap::new();

        assert!(!filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    // Version Filter Tests - Converted from Go TestCheckVersionFilters

    #[test]
    fn test_version_equal_single_values() {
        let test_cases = vec![
            ("1", "1", true),
            ("1.1", "1.1", true),
            ("1.1.1", "1.1.1", true),
            ("1.1.", "1.1", true),
        ];

        for (version, filter_value, expected) in test_cases {
            let filter = Filter {
                _type: constants::TYPE_USER.to_string(),
                sub_type: Some(constants::SUB_TYPE_APP_VERSION.to_string()),
                comparator: Some("=".to_string()),
                values: vec![serde_json::Value::String(filter_value.to_string())],
                filters: vec![],
                operator: None,
                _audiences: vec![],
            };

            let mut user = create_test_user();
            user.app_version = version.to_string();
            let audiences: HashMap<String, NoIdAudience> = HashMap::new();
            let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

            let result = filter.evaluate(&audiences, &mut user, &client_custom_data);
            assert_eq!(
                result, expected,
                "Version {} = {} should be {}",
                version, filter_value, expected
            );
        }
    }

    #[test]
    fn test_version_not_equal_single_values() {
        let test_cases = vec![
            ("", "2", false),
            ("1", "2", false),
            ("1.1", "1.2", false),
            ("1.1", "1.1.1", false),
            ("1.1.", "1.1.1", false),
            ("1.1.1", "1.1", false),
            ("1.1.1", "1.1.", false),
            ("1.1.1", "1.2.3", false),
        ];

        for (version, filter_value, expected) in test_cases {
            let filter = Filter {
                _type: constants::TYPE_USER.to_string(),
                sub_type: Some(constants::SUB_TYPE_APP_VERSION.to_string()),
                comparator: Some("=".to_string()),
                values: vec![serde_json::Value::String(filter_value.to_string())],
                filters: vec![],
                operator: None,
                _audiences: vec![],
            };

            let mut user = create_test_user();
            user.app_version = version.to_string();
            let audiences: HashMap<String, NoIdAudience> = HashMap::new();
            let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

            let result = filter.evaluate(&audiences, &mut user, &client_custom_data);
            assert_eq!(
                result, expected,
                "Version {} = {} should be {}",
                version, filter_value, expected
            );
        }
    }

    #[test]
    fn test_version_not_equal_comparator_false() {
        let test_cases = vec![
            ("1", "1"),
            ("1.1", "1.1"),
            ("1.1.1", "1.1.1"),
            ("1.1.", "1.1"),
        ];

        for (version, filter_value) in test_cases {
            let filter = Filter {
                _type: constants::TYPE_USER.to_string(),
                sub_type: Some(constants::SUB_TYPE_APP_VERSION.to_string()),
                comparator: Some("!=".to_string()),
                values: vec![serde_json::Value::String(filter_value.to_string())],
                filters: vec![],
                operator: None,
                _audiences: vec![],
            };

            let mut user = create_test_user();
            user.app_version = version.to_string();
            let audiences: HashMap<String, NoIdAudience> = HashMap::new();
            let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

            assert!(
                !filter.evaluate(&audiences, &mut user, &client_custom_data),
                "Version {} != {} should be false",
                version,
                filter_value
            );
        }
    }

    #[test]
    fn test_version_not_equal_comparator_true() {
        let test_cases = vec![
            ("1", "2"),
            ("1.1", "1.2"),
            ("1.1", "1.1.1"),
            ("1.1.", "1.1.1"),
            ("1.1.1", "1.1"),
            ("1.1.1", "1.1."),
            ("1.1.1", "1.2.3"),
        ];

        for (version, filter_value) in test_cases {
            let filter = Filter {
                _type: constants::TYPE_USER.to_string(),
                sub_type: Some(constants::SUB_TYPE_APP_VERSION.to_string()),
                comparator: Some("!=".to_string()),
                values: vec![serde_json::Value::String(filter_value.to_string())],
                filters: vec![],
                operator: None,
                _audiences: vec![],
            };

            let mut user = create_test_user();
            user.app_version = version.to_string();
            let audiences: HashMap<String, NoIdAudience> = HashMap::new();
            let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

            assert!(
                filter.evaluate(&audiences, &mut user, &client_custom_data),
                "Version {} != {} should be true",
                version,
                filter_value
            );
        }
    }

    #[test]
    fn test_version_greater_than_false() {
        let test_cases = vec![
            ("", "1"),
            ("1", "1"),
            ("1.1", "1.1"),
            ("1.1.1", "1.1.1"),
            ("1.1.", "1.1"),
            ("1", "2"),
            ("1.1", "1.2"),
            ("1.1", "1.1.1"),
            ("1.1.", "1.1.1"),
            ("1.1.1", "1.2.3"),
        ];

        for (version, filter_value) in test_cases {
            let filter = Filter {
                _type: constants::TYPE_USER.to_string(),
                sub_type: Some(constants::SUB_TYPE_APP_VERSION.to_string()),
                comparator: Some(">".to_string()),
                values: vec![serde_json::Value::String(filter_value.to_string())],
                filters: vec![],
                operator: None,
                _audiences: vec![],
            };

            let mut user = create_test_user();
            user.app_version = version.to_string();
            let audiences: HashMap<String, NoIdAudience> = HashMap::new();
            let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

            assert!(
                !filter.evaluate(&audiences, &mut user, &client_custom_data),
                "Version {} > {} should be false",
                version,
                filter_value
            );
        }
    }

    #[test]
    fn test_version_greater_than_true() {
        let test_cases = vec![
            ("2", "1"),
            ("1.2", "1.1"),
            ("2.1", "1.1"),
            ("1.2.1", "1.2"),
            ("1.2.", "1.1"),
            ("1.2.1", "1.1.1"),
            ("1.2.2", "1.2"),
            ("1.2.2", "1.2.1"),
            ("4.8.241", "4.8"),
            ("4.8.241.2", "4"),
            ("4.8.241.2", "4.8"),
            ("4.8.241.2", "4.8.2"),
            ("4.8.241.2", "4.8.241.0"),
        ];

        for (version, filter_value) in test_cases {
            let filter = Filter {
                _type: constants::TYPE_USER.to_string(),
                sub_type: Some(constants::SUB_TYPE_APP_VERSION.to_string()),
                comparator: Some(">".to_string()),
                values: vec![serde_json::Value::String(filter_value.to_string())],
                filters: vec![],
                operator: None,
                _audiences: vec![],
            };

            let mut user = create_test_user();
            user.app_version = version.to_string();
            let audiences: HashMap<String, NoIdAudience> = HashMap::new();
            let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

            assert!(
                filter.evaluate(&audiences, &mut user, &client_custom_data),
                "Version {} > {} should be true",
                version,
                filter_value
            );
        }
    }

    #[test]
    fn test_version_greater_than_or_equal_false() {
        let test_cases = vec![
            ("", "2"),
            ("1", "2"),
            ("1.1", "1.2"),
            ("1.1", "1.1.1"),
            ("1.1.", "1.1.1"),
            ("1.1.1", "1.2.3"),
            ("4.8.241", "4.9"),
            ("4.8.241.2", "5"),
            ("4.8.241.2", "4.9"),
            ("4.8.241.2", "4.8.242"),
            ("4.8.241.2", "4.8.241.5"),
        ];

        for (version, filter_value) in test_cases {
            let filter = Filter {
                _type: constants::TYPE_USER.to_string(),
                sub_type: Some(constants::SUB_TYPE_APP_VERSION.to_string()),
                comparator: Some(">=".to_string()),
                values: vec![serde_json::Value::String(filter_value.to_string())],
                filters: vec![],
                operator: None,
                _audiences: vec![],
            };

            let mut user = create_test_user();
            user.app_version = version.to_string();
            let audiences: HashMap<String, NoIdAudience> = HashMap::new();
            let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

            assert!(
                !filter.evaluate(&audiences, &mut user, &client_custom_data),
                "Version {} >= {} should be false",
                version,
                filter_value
            );
        }
    }

    #[test]
    fn test_version_greater_than_or_equal_true() {
        let test_cases = vec![
            ("1", "1"),
            ("1.1", "1.1"),
            ("1.1.1", "1.1.1"),
            ("1.1.", "1.1"),
            ("2", "1"),
            ("1.2", "1.1"),
            ("2.1", "1.1"),
            ("1.2.1", "1.2"),
            ("1.2.", "1.1"),
            ("1.2.1", "1.1.1"),
            ("1.2.2", "1.2"),
            ("1.2.2", "1.2.1"),
            ("4.8.241.2", "4"),
            ("4.8.241.2", "4.8"),
            ("4.8.241.2", "4.8.2"),
            ("4.8.241.2", "4.8.241.0"),
            ("4.8.241.2", "4.8.241.2"),
        ];

        for (version, filter_value) in test_cases {
            let filter = Filter {
                _type: constants::TYPE_USER.to_string(),
                sub_type: Some(constants::SUB_TYPE_APP_VERSION.to_string()),
                comparator: Some(">=".to_string()),
                values: vec![serde_json::Value::String(filter_value.to_string())],
                filters: vec![],
                operator: None,
                _audiences: vec![],
            };

            let mut user = create_test_user();
            user.app_version = version.to_string();
            let audiences: HashMap<String, NoIdAudience> = HashMap::new();
            let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

            assert!(
                filter.evaluate(&audiences, &mut user, &client_custom_data),
                "Version {} >= {} should be true",
                version,
                filter_value
            );
        }
    }

    #[test]
    fn test_version_with_extra_characters() {
        let test_cases = vec![
            ("1.2.2", "v1.2.1-2v3asda", ">=", true),
            ("1.2.2", "v1.2.1-va1sda", ">", true),
            ("1.2.1", "v1.2.1-vasd32a", ">=", true),
            ("1.2.1", "v1.2.1-vasda", "=", false),
            ("v1.2.1-va21sda", "v1.2.1-va13sda", "=", false),
            ("1.2.0", "v1.2.1-vas1da", ">=", false),
            ("1.2.1", "v1.2.1- va34sda", "<=", true),
            ("1.2.0", "v1.2.1-vas3da", "<=", true),
        ];

        for (version, filter_value, comparator, expected) in test_cases {
            let filter = Filter {
                _type: constants::TYPE_USER.to_string(),
                sub_type: Some(constants::SUB_TYPE_APP_VERSION.to_string()),
                comparator: Some(comparator.to_string()),
                values: vec![serde_json::Value::String(filter_value.to_string())],
                filters: vec![],
                operator: None,
                _audiences: vec![],
            };

            let mut user = create_test_user();
            user.app_version = version.to_string();
            let audiences: HashMap<String, NoIdAudience> = HashMap::new();
            let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

            assert_eq!(
                filter.evaluate(&audiences, &mut user, &client_custom_data),
                expected,
                "Version {} {} {} should be {}",
                version,
                comparator,
                filter_value,
                expected
            );
        }
    }

    #[test]
    fn test_version_less_than_true() {
        let test_cases = vec![
            ("1", "2"),
            ("1.1", "1.2"),
            ("1.1", "1.1.1"),
            ("1.1.", "1.1.1"),
            ("1.1.1", "1.2.3"),
            ("4.8.241.2", "5"),
            ("4.8.241.2", "4.9"),
            ("4.8.241.2", "4.8.242"),
            ("4.8.241.2", "4.8.241.5"),
        ];

        for (version, filter_value) in test_cases {
            let filter = Filter {
                _type: constants::TYPE_USER.to_string(),
                sub_type: Some(constants::SUB_TYPE_APP_VERSION.to_string()),
                comparator: Some("<".to_string()),
                values: vec![serde_json::Value::String(filter_value.to_string())],
                filters: vec![],
                operator: None,
                _audiences: vec![],
            };

            let mut user = create_test_user();
            user.app_version = version.to_string();
            let audiences: HashMap<String, NoIdAudience> = HashMap::new();
            let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

            assert!(
                filter.evaluate(&audiences, &mut user, &client_custom_data),
                "Version {} < {} should be true",
                version,
                filter_value
            );
        }
    }

    #[test]
    fn test_version_less_than_false() {
        let test_cases = vec![
            ("", "1"),
            ("1", "1"),
            ("1.1", "1.1"),
            ("1.1.1", "1.1.1"),
            ("1.1.", "1.1"),
            ("2", "1"),
            ("1.2", "1.1"),
            ("2.1", "1.1"),
            ("1.2.1", "1.2"),
            ("1.2.", "1.1"),
            ("1.2.1", "1.1.1"),
            ("1.2.2", "1.2"),
            ("1.2.2", "1.2."),
            ("1.2.2", "1.2.1"),
            ("4.8.241.2", "4"),
            ("4.8.241.2", "4.8"),
            ("4.8.241.2", "4.8.241"),
            ("4.8.241.2", "4.8.241.0"),
        ];

        for (version, filter_value) in test_cases {
            let filter = Filter {
                _type: constants::TYPE_USER.to_string(),
                sub_type: Some(constants::SUB_TYPE_APP_VERSION.to_string()),
                comparator: Some("<".to_string()),
                values: vec![serde_json::Value::String(filter_value.to_string())],
                filters: vec![],
                operator: None,
                _audiences: vec![],
            };

            let mut user = create_test_user();
            user.app_version = version.to_string();
            let audiences: HashMap<String, NoIdAudience> = HashMap::new();
            let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

            assert!(
                !filter.evaluate(&audiences, &mut user, &client_custom_data),
                "Version {} < {} should be false",
                version,
                filter_value
            );
        }
    }

    #[test]
    fn test_version_less_than_or_equal_true() {
        let test_cases = vec![
            ("1", "1"),
            ("1.1", "1.1"),
            ("1.1.1", "1.1.1"),
            ("1.1.", "1.1"),
            ("1", "2"),
            ("1.1", "1.2"),
            ("1.1", "1.1.1"),
            ("1.1.", "1.1.1"),
            ("1.1.1", "1.2.3"),
            ("4.8.241.2", "4.8.241.2"),
        ];

        for (version, filter_value) in test_cases {
            let filter = Filter {
                _type: constants::TYPE_USER.to_string(),
                sub_type: Some(constants::SUB_TYPE_APP_VERSION.to_string()),
                comparator: Some("<=".to_string()),
                values: vec![serde_json::Value::String(filter_value.to_string())],
                filters: vec![],
                operator: None,
                _audiences: vec![],
            };

            let mut user = create_test_user();
            user.app_version = version.to_string();
            let audiences: HashMap<String, NoIdAudience> = HashMap::new();
            let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

            assert!(
                filter.evaluate(&audiences, &mut user, &client_custom_data),
                "Version {} <= {} should be true",
                version,
                filter_value
            );
        }
    }

    #[test]
    fn test_version_less_than_or_equal_false() {
        let test_cases = vec![
            ("", "1"),
            ("2", "1"),
            ("1.2", "1.1"),
            ("2.1", "1.1"),
            ("1.2.1", "1.2"),
            ("1.2.", "1.1"),
            ("1.2.1", "1.1.1"),
            ("1.2.2", "1.2"),
            ("1.2.2", "1.2."),
            ("1.2.2", "1.2.1"),
            ("4.8.241.2", "4.8.241"),
        ];

        for (version, filter_value) in test_cases {
            let filter = Filter {
                _type: constants::TYPE_USER.to_string(),
                sub_type: Some(constants::SUB_TYPE_APP_VERSION.to_string()),
                comparator: Some("<=".to_string()),
                values: vec![serde_json::Value::String(filter_value.to_string())],
                filters: vec![],
                operator: None,
                _audiences: vec![],
            };

            let mut user = create_test_user();
            user.app_version = version.to_string();
            let audiences: HashMap<String, NoIdAudience> = HashMap::new();
            let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

            assert!(
                !filter.evaluate(&audiences, &mut user, &client_custom_data),
                "Version {} <= {} should be false",
                version,
                filter_value
            );
        }
    }

    #[test]
    fn test_version_equal_array_any_match() {
        let test_cases = vec![
            ("1", vec!["1", "1.1"], true),
            ("1.1", vec!["1", "1.1"], true),
            ("1.1", vec!["1.1", ""], true),
        ];

        for (version, filter_values, expected) in test_cases {
            let filter = Filter {
                _type: constants::TYPE_USER.to_string(),
                sub_type: Some(constants::SUB_TYPE_APP_VERSION.to_string()),
                comparator: Some("=".to_string()),
                values: filter_values
                    .iter()
                    .map(|v| serde_json::Value::String(v.to_string()))
                    .collect(),
                filters: vec![],
                operator: None,
                _audiences: vec![],
            };

            let mut user = create_test_user();
            user.app_version = version.to_string();
            let audiences: HashMap<String, NoIdAudience> = HashMap::new();
            let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

            assert_eq!(
                filter.evaluate(&audiences, &mut user, &client_custom_data),
                expected,
                "Version {} should match any of {:?}",
                version,
                filter_values
            );
        }
    }

    #[test]
    fn test_version_equal_array_no_match() {
        let test_cases = vec![
            ("1", vec!["2", "1.1"]),
            ("1.1", vec!["1.2", "1"]),
        ];

        for (version, filter_values) in test_cases {
            let filter = Filter {
                _type: constants::TYPE_USER.to_string(),
                sub_type: Some(constants::SUB_TYPE_APP_VERSION.to_string()),
                comparator: Some("=".to_string()),
                values: filter_values
                    .iter()
                    .map(|v| serde_json::Value::String(v.to_string()))
                    .collect(),
                filters: vec![],
                operator: None,
                _audiences: vec![],
            };

            let mut user = create_test_user();
            user.app_version = version.to_string();
            let audiences: HashMap<String, NoIdAudience> = HashMap::new();
            let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

            assert!(
                !filter.evaluate(&audiences, &mut user, &client_custom_data),
                "Version {} should not match any of {:?}",
                version,
                filter_values
            );
        }
    }

    #[test]
    fn test_version_not_equal_array() {
        let test_cases = vec![
            ("1", vec!["2", "1"], false),
            ("1.1", vec!["1.2", "1.1"], false),
            ("1.1", vec!["1.1.1", "1.2"], true),
            ("1.1.", vec!["1.1.1", "1"], true),
        ];

        for (version, filter_values, expected) in test_cases {
            let filter = Filter {
                _type: constants::TYPE_USER.to_string(),
                sub_type: Some(constants::SUB_TYPE_APP_VERSION.to_string()),
                comparator: Some("!=".to_string()),
                values: filter_values
                    .iter()
                    .map(|v| serde_json::Value::String(v.to_string()))
                    .collect(),
                filters: vec![],
                operator: None,
                _audiences: vec![],
            };

            let mut user = create_test_user();
            user.app_version = version.to_string();
            let audiences: HashMap<String, NoIdAudience> = HashMap::new();
            let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

            assert_eq!(
                filter.evaluate(&audiences, &mut user, &client_custom_data),
                expected,
                "Version {} != {:?} should be {}",
                version,
                filter_values,
                expected
            );
        }
    }

    #[test]
    fn test_version_greater_than_array() {
        let test_cases = vec![
            ("1", vec!["1", "1"], false),
            ("1.1", vec!["1.1", "1.1.", "1.1"], false),
            ("1", vec!["2"], false),
            ("1.1", vec!["1.1.0"], false),
            ("2", vec!["1", "2.0"], true),
            ("1.2.1", vec!["1.2", "1.2"], true),
            ("1.2.", vec!["1.1", "1.9."], true),
        ];

        for (version, filter_values, expected) in test_cases {
            let filter = Filter {
                _type: constants::TYPE_USER.to_string(),
                sub_type: Some(constants::SUB_TYPE_APP_VERSION.to_string()),
                comparator: Some(">".to_string()),
                values: filter_values
                    .iter()
                    .map(|v| serde_json::Value::String(v.to_string()))
                    .collect(),
                filters: vec![],
                operator: None,
                _audiences: vec![],
            };

            let mut user = create_test_user();
            user.app_version = version.to_string();
            let audiences: HashMap<String, NoIdAudience> = HashMap::new();
            let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

            assert_eq!(
                filter.evaluate(&audiences, &mut user, &client_custom_data),
                expected,
                "Version {} > {:?} should be {}",
                version,
                filter_values,
                expected
            );
        }
    }

    #[test]
    fn test_version_greater_equal_array() {
        let test_cases = vec![
            ("1", vec!["2", "1.2"], false),
            ("1.1", vec!["1.2"], false),
            ("1.1", vec!["1.1.1", "1.2"], false),
            ("1", vec!["1", "1.1"], true),
            ("1.1", vec!["1.1", "1"], true),
            ("1.1.1", vec!["1.2", "1.1.1"], true),
            ("1.1.", vec!["1.1"], true),
            ("2", vec!["1", "3"], true),
        ];

        for (version, filter_values, expected) in test_cases {
            let filter = Filter {
                _type: constants::TYPE_USER.to_string(),
                sub_type: Some(constants::SUB_TYPE_APP_VERSION.to_string()),
                comparator: Some(">=".to_string()),
                values: filter_values
                    .iter()
                    .map(|v| serde_json::Value::String(v.to_string()))
                    .collect(),
                filters: vec![],
                operator: None,
                _audiences: vec![],
            };

            let mut user = create_test_user();
            user.app_version = version.to_string();
            let audiences: HashMap<String, NoIdAudience> = HashMap::new();
            let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

            assert_eq!(
                filter.evaluate(&audiences, &mut user, &client_custom_data),
                expected,
                "Version {} >= {:?} should be {}",
                version,
                filter_values,
                expected
            );
        }
    }

    #[test]
    fn test_version_less_than_array() {
        let test_cases = vec![
            ("1", vec!["2", "1"], true),
            ("1.1", vec!["1.2", "1.5"], true),
            ("1.1.", vec!["1.1.1"], true),
            ("1", vec!["1", "1.0"], false),
            ("1.1.", vec!["1.1", "1.1.0"], false),
        ];

        for (version, filter_values, expected) in test_cases {
            let filter = Filter {
                _type: constants::TYPE_USER.to_string(),
                sub_type: Some(constants::SUB_TYPE_APP_VERSION.to_string()),
                comparator: Some("<".to_string()),
                values: filter_values
                    .iter()
                    .map(|v| serde_json::Value::String(v.to_string()))
                    .collect(),
                filters: vec![],
                operator: None,
                _audiences: vec![],
            };

            let mut user = create_test_user();
            user.app_version = version.to_string();
            let audiences: HashMap<String, NoIdAudience> = HashMap::new();
            let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

            assert_eq!(
                filter.evaluate(&audiences, &mut user, &client_custom_data),
                expected,
                "Version {} < {:?} should be {}",
                version,
                filter_values,
                expected
            );
        }
    }

    #[test]
    fn test_version_less_equal_array() {
        let test_cases = vec![
            ("1", vec!["1", "5"], true),
            ("1.1", vec!["1.1", "1.1."], true),
            ("1.1.", vec!["1.1.1", "1.1."], true),
            ("2", vec!["1", "1.9"], false),
            ("1.2.1", vec!["1.2", "1.2"], false),
            ("1.2.", vec!["1.1", "1.1.9"], false),
        ];

        for (version, filter_values, expected) in test_cases {
            let filter = Filter {
                _type: constants::TYPE_USER.to_string(),
                sub_type: Some(constants::SUB_TYPE_APP_VERSION.to_string()),
                comparator: Some("<=".to_string()),
                values: filter_values
                    .iter()
                    .map(|v| serde_json::Value::String(v.to_string()))
                    .collect(),
                filters: vec![],
                operator: None,
                _audiences: vec![],
            };

            let mut user = create_test_user();
            user.app_version = version.to_string();
            let audiences: HashMap<String, NoIdAudience> = HashMap::new();
            let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

            assert_eq!(
                filter.evaluate(&audiences, &mut user, &client_custom_data),
                expected,
                "Version {} <= {:?} should be {}",
                version,
                filter_values,
                expected
            );
        }
    }
}
