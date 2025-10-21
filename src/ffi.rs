// FFI (Foreign Function Interface) bindings for C library support
#![cfg(feature = "ffi")]

use crate::errors::DevCycleError;
use crate::events::EventQueueOptions;
use crate::user::{BucketedUserConfig, PopulatedUser, User};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use std::sync::Arc;

// Thread-local storage for the last error message
thread_local! {
    static LAST_ERROR: RefCell<Option<String>> = RefCell::new(None);
}

/// Set the last error message in thread-local storage
fn set_last_error(err: String) {
    LAST_ERROR.with(|last| {
        *last.borrow_mut() = Some(err);
    });
}

/// Clear the last error message
fn clear_last_error() {
    LAST_ERROR.with(|last| {
        *last.borrow_mut() = None;
    });
}

/// Get the last error message from FFI operations
/// Returns a C string that must be freed with devcycle_free_string
/// Returns null if there is no error
#[no_mangle]
pub unsafe extern "C" fn devcycle_get_last_error() -> *mut c_char {
    LAST_ERROR.with(|last| match &*last.borrow() {
        Some(err) => match CString::new(err.clone()) {
            Ok(c_str) => c_str.into_raw(),
            Err(_) => ptr::null_mut(),
        },
        None => ptr::null_mut(),
    })
}

// Opaque pointer types for C API
pub struct CBucketedUserConfig(BucketedUserConfig);
pub struct CUser(User);
pub struct CPopulatedUser(PopulatedUser);
pub struct CEventQueueOptions(EventQueueOptions);
pub struct CVariableForUserResult(crate::bucketing::bucketing::VariableForUserResult);

/// Initialize event queue
/// Returns 0 on success, non-zero on error
/// Call devcycle_get_last_error() to get detailed error message
#[no_mangle]
pub unsafe extern "C" fn devcycle_init_event_queue(
    sdk_key: *const c_char,
    options: *const CEventQueueOptions,
) -> i32 {
    clear_last_error();

    if sdk_key.is_null() {
        set_last_error("SDK key pointer is null".to_string());
        return -1;
    }

    let sdk_key_str = match CStr::from_ptr(sdk_key).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("Failed to convert SDK key from C string: {}", e));
            return -2;
        }
    };

    let event_options = if options.is_null() {
        EventQueueOptions::default()
    } else {
        (*options).0.clone()
    };

    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            set_last_error(format!("Failed to create Tokio runtime: {}", e));
            return -3;
        }
    };

    match runtime.block_on(crate::init_event_queue(sdk_key_str, event_options)) {
        Ok(_) => 0,
        Err(e) => {
            set_last_error(format!("Event queue initialization failed: {}", e));
            -4
        }
    }
}

/// Generate bucketed config from populated user
/// Returns pointer to CBucketedUserConfig on success, null on error
/// Call devcycle_get_last_error() to get detailed error message
#[no_mangle]
pub unsafe extern "C" fn devcycle_generate_bucketed_config(
    sdk_key: *const c_char,
    user: *const CPopulatedUser,
    client_custom_data_json: *const c_char,
) -> *mut CBucketedUserConfig {
    clear_last_error();

    if sdk_key.is_null() || user.is_null() {
        set_last_error("SDK key or user pointer is null".to_string());
        return ptr::null_mut();
    }

    let sdk_key_str = match CStr::from_ptr(sdk_key).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("Failed to convert SDK key from C string: {}", e));
            return ptr::null_mut();
        }
    };

    let populated_user = (*user).0.clone();

    let client_custom_data: HashMap<String, serde_json::Value> =
        if client_custom_data_json.is_null() {
            HashMap::new()
        } else {
            match CStr::from_ptr(client_custom_data_json).to_str() {
                Ok(json_str) => match serde_json::from_str(json_str) {
                    Ok(data) => data,
                    Err(e) => {
                        set_last_error(format!("Failed to parse client custom data JSON: {}", e));
                        return ptr::null_mut();
                    }
                },
                Err(e) => {
                    set_last_error(format!(
                        "Failed to convert client custom data from C string: {}",
                        e
                    ));
                    return ptr::null_mut();
                }
            }
        };

    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            set_last_error(format!("Failed to create Tokio runtime: {}", e));
            return ptr::null_mut();
        }
    };

    match runtime.block_on(crate::generate_bucketed_config(
        sdk_key_str,
        populated_user,
        client_custom_data,
    )) {
        Ok(config) => Box::into_raw(Box::new(CBucketedUserConfig(config))),
        Err(e) => {
            set_last_error(format!("Failed to generate bucketed config: {}", e));
            ptr::null_mut()
        }
    }
}

