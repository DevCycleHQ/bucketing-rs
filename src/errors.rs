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
pub fn failed_to_decide_variation() -> DevCycleError {
    DevCycleError::new("Failed to decide target variation")
}

pub fn failed_user_does_not_qualify_for_targets() -> DevCycleError {
    DevCycleError::new("User does not qualify for targets")
}

pub fn failed_user_does_not_qualify_for_rollouts() -> DevCycleError {
    DevCycleError::new("User does not qualify for rollouts")
}

pub fn missing_variable() -> DevCycleError {
    DevCycleError::new("Missing variable")
}

pub fn missing_variation() -> DevCycleError {
    DevCycleError::new("Missing variation")
}

pub fn missing_config() -> DevCycleError {
    DevCycleError::new("Missing config")
}