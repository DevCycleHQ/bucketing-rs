use crate::config::platform_data::PlatformData;
use crate::config::{Environment, Project};
use crate::feature::{Feature, FeatureVariation, ReadOnlyVariable};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
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
    #[serde(default = "Utc::now")]
    pub last_seen_date: DateTime<Utc>,
}

impl User {
    pub fn get_populated_user(&self, sdk_key: &str) -> PopulatedUser {
        self.get_populated_user_with_platform_data_and_time(sdk_key, None, Utc::now())
    }

    pub fn get_populated_user_with_platform_data_and_time(
        &self,
        sdk_key: &str,
        platform_data: Option<Arc<PlatformData>>,
        create_date: DateTime<Utc>,
    ) -> PopulatedUser {
        let platform_data = platform_data
            .unwrap_or_else(|| crate::config::platform_data::get_platform_data(sdk_key).unwrap());

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

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PopulatedUser {
    #[serde(rename = "userId")]
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
    #[serde(rename = "appVersion")]
    pub app_version: String,
    // App Build number of the running application
    #[serde(rename = "appBuild")]
    pub app_build: String,
    // User's custom data to target the user with, data will be logged to DevCycle for use in dashboard.
    #[serde(rename = "customData")]
    pub custom_data: HashMap<String, serde_json::Value>,
    // User's custom data to target the user with, data will not be logged to DevCycle only used for feature bucketing.
    #[serde(rename = "privateCustomData")]
    pub private_custom_data: HashMap<String, serde_json::Value>,
    // User's device model
    #[serde(rename = "deviceModel")]
    pub device_model: String,
    // Date the user was created, Unix epoch timestamp format
    #[serde(rename = "lastSeenDate")]
    pub last_seen_date: DateTime<Utc>,
    // Platform data of the instance (Arc for efficient sharing across threads)
    #[serde(rename = "platformData")]
    pub platform_data: Arc<PlatformData>,
    // Date the user was created, Unix epoch timestamp format
    #[serde(rename = "createdDate")]
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
    pub fn new(
        user: User,
        platform_data: Arc<PlatformData>,
        client_custom_data: HashMap<String, serde_json::Value>,
    ) -> PopulatedUser {
        let mut popuser = PopulatedUser {
            user_id: user.user_id.clone(),
            email: user.email.clone(),
            name: user.name.clone(),
            private_custom_data: user.private_custom_data.clone(),
            custom_data: user.custom_data.clone(),
            language: user.language,
            country: user.country.clone(),
            app_version: user.app_version.clone(),
            app_build: user.app_build.clone(),
            device_model: user.device_model.clone(),
            last_seen_date: user.last_seen_date.clone(),
            platform_data,
            created_date: Utc::now(),
        };
        for (k, v) in client_custom_data {
            if !popuser.custom_data.contains_key(&k)
                && !popuser.private_custom_data.contains_key(&k)
            {
                popuser.custom_data.insert(k, v);
            }
        }

        return popuser;
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BucketedUserConfig {
    pub(crate) project: Project,
    pub(crate) environment: Environment,
    pub(crate) features: HashMap<String, Feature>,
    pub(crate) feature_variation_map: HashMap<String, String>,
    pub(crate) variable_variation_map: HashMap<String, FeatureVariation>,
    pub(crate) variables: HashMap<String, ReadOnlyVariable>,
    #[serde(skip_serializing)]
    pub(crate) user: PopulatedUser,
}