/// Generate bucketed config from user
/// Returns pointer to CBucketedUserConfig on success, null on error
/// Call devcycle_get_last_error() to get detailed error message
#[no_mangle]
pub unsafe extern "C" fn devcycle_generate_bucketed_config_from_user(
    sdk_key: *const c_char,
    user: *const CUser,
    client_custom_data_json: *const c_char,
) -> *mut CBucketedUserConfig {
    clear_last_error();

    if sdk_key.is_null() || user.is_null() {
        set_last_error("SDK key or user pointer is null".to_string());
        return ptr::null_mut();
    }

    let sdk_key_str = match CStr::from_ptr(sdk_key).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("Failed to convert SDK key from C string: {}", e));
            return ptr::null_mut();
        }
    };

    let user_obj = (*user).0.clone();

    // Note: client_custom_data_json is ignored here because generate_bucketed_config_from_user
    // retrieves client custom data internally. Use devcycle_set_client_custom_data first if needed.
    let _ = client_custom_data_json; // Suppress unused parameter warning

    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            set_last_error(format!("Failed to create Tokio runtime: {}", e));
            return ptr::null_mut();
        }
    };

    match runtime.block_on(crate::generate_bucketed_config_from_user(
        sdk_key_str,
        user_obj,
    )) {
        Ok(config) => Box::into_raw(Box::new(CBucketedUserConfig(config))),
        Err(e) => {
            set_last_error(format!(
                "Failed to generate bucketed config from user: {}",
                e
            ));
            ptr::null_mut()
        }
    }
}

/// Set config from JSON string
/// Returns 0 on success, non-zero on error
/// Call devcycle_get_last_error() to get detailed error message
#[no_mangle]
pub unsafe extern "C" fn devcycle_set_config(
    sdk_key: *const c_char,
    config_json: *const c_char,
) -> i32 {
    clear_last_error();

    if sdk_key.is_null() || config_json.is_null() {
        set_last_error("SDK key or config JSON pointer is null".to_string());
        return -1;
    }

    let sdk_key_str = match CStr::from_ptr(sdk_key).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("Failed to convert SDK key from C string: {}", e));
            return -2;
        }
    };

    let config_json_str = match CStr::from_ptr(config_json).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!(
                "Failed to convert config JSON from C string: {}",
                e
            ));
            return -3;
        }
    };

    // Parse the JSON into a FullConfig first, then convert to ConfigBody
    let full_config: crate::config::FullConfig = match serde_json::from_str(config_json_str) {
        Ok(config) => config,
        Err(e) => {
            set_last_error(format!(
                "Failed to parse JSON into FullConfig: {} (JSON preview: {})",
                e,
                &config_json_str.chars().take(200).collect::<String>()
            ));
            return -4;
        }
    };

    // Convert FullConfig to ConfigBody using from_full_config
    let config_body = match crate::config::ConfigBody::from_full_config(full_config) {
        Ok(body) => body,
        Err(e) => {
            set_last_error(format!("Failed to convert FullConfig to ConfigBody: {}", e));
            return -5;
        }
    };

    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            set_last_error(format!("Failed to create Tokio runtime: {}", e));
            return -6;
        }
    };

    match runtime.block_on(crate::set_config(sdk_key_str, config_body)) {
        Ok(_) => 0,
        Err(e) => {
            set_last_error(format!("Failed to set config: {}", e));
            -7
        }
    }
}

