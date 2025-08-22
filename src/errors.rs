use crate::errors::errors::DevCycleError;

pub(crate) mod errors {
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
        details: "Failed to decide target variation".parse().unwrap(),
    };
    pub const FAILED_USER_DOES_NOT_QUALIFY_FOR_TARGETS: DevCycleError = DevCycleError {
        details: "User does not qualify for any targets for feature"
            .parse()
            .unwrap(),
    };
    pub const FAILED_USER_DOES_NOT_QUALIFY_FOR_ROLLOUTS: DevCycleError = DevCycleError {
        details: "User does not qualify for any rollouts for feature"
            .parse()
            .unwrap(),
    };
    pub const MISSING_VARIABLE: DevCycleError = DevCycleError {
        details: "Missing variable".parse().unwrap(),
    };
    pub const MISSING_VARIATION: DevCycleError = DevCycleError {
        details: "Missing variation".parse().unwrap(),
    };
}
