// FFI (Foreign Function Interface) bindings for C library support
#![cfg(feature = "ffi")]

use crate::events::EventQueueOptions;
use crate::user::{BucketedUserConfig, PopulatedUser, User};
use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

#[cfg(feature = "protobuf")]
use prost::Message;

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
    ProtobufDecodeFailed = -10,
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
        set_error(
            "SDK key pointer is null".to_string(),
            DevCycleFFIErrorCode::NullPointer,
        );
        return DevCycleFFIErrorCode::NullPointer as i32;
    }

    let sdk_key_str = match CStr::from_ptr(sdk_key).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(
                format!("Failed to convert SDK key from C string: {}", e),
                DevCycleFFIErrorCode::SdkKeyConversionFailed,
            );
            return DevCycleFFIErrorCode::SdkKeyConversionFailed as i32;
        }
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

    match runtime.block_on(crate::init_event_queue(sdk_key_str, event_options)) {
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

    if sdk_key.is_null() || user.is_null() {
        set_error(
            "SDK key or user pointer is null".to_string(),
            DevCycleFFIErrorCode::NullPointer,
        );
        return ptr::null_mut();
    }

    let sdk_key_str = match CStr::from_ptr(sdk_key).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(
                format!("Failed to convert SDK key from C string: {}", e),
                DevCycleFFIErrorCode::SdkKeyConversionFailed,
            );
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
        sdk_key_str,
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

    if sdk_key.is_null() || user.is_null() {
        set_error(
            "SDK key or user pointer is null".to_string(),
            DevCycleFFIErrorCode::NullPointer,
        );
        return ptr::null_mut();
    }

    let sdk_key_str = match CStr::from_ptr(sdk_key).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(
                format!("Failed to convert SDK key from C string: {}", e),
                DevCycleFFIErrorCode::SdkKeyConversionFailed,
            );
            return ptr::null_mut();
        }
    };

    let user_obj = (*user).0.clone();

    // Note: client_custom_data_json is ignored here because generate_bucketed_config_from_user
    // retrieves client custom data internally. Use devcycle_set_client_custom_data first if needed.
    let _ = client_custom_data_json; // Suppress unused parameter warning

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
        sdk_key_str,
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

    if sdk_key.is_null() || config_json.is_null() {
        set_error(
            "SDK key or config JSON pointer is null".to_string(),
            DevCycleFFIErrorCode::NullPointer,
        );
        return DevCycleFFIErrorCode::NullPointer as i32;
    }

    let sdk_key_str = match CStr::from_ptr(sdk_key).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(
                format!("Failed to convert SDK key from C string: {}", e),
                DevCycleFFIErrorCode::SdkKeyConversionFailed,
            );
            return DevCycleFFIErrorCode::SdkKeyConversionFailed as i32;
        }
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

    match runtime.block_on(crate::set_config(sdk_key_str, config_body)) {
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

    if sdk_key.is_null() || proto_bytes.is_null() {
        set_error(
            "SDK key or protobuf bytes pointer is null".to_string(),
            DevCycleFFIErrorCode::NullPointer,
        );
        return DevCycleFFIErrorCode::NullPointer as i32;
    }

    let sdk_key_str = match CStr::from_ptr(sdk_key).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(
                format!("Failed to convert SDK key from C string: {}", e),
                DevCycleFFIErrorCode::SdkKeyConversionFailed,
            );
            return DevCycleFFIErrorCode::SdkKeyConversionFailed as i32;
        }
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

    match runtime.block_on(crate::set_config_from_protobuf(sdk_key_str, proto_msg)) {
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

    if sdk_key.is_null() || custom_data_json.is_null() {
        set_error(
            "SDK key or custom data JSON pointer is null".to_string(),
            DevCycleFFIErrorCode::NullPointer,
        );
        return DevCycleFFIErrorCode::NullPointer as i32;
    }

    let sdk_key_str = match CStr::from_ptr(sdk_key).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(
                format!("Failed to convert SDK key from C string: {}", e),
                DevCycleFFIErrorCode::SdkKeyConversionFailed,
            );
            return DevCycleFFIErrorCode::SdkKeyConversionFailed as i32;
        }
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

    match runtime.block_on(crate::set_client_custom_data(sdk_key_str, custom_data)) {
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

    if sdk_key.is_null() || platform_data_json.is_null() {
        set_error(
            "SDK key or platform data JSON pointer is null".to_string(),
            DevCycleFFIErrorCode::NullPointer,
        );
        return DevCycleFFIErrorCode::NullPointer as i32;
    }

    let sdk_key_str = match CStr::from_ptr(sdk_key).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(
                format!("Failed to convert SDK key from C string: {}", e),
                DevCycleFFIErrorCode::SdkKeyConversionFailed,
            );
            return DevCycleFFIErrorCode::SdkKeyConversionFailed as i32;
        }
    };

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

    runtime.block_on(crate::set_platform_data(sdk_key_str, platform_data));
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

    if sdk_key.is_null() || config_json.is_null() {
        set_error(
            "SDK key or config JSON pointer is null".to_string(),
            DevCycleFFIErrorCode::NullPointer,
        );
        return DevCycleFFIErrorCode::NullPointer as i32;
    }

    let sdk_key_str = match CStr::from_ptr(sdk_key).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(
                format!("Failed to convert SDK key from C string: {}", e),
                DevCycleFFIErrorCode::SdkKeyConversionFailed,
            );
            return DevCycleFFIErrorCode::SdkKeyConversionFailed as i32;
        }
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
        sdk_key_str,
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
    if sdk_key.is_null() || user.is_null() || variable_key.is_null() || variable_type.is_null() {
        set_error(
            "One or more required pointers are null".to_string(),
            DevCycleFFIErrorCode::NullPointer,
        );
        return ptr::null_mut();
    }
    let sdk_key_str = match CStr::from_ptr(sdk_key).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(
                format!("Failed to convert SDK key from C string: {}", e),
                DevCycleFFIErrorCode::SdkKeyConversionFailed,
            );
            return ptr::null_mut();
        }
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
        sdk_key_str,
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
    match serde_json::from_str::<User>(json_str) {
        Ok(user) => {
            set_last_error_code(DevCycleFFIErrorCode::Success);
            Box::into_raw(Box::new(CUser(user)))
        }
        Err(e) => {
            set_error(
                format!("Failed to parse user JSON: {}", e),
                DevCycleFFIErrorCode::JsonParseFailed,
            );
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
    if sdk_key.is_null() || user.is_null() {
        set_error(
            "SDK key or user pointer is null".to_string(),
            DevCycleFFIErrorCode::NullPointer,
        );
        return ptr::null_mut();
    }
    let sdk_key_str = match CStr::from_ptr(sdk_key).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(
                format!("Failed to convert SDK key from C string: {}", e),
                DevCycleFFIErrorCode::SdkKeyConversionFailed,
            );
            return ptr::null_mut();
        }
    };
    let user_obj = (*user).0.clone();
    let populated = user_obj.get_populated_user(sdk_key_str);
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
