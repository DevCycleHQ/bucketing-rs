use std::collections::HashMap;

use crate::config::{Environment, Project};
use crate::feature::{Feature, FeatureVariation, ReadOnlyVariable};
use crate::platform_data::PlatformData;
use chrono::{DateTime, Utc};

pub struct User {
    // Unique id to identify the user
    pub user_id: String,
    // User's email used to identify the user on the dashboard / target audiences
    pub email: String,
    // User's name used to identify the user on the dashboard / target audiences
    pub name: String,
    // User's language in ISO 639-1 format
    pub language: String,
    // User's country in ISO 3166 alpha-2 format
    pub country: String,
    // App Version of the running application
    pub app_version: String,
    // App Build number of the running application
    pub app_build: String,
    // User's custom data to target the user with, data will be logged to DevCycle for use in dashboard.
    pub custom_data: HashMap<String, serde_json::Value>,
    // User's custom data to target the user with, data will not be logged to DevCycle only used for feature bucketing.
    pub private_custom_data: HashMap<String, serde_json::Value>,
    // User's device model
    pub device_model: String,
    // Date the user was created, Unix epoch timestamp format
    pub last_seen_date: DateTime<Utc>,
}

impl User {
    pub fn get_populated_user(&self, platform_data: PlatformData) -> PopulatedUser {
        PopulatedUser {
            user_id: self.user_id.clone(),
            email: self.email.clone(),
            name: self.name.clone(),
            private_custom_data: self.private_custom_data.clone(),
            custom_data: self.custom_data.clone(),
            language: self.language.clone(),
            country: self.country.clone(),
            app_version: self.app_version.clone(),
            app_build: self.app_build.clone(),
            device_model: self.device_model.clone(),
            last_seen_date: self.last_seen_date.clone(),
            platform_data,
            created_date: Utc::now(),
        }
    }

    // GetPopulatedUserWithTime returns a populated user with a specific created date
    pub fn get_populated_user_with_time(
        &self,
        platform_data: PlatformData,
        create_date: DateTime<Utc>,
    ) -> PopulatedUser {
        PopulatedUser {
            user_id: self.user_id.clone(),
            email: self.email.clone(),
            name: self.name.clone(),
            private_custom_data: self.private_custom_data.clone(),
            custom_data: self.custom_data.clone(),
            language: self.language.clone(),
            country: self.country.clone(),
            app_version: self.app_version.clone(),
            app_build: self.app_build.clone(),
            device_model: self.device_model.clone(),
            last_seen_date: self.last_seen_date.clone(),
            platform_data,
            created_date: create_date,
        }
    }
}

pub struct UserFeatureData<'a> {
    user: &'a User,
    feature_vars: &'a HashMap<String, String>,
}

#[derive(Clone)]
pub struct PopulatedUser {
    pub user_id: String,
    // User's email used to identify the user on the dashboard / target audiences
    pub email: String,
    // User's name used to identify the user on the dashboard / target audiences
    pub name: String,
    // User's language in ISO 639-1 format
    pub language: String,
    // User's country in ISO 3166 alpha-2 format
    pub country: String,
    // App Version of the running application
    pub app_version: String,
    // App Build number of the running application
    pub app_build: String,
    // User's custom data to target the user with, data will be logged to DevCycle for use in dashboard.
    pub custom_data: HashMap<String, serde_json::Value>,
    // User's custom data to target the user with, data will not be logged to DevCycle only used for feature bucketing.
    pub private_custom_data: HashMap<String, serde_json::Value>,
    // User's device model
    pub device_model: String,
    // Date the user was created, Unix epoch timestamp format
    pub last_seen_date: DateTime<Utc>,
    // Platform data of the instance
    pub platform_data: PlatformData,
    // Date the user was created, Unix epoch timestamp format
    pub created_date: DateTime<Utc>,
}

impl PopulatedUser {
    pub fn merge_client_custom_data(
        mut self,
        client_custom_data: HashMap<String, serde_json::Value>,
    ) {
        for (k, v) in client_custom_data {
            if !self.custom_data.contains_key(&k) && !self.private_custom_data.contains_key(&k) {
                self.custom_data.insert(k, v);
            }
        }
    }

    pub fn combined_custom_data(&self) -> HashMap<String, serde_json::Value> {
        let mut ret = HashMap::new();
        if !self.custom_data.is_empty() {
            ret.extend(self.custom_data.clone());
        }
        if !self.private_custom_data.is_empty() {
            ret.extend(self.private_custom_data.clone());
        }
        ret
    }
}

pub struct BucketedUserConfig {
    pub(crate) project: String, // Changed from Project to String to match the data we have
    pub(crate) environment: String, // Changed from Environment to String to match the data we have
    pub(crate) features: HashMap<String, Feature>,
    pub(crate) feature_variation_map: HashMap<String, String>,
    pub(crate) variable_variation_map: HashMap<String, FeatureVariation>,
    pub(crate) variables: HashMap<String, ReadOnlyVariable>,
    pub(crate) known_variable_keys: Vec<String>, // Fixed type from Vec<f64> to Vec<String>
    pub(crate) user: User,
}
