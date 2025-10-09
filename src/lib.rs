use crate::errors::DevCycleError;
use crate::user::{BucketedUserConfig, PopulatedUser, User};
use std::collections::HashMap;

pub mod bucketing;
pub mod config;
pub mod configmanager;
pub mod constants;
pub mod errors;
pub mod feature;
pub mod filters;
pub mod murmurhash;
pub mod platform_data;
pub mod target;
pub mod user;
pub mod versioncompare;

#[cfg(test)]
mod bucketing_tests;
mod client_custom_data;
#[cfg(test)]
mod config_tests;
#[cfg(test)]
mod configmanager_tests;
mod event;
mod event_queue;
#[cfg(test)]
mod filter_tests;
#[cfg(test)]
mod murmurhash_tests;
#[cfg(test)]
mod platform_data_tests;

// Export platform data types and functions for external SDKs
pub use platform_data::{get_platform_data, set_platform_data, PlatformData};

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

pub async unsafe fn generate_bucketed_config(
    sdk_key: &str,
    user: PopulatedUser,
    client_custom_data: HashMap<String, serde_json::Value>,
) -> Result<BucketedUserConfig, DevCycleError> {
    bucketing::generate_bucketed_config(sdk_key.to_string(), user, client_custom_data).await
}

pub async unsafe fn generate_bucketed_config_from_user(
    sdk_key: &str,
    user: User,
    client_custom_data: HashMap<String, serde_json::Value>,
) -> Result<BucketedUserConfig, DevCycleError> {
    let populated_user = user.get_populated_user(sdk_key);
    bucketing::generate_bucketed_config(sdk_key.to_string(), populated_user, client_custom_data)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