/// Set client custom data from JSON string
/// Returns 0 on success, non-zero on error
/// Call devcycle_get_last_error() to get detailed error message
#[no_mangle]
pub unsafe extern "C" fn devcycle_set_client_custom_data(
    sdk_key: *const c_char,
    custom_data_json: *const c_char,
) -> i32 {
    clear_last_error();

    if sdk_key.is_null() || custom_data_json.is_null() {
        set_last_error("SDK key or custom data JSON pointer is null".to_string());
        return -1;
    }

    let sdk_key_str = match CStr::from_ptr(sdk_key).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("Failed to convert SDK key from C string: {}", e));
            return -2;
        }
    };

    let custom_data_json_str = match CStr::from_ptr(custom_data_json).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!(
                "Failed to convert custom data JSON from C string: {}",
                e
            ));
            return -3;
        }
    };

    let custom_data: HashMap<String, serde_json::Value> =
        match serde_json::from_str(custom_data_json_str) {
            Ok(data) => data,
            Err(e) => {
                set_last_error(format!("Failed to parse custom data JSON: {}", e));
                return -4;
            }
        };

    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            set_last_error(format!("Failed to create Tokio runtime: {}", e));
            return -5;
        }
    };

    match runtime.block_on(crate::set_client_custom_data(sdk_key_str, custom_data)) {
        Ok(_) => 0,
        Err(e) => {
            set_last_error(format!("Failed to set client custom data: {}", e));
            -6
        }
    }
}

/// Set platform data from JSON string
/// Returns 0 on success, non-zero on error
/// Call devcycle_get_last_error() to get detailed error message
#[no_mangle]
pub unsafe extern "C" fn devcycle_set_platform_data(
    sdk_key: *const c_char,
    platform_data_json: *const c_char,
) -> i32 {
    clear_last_error();

    if sdk_key.is_null() || platform_data_json.is_null() {
        set_last_error("SDK key or platform data JSON pointer is null".to_string());
        return -1;
    }

    let sdk_key_str = match CStr::from_ptr(sdk_key).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("Failed to convert SDK key from C string: {}", e));
            return -2;
        }
    };

    let platform_data_json_str = match CStr::from_ptr(platform_data_json).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!(
                "Failed to convert platform data JSON from C string: {}",
                e
            ));
            return -3;
        }
    };

    let platform_data: crate::PlatformData = match serde_json::from_str(platform_data_json_str) {
        Ok(data) => data,
        Err(e) => {
            set_last_error(format!("Failed to parse platform data JSON: {}", e));
            return -4;
        }
    };

    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            set_last_error(format!("Failed to create Tokio runtime: {}", e));
            return -5;
        }
    };

    runtime.block_on(crate::set_platform_data(sdk_key_str, platform_data));
    0
}

/// Get variable value for a user
/// Returns pointer to CVariableForUserResult on success, null on error
/// Call devcycle_get_last_error() to get detailed error message
#[no_mangle]
pub unsafe extern "C" fn devcycle_variable_for_user(
    sdk_key: *const c_char,
    user: *const CPopulatedUser,
    variable_key: *const c_char,
    variable_type: *const c_char,
) -> *mut CVariableForUserResult {
    clear_last_error();

    if sdk_key.is_null() || user.is_null() || variable_key.is_null() || variable_type.is_null() {
        set_last_error("One or more required pointers are null".to_string());
        return ptr::null_mut();
    }

    let sdk_key_str = match CStr::from_ptr(sdk_key).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("Failed to convert SDK key from C string: {}", e));
            return ptr::null_mut();
        }
    };

    let variable_key_str = match CStr::from_ptr(variable_key).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!(
                "Failed to convert variable key from C string: {}",
                e
            ));
            return ptr::null_mut();
        }
    };

    let variable_type_str = match CStr::from_ptr(variable_type).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!(
                "Failed to convert variable type from C string: {}",
                e
            ));
            return ptr::null_mut();
        }
    };

    let populated_user = (*user).0.clone();

    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            set_last_error(format!("Failed to create Tokio runtime: {}", e));
            return ptr::null_mut();
        }
    };

    match runtime.block_on(crate::variable_for_user(
        sdk_key_str,
        populated_user,
        variable_key_str,
        variable_type_str,
    )) {
        Ok(result) => Box::into_raw(Box::new(CVariableForUserResult(result))),
        Err(e) => {
            set_last_error(format!("Failed to get variable for user: {}", e));
            ptr::null_mut()
        }
    }
}

