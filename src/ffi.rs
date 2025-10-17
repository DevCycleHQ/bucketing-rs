// FFI (Foreign Function Interface) bindings for C library support
#![cfg(feature = "ffi")]

use crate::errors::DevCycleError;
use crate::events::EventQueueOptions;
use crate::user::{BucketedUserConfig, PopulatedUser, User};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use std::sync::Arc;

// Opaque pointer types for C API
pub struct CBucketedUserConfig(BucketedUserConfig);
pub struct CUser(User);
pub struct CPopulatedUser(PopulatedUser);
pub struct CEventQueueOptions(EventQueueOptions);

/// Initialize event queue
/// Returns 0 on success, non-zero on error
#[no_mangle]
pub unsafe extern "C" fn devcycle_init_event_queue(
    sdk_key: *const c_char,
    options: *const CEventQueueOptions,
) -> i32 {
    if sdk_key.is_null() {
        return -1;
    }

    let sdk_key_str = match CStr::from_ptr(sdk_key).to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };

    let event_options = if options.is_null() {
        EventQueueOptions::default()
    } else {
        (*options).0.clone()
    };

    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return -3,
    };

    match runtime.block_on(crate::init_event_queue(sdk_key_str, event_options)) {
        Ok(_) => 0,
        Err(_) => -4,
    }
}

/// Generate bucketed config from populated user
/// Returns pointer to CBucketedUserConfig on success, null on error
#[no_mangle]
pub unsafe extern "C" fn devcycle_generate_bucketed_config(
    sdk_key: *const c_char,
    user: *const CPopulatedUser,
    client_custom_data_json: *const c_char,
) -> *mut CBucketedUserConfig {
    if sdk_key.is_null() || user.is_null() {
        return ptr::null_mut();
    }

    let sdk_key_str = match CStr::from_ptr(sdk_key).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let populated_user = (*user).0.clone();

    let client_custom_data: HashMap<String, serde_json::Value> =
        if client_custom_data_json.is_null() {
            HashMap::new()
        } else {
            match CStr::from_ptr(client_custom_data_json).to_str() {
                Ok(json_str) => serde_json::from_str(json_str).unwrap_or_default(),
                Err(_) => return ptr::null_mut(),
            }
        };

    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return ptr::null_mut(),
    };

    match runtime.block_on(crate::generate_bucketed_config(
        sdk_key_str,
        populated_user,
        client_custom_data,
    )) {
        Ok(config) => Box::into_raw(Box::new(CBucketedUserConfig(config))),
        Err(_) => ptr::null_mut(),
    }
}

/// Generate bucketed config from user
#[no_mangle]
pub unsafe extern "C" fn devcycle_generate_bucketed_config_from_user(
    sdk_key: *const c_char,
    user: *const CUser,
    client_custom_data_json: *const c_char,
) -> *mut CBucketedUserConfig {
    if sdk_key.is_null() || user.is_null() {
        return ptr::null_mut();
    }

    let sdk_key_str = match CStr::from_ptr(sdk_key).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let user_obj = (*user).0.clone();

    let client_custom_data: HashMap<String, serde_json::Value> =
        if client_custom_data_json.is_null() {
            HashMap::new()
        } else {
            match CStr::from_ptr(client_custom_data_json).to_str() {
                Ok(json_str) => serde_json::from_str(json_str).unwrap_or_default(),
                Err(_) => return ptr::null_mut(),
            }
        };

    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return ptr::null_mut(),
    };

    match runtime.block_on(crate::generate_bucketed_config_from_user(
        sdk_key_str,
        user_obj,
        client_custom_data,
    )) {
        Ok(config) => Box::into_raw(Box::new(CBucketedUserConfig(config))),
        Err(_) => ptr::null_mut(),
    }
}

/// Get JSON representation of bucketed config
/// Returns a C string that must be freed with devcycle_free_string
#[no_mangle]
pub unsafe extern "C" fn devcycle_bucketed_config_to_json(
    config: *const CBucketedUserConfig,
) -> *mut c_char {
    if config.is_null() {
        return ptr::null_mut();
    }

    match serde_json::to_string(&(*config).0) {
        Ok(json) => match CString::new(json) {
            Ok(c_str) => c_str.into_raw(),
            Err(_) => ptr::null_mut(),
        },
        Err(_) => ptr::null_mut(),
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
    if json.is_null() {
        return ptr::null_mut();
    }

    let json_str = match CStr::from_ptr(json).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    match serde_json::from_str::<User>(json_str) {
        Ok(user) => Box::into_raw(Box::new(CUser(user))),
        Err(_) => ptr::null_mut(),
    }
}
