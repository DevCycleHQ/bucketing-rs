use crate::errors::DevCycleError;
use crate::user::{BucketedUserConfig, User};
use std::collections::HashMap;

// Module declarations - now organized with mod.rs files in each folder
pub(crate) mod bucketing;
pub(crate) mod config;
pub(crate) mod events;
pub(crate) mod segmentation;
pub mod user;
pub(crate) mod util;

// FFI bindings for C library support
#[cfg(feature = "ffi")]
pub mod ffi;

// WASM bindings for WebAssembly support
#[cfg(feature = "wasm")]
pub mod wasm;

// Internal re-exports for convenience within the crate
pub(crate) use config::configmanager;
pub(crate) use config::feature;
// Re-export only what's needed for the public API
pub use config::platform_data::*;
pub use events::event::{DefaultReason, EvalDetails, EvaluationReason};
pub use events::EventQueueOptions;
pub(crate) use segmentation::filters;
pub(crate) use segmentation::target;
pub use user::BucketedUserConfig as BucketedConfig;
pub use user::PopulatedUser;
pub use user::User as DevCycleUser;
pub(crate) use util::constants;
pub(crate) use util::errors;
pub use util::errors::DevCycleError as Error;
pub(crate) use util::murmurhash;

use crate::bucketing::bucketing::VariableForUserResult;

pub async fn generate_bucketed_config(
    sdk_key: &str,
    user: user::PopulatedUser,
    client_custom_data: HashMap<String, serde_json::Value>,
) -> Result<BucketedUserConfig, DevCycleError> {
    bucketing::generate_bucketed_config(sdk_key.to_string(), user, client_custom_data).await
}

pub async fn generate_bucketed_config_from_user(
    sdk_key: &str,
    user: User,
    client_custom_data: HashMap<String, serde_json::Value>,
) -> Result<BucketedUserConfig, DevCycleError> {
    let populated_user = user.get_populated_user(sdk_key);
    bucketing::generate_bucketed_config(sdk_key.to_string(), populated_user, client_custom_data)
        .await
}

pub async fn variable_for_user(
    sdk_key: &str,
    user: user::PopulatedUser,
    variable_key: &str,
    variable_type: &str,
    client_custom_data: HashMap<String, serde_json::Value>,
) -> Result<VariableForUserResult, DevCycleError> {
    bucketing::variable_for_user(
        sdk_key,
        user,
        variable_key,
        variable_type,
        client_custom_data,
    )
    .await
}

pub async fn init_event_queue(
    sdk_key: &str,
    event_queue_options: EventQueueOptions,
) -> Result<(), DevCycleError> {
    let eq = events::event_queue::EventQueue::new(sdk_key.to_string(), event_queue_options);
    match eq {
        Ok(queue) => {
            events::event_queue_manager::set_event_queue(sdk_key, queue);
            Ok(())
        }
        Err(e) => Err(e),
    }
}

pub async fn set_client_custom_data(
    sdk_key: &str,
    client_custom_data: HashMap<String, serde_json::Value>,
) -> Result<(), DevCycleError> {
    if config::client_custom_data::set_client_custom_data(sdk_key.to_string(), client_custom_data)
        .is_some()
    {
        Ok(())
    } else {
        Err(errors::failed_to_set_client_custom_data())
    }
}
