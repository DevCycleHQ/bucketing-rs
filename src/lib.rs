use std::collections::HashMap;
use crate::errors::errors::DevCycleError;
use crate::user::user::PopulatedUser;

pub mod murmurhash;
pub mod constants;
pub mod versioncompare;
pub mod filters;
pub mod feature;
pub mod user;
pub mod platform_data;
pub mod target;
pub mod errors;
pub mod config;
pub mod bucketing;
pub mod configmanager;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}
pub async unsafe fn generate_bucketed_config(
    sdk_key: &str,
    user: PopulatedUser,
    client_custom_data: HashMap<String, serde_json::Value>,
) -> Result<user::user::BucketedUserConfig, DevCycleError> {
    bucketing::bucketing::generate_bucketed_config(sdk_key, user, client_custom_data).await
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