/// Get JSON representation of variable for user result
/// Returns a C string that must be freed with devcycle_free_string
#[no_mangle]
pub unsafe extern "C" fn devcycle_variable_result_to_json(
    result: *const CVariableForUserResult,
) -> *mut c_char {
    if result.is_null() {
        return ptr::null_mut();
    }

    match serde_json::to_string(&(*result).0.variable_value) {
        Ok(json) => match CString::new(json) {
            Ok(c_str) => c_str.into_raw(),
            Err(_) => ptr::null_mut(),
        },
        Err(_) => ptr::null_mut(),
    }
}

/// Get variable type from result
/// Returns a C string that must be freed with devcycle_free_string
#[no_mangle]
pub unsafe extern "C" fn devcycle_variable_result_get_type(
    result: *const CVariableForUserResult,
) -> *mut c_char {
    if result.is_null() {
        return ptr::null_mut();
    }

    match CString::new((*result).0.variable_type.clone()) {
        Ok(c_str) => c_str.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// Get JSON representation of bucketed config
/// Returns a C string that must be freed with devcycle_free_string
#[no_mangle]
pub unsafe extern "C" fn devcycle_bucketed_config_to_json(
    config: *const CBucketedUserConfig,
) -> *mut c_char {
    clear_last_error();

    if config.is_null() {
        set_last_error("Bucketed config pointer is null".to_string());
        return ptr::null_mut();
    }

    match serde_json::to_string(&(*config).0) {
        Ok(json) => match CString::new(json) {
            Ok(c_str) => c_str.into_raw(),
            Err(e) => {
                set_last_error(format!("Failed to convert JSON to C string: {}", e));
                ptr::null_mut()
            }
        },
        Err(e) => {
            set_last_error(format!(
                "Failed to serialize bucketed config to JSON: {}",
                e
            ));
            ptr::null_mut()
        }
    }
}

/// Free a variable for user result
#[no_mangle]
pub unsafe extern "C" fn devcycle_free_variable_result(result: *mut CVariableForUserResult) {
    if !result.is_null() {
        let _ = Box::from_raw(result);
    }
}

/// Free a C string returned by this library
#[no_mangle]
pub unsafe extern "C" fn devcycle_free_string(s: *mut c_char) {
    if !s.is_null() {
        let _ = CString::from_raw(s);
    }
}

/// Free a bucketed config
#[no_mangle]
pub unsafe extern "C" fn devcycle_free_bucketed_config(config: *mut CBucketedUserConfig) {
    if !config.is_null() {
        let _ = Box::from_raw(config);
    }
}

/// Free a user
#[no_mangle]
pub unsafe extern "C" fn devcycle_free_user(user: *mut CUser) {
    if !user.is_null() {
        let _ = Box::from_raw(user);
    }
}

/// Free a populated user
#[no_mangle]
pub unsafe extern "C" fn devcycle_free_populated_user(user: *mut CPopulatedUser) {
    if !user.is_null() {
        let _ = Box::from_raw(user);
    }
}

/// Free event queue options
#[no_mangle]
pub unsafe extern "C" fn devcycle_free_event_queue_options(options: *mut CEventQueueOptions) {
    if !options.is_null() {
        let _ = Box::from_raw(options);
    }
}

/// Create user from JSON string
#[no_mangle]
pub unsafe extern "C" fn devcycle_user_from_json(json: *const c_char) -> *mut CUser {
    clear_last_error();

    if json.is_null() {
        set_last_error("JSON pointer is null".to_string());
        return ptr::null_mut();
    }

    let json_str = match CStr::from_ptr(json).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("Failed to convert JSON from C string: {}", e));
            return ptr::null_mut();
        }
    };

    match serde_json::from_str::<User>(json_str) {
        Ok(user) => Box::into_raw(Box::new(CUser(user))),
        Err(e) => {
            set_last_error(format!("Failed to parse user JSON: {}", e));
            ptr::null_mut()
        }
    }
}
