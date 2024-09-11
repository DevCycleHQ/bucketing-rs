mod user {
    use std::collections::HashMap;
    use serde::{Serialize, Deserialize};
    use chrono::{DateTime, Utc};
    use crate::platform_data;
    #[derive(Serialize, Deserialize, Debug)]
    pub struct User {
        // Unique id to identify the user
        user_id: str,
        // User's email used to identify the user on the dashboard / target audiences
        email: str,
        // User's name used to identify the user on the dashboard / target audiences
        name: str,
        // User's language in ISO 639-1 format
        language: str,
        // User's country in ISO 3166 alpha-2 format
        country: str,
        // App Version of the running application
        app_version: str,
        // App Build number of the running application
        app_build: str,
        // User's custom data to target the user with, data will be logged to DevCycle for use in dashboard.
        custom_data: HashMap<str, std::any>,
        // User's custom data to target the user with, data will not be logged to DevCycle only used for feature bucketing.
        private_custom_data: HashMap<str, std::any>,
        // User's device model
        device_model: str,
        // Date the user was created, Unix epoch timestamp format
        last_seen_date: DateTime<Utc>
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct PopulatedUser {
        user: &'a User,
        platform_data: platform_data,
        // Date the user was created, Unix epoch timestamp format
        created_date: Option<DateTime<Utc>>,
    }

    impl User {
        pub fn get_populated_user(&self, platform_data: platform_data) -> PopulatedUser {
            PopulatedUser {
                user: self.clone(),
                platform_data,
                created_date: Some(Utc::now()),
            }
        }

        // GetPopulatedUserWithTime returns a populated user with a specific created date
        pub fn get_populated_user_with_time(&self, platform_data: platform_data, create_date: DateTime<Utc>) -> PopulatedUser {
            PopulatedUser {
                user: self.clone()
                platform_data,
                created_date: Some(create_date),
            }
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct UserFeatureData {
        user: User,
        feature_vars: HashMap<String, String>,
    }

    impl PopulatedUser {
        pub fn merge_client_custom_data(&mut self, ccd: HashMap<String, serde_json::Value>) {
            if self.user.custom_data.is_none() {
                self.user.custom_data = Some(HashMap::new());
            }
            let custom_data = self.user.custom_data.as_mut().unwrap();
            for (k, v) in ccd {
                if !custom_data.contains_key(&k) && self.user.private_custom_data.as_ref().map_or(true, |pd| pd.get(&k).is_none()) {
                    custom_data.insert(k, v);
                }
            }
        }

        pub fn combined_custom_data(&self) -> HashMap<String, serde_json::Value> {
            let mut ret = HashMap::new();
            if let Some(ref custom_data) = self.user.custom_data {
                ret.extend(custom_data.clone());
            }
            if let Some(ref private_custom_data) = self.user.private_custom_data {
                ret.extend(private_custom_data.clone());
            }
            ret
        }
    }
}