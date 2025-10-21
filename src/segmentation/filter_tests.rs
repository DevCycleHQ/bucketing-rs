#[cfg(test)]
mod tests {
    use crate::constants;
    use crate::filters::*;
    use std::collections::HashMap;

    use crate::config::platform_data::{self, PlatformData};
    use crate::user::PopulatedUser;

    use chrono::Utc;
    use std::sync::{Arc, Once};

    const TEST_SDK_KEY: &str = "test-sdk-key";

    static INIT: Once = Once::new();
    fn initialize_test_platform_data() {
        INIT.call_once(|| {
            let platform_data = PlatformData {
                sdk_type: "server".to_string(),
                sdk_version: "1.0.0".to_string(),
                platform_version: "test".to_string(),
                device_model: "test-device".to_string(),
                platform: "Rust".to_string(),
                hostname: "localhost".to_string(),
            };
            platform_data::set_platform_data(TEST_SDK_KEY.to_string(), platform_data);
        });
    }

    fn create_test_user() -> PopulatedUser {
        initialize_test_platform_data();

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
            platform_data: platform_data::get_platform_data(TEST_SDK_KEY).unwrap(),
            created_date: Utc::now(),
        }
    }

    fn create_brooks_user() -> PopulatedUser {
        PopulatedUser {
            user_id: "brooks_user".to_string(),
            email: "brooks@big.lunch".to_string(),
            name: "Brooks".to_string(),
            language: "en".to_string(),
            country: "Canada".to_string(),
            app_version: "2.0.2".to_string(),
            app_build: "100".to_string(),
            custom_data: HashMap::new(),
            private_custom_data: HashMap::new(),
            device_model: "".to_string(),
            last_seen_date: Utc::now(),
            platform_data: Arc::new(PlatformData {
                sdk_type: "mobile".to_string(),
                sdk_version: "1.0.0".to_string(),
                platform_version: "10.3.1".to_string(),
                device_model: "".to_string(),
                platform: "iOS".to_string(),
                hostname: "".to_string(),
            }),
            created_date: Utc::now(),
        }
    }

    #[test]
    fn test_evaluate_operator_fail_empty() {
        let mut user = create_brooks_user();
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        // Empty AND operator should pass (vacuous truth in boolean logic)
        let and_operator = AudienceOperator {
            operator: constants::OPERATOR_AND.to_string(),
            filters: vec![],
        };
        assert!(and_operator.evaluate(&audiences, &mut user, &client_custom_data));

        // Empty OR operator should fail (no conditions to satisfy)
        let or_operator = AudienceOperator {
            operator: constants::OPERATOR_OR.to_string(),
            filters: vec![],
        };
        assert!(!or_operator.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_evaluate_operator_pass_all() {
        let mut user = create_brooks_user();
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        let all_filter = Filter {
            _type: constants::TYPE_ALL.to_string(),
            sub_type: None,
            comparator: None,
            values: vec![],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        // AND with all filter should pass
        let and_operator = AudienceOperator {
            operator: constants::OPERATOR_AND.to_string(),
            filters: vec![all_filter.clone()],
        };
        assert!(and_operator.evaluate(&audiences, &mut user, &client_custom_data));

        // OR with all filter should pass
        let or_operator = AudienceOperator {
            operator: constants::OPERATOR_OR.to_string(),
            filters: vec![all_filter],
        };
        assert!(or_operator.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_evaluate_operator_unknown_filter() {
        let mut user = create_brooks_user();
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        let unknown_filter = Filter {
            _type: "myNewFilter".to_string(),
            sub_type: None,
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        // AND with unknown filter should fail
        let and_operator = AudienceOperator {
            operator: constants::OPERATOR_AND.to_string(),
            filters: vec![unknown_filter.clone()],
        };
        assert!(!and_operator.evaluate(&audiences, &mut user, &client_custom_data));

        // OR with unknown filter should fail
        let or_operator = AudienceOperator {
            operator: constants::OPERATOR_OR.to_string(),
            filters: vec![unknown_filter],
        };
        assert!(!or_operator.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_evaluate_operator_invalid_comparator() {
        let mut user = create_brooks_user();
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        let email_filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_EMAIL.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![serde_json::Value::String("brooks@big.lunch".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        // Invalid operator should fail
        let invalid_operator = AudienceOperator {
            operator: "xylophone".to_string(),
            filters: vec![email_filter],
        };
        assert!(!invalid_operator.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_evaluate_operator_audience_filter_match() {
        let mut user = create_brooks_user();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        // Create filters
        let country_filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_COUNTRY.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![serde_json::Value::String("Canada".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let email_filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_EMAIL.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![
                serde_json::Value::String("dexter@smells.nice".to_string()),
                serde_json::Value::String("brooks@big.lunch".to_string()),
            ],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let version_filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_APP_VERSION.to_string()),
            comparator: Some(constants::COMPARATOR_GREATER.to_string()),
            values: vec![serde_json::Value::String("1.0.0".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        // Create audience
        let audience = NoIdAudience {
            filters: AudienceOperator {
                operator: constants::OPERATOR_AND.to_string(),
                filters: vec![country_filter, email_filter, version_filter],
            },
        };

        let mut audiences: HashMap<String, NoIdAudience> = HashMap::new();
        audiences.insert("test".to_string(), audience);

        // Test audience match filter with = comparator (user is in audience)
        let audience_match_filter = Filter {
            _type: constants::TYPE_AUDIENCE_MATCH.to_string(),
            sub_type: None,
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![],
            filters: vec![],
            operator: None,
            _audiences: vec!["test".to_string()],
        };

        let operator = AudienceOperator {
            operator: constants::OPERATOR_AND.to_string(),
            filters: vec![audience_match_filter],
        };
        assert!(operator.evaluate(&audiences, &mut user, &client_custom_data));

        // Test audience match filter with != comparator (user is in audience, so should fail)
        let audience_not_match_filter = Filter {
            _type: constants::TYPE_AUDIENCE_MATCH.to_string(),
            sub_type: None,
            comparator: Some(constants::COMPARATOR_NOT_EQUAL.to_string()),
            values: vec![],
            filters: vec![],
            operator: None,
            _audiences: vec!["test".to_string()],
        };

        let operator2 = AudienceOperator {
            operator: constants::OPERATOR_AND.to_string(),
            filters: vec![audience_not_match_filter],
        };
        assert!(!operator2.evaluate(&audiences, &mut user, &client_custom_data));

        // Test with audience ID not in list
        let empty_audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let audience_match_filter2 = Filter {
            _type: constants::TYPE_AUDIENCE_MATCH.to_string(),
            sub_type: None,
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![],
            filters: vec![],
            operator: None,
            _audiences: vec!["test".to_string()],
        };

        let operator3 = AudienceOperator {
            operator: constants::OPERATOR_AND.to_string(),
            filters: vec![audience_match_filter2],
        };
        assert!(!operator3.evaluate(&empty_audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_evaluate_operator_audience_filter_for_platforms() {
        let mut user = PopulatedUser {
            user_id: "9999".to_string(),
            email: "".to_string(),
            name: "".to_string(),
            language: "".to_string(),
            country: "".to_string(),
            app_version: "".to_string(),
            app_build: "".to_string(),
            custom_data: HashMap::new(),
            private_custom_data: HashMap::new(),
            device_model: "".to_string(),
            last_seen_date: Utc::now(),
            platform_data: Arc::new(PlatformData {
                sdk_type: "".to_string(),
                sdk_version: "".to_string(),
                platform_version: "".to_string(),
                device_model: "".to_string(),
                platform: "Android TV".to_string(),
                hostname: "".to_string(),
            }),
            created_date: Utc::now(),
        };

        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        let test_cases = vec![
            (
                "should filter all Android TV audiences properly if it is included in data - 3 elements",
                vec!["Android", "Fire TV", "Android TV"],
                true,
            ),
            (
                "should filter all Android TV audiences properly if it is included in data - 2 elements",
                vec!["Fire TV", "Android TV"],
                true,
            ),
            (
                "should filter all Android TV audiences properly if it is included in data - 1 element",
                vec!["Android TV"],
                true,
            ),
            (
                "should filter all Android TV audiences properly if it is included in data - similar but !=",
                vec!["Android"],
                false,
            ),
            (
                "should filter all Android TV audiences properly if it is included in data - different platform",
                vec!["iOS"],
                false,
            ),
        ];

        for (name, platform_values, expected) in test_cases {
            let platform_filter = Filter {
                _type: constants::TYPE_USER.to_string(),
                sub_type: Some(constants::SUB_TYPE_PLATFORM.to_string()),
                comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
                values: platform_values
                    .iter()
                    .map(|s| serde_json::Value::String(s.to_string()))
                    .collect(),
                filters: vec![],
                operator: None,
                _audiences: vec![],
            };

            let audience_id = "test";
            let audience = NoIdAudience {
                filters: AudienceOperator {
                    operator: constants::OPERATOR_AND.to_string(),
                    filters: vec![platform_filter],
                },
            };

            let mut audiences: HashMap<String, NoIdAudience> = HashMap::new();
            audiences.insert(audience_id.to_string(), audience);

            let audience_match_filter = Filter {
                _type: constants::TYPE_AUDIENCE_MATCH.to_string(),
                sub_type: None,
                comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
                values: vec![],
                filters: vec![],
                operator: None,
                _audiences: vec![audience_id.to_string()],
            };

            let operator = AudienceOperator {
                operator: constants::OPERATOR_AND.to_string(),
                filters: vec![audience_match_filter],
            };

            let result = operator.evaluate(&audiences, &mut user, &client_custom_data);
            assert_eq!(result, expected, "{}", name);
        }
    }

    #[test]
    fn test_evaluate_operator_audience_nested() {
        let mut user = create_brooks_user();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        let country_filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_COUNTRY.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![serde_json::Value::String("Canada".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let audience_inner = NoIdAudience {
            filters: AudienceOperator {
                operator: constants::OPERATOR_AND.to_string(),
                filters: vec![country_filter.clone()],
            },
        };

        let audience_outer = NoIdAudience {
            filters: AudienceOperator {
                operator: constants::OPERATOR_AND.to_string(),
                filters: vec![Filter {
                    _type: constants::TYPE_AUDIENCE_MATCH.to_string(),
                    sub_type: None,
                    comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
                    values: vec![],
                    filters: vec![],
                    operator: None,
                    _audiences: vec!["inner".to_string()],
                }],
            },
        };

        let mut audiences: HashMap<String, NoIdAudience> = HashMap::new();
        audiences.insert("outer".to_string(), audience_outer);
        audiences.insert("inner".to_string(), audience_inner);

        // Test nested audience matching
        let operator = AudienceOperator {
            operator: constants::OPERATOR_AND.to_string(),
            filters: vec![Filter {
                _type: constants::TYPE_AUDIENCE_MATCH.to_string(),
                sub_type: None,
                comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
                values: vec![],
                filters: vec![],
                operator: None,
                _audiences: vec!["outer".to_string()],
            }],
        };

        assert!(operator.evaluate(&audiences, &mut user, &client_custom_data));

        // Test mixing direct filter with audience match
        let operator2 = AudienceOperator {
            operator: constants::OPERATOR_AND.to_string(),
            filters: vec![
                country_filter,
                Filter {
                    _type: constants::TYPE_AUDIENCE_MATCH.to_string(),
                    sub_type: None,
                    comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
                    values: vec![],
                    filters: vec![],
                    operator: None,
                    _audiences: vec!["inner".to_string()],
                },
            ],
        };

        assert!(operator2.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_evaluate_operator_user_sub_filter_invalid() {
        let mut user = create_brooks_user();
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        let invalid_filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some("myNewFilter".to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let operator = AudienceOperator {
            operator: constants::OPERATOR_AND.to_string(),
            filters: vec![invalid_filter],
        };

        assert!(!operator.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_evaluate_operator_user_new_comparator() {
        let mut user = create_brooks_user();
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        let filter_with_unknown_comparator = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_EMAIL.to_string()),
            comparator: Some("wowNewComparator".to_string()),
            values: vec![],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let operator = AudienceOperator {
            operator: constants::OPERATOR_AND.to_string(),
            filters: vec![filter_with_unknown_comparator],
        };

        assert!(!operator.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_evaluate_operator_user_filters_and() {
        let mut user = create_brooks_user();
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        let country_filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_COUNTRY.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![serde_json::Value::String("Canada".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let email_filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_EMAIL.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![
                serde_json::Value::String("dexter@smells.nice".to_string()),
                serde_json::Value::String("brooks@big.lunch".to_string()),
            ],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let app_ver_filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_APP_VERSION.to_string()),
            comparator: Some(constants::COMPARATOR_GREATER.to_string()),
            values: vec![serde_json::Value::String("1.0.0".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let operator = AudienceOperator {
            operator: constants::OPERATOR_AND.to_string(),
            filters: vec![country_filter, email_filter, app_ver_filter],
        };

        assert!(operator.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_evaluate_operator_user_filters_or() {
        let mut user = create_brooks_user();
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        let country_filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_COUNTRY.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![serde_json::Value::String("Banada".to_string())], // Wrong country
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let email_filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_EMAIL.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![serde_json::Value::String("dexter@smells.nice".to_string())], // Wrong email
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let app_ver_filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_APP_VERSION.to_string()),
            comparator: Some(constants::COMPARATOR_GREATER.to_string()),
            values: vec![serde_json::Value::String("1.0.0".to_string())], // This will pass
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let operator = AudienceOperator {
            operator: constants::OPERATOR_OR.to_string(),
            filters: vec![country_filter, email_filter, app_ver_filter],
        };

        assert!(operator.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_evaluate_operator_nested_and() {
        let mut user = create_brooks_user();
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        let country_filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_COUNTRY.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![serde_json::Value::String("Canada".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let email_filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_EMAIL.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![
                serde_json::Value::String("dexter@smells.nice".to_string()),
                serde_json::Value::String("brooks@big.lunch".to_string()),
            ],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let app_ver_filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_APP_VERSION.to_string()),
            comparator: Some(constants::COMPARATOR_GREATER.to_string()),
            values: vec![serde_json::Value::String("1.0.0".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let nested_operator = Filter {
            _type: "".to_string(),
            sub_type: None,
            comparator: None,
            values: vec![],
            filters: vec![country_filter, email_filter, app_ver_filter],
            operator: Some(constants::OPERATOR_AND.to_string()),
            _audiences: vec![],
        };

        let top_level_filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_COUNTRY.to_string()),
            comparator: Some(constants::COMPARATOR_NOT_EQUAL.to_string()),
            values: vec![serde_json::Value::String("Nanada".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let operator = AudienceOperator {
            operator: constants::OPERATOR_AND.to_string(),
            filters: vec![nested_operator, top_level_filter.clone()],
        };

        assert!(operator.evaluate(&audiences, &mut user, &client_custom_data));

        // If the second AND filter fails, should fail to match
        let top_level_filter_fail = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_COUNTRY.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![serde_json::Value::String("Nanada".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let country_filter2 = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_COUNTRY.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![serde_json::Value::String("Canada".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let email_filter2 = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_EMAIL.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![
                serde_json::Value::String("dexter@smells.nice".to_string()),
                serde_json::Value::String("brooks@big.lunch".to_string()),
            ],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let app_ver_filter2 = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_APP_VERSION.to_string()),
            comparator: Some(constants::COMPARATOR_GREATER.to_string()),
            values: vec![serde_json::Value::String("1.0.0".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let nested_operator2 = Filter {
            _type: "".to_string(),
            sub_type: None,
            comparator: None,
            values: vec![],
            filters: vec![country_filter2, email_filter2, app_ver_filter2],
            operator: Some(constants::OPERATOR_AND.to_string()),
            _audiences: vec![],
        };

        let operator2 = AudienceOperator {
            operator: constants::OPERATOR_AND.to_string(),
            filters: vec![nested_operator2, top_level_filter_fail],
        };

        assert!(!operator2.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_evaluate_operator_nested_or() {
        let mut user = create_brooks_user();
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        let country_filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_COUNTRY.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![serde_json::Value::String("Nanada".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let email_filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_EMAIL.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![
                serde_json::Value::String("dexter@smells.nice".to_string()),
                serde_json::Value::String("brooks@big.lunch".to_string()),
            ],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let app_ver_filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_APP_VERSION.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![serde_json::Value::String("1.0.0".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let nested_operator = Filter {
            _type: "".to_string(),
            sub_type: None,
            comparator: None,
            values: vec![],
            filters: vec![country_filter, email_filter, app_ver_filter],
            operator: Some(constants::OPERATOR_OR.to_string()),
            _audiences: vec![],
        };

        let top_level_filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_COUNTRY.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![serde_json::Value::String("Nanada".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let operator = AudienceOperator {
            operator: constants::OPERATOR_OR.to_string(),
            filters: vec![nested_operator, top_level_filter],
        };

        assert!(operator.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_evaluate_operator_and_custom_data() {
        let mut user = create_brooks_user();
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        let country_filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_COUNTRY.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![serde_json::Value::String("Canada".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let custom_data_filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_CUSTOM_DATA.to_string()),
            comparator: Some(constants::COMPARATOR_NOT_EQUAL.to_string()),
            values: vec![
                serde_json::Value::String("something".to_string()),
                serde_json::Value::String("Canada".to_string()),
            ],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let operator = AudienceOperator {
            operator: constants::OPERATOR_AND.to_string(),
            filters: vec![country_filter, custom_data_filter],
        };

        assert!(operator.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_evaluate_operator_and_custom_data_prioritize_user_data() {
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let mut client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();
        client_custom_data.insert(
            "user_tier".to_string(),
            serde_json::Value::String("Silver".to_string()),
        );

        let mut user = PopulatedUser {
            user_id: "test".to_string(),
            email: "brooks@big.lunch".to_string(),
            name: "Brooks".to_string(),
            language: "en".to_string(),
            country: "Canada".to_string(),
            app_version: "2.0.2".to_string(),
            app_build: "100".to_string(),
            custom_data: {
                let mut data = HashMap::new();
                data.insert(
                    "user_tier".to_string(),
                    serde_json::Value::String("Gold".to_string()),
                );
                data
            },
            private_custom_data: HashMap::new(),
            device_model: "".to_string(),
            last_seen_date: Utc::now(),
            platform_data: Arc::new(PlatformData {
                sdk_type: "mobile".to_string(),
                sdk_version: "1.0.0".to_string(),
                platform_version: "2.0.0".to_string(),
                device_model: "".to_string(),
                platform: "iOS".to_string(),
                hostname: "".to_string(),
            }),
            created_date: Utc::now(),
        };

        let custom_data_filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_CUSTOM_DATA.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![
                serde_json::Value::String("user_tier".to_string()),
                serde_json::Value::String("Gold".to_string()),
            ],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let operator = AudienceOperator {
            operator: constants::OPERATOR_AND.to_string(),
            filters: vec![custom_data_filter.clone()],
        };

        // User has "Gold" in custom_data, should match even though clientCustomData has "Silver"
        assert!(operator.evaluate(&audiences, &mut user, &client_custom_data));

        // Now test with user having "Silver"
        user.custom_data.insert(
            "user_tier".to_string(),
            serde_json::Value::String("Silver".to_string()),
        );

        let operator2 = AudienceOperator {
            operator: constants::OPERATOR_AND.to_string(),
            filters: vec![custom_data_filter],
        };

        // User has "Silver", filter looking for "Gold", should fail
        assert!(!operator2.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_evaluate_operator_opt_in() {
        let mut user = PopulatedUser {
            user_id: "test".to_string(),
            email: "brooks@big.lunch".to_string(),
            name: "Brooks".to_string(),
            language: "en".to_string(),
            country: "Canada".to_string(),
            app_version: "2.0.2".to_string(),
            app_build: "100".to_string(),
            custom_data: HashMap::new(),
            private_custom_data: HashMap::new(),
            device_model: "".to_string(),
            last_seen_date: Utc::now(),
            platform_data: Arc::new(PlatformData {
                sdk_type: "mobile".to_string(),
                sdk_version: "1.0.0".to_string(),
                platform_version: "2.0.0".to_string(),
                device_model: "".to_string(),
                platform: "iOS".to_string(),
                hostname: "".to_string(),
            }),
            created_date: Utc::now(),
        };

        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        let opt_in_filter = Filter {
            _type: constants::TYPE_OPT_IN.to_string(),
            sub_type: None,
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let operator = AudienceOperator {
            operator: constants::OPERATOR_AND.to_string(),
            filters: vec![opt_in_filter],
        };

        // OptIn filter should always fail
        assert!(!operator.evaluate(&audiences, &mut user, &client_custom_data));
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
    fn test_user_id_filter_contain() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_USER_ID.to_string()),
            comparator: Some(constants::COMPARATOR_CONTAIN.to_string()),
            values: vec![
                serde_json::Value::String("5678".to_string()),
                serde_json::Value::String("1234".to_string()),
                serde_json::Value::String("000099".to_string()),
            ],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = PopulatedUser {
            user_id: "1234".to_string(),
            email: "".to_string(),
            name: "".to_string(),
            language: "".to_string(),
            country: "".to_string(),
            app_version: "".to_string(),
            app_build: "".to_string(),
            custom_data: HashMap::new(),
            private_custom_data: HashMap::new(),
            device_model: "".to_string(),
            last_seen_date: Utc::now(),
            platform_data: Arc::new(PlatformData {
                sdk_type: "".to_string(),
                sdk_version: "".to_string(),
                platform_version: "".to_string(),
                device_model: "".to_string(),
                platform: "".to_string(),
                hostname: "".to_string(),
            }),
            created_date: Utc::now(),
        };

        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_does_user_pass_filter_with_user_country_filter() {
        let mut user = PopulatedUser {
            user_id: "1234".to_string(),
            email: "".to_string(),
            name: "".to_string(),
            language: "".to_string(),
            country: "CA".to_string(),
            app_version: "".to_string(),
            app_build: "".to_string(),
            custom_data: HashMap::new(),
            private_custom_data: HashMap::new(),
            device_model: "".to_string(),
            last_seen_date: Utc::now(),
            platform_data: Arc::new(PlatformData {
                sdk_type: "".to_string(),
                sdk_version: "".to_string(),
                platform_version: "".to_string(),
                device_model: "".to_string(),
                platform: "".to_string(),
                hostname: "".to_string(),
            }),
            created_date: Utc::now(),
        };

        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        // User country equals filter
        let filter1 = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_COUNTRY.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![serde_json::Value::String("CA".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };
        assert!(filter1.evaluate(&audiences, &mut user, &client_custom_data));

        // User country does not equal filter
        let filter2 = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_COUNTRY.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![serde_json::Value::String("JP".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };
        assert!(!filter2.evaluate(&audiences, &mut user, &client_custom_data));

        // User country in filter set
        let filter3 = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_COUNTRY.to_string()),
            comparator: Some(constants::COMPARATOR_CONTAIN.to_string()),
            values: vec![
                serde_json::Value::String("US".to_string()),
                serde_json::Value::String("JP".to_string()),
                serde_json::Value::String("CA".to_string()),
            ],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };
        assert!(filter3.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_does_user_pass_filter_with_user_email_filter() {
        let mut user = PopulatedUser {
            user_id: "1234".to_string(),
            email: "test@devcycle.com".to_string(),
            name: "".to_string(),
            language: "".to_string(),
            country: "".to_string(),
            app_version: "".to_string(),
            app_build: "".to_string(),
            custom_data: HashMap::new(),
            private_custom_data: HashMap::new(),
            device_model: "".to_string(),
            last_seen_date: Utc::now(),
            platform_data: Arc::new(PlatformData {
                sdk_type: "".to_string(),
                sdk_version: "".to_string(),
                platform_version: "".to_string(),
                device_model: "".to_string(),
                platform: "".to_string(),
                hostname: "".to_string(),
            }),
            created_date: Utc::now(),
        };

        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        let test_cases = vec![
            (
                "User email equals filter",
                constants::COMPARATOR_EQUAL,
                vec!["test@devcycle.com"],
                true,
            ),
            (
                "User email does not equal filter",
                constants::COMPARATOR_EQUAL,
                vec!["someone.else@devcycle.com"],
                false,
            ),
            (
                "User email in filter set",
                constants::COMPARATOR_CONTAIN,
                vec!["@gmail.com", "@devcycle.com", "@hotmail.com"],
                true,
            ),
            (
                "User email starts with filter",
                constants::COMPARATOR_START_WITH,
                vec!["test"],
                true,
            ),
            (
                "User email ends with filter",
                constants::COMPARATOR_END_WITH,
                vec!["@devcycle.com"],
                true,
            ),
            (
                "User email does not start with filter",
                constants::COMPARATOR_NOT_START_WITH,
                vec!["user"],
                true,
            ),
            (
                "User email does not end with filter",
                constants::COMPARATOR_NOT_END_WITH,
                vec!["@devcycle.io"],
                true,
            ),
            (
                "User email does start with filter with empty value",
                constants::COMPARATOR_START_WITH,
                vec![""],
                false,
            ),
            (
                "User email does end with filter with empty value",
                constants::COMPARATOR_END_WITH,
                vec![""],
                false,
            ),
            (
                "User email does contain filter with empty value",
                constants::COMPARATOR_CONTAIN,
                vec![""],
                false,
            ),
            (
                "User email does not start with filter with empty value",
                constants::COMPARATOR_NOT_START_WITH,
                vec![""],
                true,
            ),
            (
                "User email does not end with filter with empty value",
                constants::COMPARATOR_NOT_END_WITH,
                vec![""],
                true,
            ),
            (
                "User email does not contain filter with empty value",
                constants::COMPARATOR_NOT_CONTAIN,
                vec![""],
                true,
            ),
        ];

        for (name, comparator, values, expected) in test_cases {
            let filter = Filter {
                _type: constants::TYPE_USER.to_string(),
                sub_type: Some(constants::SUB_TYPE_EMAIL.to_string()),
                comparator: Some(comparator.to_string()),
                values: values
                    .iter()
                    .map(|s| serde_json::Value::String(s.to_string()))
                    .collect(),
                filters: vec![],
                operator: None,
                _audiences: vec![],
            };

            let result = filter.evaluate(&audiences, &mut user, &client_custom_data);
            assert_eq!(result, expected, "{}", name);
        }
    }

    #[test]
    fn test_does_user_pass_filter_with_user_platform_filter() {
        let mut user = PopulatedUser {
            user_id: "1234".to_string(),
            email: "".to_string(),
            name: "".to_string(),
            language: "".to_string(),
            country: "".to_string(),
            app_version: "".to_string(),
            app_build: "".to_string(),
            custom_data: HashMap::new(),
            private_custom_data: HashMap::new(),
            device_model: "".to_string(),
            last_seen_date: Utc::now(),
            platform_data: Arc::new(PlatformData {
                sdk_type: "".to_string(),
                sdk_version: "".to_string(),
                platform_version: "10.3.1".to_string(),
                device_model: "".to_string(),
                platform: "iOS".to_string(),
                hostname: "".to_string(),
            }),
            created_date: Utc::now(),
        };

        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        let test_cases = vec![
            (
                "User platform equals filter",
                constants::COMPARATOR_EQUAL,
                vec!["iOS"],
                true,
            ),
            (
                "User platform does not equal filter",
                constants::COMPARATOR_EQUAL,
                vec!["Linux"],
                false,
            ),
            (
                "User platform in filter set",
                constants::COMPARATOR_CONTAIN,
                vec!["Linux", "macOS", "iOS"],
                true,
            ),
        ];

        for (name, comparator, values, expected) in test_cases {
            let filter = Filter {
                _type: constants::TYPE_USER.to_string(),
                sub_type: Some(constants::SUB_TYPE_PLATFORM.to_string()),
                comparator: Some(comparator.to_string()),
                values: values
                    .iter()
                    .map(|s| serde_json::Value::String(s.to_string()))
                    .collect(),
                filters: vec![],
                operator: None,
                _audiences: vec![],
            };

            let result = filter.evaluate(&audiences, &mut user, &client_custom_data);
            assert_eq!(result, expected, "{}", name);
        }
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
            values: vec![serde_json::Value::String("Rust".to_string())],
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

        assert_eq!(
            true,
            filter_gt.evaluate(&audiences, &mut user, &client_custom_data)
        );
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
        let test_cases = vec![("1", vec!["2", "1.1"]), ("1.1", vec!["1.2", "1"])];

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
                filters: vec![],
                operator: None,
                values: filter_values
                    .iter()
                    .map(|v| serde_json::Value::String(v.to_string()))
                    .collect(),
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

    #[test]
    fn test_custom_data_string_equal_no_data() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_CUSTOM_DATA.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![
                serde_json::Value::String("strKey".to_string()),
                serde_json::Value::String("value".to_string()),
            ],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        user.custom_data = HashMap::new();
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(!filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_custom_data_string_equal_match() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_CUSTOM_DATA.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![
                serde_json::Value::String("strKey".to_string()),
                serde_json::Value::String("value".to_string()),
            ],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        user.custom_data = HashMap::new();
        user.custom_data.insert(
            "strKey".to_string(),
            serde_json::Value::String("value".to_string()),
        );
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_custom_data_string_equal_or_values() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_CUSTOM_DATA.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![
                serde_json::Value::String("strKey".to_string()),
                serde_json::Value::String("value".to_string()),
                serde_json::Value::String("value too".to_string()),
            ],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        user.custom_data = HashMap::new();
        user.custom_data.insert(
            "strKey".to_string(),
            serde_json::Value::String("value".to_string()),
        );
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_custom_data_string_not_equal() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_CUSTOM_DATA.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![
                serde_json::Value::String("strKey".to_string()),
                serde_json::Value::String("value".to_string()),
            ],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        user.custom_data = HashMap::new();
        user.custom_data.insert(
            "strKey".to_string(),
            serde_json::Value::String("rutabaga".to_string()),
        );
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(!filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_custom_data_string_empty_string() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_CUSTOM_DATA.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![
                serde_json::Value::String("strKey".to_string()),
                serde_json::Value::String("value".to_string()),
            ],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        user.custom_data = HashMap::new();
        user.custom_data.insert(
            "strKey".to_string(),
            serde_json::Value::String("".to_string()),
        );
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(!filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_custom_data_string_key_not_present() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_CUSTOM_DATA.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![
                serde_json::Value::String("strKey".to_string()),
                serde_json::Value::String("value".to_string()),
            ],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        user.custom_data = HashMap::new();
        user.custom_data.insert(
            "otherKey".to_string(),
            serde_json::Value::String("something else".to_string()),
        );
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(!filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_custom_data_string_not_equal_multiple() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_CUSTOM_DATA.to_string()),
            comparator: Some(constants::COMPARATOR_NOT_EQUAL.to_string()),
            values: vec![
                serde_json::Value::String("strKey".to_string()),
                serde_json::Value::String("value1".to_string()),
                serde_json::Value::String("value2".to_string()),
                serde_json::Value::String("value3".to_string()),
            ],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        user.custom_data = HashMap::new();
        user.custom_data.insert(
            "strKey".to_string(),
            serde_json::Value::String("value".to_string()),
        );
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_custom_data_number_equal() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_CUSTOM_DATA.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![
                serde_json::Value::String("numKey".to_string()),
                serde_json::Value::Number(serde_json::Number::from(0)),
            ],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        user.custom_data = HashMap::new();
        user.custom_data.insert(
            "numKey".to_string(),
            serde_json::Value::Number(serde_json::Number::from(0)),
        );
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_custom_data_number_or_values() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_CUSTOM_DATA.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![
                serde_json::Value::String("numKey".to_string()),
                serde_json::Value::Number(serde_json::Number::from(0)),
                serde_json::Value::Number(serde_json::Number::from(1)),
            ],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        user.custom_data = HashMap::new();
        user.custom_data.insert(
            "numKey".to_string(),
            serde_json::Value::Number(serde_json::Number::from(1)),
        );
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_custom_data_number_not_equal() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_CUSTOM_DATA.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![
                serde_json::Value::String("numKey".to_string()),
                serde_json::Value::Number(serde_json::Number::from(0)),
            ],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        user.custom_data = HashMap::new();
        user.custom_data.insert(
            "numKey".to_string(),
            serde_json::Value::Number(serde_json::Number::from(1)),
        );
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(!filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_custom_data_bool_equal_true() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_CUSTOM_DATA.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![
                serde_json::Value::String("boolKey".to_string()),
                serde_json::Value::Bool(false),
            ],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        user.custom_data = HashMap::new();
        user.custom_data
            .insert("boolKey".to_string(), serde_json::Value::Bool(false));
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_custom_data_bool_not_equal() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_CUSTOM_DATA.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![
                serde_json::Value::String("boolKey".to_string()),
                serde_json::Value::Bool(false),
            ],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        user.custom_data = HashMap::new();
        user.custom_data
            .insert("boolKey".to_string(), serde_json::Value::Bool(true));
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(!filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_custom_data_not_equal_no_data() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_CUSTOM_DATA.to_string()),
            comparator: Some(constants::COMPARATOR_NOT_EQUAL.to_string()),
            values: vec![
                serde_json::Value::String("strKey".to_string()),
                serde_json::Value::String("value".to_string()),
            ],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        user.custom_data = HashMap::new();
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_custom_data_not_exist_no_data() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_CUSTOM_DATA.to_string()),
            comparator: Some(constants::COMPARATOR_NOT_EXIST.to_string()),
            values: vec![serde_json::Value::String("strKey".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        user.custom_data = HashMap::new();
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_custom_data_not_exist_with_data() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_CUSTOM_DATA.to_string()),
            comparator: Some(constants::COMPARATOR_NOT_EXIST.to_string()),
            values: vec![serde_json::Value::String("strKey".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        user.custom_data = HashMap::new();
        user.custom_data.insert(
            "strKey".to_string(),
            serde_json::Value::String("value".to_string()),
        );
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(!filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_custom_data_contains() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_CUSTOM_DATA.to_string()),
            comparator: Some(constants::COMPARATOR_CONTAIN.to_string()),
            values: vec![
                serde_json::Value::String("last_order_no".to_string()),
                serde_json::Value::String("FP".to_string()),
            ],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        user.custom_data = HashMap::new();
        user.custom_data.insert(
            "last_order_no".to_string(),
            serde_json::Value::String("FP2423423".to_string()),
        );
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_custom_data_not_contains() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_CUSTOM_DATA.to_string()),
            comparator: Some(constants::COMPARATOR_NOT_CONTAIN.to_string()),
            values: vec![
                serde_json::Value::String("last_order_no".to_string()),
                serde_json::Value::String("FP".to_string()),
            ],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        user.custom_data = HashMap::new();
        user.custom_data.insert(
            "last_order_no".to_string(),
            serde_json::Value::String("FP2423423".to_string()),
        );
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(!filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_custom_data_exist_with_value() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_CUSTOM_DATA.to_string()),
            comparator: Some(constants::COMPARATOR_EXIST.to_string()),
            values: vec![serde_json::Value::String("last_order_no".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        user.custom_data = HashMap::new();
        user.custom_data.insert(
            "last_order_no".to_string(),
            serde_json::Value::String("FP2423423".to_string()),
        );
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_custom_data_exist_without_field() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_CUSTOM_DATA.to_string()),
            comparator: Some(constants::COMPARATOR_EXIST.to_string()),
            values: vec![serde_json::Value::String("last_order_no".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        user.custom_data = HashMap::new();
        user.custom_data.insert(
            "otherField".to_string(),
            serde_json::Value::String("value".to_string()),
        );
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(!filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_custom_data_exist_empty_data() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_CUSTOM_DATA.to_string()),
            comparator: Some(constants::COMPARATOR_EXIST.to_string()),
            values: vec![serde_json::Value::String("last_order_no".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        user.custom_data = HashMap::new();
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(!filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_custom_data_with_client_custom_data() {
        // Test that custom data works when passed via client_custom_data instead of user.custom_data
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_CUSTOM_DATA.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![
                serde_json::Value::String("strKey".to_string()),
                serde_json::Value::String("value".to_string()),
            ],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        user.custom_data = HashMap::new();
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let mut client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();
        client_custom_data.insert(
            "strKey".to_string(),
            serde_json::Value::String("value".to_string()),
        );

        assert!(filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    // String Filter Tests - Converted from Go TestCheckStringsFilter

    #[test]
    fn test_string_filter_equal_empty_no_values() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_EMAIL.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        user.email = "".to_string();
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(!filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_string_filter_equal_match() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_PLATFORM.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![serde_json::Value::String("foo".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        user.platform_data = Arc::new(PlatformData {
            platform: "foo".to_string(),
            ..(*user.platform_data).clone()
        });
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_string_filter_equal_match_in_list() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_PLATFORM.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![
                serde_json::Value::String("iPhone OS".to_string()),
                serde_json::Value::String("Android".to_string()),
                serde_json::Value::String("Blackberry".to_string()),
            ],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        user.platform_data = Arc::new(PlatformData {
            platform: "Android".to_string(),
            ..(*user.platform_data).clone()
        });
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_string_filter_equal_no_match() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_PLATFORM.to_string()),
            comparator: Some(constants::COMPARATOR_EQUAL.to_string()),
            values: vec![serde_json::Value::String("foo".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        user.platform_data = Arc::new(PlatformData {
            platform: "fo".to_string(),
            ..(*user.platform_data).clone()
        });
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(!filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_string_filter_not_equal_match() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_PLATFORM.to_string()),
            comparator: Some(constants::COMPARATOR_NOT_EQUAL.to_string()),
            values: vec![serde_json::Value::String("foo".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        user.platform_data = Arc::new(PlatformData {
            platform: "foo".to_string(),
            ..(*user.platform_data).clone()
        });
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(!filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_string_filter_not_equal_match_in_list() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_PLATFORM.to_string()),
            comparator: Some(constants::COMPARATOR_NOT_EQUAL.to_string()),
            values: vec![
                serde_json::Value::String("iPhone OS".to_string()),
                serde_json::Value::String("Android".to_string()),
                serde_json::Value::String("Blackberry".to_string()),
            ],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        user.platform_data = Arc::new(PlatformData {
            platform: "Android".to_string(),
            ..(*user.platform_data).clone()
        });
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(!filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_string_filter_not_equal_no_match() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_PLATFORM.to_string()),
            comparator: Some(constants::COMPARATOR_NOT_EQUAL.to_string()),
            values: vec![serde_json::Value::String("foo".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        user.platform_data = Arc::new(PlatformData {
            platform: "bar".to_string(),
            ..(*user.platform_data).clone()
        });
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_string_filter_exist_empty() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_EMAIL.to_string()),
            comparator: Some(constants::COMPARATOR_EXIST.to_string()),
            values: vec![],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        user.email = "".to_string();
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(!filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_string_filter_exist_not_empty() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_EMAIL.to_string()),
            comparator: Some(constants::COMPARATOR_EXIST.to_string()),
            values: vec![],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        user.email = "string".to_string();
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_string_filter_not_exist_empty() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_EMAIL.to_string()),
            comparator: Some(constants::COMPARATOR_NOT_EXIST.to_string()),
            values: vec![],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        user.email = "".to_string();
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_string_filter_not_exist_not_empty() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_EMAIL.to_string()),
            comparator: Some(constants::COMPARATOR_NOT_EXIST.to_string()),
            values: vec![],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        user.email = "exists".to_string();
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(!filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_string_filter_contain_match() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_PLATFORM.to_string()),
            comparator: Some(constants::COMPARATOR_CONTAIN.to_string()),
            values: vec![serde_json::Value::String("Chrome".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        user.platform_data = Arc::new(PlatformData {
            platform: "Chrome".to_string(),
            ..(*user.platform_data).clone()
        });
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_string_filter_contain_partial_match() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_DEVICE_MODEL.to_string()),
            comparator: Some(constants::COMPARATOR_CONTAIN.to_string()),
            values: vec![serde_json::Value::String("hello".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        user.device_model = "helloWorld".to_string();
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_string_filter_contain_no_match() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_PLATFORM.to_string()),
            comparator: Some(constants::COMPARATOR_CONTAIN.to_string()),
            values: vec![serde_json::Value::String("foo".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        user.platform_data = Arc::new(PlatformData {
            platform: "bar".to_string(),
            ..(*user.platform_data).clone()
        });
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(!filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_string_filter_not_contain_match() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_DEVICE_MODEL.to_string()),
            comparator: Some(constants::COMPARATOR_NOT_CONTAIN.to_string()),
            values: vec![serde_json::Value::String("Desktop".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        user.device_model = "Desktop".to_string();
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(!filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_string_filter_not_contain_partial_match() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_DEVICE_MODEL.to_string()),
            comparator: Some(constants::COMPARATOR_NOT_CONTAIN.to_string()),
            values: vec![serde_json::Value::String("oob".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        user.device_model = "foobar".to_string();
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(!filter.evaluate(&audiences, &mut user, &client_custom_data));
    }

    #[test]
    fn test_string_filter_not_contain_no_match() {
        let filter = Filter {
            _type: constants::TYPE_USER.to_string(),
            sub_type: Some(constants::SUB_TYPE_PLATFORM.to_string()),
            comparator: Some(constants::COMPARATOR_NOT_CONTAIN.to_string()),
            values: vec![serde_json::Value::String("foo".to_string())],
            filters: vec![],
            operator: None,
            _audiences: vec![],
        };

        let mut user = create_test_user();
        user.platform_data = Arc::new(PlatformData {
            platform: "bar".to_string(),
            ..(*user.platform_data).clone()
        });
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();
        let client_custom_data: HashMap<String, serde_json::Value> = HashMap::new();

        assert!(filter.evaluate(&audiences, &mut user, &client_custom_data));
    }
}
