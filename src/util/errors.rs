use crate::events::event::DefaultReason;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct DevCycleError {
    pub(crate) details: String,
}

impl DevCycleError {
    pub fn new(msg: &str) -> DevCycleError {
        DevCycleError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for DevCycleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for DevCycleError {
    fn description(&self) -> &str {
        &self.details
    }
}

pub const FAILED_TO_DECIDE_VARIATION: DevCycleError = DevCycleError {
    details: String::new(),
};
pub const FAILED_USER_DOES_NOT_QUALIFY_FOR_TARGETS: DevCycleError = DevCycleError {
    details: String::new(),
};
pub const FAILED_USER_DOES_NOT_QUALIFY_FOR_ROLLOUTS: DevCycleError = DevCycleError {
    details: String::new(),
};
pub const MISSING_VARIABLE: DevCycleError = DevCycleError {
    details: String::new(),
};
pub const MISSING_VARIATION: DevCycleError = DevCycleError {
    details: String::new(),
};

// Helper functions to create errors with proper messages
pub(crate) fn failed_to_decide_variation() -> DevCycleError {
    DevCycleError::new("Failed to decide target variation")
}

pub(crate) fn failed_user_does_not_qualify_for_targets() -> DevCycleError {
    DevCycleError::new("User does not qualify for any targets")
}

pub(crate) fn failed_user_does_not_qualify_for_rollouts() -> DevCycleError {
    DevCycleError::new("User does not qualify for rollouts")
}

pub(crate) fn missing_variable() -> DevCycleError {
    DevCycleError::new("Variable not found")
}

pub(crate) fn missing_variation() -> DevCycleError {
    DevCycleError::new("Variation not found")
}

pub(crate) fn missing_config() -> DevCycleError {
    DevCycleError::new("Config not found")
}

pub(crate) fn missing_feature() -> DevCycleError {
    DevCycleError::new("Feature not found")
}

pub(crate) fn missing_variable_for_variation() -> DevCycleError {
    DevCycleError::new("Variable not found for variation")
}

pub(crate) fn invalid_variable_type() -> DevCycleError {
    DevCycleError::new("Invalid variable type")
}

pub(crate) fn variable_type_mismatch() -> DevCycleError {
    DevCycleError::new("Variable type mismatch")
}

pub(crate) fn bucket_result_error_to_default_reason(err: &DevCycleError) -> DefaultReason {
    match err.details.as_str() {
        "Missing config" => DefaultReason::MissingConfig,
        "Missing variable" => DefaultReason::MissingVariable,
        "Missing feature" => DefaultReason::MissingFeature,
        "Missing variation" => DefaultReason::MissingVariation,
        "Missing variable for variation" => DefaultReason::MissingVariableForVariation,
        "User does not qualify for rollouts" => DefaultReason::UserNotInRollout,
        "User does not qualify for targets" => DefaultReason::UserNotTargeted,
        "Invalid variable type" => DefaultReason::InvalidVariableType,
        "Variable type mismatch" => DefaultReason::VariableTypeMismatch,
        "" => DefaultReason::NotDefaulted,
        _ => DefaultReason::Unknown,
    }
}

pub(crate) fn event_queue_not_initialized() -> DevCycleError {
    return DevCycleError::new("Event queue not initialized");
}

pub(crate) fn failed_to_set_client_custom_data() -> DevCycleError {
    return DevCycleError::new("Failed to set client custom data");
}
