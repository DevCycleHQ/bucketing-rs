// FFI (Foreign Function Interface) bindings for C library support
#![cfg(feature = "ffi")]

use crate::events::EventQueueOptions;
use crate::user::{BucketedUserConfig, PopulatedUser, User};
use once_cell::sync::Lazy;
#[cfg(feature = "protobuf")]
use prost::Message;
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

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DevCycleFFIErrorCode {
    Success = 0,
    NullPointer = -1,
    SdkKeyConversionFailed = -2,
    InputStringConversionFailed = -3,
    JsonParseFailed = -4,
    ConfigBodyConversionFailed = -5,
    RuntimeUnavailable = -6,
    OperationFailed = -7,
    /// Reserved for future use (was -8)
    Reserved8 = -8,
    /// Reserved for future use (was -9)
    Reserved9 = -9,
    ProtobufDecodeFailed = -10,
    /// Reserved for future use (was -11)
    Reserved11 = -11,
    /// Reserved for future use (was -12)
    Reserved12 = -12,
    EventQueueInitFailed = -13,
    InitSdkKeyFailed = -14,
}

thread_local! {
    static LAST_ERROR_CODE: RefCell<DevCycleFFIErrorCode> = RefCell::new(DevCycleFFIErrorCode::Success);
}

/// Set the last error message in thread-local storage
fn set_last_error(err: String) {
    LAST_ERROR.with(|last| {
        *last.borrow_mut() = Some(err);
    });
}

fn set_last_error_code(code: DevCycleFFIErrorCode) {
    LAST_ERROR_CODE.with(|c| *c.borrow_mut() = code);
}

