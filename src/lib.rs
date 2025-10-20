use crate::errors::DevCycleError;
use crate::user::{BucketedUserConfig, PopulatedUser, User};
use std::collections::HashMap;

// Module declarations - now organized with mod.rs files in each folder
pub mod bucketing;
pub mod config;
pub mod events;
pub mod segmentation;
pub mod user;
pub mod util;

// FFI bindings for C library support
#[cfg(feature = "ffi")]
pub mod ffi;

// WASM bindings for WebAssembly support
#[cfg(feature = "wasm")]
pub mod wasm;

// Re-export commonly used items from submodules
pub use config::configmanager;
pub use config::feature;
pub use events::event;
pub use segmentation::filters;
pub use segmentation::target;
pub use segmentation::versioncompare;
pub use user::platform_data;
pub use util::constants;
pub use util::errors;
pub use util::murmurhash;

// Export platform data types and functions for external SDKs
pub use platform_data::{get_platform_data, set_platform_data, PlatformData};

// Export evaluation reason types for external use
use crate::events::EventQueueOptions;
pub use event::{DefaultReason, EvalDetails, EvaluationReason};

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

pub async unsafe fn init_event_queue(
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
