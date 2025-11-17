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

// FFI bindings for C library support (exclude when building for WASM)
#[cfg(not(feature = "wasm"))]
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
use crate::config::client_custom_data::get_client_custom_data;
use crate::config::ConfigBody;

pub async fn init_sdk_key(
    sdk_key: &str,
    config_body: ConfigBody,
    event_queue_options: EventQueueOptions,
    client_custom_data: HashMap<String, serde_json::Value>,
    platform_data: PlatformData,
) -> Result<(), DevCycleError> {
    set_platform_data(sdk_key, platform_data).await;
    set_client_custom_data(sdk_key, client_custom_data).await?;
    set_config(sdk_key, config_body).await?;
    init_event_queue(sdk_key, event_queue_options).await?;

    Ok(())
}

pub async fn set_config(sdk_key: &str, config_body: ConfigBody) -> Result<(), DevCycleError> {
    Ok(configmanager::set_config(sdk_key, config_body))
}

#[cfg(feature = "protobuf")]
pub async fn set_config_from_protobuf(
    sdk_key: &str,
    proto_config: protobuf::proto::ConfigBodyProto,
) -> Result<(), DevCycleError> {
    let config_body = protobuf::convert_proto_to_config_body(proto_config)?;
    Ok(configmanager::set_config(sdk_key, config_body))
}

pub async fn generate_bucketed_config(
    sdk_key: &str,
    user: PopulatedUser,
    client_custom_data: HashMap<String, serde_json::Value>,
) -> Result<BucketedUserConfig, DevCycleError> {
    bucketing::generate_bucketed_config(sdk_key.to_string(), user, client_custom_data).await
}

pub async fn generate_bucketed_config_from_user(
    sdk_key: &str,
    user: User,
) -> Result<BucketedUserConfig, DevCycleError> {
    let populated_user = user.get_populated_user(sdk_key);
    bucketing::generate_bucketed_config(
        sdk_key.to_string(),
        populated_user,
        get_client_custom_data(sdk_key.to_string()),
    )
    .await
}

pub async fn variable_for_user(
    sdk_key: &str,
    user: PopulatedUser,
    variable_key: &str,
    variable_type: &str,
) -> Result<VariableForUserResult, DevCycleError> {
    bucketing::variable_for_user(
        sdk_key,
        user,
        variable_key,
        variable_type,
        get_client_custom_data(sdk_key.to_string()),
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
    // The insert operation always succeeds, it returns the old value if one existed
    config::client_custom_data::set_client_custom_data(sdk_key.to_string(), client_custom_data);
    Ok(())
}

pub async fn set_platform_data(sdk_key: &str, platform_data: PlatformData) {
    config::platform_data::set_platform_data(sdk_key.to_string(), platform_data);
}