/// Clear last error (message & code)
fn clear_last_error() {
    LAST_ERROR.with(|last| *last.borrow_mut() = None);
    set_last_error_code(DevCycleFFIErrorCode::Success);
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

#[no_mangle]
pub unsafe extern "C" fn devcycle_get_last_error_code() -> DevCycleFFIErrorCode {
    LAST_ERROR_CODE.with(|c| *c.borrow())
}

#[no_mangle]
pub unsafe extern "C" fn devcycle_error_code_to_string(code: DevCycleFFIErrorCode) -> *mut c_char {
    let s = match code {
        DevCycleFFIErrorCode::Success => "Success",
        DevCycleFFIErrorCode::NullPointer => "NullPointer",
        DevCycleFFIErrorCode::SdkKeyConversionFailed => "SdkKeyConversionFailed",
        DevCycleFFIErrorCode::InputStringConversionFailed => "InputStringConversionFailed",
        DevCycleFFIErrorCode::JsonParseFailed => "JsonParseFailed",
        DevCycleFFIErrorCode::ConfigBodyConversionFailed => "ConfigBodyConversionFailed",
        DevCycleFFIErrorCode::RuntimeUnavailable => "RuntimeUnavailable",
        DevCycleFFIErrorCode::OperationFailed => "OperationFailed",
        DevCycleFFIErrorCode::ProtobufDecodeFailed => "ProtobufDecodeFailed",
        DevCycleFFIErrorCode::EventQueueInitFailed => "EventQueueInitFailed",
        DevCycleFFIErrorCode::InitSdkKeyFailed => "InitSdkKeyFailed",
    };
    match CString::new(s) {
        Ok(c_str) => c_str.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

// Opaque pointer types for C API
pub struct CBucketedUserConfig(BucketedUserConfig);
pub struct CUser(User);
pub struct CPopulatedUser(PopulatedUser);
pub struct CEventQueueOptions(EventQueueOptions);
pub struct CVariableForUserResult(crate::bucketing::bucketing::VariableForUserResult);

static TOKIO_RUNTIME: Lazy<Result<tokio::runtime::Runtime, String>> =
    Lazy::new(|| match tokio::runtime::Runtime::new() {
        Ok(rt) => Ok(rt),
        Err(e) => Err(format!("Failed to create Tokio runtime: {}", e)),
    });

// Tolerant user JSON parsing: accepts either full User JSON or minimal {"userId":"..."} / {"user_id":"..."}
fn parse_user_json_tolerant(json_str: &str) -> Result<User, DevCycleFFIErrorCode> {
    // Try full deserialization first
    if let Ok(user) = serde_json::from_str::<User>(json_str) {
        return Ok(user);
    }
    let value: serde_json::Value =
        serde_json::from_str(json_str).map_err(|_| DevCycleFFIErrorCode::JsonParseFailed)?;
    let obj = value
        .as_object()
        .ok_or(DevCycleFFIErrorCode::JsonParseFailed)?;
    let user_id = obj
        .get("userId")
        .or_else(|| obj.get("user_id"))
        .and_then(|v| v.as_str())
        .ok_or(DevCycleFFIErrorCode::JsonParseFailed)?;
    use chrono::Utc;
    Ok(User {
        user_id: user_id.to_string(),
        email: obj
            .get("email")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        name: obj
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        language: obj
            .get("language")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        country: obj
            .get("country")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        app_version: obj
            .get("appVersion")
            .or_else(|| obj.get("app_version"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        app_build: obj
            .get("appBuild")
            .or_else(|| obj.get("app_build"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        custom_data: obj
            .get("customData")
            .or_else(|| obj.get("custom_data"))
            .and_then(|v| v.as_object())
            .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default(),
        private_custom_data: obj
            .get("privateCustomData")
            .or_else(|| obj.get("private_custom_data"))
            .and_then(|v| v.as_object())
            .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default(),
        device_model: obj
            .get("deviceModel")
            .or_else(|| obj.get("device_model"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        last_seen_date: Utc::now(),
    })
}

// Helper to get runtime or set last error and return error code/null
fn get_runtime_or_set_error() -> Option<&'static tokio::runtime::Runtime> {
    match &*TOKIO_RUNTIME {
        Ok(rt) => Some(rt),
        Err(e) => {
            set_last_error(e.clone());
            None
        }
    }
}

/// Set error message and code
fn set_error(msg: String, code: DevCycleFFIErrorCode) {
    set_last_error(msg);
    set_last_error_code(code);
}

// Helper to parse SDK key C string and set appropriate errors.
// Returns owned String so caller can safely use &str without lifetime issues.
unsafe fn parse_sdk_key(sdk_key: *const c_char) -> Result<String, DevCycleFFIErrorCode> {
    if sdk_key.is_null() {
        set_error(
            "SDK key pointer is null".to_string(),
            DevCycleFFIErrorCode::NullPointer,
        );
        return Err(DevCycleFFIErrorCode::NullPointer);
    }
    match CStr::from_ptr(sdk_key).to_str() {
        Ok(s) => Ok(s.to_string()),
        Err(e) => {
            set_error(
                format!("Failed to convert SDK key from C string: {}", e),
                DevCycleFFIErrorCode::SdkKeyConversionFailed,
            );
            Err(DevCycleFFIErrorCode::SdkKeyConversionFailed)
        }
    }
}

/// Initialize event queue
/// Returns 0 on success, non-zero on error
/// Call devcycle_get_last_error() to get detailed error message
#[no_mangle]
pub unsafe extern "C" fn devcycle_init_event_queue(
    sdk_key: *const c_char,
    options: *const CEventQueueOptions,
) -> i32 {
    clear_last_error();

    let sdk_key_str = match parse_sdk_key(sdk_key) {
        Ok(s) => s,
        Err(code) => return code as i32,
    };

    let event_options = if options.is_null() {
        EventQueueOptions::default()
    } else {
        (*options).0.clone()
    };

    // Replace runtime creation
    let runtime = match get_runtime_or_set_error() {
        Some(rt) => rt,
        None => {
            set_error(
                "Runtime unavailable".to_string(),
                DevCycleFFIErrorCode::RuntimeUnavailable,
            );
            return DevCycleFFIErrorCode::RuntimeUnavailable as i32;
        }
    };

    match runtime.block_on(crate::init_event_queue(&sdk_key_str, event_options)) {
        Ok(_) => {
            set_last_error_code(DevCycleFFIErrorCode::Success);
            0
        }
        Err(e) => {
            set_error(
                format!("Event queue initialization failed: {}", e),
                DevCycleFFIErrorCode::EventQueueInitFailed,
            );
            DevCycleFFIErrorCode::EventQueueInitFailed as i32
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

    if user.is_null() {
        set_error(
            "User pointer is null".to_string(),
            DevCycleFFIErrorCode::NullPointer,
        );
        return ptr::null_mut();
    }

    let sdk_key_str = match parse_sdk_key(sdk_key) {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
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
                        set_error(
                            format!("Failed to parse client custom data JSON: {}", e),
                            DevCycleFFIErrorCode::JsonParseFailed,
                        );
                        return ptr::null_mut();
                    }
                },
                Err(e) => {
                    set_error(
                        format!("Failed to convert client custom data from C string: {}", e),
                        DevCycleFFIErrorCode::InputStringConversionFailed,
                    );
                    return ptr::null_mut();
                }
            }
        };

    let runtime = match get_runtime_or_set_error() {
        Some(rt) => rt,
        None => {
            set_error(
                "Runtime unavailable".to_string(),
                DevCycleFFIErrorCode::RuntimeUnavailable,
            );
            return ptr::null_mut();
        }
    };

    match runtime.block_on(crate::generate_bucketed_config(
        &sdk_key_str,
        populated_user,
        client_custom_data,
    )) {
        Ok(config) => Box::into_raw(Box::new(CBucketedUserConfig(config))),
        Err(e) => {
            set_error(
                format!("Failed to generate bucketed config: {}", e),
                DevCycleFFIErrorCode::OperationFailed,
            );
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

    if user.is_null() {
        set_error(
            "User pointer is null".to_string(),
            DevCycleFFIErrorCode::NullPointer,
        );
        return ptr::null_mut();
    }

    let sdk_key_str = match parse_sdk_key(sdk_key) {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let user_obj = (*user).0.clone();

    let _ = client_custom_data_json; // ignored

    let runtime = match get_runtime_or_set_error() {
        Some(rt) => rt,
        None => {
            set_error(
                "Runtime unavailable".to_string(),
                DevCycleFFIErrorCode::RuntimeUnavailable,
            );
            return ptr::null_mut();
        }
    };

    match runtime.block_on(crate::generate_bucketed_config_from_user(
        &sdk_key_str,
        user_obj,
    )) {
        Ok(config) => Box::into_raw(Box::new(CBucketedUserConfig(config))),
        Err(e) => {
            set_error(
                format!("Failed to generate bucketed config from user: {}", e),
                DevCycleFFIErrorCode::OperationFailed,
            );
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

    if config_json.is_null() {
        set_error(
            "Config JSON pointer is null".to_string(),
            DevCycleFFIErrorCode::NullPointer,
        );
        return DevCycleFFIErrorCode::NullPointer as i32;
    }

    let sdk_key_str = match parse_sdk_key(sdk_key) {
        Ok(s) => s,
        Err(code) => return code as i32,
    };

    let config_json_str = match CStr::from_ptr(config_json).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(
                format!("Failed to convert config JSON from C string: {}", e),
                DevCycleFFIErrorCode::InputStringConversionFailed,
            );
            return DevCycleFFIErrorCode::InputStringConversionFailed as i32;
        }
    };

    // Parse the JSON into a FullConfig first, then convert to ConfigBody
    let full_config: crate::config::FullConfig = match serde_json::from_str(config_json_str) {
        Ok(config) => config,
        Err(e) => {
            set_error(
                format!("Failed to parse JSON into FullConfig: {}", e),
                DevCycleFFIErrorCode::JsonParseFailed,
            );
            return DevCycleFFIErrorCode::JsonParseFailed as i32;
        }
    };

    // Convert FullConfig to ConfigBody using from_full_config
    let config_body = match crate::config::ConfigBody::from_full_config(full_config) {
        Ok(body) => body,
        Err(e) => {
            set_error(
                format!("Failed to convert FullConfig to ConfigBody: {}", e),
                DevCycleFFIErrorCode::ConfigBodyConversionFailed,
            );
            return DevCycleFFIErrorCode::ConfigBodyConversionFailed as i32;
        }
    };

    let runtime = match get_runtime_or_set_error() {
        Some(rt) => rt,
        None => {
            set_error(
                "Runtime unavailable".to_string(),
                DevCycleFFIErrorCode::RuntimeUnavailable,
            );
            return DevCycleFFIErrorCode::RuntimeUnavailable as i32;
        }
    };

    match runtime.block_on(crate::set_config(&sdk_key_str, config_body)) {
        Ok(_) => {
            set_last_error_code(DevCycleFFIErrorCode::Success);
            0
        }
        Err(e) => {
            set_error(
                format!("Failed to set config: {}", e),
                DevCycleFFIErrorCode::OperationFailed,
            );
            DevCycleFFIErrorCode::OperationFailed as i32
        }
    }
}

/// Set config from protobuf bytes (serialized ConfigBodyProto)
/// Returns 0 on success, non-zero on error
/// Call devcycle_get_last_error() to get detailed error message
#[cfg(feature = "protobuf")]
#[no_mangle]
pub unsafe extern "C" fn devcycle_set_config_from_protobuf(
    sdk_key: *const c_char,
    proto_bytes: *const u8,
    len: usize,
) -> i32 {
    clear_last_error();

    if proto_bytes.is_null() {
        set_error(
            "Protobuf bytes pointer is null".to_string(),
            DevCycleFFIErrorCode::NullPointer,
        );
        return DevCycleFFIErrorCode::NullPointer as i32;
    }

    let sdk_key_str = match parse_sdk_key(sdk_key) {
        Ok(s) => s,
        Err(code) => return code as i32,
    };

    // Safety: we trust len provided by caller; create slice from raw pointer
    let bytes_slice = std::slice::from_raw_parts(proto_bytes, len);

    let proto_msg = match crate::protobuf::proto::ConfigBodyProto::decode(bytes_slice) {
        Ok(m) => m,
        Err(e) => {
            set_error(
                format!("Failed to decode protobuf bytes: {}", e),
                DevCycleFFIErrorCode::ProtobufDecodeFailed,
            );
            return DevCycleFFIErrorCode::ProtobufDecodeFailed as i32;
        }
    };

    let runtime = match get_runtime_or_set_error() {
        Some(rt) => rt,
        None => {
            set_error(
                "Runtime unavailable".to_string(),
                DevCycleFFIErrorCode::RuntimeUnavailable,
            );
            return DevCycleFFIErrorCode::RuntimeUnavailable as i32;
        }
    };

    match runtime.block_on(crate::set_config_from_protobuf(&sdk_key_str, proto_msg)) {
        Ok(_) => {
            set_last_error_code(DevCycleFFIErrorCode::Success);
            0
        }
        Err(e) => {
            set_error(
                format!("Failed to set config from protobuf: {}", e),
                DevCycleFFIErrorCode::OperationFailed,
            );
            DevCycleFFIErrorCode::OperationFailed as i32
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

    if custom_data_json.is_null() {
        set_error(
            "Custom data JSON pointer is null".to_string(),
            DevCycleFFIErrorCode::NullPointer,
        );
        return DevCycleFFIErrorCode::NullPointer as i32;
    }

    let sdk_key_str = match parse_sdk_key(sdk_key) {
        Ok(s) => s,
        Err(code) => return code as i32,
    };

    let custom_data_json_str = match CStr::from_ptr(custom_data_json).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(
                format!("Failed to convert custom data JSON from C string: {}", e),
                DevCycleFFIErrorCode::InputStringConversionFailed,
            );
            return DevCycleFFIErrorCode::InputStringConversionFailed as i32;
        }
    };

    let custom_data: HashMap<String, serde_json::Value> =
        match serde_json::from_str(custom_data_json_str) {
            Ok(data) => data,
            Err(e) => {
                set_error(
                    format!("Failed to parse custom data JSON: {}", e),
                    DevCycleFFIErrorCode::JsonParseFailed,
                );
                return DevCycleFFIErrorCode::JsonParseFailed as i32;
            }
        };

    let runtime = match get_runtime_or_set_error() {
        Some(rt) => rt,
        None => {
            set_error(
                "Runtime unavailable".to_string(),
                DevCycleFFIErrorCode::RuntimeUnavailable,
            );
            return DevCycleFFIErrorCode::RuntimeUnavailable as i32;
        }
    };

    match runtime.block_on(crate::set_client_custom_data(&sdk_key_str, custom_data)) {
        Ok(_) => {
            set_last_error_code(DevCycleFFIErrorCode::Success);
            0
        }
        Err(e) => {
            set_error(
                format!("Failed to set client custom data: {}", e),
                DevCycleFFIErrorCode::OperationFailed,
            );
            DevCycleFFIErrorCode::OperationFailed as i32
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

    let sdk_key_str = match parse_sdk_key(sdk_key) {
        Ok(s) => s,
        Err(code) => return code as i32,
    };

    if platform_data_json.is_null() {
        set_error(
            "Platform data JSON pointer is null".to_string(),
            DevCycleFFIErrorCode::NullPointer,
        );
        return DevCycleFFIErrorCode::NullPointer as i32;
    }

    let platform_data_json_str = match CStr::from_ptr(platform_data_json).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(
                format!("Failed to convert platform data JSON from C string: {}", e),
                DevCycleFFIErrorCode::InputStringConversionFailed,
            );
            return DevCycleFFIErrorCode::InputStringConversionFailed as i32;
        }
    };

    let platform_data: crate::PlatformData = match serde_json::from_str(platform_data_json_str) {
        Ok(data) => data,
        Err(e) => {
            set_error(
                format!("Failed to parse platform data JSON: {}", e),
                DevCycleFFIErrorCode::JsonParseFailed,
            );
            return DevCycleFFIErrorCode::JsonParseFailed as i32;
        }
    };

    let runtime = match get_runtime_or_set_error() {
        Some(rt) => rt,
        None => {
            set_error(
                "Runtime unavailable".to_string(),
                DevCycleFFIErrorCode::RuntimeUnavailable,
            );
            return DevCycleFFIErrorCode::RuntimeUnavailable as i32;
        }
    };

    runtime.block_on(crate::set_platform_data(&sdk_key_str, platform_data));
    set_last_error_code(DevCycleFFIErrorCode::Success);
    0
}

/// Initialize SDK key with config, event queue options, client custom data, and platform data
/// This is a convenience function that calls set_config, init_event_queue, set_client_custom_data, and set_platform_data
/// Returns 0 on success, non-zero on error
/// Call devcycle_get_last_error() to get detailed error message
#[no_mangle]
pub unsafe extern "C" fn devcycle_init_sdk_key(
    sdk_key: *const c_char,
    config_json: *const c_char,
    event_queue_options: *const CEventQueueOptions,
    client_custom_data_json: *const c_char,
    platform_data_json: *const c_char,
) -> i32 {
    clear_last_error();

    if config_json.is_null() {
        set_error(
            "Config JSON pointer is null".to_string(),
            DevCycleFFIErrorCode::NullPointer,
        );
        return DevCycleFFIErrorCode::NullPointer as i32;
    }

    let sdk_key_str = match parse_sdk_key(sdk_key) {
        Ok(s) => s,
        Err(code) => return code as i32,
    };

    let config_json_str = match CStr::from_ptr(config_json).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(
                format!("Failed to convert config JSON from C string: {}", e),
                DevCycleFFIErrorCode::InputStringConversionFailed,
            );
            return DevCycleFFIErrorCode::InputStringConversionFailed as i32;
        }
    };

    // Parse platform data
    let platform_data: crate::PlatformData = if platform_data_json.is_null() {
        crate::PlatformData::generate()
    } else {
        match CStr::from_ptr(platform_data_json).to_str() {
            Ok(json_str) => match serde_json::from_str(json_str) {
                Ok(data) => data,
                Err(e) => {
                    set_error(
                        format!("Failed to parse platform data JSON: {}", e),
                        DevCycleFFIErrorCode::JsonParseFailed,
                    );
                    return DevCycleFFIErrorCode::JsonParseFailed as i32;
                }
            },
            Err(e) => {
                set_error(
                    format!("Failed to convert platform data from C string: {}", e),
                    DevCycleFFIErrorCode::InputStringConversionFailed,
                );
                return DevCycleFFIErrorCode::InputStringConversionFailed as i32;
            }
        }
    };

    // Parse the JSON into a FullConfig first, then convert to ConfigBody
    let full_config: crate::config::FullConfig = match serde_json::from_str(config_json_str) {
        Ok(config) => config,
        Err(e) => {
            set_error(
                format!("Failed to parse JSON into FullConfig: {}", e),
                DevCycleFFIErrorCode::JsonParseFailed,
            );
            return DevCycleFFIErrorCode::JsonParseFailed as i32;
        }
    };

    // Convert FullConfig to ConfigBody using from_full_config
    let config_body = match crate::config::ConfigBody::from_full_config(full_config) {
        Ok(body) => body,
        Err(e) => {
            set_error(
                format!("Failed to convert FullConfig to ConfigBody: {}", e),
                DevCycleFFIErrorCode::ConfigBodyConversionFailed,
            );
            return DevCycleFFIErrorCode::ConfigBodyConversionFailed as i32;
        }
    };

    // Parse event queue options
    let event_options = if event_queue_options.is_null() {
        crate::EventQueueOptions::default()
    } else {
        (*event_queue_options).0.clone()
    };

    // Parse client custom data
    let client_custom_data: HashMap<String, serde_json::Value> =
        if client_custom_data_json.is_null() {
            HashMap::new()
        } else {
            match CStr::from_ptr(client_custom_data_json).to_str() {
                Ok(json_str) => match serde_json::from_str(json_str) {
                    Ok(data) => data,
                    Err(e) => {
                        set_error(
                            format!("Failed to parse client custom data JSON: {}", e),
                            DevCycleFFIErrorCode::JsonParseFailed,
                        );
                        return DevCycleFFIErrorCode::JsonParseFailed as i32;
                    }
                },
                Err(e) => {
                    set_error(
                        format!("Failed to convert client custom data from C string: {}", e),
                        DevCycleFFIErrorCode::InputStringConversionFailed,
                    );
                    return DevCycleFFIErrorCode::InputStringConversionFailed as i32;
                }
            }
        };

    let runtime = match get_runtime_or_set_error() {
        Some(rt) => rt,
        None => {
            set_error(
                "Runtime unavailable".to_string(),
                DevCycleFFIErrorCode::RuntimeUnavailable,
            );
            return DevCycleFFIErrorCode::RuntimeUnavailable as i32;
        }
    };

    match runtime.block_on(crate::init_sdk_key(
        &sdk_key_str,
        config_body,
        event_options,
        client_custom_data,
        platform_data,
    )) {
        Ok(_) => {
            set_last_error_code(DevCycleFFIErrorCode::Success);
            0
        }
        Err(e) => {
            set_error(
                format!("Failed to initialize SDK key: {}", e),
                DevCycleFFIErrorCode::InitSdkKeyFailed,
            );
            DevCycleFFIErrorCode::InitSdkKeyFailed as i32
        }
    }
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
    if user.is_null() || variable_key.is_null() || variable_type.is_null() {
        set_error(
            "User, variable key, or variable type pointer is null".to_string(),
            DevCycleFFIErrorCode::NullPointer,
        );
        return ptr::null_mut();
    }
    let sdk_key_str = match parse_sdk_key(sdk_key) {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    let variable_key_str = match CStr::from_ptr(variable_key).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(
                format!("Failed to convert variable key from C string: {}", e),
                DevCycleFFIErrorCode::InputStringConversionFailed,
            );
            return ptr::null_mut();
        }
    };
    let variable_type_str = match CStr::from_ptr(variable_type).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(
                format!("Failed to convert variable type from C string: {}", e),
                DevCycleFFIErrorCode::InputStringConversionFailed,
            );
            return ptr::null_mut();
        }
    };
    let populated_user = (*user).0.clone();
    let runtime = match get_runtime_or_set_error() {
        Some(rt) => rt,
        None => {
            set_error(
                "Runtime unavailable".to_string(),
                DevCycleFFIErrorCode::RuntimeUnavailable,
            );
            return ptr::null_mut();
        }
    };
    match runtime.block_on(crate::variable_for_user(
        &sdk_key_str,
        populated_user,
        variable_key_str,
        variable_type_str,
    )) {
        Ok(result) => Box::into_raw(Box::new(CVariableForUserResult(result))),
        Err(e) => {
            set_error(
                format!("Failed to get variable for user: {}", e),
                DevCycleFFIErrorCode::OperationFailed,
            );
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
    clear_last_error();
    if result.is_null() {
        set_error(
            "Variable result pointer is null".to_string(),
            DevCycleFFIErrorCode::NullPointer,
        );
        return ptr::null_mut();
    }
    match serde_json::to_string(&(*result).0.variable_value) {
        Ok(json) => match CString::new(json) {
            Ok(c_str) => {
                set_last_error_code(DevCycleFFIErrorCode::Success);
                c_str.into_raw()
            }
            Err(e) => {
                set_error(
                    format!("Failed to build CString: {}", e),
                    DevCycleFFIErrorCode::OperationFailed,
                );
                ptr::null_mut()
            }
        },
        Err(e) => {
            set_error(
                format!("Failed to serialize variable value JSON: {}", e),
                DevCycleFFIErrorCode::OperationFailed,
            );
            ptr::null_mut()
        }
    }
}

/// Get variable type from result
/// Returns a C string that must be freed with devcycle_free_string
#[no_mangle]
pub unsafe extern "C" fn devcycle_variable_result_get_type(
    result: *const CVariableForUserResult,
) -> *mut c_char {
    clear_last_error();
    if result.is_null() {
        set_error(
            "Variable result pointer is null".to_string(),
            DevCycleFFIErrorCode::NullPointer,
        );
        return ptr::null_mut();
    }
    match CString::new((*result).0.variable_type.clone()) {
        Ok(c_str) => {
            set_last_error_code(DevCycleFFIErrorCode::Success);
            c_str.into_raw()
        }
        Err(e) => {
            set_error(
                format!("Failed to build CString: {}", e),
                DevCycleFFIErrorCode::OperationFailed,
            );
            ptr::null_mut()
        }
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
        set_error(
            "Bucketed config pointer is null".to_string(),
            DevCycleFFIErrorCode::NullPointer,
        );
        return ptr::null_mut();
    }
    match serde_json::to_string(&(*config).0) {
        Ok(json) => match CString::new(json) {
            Ok(c_str) => {
                set_last_error_code(DevCycleFFIErrorCode::Success);
                c_str.into_raw()
            }
            Err(e) => {
                set_error(
                    format!("Failed to convert JSON to C string: {}", e),
                    DevCycleFFIErrorCode::OperationFailed,
                );
                ptr::null_mut()
            }
        },
        Err(e) => {
            set_error(
                format!("Failed to serialize bucketed config to JSON: {}", e),
                DevCycleFFIErrorCode::OperationFailed,
            );
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
        set_error(
            "JSON pointer is null".to_string(),
            DevCycleFFIErrorCode::NullPointer,
        );
        return ptr::null_mut();
    }
    let json_str = match CStr::from_ptr(json).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(
                format!("Failed to convert JSON from C string: {}", e),
                DevCycleFFIErrorCode::InputStringConversionFailed,
            );
            return ptr::null_mut();
        }
    };
    match parse_user_json_tolerant(json_str) {
        Ok(user) => {
            set_last_error_code(DevCycleFFIErrorCode::Success);
            Box::into_raw(Box::new(CUser(user)))
        }
        Err(code) => {
            if code == DevCycleFFIErrorCode::JsonParseFailed {
                set_error(
                    "Failed to parse user JSON (missing required userId)".to_string(),
                    DevCycleFFIErrorCode::JsonParseFailed,
                );
            }
            ptr::null_mut()
        }
    }
}

/// Create a populated user from a user and sdk key
/// Returns pointer to CPopulatedUser on success, null on error
#[no_mangle]
pub unsafe extern "C" fn devcycle_populate_user(
    sdk_key: *const c_char,
    user: *const CUser,
) -> *mut CPopulatedUser {
    clear_last_error();
    if user.is_null() {
        set_error(
            "User pointer is null".to_string(),
            DevCycleFFIErrorCode::NullPointer,
        );
        return ptr::null_mut();
    }
    let sdk_key_str = match parse_sdk_key(sdk_key) {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    let user_obj = (*user).0.clone();
    let populated = user_obj.get_populated_user(&sdk_key_str);
    set_last_error_code(DevCycleFFIErrorCode::Success);
    Box::into_raw(Box::new(CPopulatedUser(populated)))
}

/// Merge client custom data JSON into an existing populated user (adds keys if not already present)
/// Returns 0 on success, non-zero on error
#[no_mangle]
pub unsafe extern "C" fn devcycle_populated_user_merge_client_custom_data(
    populated_user: *mut CPopulatedUser,
    client_custom_data_json: *const c_char,
) -> i32 {
    clear_last_error();
    if populated_user.is_null() || client_custom_data_json.is_null() {
        set_error(
            "Populated user or custom data JSON pointer is null".to_string(),
            DevCycleFFIErrorCode::NullPointer,
        );
        return DevCycleFFIErrorCode::NullPointer as i32;
    }
    let json_str = match CStr::from_ptr(client_custom_data_json).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(
                format!(
                    "Failed to convert client custom data JSON from C string: {}",
                    e
                ),
                DevCycleFFIErrorCode::InputStringConversionFailed,
            );
            return DevCycleFFIErrorCode::InputStringConversionFailed as i32;
        }
    };
    let custom_data: HashMap<String, serde_json::Value> = match serde_json::from_str(json_str) {
        Ok(data) => data,
        Err(e) => {
            set_error(
                format!("Failed to parse client custom data JSON: {}", e),
                DevCycleFFIErrorCode::JsonParseFailed,
            );
            return DevCycleFFIErrorCode::JsonParseFailed as i32;
        }
    };
    let pop = &mut (*populated_user).0;
    for (k, v) in custom_data.into_iter() {
        if !pop.custom_data.contains_key(&k) && !pop.private_custom_data.contains_key(&k) {
            pop.custom_data.insert(k, v);
        }
    }
    set_last_error_code(DevCycleFFIErrorCode::Success);
    DevCycleFFIErrorCode::Success as i32
}

/// Get feature id from a variable result
/// Returns a C string that must be freed with devcycle_free_string
#[no_mangle]
pub unsafe extern "C" fn devcycle_variable_result_get_feature_id(
    result: *const CVariableForUserResult,
) -> *mut c_char {
    clear_last_error();
    if result.is_null() {
        set_error(
            "Variable result pointer is null".to_string(),
            DevCycleFFIErrorCode::NullPointer,
        );
        return ptr::null_mut();
    }
    match CString::new((*result).0.feature_id.clone()) {
        Ok(c_str) => {
            set_last_error_code(DevCycleFFIErrorCode::Success);
            c_str.into_raw()
        }
        Err(e) => {
            set_error(
                format!("Failed to build CString: {}", e),
                DevCycleFFIErrorCode::OperationFailed,
            );
            ptr::null_mut()
        }
    }
}

/// Get variation id from a variable result
/// Returns a C string that must be freed with devcycle_free_string
#[no_mangle]
pub unsafe extern "C" fn devcycle_variable_result_get_variation_id(
    result: *const CVariableForUserResult,
) -> *mut c_char {
    clear_last_error();
    if result.is_null() {
        set_error(
            "Variable result pointer is null".to_string(),
            DevCycleFFIErrorCode::NullPointer,
        );
        return ptr::null_mut();
    }
    match CString::new((*result).0.variation_id.clone()) {
        Ok(c_str) => {
            set_last_error_code(DevCycleFFIErrorCode::Success);
            c_str.into_raw()
        }
        Err(e) => {
            set_error(
                format!("Failed to build CString: {}", e),
                DevCycleFFIErrorCode::OperationFailed,
            );
            ptr::null_mut()
        }
    }
}

/// Get evaluation reason from a variable result ("ERROR" if error)
/// Returns a C string that must be freed with devcycle_free_string
#[no_mangle]
pub unsafe extern "C" fn devcycle_variable_result_get_evaluation_reason(
    result: *const CVariableForUserResult,
) -> *mut c_char {
    clear_last_error();
    if result.is_null() {
        set_error(
            "Variable result pointer is null".to_string(),
            DevCycleFFIErrorCode::NullPointer,
        );
        return ptr::null_mut();
    }
    let reason_str = match &(*result).0.eval_reason {
        Ok(r) => r.to_string(),
        Err(_) => "ERROR".to_string(),
    };
    match CString::new(reason_str) {
        Ok(c_str) => {
            set_last_error_code(DevCycleFFIErrorCode::Success);
            c_str.into_raw()
        }
        Err(e) => {
            set_error(
                format!("Failed to build CString: {}", e),
                DevCycleFFIErrorCode::OperationFailed,
            );
            ptr::null_mut()
        }
    }
}

/// Check if variable result contains an error (returns 1 if error, 0 otherwise)
#[no_mangle]
pub unsafe extern "C" fn devcycle_variable_result_is_error(
    result: *const CVariableForUserResult,
) -> i32 {
    if result.is_null() {
        set_error(
            "Variable result pointer is null".to_string(),
            DevCycleFFIErrorCode::NullPointer,
        );
        return 1; // preserve API semantics (1 indicates error)
    }
    match &(*result).0.eval_reason {
        Ok(_) => {
            set_last_error_code(DevCycleFFIErrorCode::Success);
            0
        }
        Err(_) => {
            set_last_error_code(DevCycleFFIErrorCode::OperationFailed);
            1
        }
    }
}

/// Get error message from variable result (null if no error)
/// Returns C string that must be freed with devcycle_free_string
#[no_mangle]
pub unsafe extern "C" fn devcycle_variable_result_get_error(
    result: *const CVariableForUserResult,
) -> *mut c_char {
    clear_last_error();
    if result.is_null() {
        set_error(
            "Variable result pointer is null".to_string(),
            DevCycleFFIErrorCode::NullPointer,
        );
        return ptr::null_mut();
    }
    match &(*result).0.eval_reason {
        Ok(_) => {
            set_last_error_code(DevCycleFFIErrorCode::Success);
            ptr::null_mut()
        }
        Err(e) => match CString::new(e.to_string()) {
            Ok(c_str) => {
                set_last_error_code(DevCycleFFIErrorCode::OperationFailed);
                c_str.into_raw()
            }
            Err(err2) => {
                set_error(
                    format!("Failed to build CString: {}", err2),
                    DevCycleFFIErrorCode::OperationFailed,
                );
                ptr::null_mut()
            }
        },
    }
}

/// Get a full JSON representation of the variable result including metadata
/// Returns C string that must be freed with devcycle_free_string
#[no_mangle]
pub unsafe extern "C" fn devcycle_variable_result_to_full_json(
    result: *const CVariableForUserResult,
) -> *mut c_char {
    clear_last_error();
    if result.is_null() {
        set_error(
            "Variable result pointer is null".to_string(),
            DevCycleFFIErrorCode::NullPointer,
        );
        return ptr::null_mut();
    }
    let (is_error, eval_reason_str, error_message) = match &(*result).0.eval_reason {
        Ok(r) => (false, r.to_string(), None),
        Err(e) => (true, "ERROR".to_string(), Some(e.to_string())),
    };
    let json_obj = serde_json::json!({
        "variableType": (*result).0.variable_type,
        "variableValue": (*result).0.variable_value,
        "featureId": (*result).0.feature_id,
        "variationId": (*result).0.variation_id,
        "evaluationReason": eval_reason_str,
        "isError": is_error,
        "errorMessage": error_message,
    });
    match serde_json::to_string(&json_obj) {
        Ok(json) => match CString::new(json) {
            Ok(c_str) => {
                set_last_error_code(DevCycleFFIErrorCode::Success);
                c_str.into_raw()
            }
            Err(e) => {
                set_error(
                    format!("Failed to build CString: {}", e),
                    DevCycleFFIErrorCode::OperationFailed,
                );
                ptr::null_mut()
            }
        },
        Err(e) => {
            set_error(
                format!("Failed to serialize variable result JSON: {}", e),
                DevCycleFFIErrorCode::OperationFailed,
            );
            ptr::null_mut()
        }
    }
}

// Helper: parse event type string to EventType
fn parse_event_type(s: &str) -> Option<crate::events::event::EventType> {
    use crate::events::event::EventType;
    match s {
        "AggregateVariableEvaluated" => Some(EventType::AggregateVariableEvaluated),
        "AggregateVariableDefaulted" => Some(EventType::AggregateVariableDefaulted),
        "VariableEvaluated" => Some(EventType::VariableEvaluated),
        "VariableDefaulted" => Some(EventType::VariableDefaulted),
        "SDKConfig" => Some(EventType::SDKConfig),
        "CustomEvent" => Some(EventType::CustomEvent),
        _ => None,
    }
}

/// Queue a manual event for a raw user JSON. The user will be populated internally using platform & client custom data.
/// Returns 0 on success, non-zero (DevCycleFFIErrorCode) on error. Use devcycle_get_last_error() for message.
/// event_type: one of the supported EventType strings (e.g., "CustomEvent"). For manual custom events use "CustomEvent".
/// meta_data_json: JSON object string for metadata (may be null for empty)
#[no_mangle]
pub unsafe extern "C" fn devcycle_queue_event(
    sdk_key: *const c_char,
    user_json: *const c_char,
    event_type_str: *const c_char,
    custom_type: *const c_char,
    target: *const c_char,
    value: f64,
    meta_data_json: *const c_char,
) -> i32 {
    clear_last_error();
    if user_json.is_null() || event_type_str.is_null() {
        set_error(
            "User JSON or event type pointer is null".to_string(),
            DevCycleFFIErrorCode::NullPointer,
        );
        return DevCycleFFIErrorCode::NullPointer as i32;
    }

    let sdk_key_str = match parse_sdk_key(sdk_key) {
        Ok(s) => s,
        Err(code) => return code as i32,
    };
    let user_json_str = match CStr::from_ptr(user_json).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(
                format!("Failed to convert user JSON from C string: {}", e),
                DevCycleFFIErrorCode::InputStringConversionFailed,
            );
            return DevCycleFFIErrorCode::InputStringConversionFailed as i32;
        }
    };
    let event_type_raw = match CStr::from_ptr(event_type_str).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(
                format!("Failed to convert event type from C string: {}", e),
                DevCycleFFIErrorCode::InputStringConversionFailed,
            );
            return DevCycleFFIErrorCode::InputStringConversionFailed as i32;
        }
    };

    let event_type = match parse_event_type(event_type_raw) {
        Some(et) => et,
        None => {
            set_error(
                format!("Unsupported event type: {}", event_type_raw),
                DevCycleFFIErrorCode::OperationFailed,
            );
            return DevCycleFFIErrorCode::OperationFailed as i32;
        }
    };

    // Parse optional strings (treat null as empty string)
    let custom_type_str = if custom_type.is_null() {
        ""
    } else {
        match CStr::from_ptr(custom_type).to_str() {
            Ok(s) => s,
            Err(e) => {
                set_error(
                    format!("Failed to convert custom type from C string: {}", e),
                    DevCycleFFIErrorCode::InputStringConversionFailed,
                );
                return DevCycleFFIErrorCode::InputStringConversionFailed as i32;
            }
        }
    };

    let target_str = if target.is_null() {
        ""
    } else {
        match CStr::from_ptr(target).to_str() {
            Ok(s) => s,
            Err(e) => {
                set_error(
                    format!("Failed to convert target from C string: {}", e),
                    DevCycleFFIErrorCode::InputStringConversionFailed,
                );
                return DevCycleFFIErrorCode::InputStringConversionFailed as i32;
            }
        }
    };

    // Parse metadata JSON (object) or use empty map
    let meta_data: HashMap<String, serde_json::Value> = if meta_data_json.is_null() {
        HashMap::new()
    } else {
        match CStr::from_ptr(meta_data_json).to_str() {
            Ok(json_str) => match serde_json::from_str(json_str) {
                Ok(map) => map,
                Err(e) => {
                    set_error(
                        format!("Failed to parse meta_data JSON: {}", e),
                        DevCycleFFIErrorCode::JsonParseFailed,
                    );
                    return DevCycleFFIErrorCode::JsonParseFailed as i32;
                }
            },
            Err(e) => {
                set_error(
                    format!("Failed to convert meta_data JSON from C string: {}", e),
                    DevCycleFFIErrorCode::InputStringConversionFailed,
                );
                return DevCycleFFIErrorCode::InputStringConversionFailed as i32;
            }
        }
    };

    // Deserialize User
    let user: User = match parse_user_json_tolerant(user_json_str) {
        Ok(u) => u,
        Err(_) => {
            set_error(
                "Failed to parse user JSON (must include userId)".to_string(),
                DevCycleFFIErrorCode::JsonParseFailed,
            );
            return DevCycleFFIErrorCode::JsonParseFailed as i32;
        }
    };

    // Obtain runtime
    let runtime = match get_runtime_or_set_error() {
        Some(rt) => rt,
        None => {
            set_error(
                "Runtime unavailable".to_string(),
                DevCycleFFIErrorCode::RuntimeUnavailable,
            );
            return DevCycleFFIErrorCode::RuntimeUnavailable as i32;
        }
    };

    // Get event queue
    let event_queue = match crate::events::event_queue_manager::get_event_queue(&sdk_key_str) {
        Some(eq) => eq,
        None => {
            set_error(
                format!("Event queue not initialized for SDK key: {}", sdk_key_str),
                DevCycleFFIErrorCode::EventQueueInitFailed,
            );
            return DevCycleFFIErrorCode::EventQueueInitFailed as i32;
        }
    };

    // Build base Event
    let mut event = crate::events::event::Event {
        event_type,
        target: target_str.to_string(),
        custom_type: custom_type_str.to_string(),
        user_id: String::new(),
        client_date: std::time::Instant::now(),
        value,
        feature_vars: HashMap::new(),
        meta_data,
    };

    // Synchronously populate user & enrich event similar to process_user_events
    let populated_result = runtime.block_on(async {
        // Acquire client custom data
        let client_custom_data =
            crate::config::client_custom_data::get_client_custom_data(sdk_key_str.to_string());
        // Platform data already stored
        let platform_data = match crate::config::platform_data::get_platform_data(&sdk_key_str) {
            Ok(pd) => pd,
            Err(e) => {
                set_error(
                    format!("Failed to get platform data for SDK key {}: {}", sdk_key_str, e),
                    DevCycleFFIErrorCode::OperationFailed,
                );
                return Err(DevCycleFFIErrorCode::OperationFailed);
            }
        };
        let populated_user = crate::user::PopulatedUser::new(
            user.clone(),
            platform_data.clone(),
            client_custom_data.clone(),
        );
        // Generate bucketed config
        match crate::generate_bucketed_config(
            &sdk_key_str,
            populated_user.clone(),
            client_custom_data,
        )
        .await
        {
            Ok(bucketed) => {
                event.feature_vars = bucketed.feature_variation_map;
                if event.event_type == crate::events::event::EventType::CustomEvent {
                    event.user_id = user.user_id.clone();
                }
                // Insert into user_event_queue using interior mutability (Mutex)
                let mut queue_guard = event_queue.user_event_queue.lock().await;
                let user_id_key = user.user_id.clone();
                queue_guard
                    .entry(user_id_key)
                    .or_insert_with(|| crate::events::event::UserEventsBatchRecord {
                        user: populated_user,
                        events: Vec::new(),
                    })
                    .events
                    .push(event);
                // Increment the event queue count
                let mut count_guard = event_queue.user_event_queue_count.lock().await;
                *count_guard += 1;
                Ok(())
            }
            Err(_) => Err(DevCycleFFIErrorCode::OperationFailed),
        }
    });

    match populated_result {
        Ok(_) => {
            set_last_error_code(DevCycleFFIErrorCode::Success);
            0
        }
        Err(code) => {
            if code == DevCycleFFIErrorCode::OperationFailed {
                set_error(
                    "Failed to bucket config while queueing event".to_string(),
                    DevCycleFFIErrorCode::OperationFailed,
                );
            }
            code as i32
        }
    }
}

#[cfg(all(test, feature = "ffi"))]
mod ffi_tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_init_and_queue_event() {
        let config_json = include_str!("../tests/resources/test_config.json");
        let sdk_key = CString::new("test-sdk-key").unwrap();
        let config_c = CString::new(config_json).unwrap();
        let client_data = CString::new("{}").unwrap();
        let rc = unsafe {
            devcycle_init_sdk_key(
                sdk_key.as_ptr(),
                config_c.as_ptr(),
                std::ptr::null(),
                client_data.as_ptr(),
                std::ptr::null(),
            )
        };
        if rc != 0 {
            // Attempt to fetch error message safely
            let err_ptr = unsafe { devcycle_get_last_error() };
            if !err_ptr.is_null() {
                let err = unsafe { CStr::from_ptr(err_ptr).to_string_lossy().into_owned() };
                unsafe { devcycle_free_string(err_ptr) };
                panic!("init_sdk_key failed rc={}, err={}", rc, err);
            } else {
                panic!("init_sdk_key failed rc={} with null error string", rc);
            }
        }
        assert!(
            crate::events::event_queue_manager::get_event_queue("test-sdk-key").is_some(),
            "Event queue should be initialized"
        );

        let user_json = CString::new("{\"userId\":\"user-1\"}").unwrap();
        let event_type = CString::new("CustomEvent").unwrap();
        let custom_type = CString::new("purchase").unwrap();
        let target = CString::new("sku-123").unwrap();
        let meta = CString::new("{\"amount\": 19.99}").unwrap();
        let queue_rc = unsafe {
            devcycle_queue_event(
                sdk_key.as_ptr(),
                user_json.as_ptr(),
                event_type.as_ptr(),
                custom_type.as_ptr(),
                target.as_ptr(),
                19.99,
                meta.as_ptr(),
            )
        };
        if queue_rc != 0 {
            let err_ptr = unsafe { devcycle_get_last_error() };
            let err = if !err_ptr.is_null() {
                unsafe { CStr::from_ptr(err_ptr).to_string_lossy().into_owned() }
            } else {
                "<null>".to_string()
            };
            if !err_ptr.is_null() {
                unsafe { devcycle_free_string(err_ptr) };
            }
            panic!("queue_event failed rc={}, err={}", queue_rc, err);
        }
        let eq = crate::events::event_queue_manager::get_event_queue("test-sdk-key").unwrap();
        // Access underlying struct for count (Arc deref)
        assert!(
            eq.user_event_queue_count > 0,
            "Expected user_event_queue_count > 0 after queueing event"
        );
    }
}
